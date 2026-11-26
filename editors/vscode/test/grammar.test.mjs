// Tokenises the examples with the same TextMate engine VS Code uses and
// checks the scopes the grammar assigns.
import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import { fileURLToPath } from "node:url";
import oniguruma from "vscode-oniguruma";
import textmate from "vscode-textmate";

const require = createRequire(import.meta.url);
const grammarPath = fileURLToPath(new URL("../syntaxes/qsystems.tmLanguage.json", import.meta.url));
const examples = fileURLToPath(new URL("../../../examples/", import.meta.url));

const wasm = readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
await oniguruma.loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
const registry = new textmate.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (text) => new oniguruma.OnigString(text),
  }),
  loadGrammar: async () => textmate.parseRawGrammar(readFileSync(grammarPath, "utf8"), grammarPath),
});
const grammar = await registry.loadGrammar("source.qsystems");

/** Each non-blank piece of text with its most specific scope. */
function scopes(source) {
  const out = [];
  let state = textmate.INITIAL;
  for (const line of source.split("\n")) {
    const r = grammar.tokenizeLine(line, state);
    for (const t of r.tokens) {
      const text = line.slice(t.startIndex, t.endIndex);
      if (text.trim()) out.push([text.trim(), t.scopes[t.scopes.length - 1]]);
    }
    state = r.ruleStack;
  }
  return out;
}
const scopeOf = (source, piece) => scopes(source).find(([t]) => t === piece)?.[1];

test("comments, phases, relations, parameters and operators", () => {
  const text = "-INV- ** FORME ACTIVE.\nPI(V(A*),I*,J*) == SUJ + I*.\n-- == N(A*) / A* -DANS- B,C -ET- -NON- U* = -NUL-.\n";
  assert.match(scopeOf(text, "FORME ACTIVE"), /^comment/);
  assert.match(scopeOf(text, "-INV-"), /^keyword.control/);
  assert.match(scopeOf(text, "-DANS-"), /^keyword.operator.word/);
  assert.match(scopeOf(text, "-NUL-"), /^constant.language/);
  assert.match(scopeOf(text, "A*"), /^variable.parameter/);
  assert.match(scopeOf(text, "=="), /^keyword.operator.rewrite/);
  assert.match(scopeOf(text, "--"), /^keyword.operator.ditto/);
  assert.match(scopeOf(text, "SUJ"), /^entity.name.tag/);
});

test("every example tokenises, and its comments are comments", () => {
  for (const f of readdirSync(examples).filter((n) => n.endsWith(".q"))) {
    const text = readFileSync(examples + f, "utf8");
    const first = scopes(text)[0];
    assert.match(first[1], /^(comment|keyword)/, `${f}: ${first}`);
  }
});
