// Runs every example through the runner the "Run" command uses, and compares
// what it prints with the example's .out file and, when it has been built,
// with what the native qsys prints.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const runner = fileURLToPath(new URL("../server/run.js", import.meta.url));
const examples = fileURLToPath(new URL("../../../examples/", import.meta.url));
const native = fileURLToPath(new URL("../../../target/release/qsys", import.meta.url));

function run(program, data, options = []) {
  const dir = mkdtempSync(join(tmpdir(), "qsys-"));
  writeFileSync(join(dir, "text.q"), program);
  writeFileSync(join(dir, "text.dat"), data);
  return spawnSync(process.execPath, [runner, ...options, join(dir, "text.q"), join(dir, "text.dat")]);
}

test("the AAABBBCCC system of C70 p. 27", () => {
  const r = run("A* + A*(U*) == A*(1,U*) / A* -DANS- A,B,C.\nA(U*) + B(U*) + C(U*) == S(1,U*).\n", "-0- A + A + A + B + B + B + C + C + C -1-\n");
  assert.equal(r.status, 0, r.stderr.toString());
  assert.match(r.stdout.toString(), /PHASE 1\n\n-01- S\(1,1,1\) -02-/);
});

test("--det prints every application", () => {
  const r = run("A + B == C.\n", "-0- A + B -1-\n", ["--det"]);
  assert.match(r.stdout.toString(), /rule 1 \(line 1\) A \+ B == C/);
});

test("a program not in the grammar stops with its place", () => {
  const r = run("A + B == C\n", "-0- A -1-\n");
  assert.equal(r.status, 1);
  assert.match(r.stderr.toString(), /line 2, column 1: '\.' expected/);
});

test("every example prints its .out file", () => {
  for (const f of readdirSync(examples).filter((n) => n.endsWith(".q"))) {
    const program = readFileSync(examples + f, "utf8");
    const data = readFileSync(examples + f.replace(/\.q$/, ".dat"), "utf8");
    const want = readFileSync(examples + f.replace(/\.q$/, ".out"), "utf8");
    assert.equal(run(program, data).stdout.toString(), want, f);
  }
});

test("the runner agrees with the native qsys", { skip: !existsSync(native) && "no native build" }, () => {
  const f = "paraphrase";
  const mine = run(readFileSync(`${examples}${f}.q`, "utf8"), readFileSync(`${examples}${f}.dat`, "utf8"), ["--det"]);
  const theirs = spawnSync(native, ["--det", `${examples}${f}.q`, `${examples}${f}.dat`]);
  assert.equal(mine.stdout.toString(), theirs.stdout.toString());
});
