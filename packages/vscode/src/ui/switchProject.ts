import * as os from "node:os";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import { switchItemAt, switchItems, tildeFolder, type SwitchButton } from "./describe.js";
import type { ProjectBuilds } from "./projectBuilds.js";

export const SWITCH_PROJECT_COMMAND = "ascribe.switchProject";

/** What Switch Project needs of the sidebar: the project it shows, and a way to show another. */
export interface ProjectChooser {
  readonly project: ProjectServer | undefined;
  choose(server: ProjectServer): void;
}

/** An item of Switch Project, as the tests see it. */
export interface SwitchProjectItem {
  folder: string;
  label: string;
  description: string;
  detail: string;
  current: boolean;
  buttons: SwitchButton[];
}

/** Switch Project, for tests. */
export interface SwitchProjectApi {
  /** Its items, as it would list them now. */
  items(): SwitchProjectItem[];
  /** Answers the next Switch Project with the project in this folder, instead of asking. */
  answerNext(folder: string): void;
  /** Runs an item's button on the project in this folder, as clicking it does. */
  run(button: SwitchButton, folder: string): Promise<void>;
  /** Asks the running servers for their builds again, as its title's Refresh button does. */
  refresh(): Promise<void>;
}

const BUTTONS: Record<SwitchButton, vscode.QuickInputButton & { id: SwitchButton }> = {
  showOutput: {
    id: "showOutput",
    iconPath: new vscode.ThemeIcon("output"),
    tooltip: "Show Output",
  },
  restart: {
    id: "restart",
    iconPath: new vscode.ThemeIcon("debug-restart"),
    tooltip: "Restart Server",
  },
};

const REFRESH: vscode.QuickInputButton = {
  iconPath: new vscode.ThemeIcon("refresh"),
  tooltip: "Refresh",
};

type PickItem = vscode.QuickPickItem & { config: string };

/**
 * **Ascribe: Switch Project**: a quick pick of the workspace's projects,
 * the sidebar's first. Choosing one shows it in the sidebar and starts its
 * server if it hasn't started. Listing them starts nothing.
 */
export class SwitchProject implements vscode.Disposable {
  private readonly answers: string[] = [];
  private readonly disposables: vscode.Disposable[] = [];

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly builds: ProjectBuilds,
    private readonly sidebar: ProjectChooser,
  ) {}

  get api(): SwitchProjectApi {
    return {
      items: () =>
        this.items().map(({ folder, label, description, detail, current, buttons }) => ({
          folder,
          label,
          description,
          detail,
          current,
          buttons,
        })),
      answerNext: (folder) => this.answers.push(folder),
      run: async (button, folder) => {
        const server = this.projects.serverAt(folder);
        if (!server) throw new Error(`no project in ${folder}`);
        await this.press(button, server);
      },
      refresh: () => this.refresh(),
    };
  }

  register(): void {
    this.disposables.push(
      vscode.commands.registerCommand(SWITCH_PROJECT_COMMAND, () => this.show()),
    );
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
  }

  private items() {
    return switchItems(
      this.projects.servers.map((server) => this.builds.info(server)),
      this.sidebar.project?.project.config,
      (folder) => tildeFolder(folder, os.homedir()),
    );
  }

  private async show(): Promise<void> {
    const answer = this.answers.shift();
    if (answer !== undefined) {
      const item = switchItemAt(this.items(), answer);
      const server = item && this.serverOf(item.config);
      if (server) this.sidebar.choose(server);
      return;
    }
    const pick = vscode.window.createQuickPick<PickItem>();
    pick.title = "Switch Project";
    pick.placeholder = "The project the Ascribe sidebar shows";
    pick.matchOnDescription = true;
    pick.buttons = [REFRESH];
    const fill = () => {
      pick.items = this.items().map((item) => ({
        label: item.label,
        description: item.description,
        detail: item.detail,
        config: item.config,
        buttons: item.buttons.map((button) => BUTTONS[button]),
      }));
    };
    fill();
    const done = new Promise<void>((resolve) => {
      const listeners = [
        this.builds.onDidChange(fill),
        pick.onDidTriggerButton(() => void this.refresh()),
        pick.onDidTriggerItemButton(({ item, button }) => {
          const server = this.serverOf(item.config);
          const id = (button as { id?: SwitchButton }).id;
          if (server && id) void this.press(id, server);
        }),
        pick.onDidAccept(() => {
          const server = pick.selectedItems[0] && this.serverOf(pick.selectedItems[0].config);
          if (server) this.sidebar.choose(server);
          pick.hide();
        }),
        pick.onDidHide(() => {
          for (const listener of listeners) listener.dispose();
          pick.dispose();
          resolve();
        }),
      ];
    });
    pick.show();
    await done;
  }

  private async press(button: SwitchButton, server: ProjectServer): Promise<void> {
    if (button === "showOutput") {
      server.showOutput();
      return;
    }
    // A server that hasn't started stays off until it's needed.
    if (server.state === "running" || server.state === "failed") await server.restart();
  }

  /** Asks the running servers for their builds again. Starts nothing. */
  private async refresh(): Promise<void> {
    await Promise.all(
      this.projects.servers
        .filter((server) => server.state === "running")
        .map((server) => this.builds.refresh(server)),
    );
  }

  private serverOf(config: string): ProjectServer | undefined {
    return this.projects.servers.find((server) => server.project.config === config);
  }
}
