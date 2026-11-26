// Runs every example through the WebAssembly build and compares it with the
// native build:  node web/test.mjs   (after node web/build.mjs and cargo build --release)
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { qsys } from "./worker.js";

const root = fileURLToPath(new URL("..", import.meta.url));
const module = await WebAssembly.compile(readFileSync(`${root}web/qsys.wasm`));
const native = `${root}target/release/qsys`;
let failed = 0;
const fail = (m) => (failed++, console.log("FAIL " + m));
const examples = JSON.parse(readFileSync(`${root}web/examples.json`, "utf8"));
for (const ex of examples) {
  const args = ["json", "--max-arrows", "20000", "--data", ex.data, "-"];
  const wasm = qsys(module, args, ex.program);
  const want = execFileSync(native, args, { input: ex.program, maxBuffer: 64 << 20 }).toString();
  if (wasm.out !== want) fail(`${ex.name}: the wasm build and the native build differ`);
  const run = JSON.parse(wasm.out);
  if (run.error) fail(`${ex.name}: ${run.error.message}`);
  if (!run.graphs.length || !run.graphs[0].phases.length) fail(`${ex.name}: no phases`);
}
// the paraphrase treatment: phase 2 is the graph of C70 pp. 38-40
const para = examples.find((e) => e.name === "paraphrase");
const r = JSON.parse(qsys(module, ["json", "--data", para.data, "-"], para.program).out);
const want = readFileSync(`${root}tests/c70/paraphrase-phase2.txt`, "utf8");
if (r.graphs[0].phases[1].printed !== want) fail("paraphrase: phase 2 is not the graph of pp. 38-40");
// the printout of a notebook cell agrees with the .out file
const abc = examples.find((e) => e.name === "abc");
const cell = qsys(module, ["--data", abc.data, "-"], abc.program).out;
if (cell !== readFileSync(`${root}examples/abc.out`, "utf8")) fail("abc: the cell printout differs from examples/abc.out");
// the language server answers in the page as it does in an editor
import { answer } from "./lsp-worker.js";
const line = para.program.split("\n").findIndex((l) => l.startsWith("SUJ + PH(U*)"));
const lsp = answer(module, para.program, [
  { method: "textDocument/semanticTokens/full" },
  { method: "textDocument/hover", params: { position: { line, character: 2 } } },
]);
if (!lsp.results[0]?.data?.length) fail("lsp: no semantic tokens");
if (!/Phase 1, rule 4/.test(lsp.results[1]?.contents?.value ?? "")) fail("lsp: hover on a rule of the -INV- phase");
if (lsp.diagnostics.length) fail("lsp: problems reported in the paraphrase treatment");
const bad = answer(module, "-1- B -2-\n-0- A -1-\n", [], "file:///text.dat");
if (bad.diagnostics.length !== 1) fail("lsp: the data's order of paths is not checked");
console.log(`${examples.length} texts through the wasm build, ${failed ? failed + " failed" : "all agree with the native build"}`);
process.exit(failed ? 1 : 0);
