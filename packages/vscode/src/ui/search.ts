import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import type { SearchPlace, SearchResult } from "../shapes.js";
import type { ProjectBuilds } from "./projectBuilds.js";
import { searchItems, searchPlaceholder, type SearchItem } from "./searchItems.js";

const SEARCH_COMMAND = "ascribe.search";

/** How long after the last keystroke the server is asked. */
const DEBOUNCE_MS = 150;

/** A search to run instead of asking, for tests. */
export interface SearchAnswer {
  /** What's typed. */
  query: string;
  /** Whether Search Every Build is pressed. */
  everyBuild?: boolean;
  /** The item to choose, by its place's `file:line`; none to close without choosing. */
  choose?: string;
}

/** Search Pages as Readers See Them, for tests. */
export interface SearchApi {
  /** The items it shows for a query, in the active file's project. */
  items(query: string, everyBuild?: boolean): Promise<SearchItem[]>;
  /** Answers the next search with this, instead of showing the quick pick. */
  answerNext(answer: SearchAnswer): void;
  /** Settles when the last search's choice has opened. */
  whenOpened(): Promise<void>;
}

const EVERY_BUILD: vscode.QuickInputButton = {
  iconPath: new vscode.ThemeIcon("layers"),
  tooltip: "Search Every Build",
};

type PickItem = vscode.QuickPickItem & { place?: SearchPlace };

/**
 * **Ascribe: Search Pages as Readers See Them**: a quick pick that asks the
 * project's server as you type (`ascribe/search`), in the build the status
 * bar names, or with its button in every build. Choosing a place opens its
 * line; one in a phrase's value opens the phrase in `ascribe.toml`.
 */
export class SearchPages implements vscode.Disposable {
  private readonly answers: SearchAnswer[] = [];
  private readonly disposables: vscode.Disposable[] = [];
  private opened: Promise<void> = Promise.resolve();

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly builds: ProjectBuilds,
  ) {}

  get api(): SearchApi {
    return {
      items: async (query, everyBuild = false) => {
        const server = await this.server();
        if (!server) return [];
        return searchItems(await this.ask(server, query, everyBuild));
      },
      answerNext: (answer) => this.answers.push(answer),
      whenOpened: () => this.opened,
    };
  }

  register(): void {
    this.disposables.push(vscode.commands.registerCommand(SEARCH_COMMAND, () => this.show()));
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  /** The active file's project's server, started: the first project's when no file is active. */
  private async server(): Promise<ProjectServer | undefined> {
    const server = this.projects.current();
    if (!server) return undefined;
    await server.start();
    return server.state === "running" ? server : undefined;
  }

  /** The build the status bar names for the project, or none for every build. */
  private build(server: ProjectServer, everyBuild: boolean): string | undefined {
    return everyBuild ? undefined : this.builds.info(server).build;
  }

  private async ask(
    server: ProjectServer,
    query: string,
    everyBuild: boolean,
  ): Promise<SearchResult> {
    const build = this.build(server, everyBuild);
    return (await server.request("ascribe/search", {
      textDocument: { uri: vscode.Uri.file(server.project.config).toString() },
      query,
      ...(build === undefined ? {} : { build }),
    })) as SearchResult;
  }

  private async show(): Promise<void> {
    const server = await this.server();
    if (!server) {
      void vscode.window.showInformationMessage(
        "Ascribe: there's no running Ascribe project to search in this workspace.",
      );
      return;
    }
    const answer = this.answers.shift();
    if (answer !== undefined) {
      const items = searchItems(await this.ask(server, answer.query, answer.everyBuild ?? false));
      const chosen = items.find(
        (item) =>
          item.place && `${item.place.file}:${item.place.range.start.line + 1}` === answer.choose,
      );
      if (chosen?.place) {
        this.opened = this.open(chosen.place);
        await this.opened;
      }
      return;
    }
    const pick = vscode.window.createQuickPick<PickItem>();
    pick.title = "Search Pages as Readers See Them";
    // The server matches; the quick pick only shows what it found.
    pick.matchOnDescription = false;
    pick.matchOnDetail = false;
    let everyBuild = false;
    let asked = 0;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const frame = () => {
      pick.placeholder = searchPlaceholder(this.build(server, everyBuild));
      pick.buttons = everyBuild
        ? [
            {
              iconPath: new vscode.ThemeIcon("layers-active"),
              tooltip: "Search Only the Build Shown",
            },
          ]
        : [EVERY_BUILD];
    };
    const search = async () => {
      const query = pick.value;
      const mine = ++asked;
      if (query.trim() === "") {
        pick.items = [];
        pick.busy = false;
        return;
      }
      pick.busy = true;
      try {
        const result = await this.ask(server, query, everyBuild);
        if (mine !== asked) return;
        pick.items = searchItems(result).map((item) => {
          const shown: PickItem = {
            label: item.label,
            description: item.description,
            alwaysShow: true,
          };
          if (item.detail !== "") shown.detail = item.detail;
          if (item.place) shown.place = item.place;
          return shown;
        });
      } catch (error) {
        if (mine === asked) server.reportFeatureError("search", error);
      } finally {
        if (mine === asked) pick.busy = false;
      }
    };
    const soon = () => {
      clearTimeout(timer);
      timer = setTimeout(() => void search(), DEBOUNCE_MS);
    };
    frame();
    const done = new Promise<void>((resolve) => {
      const listeners = [
        pick.onDidChangeValue(soon),
        pick.onDidTriggerButton(() => {
          everyBuild = !everyBuild;
          frame();
          void search();
        }),
        pick.onDidAccept(() => {
          const place = pick.selectedItems[0]?.place;
          if (!place) return;
          pick.hide();
          this.opened = this.open(place);
        }),
        pick.onDidHide(() => {
          clearTimeout(timer);
          for (const listener of listeners) listener.dispose();
          pick.dispose();
          resolve();
        }),
      ];
    });
    pick.show();
    await done;
  }

  /** Opens a place's file with the cursor at the match, its line in view. */
  private async open(place: SearchPlace): Promise<void> {
    const { start, end } = place.range;
    const range = new vscode.Range(start.line, start.character, end.line, end.character);
    const editor = await vscode.window.showTextDocument(vscode.Uri.parse(place.uri), {
      selection: new vscode.Selection(range.start, range.start),
    });
    editor.revealRange(range, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
  }
}
