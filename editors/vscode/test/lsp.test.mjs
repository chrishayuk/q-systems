// Talks to the language server the way an editor does: over standard input
// and output. It tests the WebAssembly build the extension ships and, when
// it has been built, the native program other editors would use.
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL("..", import.meta.url));
const native = fileURLToPath(new URL("../../../target/release/qsys", import.meta.url));
const servers = [["WebAssembly", process.execPath, [`${here}server/lsp.js`, "--stdio"]]];
if (existsSync(native)) servers.push(["native", native, ["lsp"]]);

function connect(command, args) {
  const child = spawn(command, args, { stdio: ["pipe", "pipe", "inherit"] });
  let buffer = Buffer.alloc(0);
  const waiting = [];
  const arrived = [];
  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, chunk]);
    for (;;) {
      const header = buffer.indexOf("\r\n\r\n");
      if (header === -1) return;
      const length = Number(/Content-Length: (\d+)/i.exec(buffer.subarray(0, header).toString())[1]);
      if (buffer.length < header + 4 + length) return;
      const message = JSON.parse(buffer.subarray(header + 4, header + 4 + length).toString());
      buffer = buffer.subarray(header + 4 + length);
      const next = waiting.shift();
      if (next) next(message);
      else arrived.push(message);
    }
  });
  const send = (message) => {
    const body = JSON.stringify({ jsonrpc: "2.0", ...message });
    child.stdin.write(`Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`);
  };
  const receive = () => new Promise((resolve) => (arrived.length ? resolve(arrived.shift()) : waiting.push(resolve)));
  let id = 0;
  const request = async (method, params) => {
    send({ id: ++id, method, params });
    const reply = await receive();
    assert.equal(reply.id, id);
    return reply.result;
  };
  const exited = new Promise((resolve) => child.on("exit", resolve));
  return { send, receive, request, exited, kill: () => child.kill() };
}


const PROGRAM = `** C70 PAGE 27.
A* + A*(U*) == A*(1,U*) / A* -DANS- A,B,C.
A(U*) + B(U*) + C(U*) == S(1,U*).
-REQ- -INV-
-- == B(I*).
`;

/** Semantic tokens as [line, column, length, type name]. */
function decode(data, legend) {
  const out = [];
  let line = 0;
  let column = 0;
  for (let i = 0; i < data.length; i += 5) {
    line += data[i];
    column = data[i] ? data[i + 1] : column + data[i + 1];
    out.push([line, column, data[i + 2], legend[data[i + 3]]]);
  }
  return out;
}

for (const [kind, command, args] of servers) {
  test(`the ${kind} language server`, async () => {
    const s = connect(command, args);
    try {
      const init = await s.request("initialize", { capabilities: {} });
      const legend = init.capabilities.semanticTokensProvider.legend.tokenTypes;
      assert.equal(init.serverInfo.name, "qsys");
      s.send({ method: "initialized", params: {} });

      // the first rule of the second phase is "--" with nothing before it in that phase
      const uri = "file:///abc.q";
      s.send({ method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "qsystems", version: 1, text: PROGRAM } } });
      const problems = (await s.receive()).params.diagnostics;
      assert.equal(problems.length, 1, JSON.stringify(problems));
      assert.equal(problems[0].range.start.line, 4);
      assert.match(problems[0].message, /first rule of a system cannot contain --/);

      const fixed = PROGRAM.replace("-- == B(I*).", "S(1,U*) == A(U*) + B(U*) + C(U*).");
      s.send({ method: "textDocument/didChange", params: { textDocument: { uri, version: 2 }, contentChanges: [{ text: fixed }] } });
      assert.equal((await s.receive()).params.diagnostics.length, 0);

      const hover = await s.request("textDocument/hover", { textDocument: { uri }, position: { line: 4, character: 3 } });
      assert.match(hover.contents.value, /Phase 2, rule 1/);
      assert.match(hover.contents.value, /A\(U\*\) \+ B\(U\*\) \+ C\(U\*\) == S\(1,U\*\)/, "shown inverted");
      const onParam = await s.request("textDocument/hover", { textDocument: { uri }, position: { line: 1, character: 0 } });
      assert.match(onParam.contents.value, /label parameter/);

      const tokens = decode((await s.request("textDocument/semanticTokens/full", { textDocument: { uri } })).data, legend);
      assert.deepEqual(tokens[0], [0, 0, 15, "comment"]);
      assert.deepEqual(tokens[1], [1, 0, 2, "parameter"]);
      assert.ok(tokens.some((t) => t[3] === "keyword") && tokens.some((t) => t[3] === "macro"));
      const symbols = await s.request("textDocument/documentSymbol", { textDocument: { uri } });
      assert.deepEqual(symbols.map((x) => x.name), ["phase 1", "phase 2"]);
      assert.equal(symbols[0].children.length, 2);
      const items = await s.request("textDocument/completion", { textDocument: { uri }, position: { line: 2, character: 0 } });
      assert.ok(items.some((i) => i.label === "-INV-") && items.some((i) => i.label === "-DANS-"));

      // data: a path from a vertex written before the path into it
      const data = "file:///abc.dat";
      s.send({ method: "textDocument/didOpen", params: { textDocument: { uri: data, languageId: "qsystems-data", version: 1, text: "-1- B -2-\n-0- A -1-\n" } } });
      const p = (await s.receive()).params.diagnostics;
      assert.equal(p.length, 1);
      assert.match(p[0].message, /written before/);

      await s.request("shutdown");
      s.send({ method: "exit" });
      assert.equal(await s.exited, 0);
    } catch (e) {
      s.kill();
      throw e;
    }
  });
}
