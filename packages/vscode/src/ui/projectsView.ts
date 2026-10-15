import * as os from "node:os";
import * as vscode from "vscode";
import type { ProjectServer } from "../client.js";
import type { ProjectRegistry } from "../registry.js";
import {
  childNodes,
  nodeLook,
  projectNodes,
  tildeFolder,
  type NodeLook,
  type ProjectNode,
} from "./describe.js";
import type { ProjectBuilds } from "./projectBuilds.js";

const VIEW_ID = "ascribe.projects";

/** An item of the view, as the tests see it. */
interface ProjectsItem extends NodeLook {
  children: ProjectsItem[];
}

/** The Projects view, for tests. */
export interface ProjectsViewApi {
  /** Every item, as the view would show it with every project expanded. */
  items(): Promise<ProjectsItem[]>;
  /** Refreshes the view, as its title bar button does. */
  refresh(): Promise<void>;
  /** Runs an item's inline button on a project. */
  run(button: "showOutput" | "restart", folder: string): Promise<void>;
}

/**
 * The Projects view in the Ascribe sidebar: every project the registry
 * knows, started or not. It reads the registry and never starts a server.
 */
export class ProjectsView implements vscode.TreeDataProvider<ProjectNode>, vscode.Disposable {
  private readonly changed = new vscode.EventEmitter<void>();
  private readonly disposables: vscode.Disposable[] = [];

  readonly onDidChangeTreeData = this.changed.event;

  constructor(
    private readonly projects: ProjectRegistry,
    private readonly builds: ProjectBuilds,
  ) {}

  get api(): ProjectsViewApi {
    const expand = async (node?: ProjectNode): Promise<ProjectsItem[]> =>
      Promise.all(
        this.getChildren(node).map(async (child) => ({
          ...this.look(child),
          children: await expand(child),
        })),
      );
    return {
      items: () => expand(),
      refresh: () => this.refresh(),
      run: async (button, folder) => {
        const server = this.projects.serverAt(folder);
        if (!server) throw new Error(`no project in ${folder}`);
        await vscode.commands.executeCommand(`ascribe.projects.${button}`, this.nodeOf(server));
      },
    };
  }

  register(): void {
    this.disposables.push(
      vscode.window.createTreeView(VIEW_ID, { treeDataProvider: this, showCollapseAll: true }),
      this.builds.onDidChange(() => this.changed.fire()),
      vscode.commands.registerCommand("ascribe.projects.refresh", () => this.refresh()),
      vscode.commands.registerCommand("ascribe.projects.showOutput", (node?: ProjectNode) =>
        this.serverOf(node)?.showOutput(),
      ),
      vscode.commands.registerCommand("ascribe.projects.restart", (node?: ProjectNode) => {
        // A server that hasn't started stays off until it's needed.
        const server = this.serverOf(node);
        if (server?.state === "running" || server?.state === "failed") return server.restart();
        return undefined;
      }),
    );
  }

  getTreeItem(node: ProjectNode): vscode.TreeItem {
    const look = this.look(node);
    const item = new vscode.TreeItem(
      look.label,
      look.expandable
        ? vscode.TreeItemCollapsibleState.Collapsed
        : vscode.TreeItemCollapsibleState.None,
    );
    item.description = look.description;
    item.tooltip = look.tooltip;
    item.contextValue = look.contextValue;
    item.id = `${node.kind}:${node.project.config}`;
    if (look.icon) {
      item.iconPath = new vscode.ThemeIcon(
        look.icon,
        node.kind === "project" && node.project.state === "stopped"
          ? new vscode.ThemeColor("disabledForeground")
          : node.kind === "project" && node.project.state === "failed"
            ? new vscode.ThemeColor("problemsWarningIcon.foreground")
            : undefined,
      );
    }
    if (node.kind === "model") {
      const uri = vscode.Uri.file(node.project.config);
      item.resourceUri = uri;
      item.command = { command: "vscode.open", title: "Open", arguments: [uri] };
    }
    return item;
  }

  getChildren(node?: ProjectNode): ProjectNode[] {
    if (node) return childNodes(this.fresh(node));
    return projectNodes(this.projects.servers.map((server) => this.builds.info(server)));
  }

  dispose(): void {
    for (const disposable of this.disposables.splice(0)) disposable.dispose();
    this.changed.dispose();
  }

  /** Asks the running servers for their builds again, and redraws. Starts nothing. */
  private async refresh(): Promise<void> {
    await Promise.all(
      this.projects.servers
        .filter((server) => server.state === "running")
        .map((server) => this.builds.refresh(server)),
    );
    this.changed.fire();
  }

  private look(node: ProjectNode): NodeLook {
    return nodeLook(node, (folder) => tildeFolder(folder, os.homedir()));
  }

  /** The node with its project as it is now: a node VS Code hands back may be from an earlier draw. */
  private fresh(node: ProjectNode): ProjectNode {
    const server = this.serverOf(node);
    return server ? { ...node, project: this.builds.info(server) } : node;
  }

  private nodeOf(server: ProjectServer): ProjectNode {
    return { kind: "project", project: this.builds.info(server) };
  }

  private serverOf(node: ProjectNode | undefined): ProjectServer | undefined {
    return node && this.projects.servers.find((s) => s.project.config === node.project.config);
  }
}
