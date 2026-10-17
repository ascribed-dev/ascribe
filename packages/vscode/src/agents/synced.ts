// Where a synced copy of the skill is, for the context key that decides
// whether chat gets the extension's copy. Apart from VS Code, for the unit tests.
import * as path from "node:path";

/**
 * The context key the manifest's `chatSkills` entry reads: true while a
 * workspace folder has a synced copy of the skill, so chat lists the
 * project's copy and not the extension's too.
 */
export const SYNCED_KEY = "ascribe.agentsSynced";

/**
 * The folders of a workspace folder VS Code's chat loads skills from, the
 * two `ascribe agents sync` writes among them.
 */
const SKILL_FOLDERS = [".github/skills", ".claude/skills", ".agents/skills"] as const;

/** Where a synced copy of the skill would be in a workspace folder, one per folder chat reads. */
export function syncedSkillPaths(workspaceFolder: string): string[] {
  return SKILL_FOLDERS.map((folder) => path.join(workspaceFolder, folder, "ascribe", "SKILL.md"));
}

/** Whether any of the workspace folders has a synced copy, by `exists`. */
export async function anySynced(
  workspaceFolders: readonly string[],
  exists: (file: string) => Promise<boolean>,
): Promise<boolean> {
  const found = await Promise.all(
    workspaceFolders.flatMap((folder) => syncedSkillPaths(folder).map(exists)),
  );
  return found.includes(true);
}

/**
 * The glob, relative to a workspace folder, of everything whose creation or
 * deletion can change the answer: the copies, and the folders above them,
 * since deleting a folder reports the folder, not the files in it.
 */
export const WATCHED = `{${SKILL_FOLDERS.flatMap((folder) => {
  const parts = folder.split("/");
  return [parts[0], folder, `${folder}/ascribe`, `${folder}/ascribe/SKILL.md`];
}).join(",")}}`;
