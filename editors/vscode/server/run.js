// Runs a treatment on its data:  node run.js [options] program.q data.dat
// The two files are read here and given to qsys as the only files it can
// open, under their own names, so qsys needs no access to the disk. The
// options are qsys's own (--det, --width N, --max-arrows N).
"use strict";

const fs = require("fs");
const path = require("path");
const { run } = require("./wasi.js");

const wasm = path.join(__dirname, "qsys.wasm");
const args = process.argv.slice(2);
const data = args.pop();
const program = args.pop();
if (!program || !data || program.startsWith("--") || data.startsWith("--")) {
  console.error("usage: run.js [options] program.q data.dat");
  process.exit(2);
}
const files = {};
const names = [];
for (const file of [program, data]) {
  try {
    const base = path.basename(file);
    // two files of one name, in different folders
    const key = files[base] ? `2-${base}` : base;
    files[key] = fs.readFileSync(file);
    // a name that starts with a dash would be read as an option
    names.push(key.startsWith("-") ? `./${key}` : key);
  } catch (error) {
    console.error(`qsys: cannot read ${file}: ${error.message}`);
    process.exit(2);
  }
}
process.exitCode = run(wasm, { args: [...args, ...names], files, keyboard: false });
