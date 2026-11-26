// Runs inside VS Code. Exercises the extension the way a person would: open
// an example, hover, see its problems, outline and colours, and run it on
// its data.
"use strict";

const assert = require("node:assert/strict");
const path = require("node:path");
const vscode = require("vscode");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function until(what, probe, seconds = 20) {
  for (let i = 0; i < seconds * 10; i++) {
    const value = await probe();
    if (value) return value;
    await sleep(100);
  }
  throw new Error(`timed out waiting for ${what}`);
}

const hoverText = (hovers) =>
  hovers.flatMap((h) => h.contents).map((c) => (typeof c === "string" ? c : c.value)).join("\n");

/** Runs a command that starts a qsys task and returns the task's exit code. */
async function taskExit(command) {
  const finished = new Promise((resolve) => {
    const listener = vscode.tasks.onDidEndTaskProcess((event) => {
      if (event.execution.task.definition.type === "qsystems") {
        listener.dispose();
        resolve(event.exitCode);
      }
    });
  });
  await vscode.commands.executeCommand(command);
  return Promise.race([finished, sleep(20000).then(() => "timed out")]);
}

exports.run = async function run() {
  const folder = vscode.workspace.workspaceFolders[0].uri.fsPath;
  const document = await vscode.workspace.openTextDocument(path.join(folder, "paraphrase.q"));
  await vscode.window.showTextDocument(document);
  assert.equal(document.languageId, "qsystems", "the .q extension selects the language");
  const line = (text) => {
    for (let i = 0; i < document.lineCount; i++) if (document.lineAt(i).text.startsWith(text)) return i;
    throw new Error(`no line ${text}`);
  };

  // The server is ready when it answers a hover. On a rule of the -INV- phase: the rule inverted.
  const rule = new vscode.Position(line("SUJ + PH(U*)"), 2);
  const hovers = await until("a hover from the language server", async () => {
    const found = await vscode.commands.executeCommand("vscode.executeHoverProvider", document.uri, rule);
    return found.length ? found : undefined;
  });
  assert.match(hoverText(hovers), /Phase 1, rule 4/);
  assert.match(hoverText(hovers), /LE \+ FAIT \+ QUE \+ PI\(U\*\) == SUJ \+ PH\(U\*\)/);
  console.log("hover ok");

  const legend = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokensLegend", document.uri);
  assert.deepEqual(legend.tokenTypes, ["comment", "keyword", "operator", "parameter", "number", "macro"]);
  const tokens = await vscode.commands.executeCommand("vscode.provideDocumentSemanticTokens", document.uri);
  assert.ok(tokens.data.length > 0);
  const symbols = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", document.uri);
  assert.deepEqual(symbols.map((s) => s.name), ["phase 1", "phase 2", "phase 3", "phase 4"]);
  console.log("symbols and semantic tokens ok");

  // A rule that breaks C70's restriction on parameters.
  const broken = await vscode.workspace.openTextDocument({ language: "qsystems", content: "A* == B(I*).\n" });
  await vscode.window.showTextDocument(broken);
  const problems = await until("diagnostics", () => {
    const found = vscode.languages.getDiagnostics(broken.uri);
    return found.length ? found : undefined;
  });
  assert.equal(problems[0].source, "qsys");
  assert.match(problems[0].message, /I\*/);
  console.log("diagnostics ok");

  // The data beside the treatment is its own language.
  const data = await vscode.workspace.openTextDocument(path.join(folder, "paraphrase.dat"));
  assert.equal(data.languageId, "qsystems-data");

  // Run, and Run with details: tasks that end with exit code 0, using paraphrase.dat.
  await vscode.window.showTextDocument(document);
  assert.equal(await taskExit("qsystems.run"), 0);
  console.log("run ok");
  assert.equal(await taskExit("qsystems.det"), 0);
  console.log("details ok");
};
