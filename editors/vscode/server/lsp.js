// Starts the language server: qsys compiled for WASI, run as `qsys lsp`,
// talking the Language Server Protocol on standard input and output.
"use strict";

const path = require("path");
const { run } = require("./wasi.js");

process.exitCode = run(path.join(__dirname, "qsys.wasm"), { args: ["lsp"] });
