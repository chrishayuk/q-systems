// The Q-systems extension: starts the language server (`qsys lsp`) and adds
// commands that run the treatment in the editor on its data: the graph after
// each phase, or every application of every rule as well.
"use strict";

const vscode = require("vscode");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;

function setting(name) {
  const value = vscode.workspace.getConfiguration("qsystems").get(name);
  return typeof value === "string" ? value.trim() : value;
}

async function startServer(context) {
  const custom = setting("path");
  // Built in: qsys compiled to WebAssembly, run by the editor's own Node,
  // so nothing has to be installed.
  const builtIn = { module: context.asAbsolutePath(path.join("server", "lsp.js")), transport: TransportKind.stdio };
  const serverOptions = custom ? { command: custom, args: ["lsp"] } : { run: builtIn, debug: builtIn };
  client = new LanguageClient("qsystems", "Q-systems", serverOptions, {
    documentSelector: [{ language: "qsystems" }, { language: "qsystems-data" }],
  });
  await client.start();
}

async function restartServer(context) {
  if (client) await client.stop().catch(() => {});
  client = undefined;
  await startServer(context);
}

/** Runs qsys as a task in a terminal of its own. */
async function runTask(context, args, title, program, data) {
  const custom = setting("path");
  const width = setting("width");
  const all = [...args, ...(width ? ["--width", String(width)] : []), program, data];
  const execution = custom
    ? new vscode.ProcessExecution(custom, all)
    : new vscode.ProcessExecution(process.execPath, [context.asAbsolutePath(path.join("server", "run.js")), ...all], {
        env: { ELECTRON_RUN_AS_NODE: "1" },
      });
  const task = new vscode.Task({ type: "qsystems" }, vscode.TaskScope.Workspace, `${title} ${path.basename(program)}`, "qsys", execution, []);
  task.presentationOptions = {
    reveal: vscode.TaskRevealKind.Always,
    panel: vscode.TaskPanelKind.Dedicated,
    focus: true,
    clear: true,
    echo: false,
    showReuseMessage: false,
  };
  await vscode.tasks.executeTask(task);
}

/** A document's text on disk: saved if it is a file, copied if it is not. */
async function onDisk(document, name) {
  if (document.isUntitled || document.uri.scheme !== "file") {
    const file = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "qsys-")), name);
    fs.writeFileSync(file, document.getText());
    return file;
  }
  if (document.isDirty) await document.save();
  return document.uri.fsPath;
}

/**
 * The treatment and its data. In a .q file, the data is the .dat file of the
 * same name beside it; in a .dat file, the treatment is the .q of that name.
 * Failing that, the person is asked.
 */
async function pair(document) {
  const here = await onDisk(document, document.languageId === "qsystems-data" ? "data.dat" : "program.q");
  const wantData = document.languageId === "qsystems";
  const other = here.replace(/\.(q|dat|qdat)$/, wantData ? ".dat" : ".q");
  if (other !== here && fs.existsSync(other)) return wantData ? [here, other] : [other, here];
  const picked = await vscode.window.showOpenDialog({
    canSelectMany: false,
    openLabel: wantData ? "Use as data" : "Use as treatment",
    filters: wantData ? { "Q-system data": ["dat", "qdat"] } : { "Q-system treatment": ["q"] },
    defaultUri: vscode.Uri.file(path.dirname(here)),
  });
  if (!picked || !picked.length) return null;
  return wantData ? [here, picked[0].fsPath] : [picked[0].fsPath, here];
}

async function runText(context, args, title) {
  const editor = vscode.window.activeTextEditor;
  if (!editor || !["qsystems", "qsystems-data"].includes(editor.document.languageId)) {
    vscode.window.showInformationMessage("Open a Q-system treatment (.q) or its data (.dat) to run it.");
    return;
  }
  const files = await pair(editor.document);
  if (files) await runTask(context, args, title, ...files);
}

async function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("qsystems.run", () => runText(context, [], "Run")),
    vscode.commands.registerCommand("qsystems.det", () => runText(context, ["--det"], "Details")),
    vscode.commands.registerCommand("qsystems.restartServer", () => restartServer(context)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration("qsystems.path")) restartServer(context);
    })
  );
  await startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
