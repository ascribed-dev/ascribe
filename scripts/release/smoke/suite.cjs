// Runs inside VS Code, with the packaged extension installed. Checks that it
// starts a language server from the binary it's expected to use, and that the
// server reports the broken page's diagnostic.
const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

exports.run = async function run() {
  const expected = process.env.SMOKE_EXPECT_SOURCE;
  const workspace = process.env.SMOKE_WORKSPACE;
  const extension = vscode.extensions.getExtension("Ascribe.ascribe-vscode");
  assert.ok(extension, "the extension Ascribe.ascribe-vscode isn't installed");
  const api = await extension.activate();
  await api.whenSettled();

  const binary = api.binary();
  assert.ok(binary, "the extension found no binary");
  assert.equal(api.state(), "running", `the server isn't running (${binary.path})`);
  assert.equal(binary.source, expected, `expected the ${expected} binary, got ${binary.path}`);
  if (expected === "bundled") {
    const dir = path.join(extension.extensionPath, "bin", `${process.platform}-${process.arch}`);
    assert.equal(path.dirname(binary.path), dir);
  }
  const version = binary.version.parts.join(".");
  assert.equal(version, extension.packageJSON.version, "the binary's version");
  assert.equal(binary.warning, undefined);

  const page = vscode.Uri.file(path.join(workspace, "docs", "broken.md"));
  await vscode.window.showTextDocument(await vscode.workspace.openTextDocument(page));
  const deadline = Date.now() + 30_000;
  for (;;) {
    const codes = vscode.languages
      .getDiagnostics(page)
      .map((d) => String(typeof d.code === "object" ? d.code.value : d.code));
    if (codes.includes("ASC001")) break;
    if (Date.now() > deadline) assert.fail(`no ASC001 on broken.md; got ${codes.join(", ")}`);
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  process.stdout.write(
    `smoke: ${extension.id} ${extension.packageJSON.version} on ${process.platform}-${process.arch} ` +
      `used the ${binary.source} binary and reported ASC001\n`,
  );
};
