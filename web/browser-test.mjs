// Drives the playground in headless Chrome over the DevTools protocol:
// loads examples, steps through their phases, and reads the drawn graph.
//   node web/browser-test.mjs [screenshot.png [example:phase:step]]   (needs Chrome and python3)
import { spawn } from "node:child_process";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";

const root = new URL("..", import.meta.url).pathname;
const chromePath = process.env.CHROME || "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const [port, debug] = [8782, 9344];
const server = spawn("python3", ["-m", "http.server", "-d", root + "web", "--bind", "127.0.0.1", String(port)], { stdio: "ignore" });
const chrome = spawn(chromePath, ["--headless=new", "--disable-gpu", `--remote-debugging-port=${debug}`,
  `--user-data-dir=${mkdtempSync(tmpdir() + "/qsys-chrome-")}`, "--window-size=1440,1400", "about:blank"], { stdio: "ignore" });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const quit = (code) => (server.kill(), chrome.kill(), process.exit(code));

let target;
for (let i = 0; i < 50 && !target; i++) {
  await sleep(200);
  try {
    target = (await (await fetch(`http://127.0.0.1:${debug}/json`)).json()).find((t) => t.type === "page");
  } catch {}
}
if (!target) (console.log("Chrome did not start"), quit(2));
const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((r) => (socket.onopen = r));
let id = 0;
const waiting = new Map();
const errors = [];
socket.onmessage = (event) => {
  const m = JSON.parse(event.data);
  if (m.id && waiting.has(m.id)) waiting.get(m.id)(m.result ?? m);
  if (m.method === "Runtime.exceptionThrown") errors.push(m.params.exceptionDetails.exception?.description ?? m.params.exceptionDetails.text);
};
const send = (method, params = {}) => new Promise((r) => (waiting.set(++id, r), socket.send(JSON.stringify({ id, method, params }))));
const js = async (expression) => {
  const r = await send("Runtime.evaluate", { expression, awaitPromise: true, returnByValue: true });
  if (r.exceptionDetails) throw new Error(r.exceptionDetails.exception?.description ?? r.exceptionDetails.text);
  return r.result.value;
};
const until = async (expression, what) => {
  for (let i = 0; i < 100; i++) {
    if (await js(expression)) return;
    await sleep(100);
  }
  throw new Error(`timed out: ${what}`);
};
const open = async (example) => {
  await send("Page.navigate", { url: `http://127.0.0.1:${port}/index.html?example=${example}` });
  await until(`document.querySelector(".qs-tabs button") && !document.querySelector(".qs-run").disabled && document.querySelector(".qs-result").textContent.length > 0`, `${example} runs`);
};
const slide = (i) => js(`(() => { const s = document.querySelector(".qs-slider"); s.value = ${i}; s.dispatchEvent(new Event("input")); return s.max; })()`);
const visible = (cls = "") => js(`[...document.querySelectorAll(".qs-graph .qs-arrow${cls}")].filter((g) => !g.classList.contains("hidden")).map((g) => g.querySelector("title").textContent)`);
const phase = (k) => js(`document.querySelectorAll(".qs-tabs:not(.qs-graphs) button")[${k}].click()`);

let checked = 0, failed = 0;
const check = (ok, what) => (checked++, ok || (failed++, console.log(`FAIL ${what}`)));

try {
  await send("Runtime.enable");
  await send("Page.enable");

  // C70 p. 29: the first phase on AAABBBCCC, then the second
  await open("abc");
  const max = Number(await slide(0));
  check(max === 10 + 2, "abc: ten applications, and the two steps of the second phase");
  check((await visible()).length === 9, "abc: the data, nine arrows");
  await slide(10);
  const grown = await visible();
  check(grown.length === 19 && grown.filter((t) => t === "A(1)").length === 2 && grown.includes("S(1,1,1)"), "abc: the graph of p. 29 after the first phase");
  check((await visible(".match")).length > 0 && (await visible(".new")).join() === "S(1,1,1)", "abc: the last application matched a path and added S(1,1,1)");
  await slide(11);
  check((await visible(".rewritten")).length === 15 && (await visible(".stranded")).length === 3, "abc: 15 arrows rewritten, 3 stranded");
  await slide(12);
  check((await visible()).join() === "S(1,1,1)", "abc: only S(1,1,1) is left");
  check((await js(`document.querySelector(".qs-result").textContent`)).trim() === "-01- S(1,1,1) -02-", "abc: the printed result");

  // the paraphrase treatment: four phases, the second is pp. 38-40
  await open("paraphrase");
  check((await js(`document.querySelectorAll(".qs-tabs:not(.qs-graphs) button").length`)) === 4, "paraphrase: four phases");
  check((await js(`document.querySelector(".qs-result").textContent`)).startsWith("-01- PI(V(CONSTATE)"), "paraphrase: phase 1 analyses the sentence");
  await phase(1);
  check((await js(`document.querySelector(".qs-result").textContent.trim().split("\\n").length`)) === 35, "paraphrase: phase 2 prints 35 paths");
  check((await js(`document.querySelectorAll(".qs-graph .qs-arrow").length`)) === 330, "paraphrase: phase 2 draws its 330 arrows");

  // the non-converging rules of p. 30
  await open("structure");
  check(await js(`document.querySelector(".qs-tabs small.bad") !== null`), "structure: the phase is marked as not converging");

  // two graphs of data
  await open("e05-two-readings");
  const two = await visible();
  check(two.length === 2, `e05: two readings, two arrows: ${two.join(" | ")}`);

  // a notebook cell
  await js(`(async () => {
    const m = await import("./playground.js");
    const div = document.createElement("div");
    div.id = "cell";
    document.body.append(div);
    await m.mountRunner(div, { example: "abc-inverse", label: "the inverse" }).run();
  })()`);
  check(/A \+ A \+ A \+ B/.test(await js(`document.querySelector("#cell .qs-cell-out").textContent`)), "a cell runs the inverse system");
  await js(`document.querySelector("#cell").remove()`);

  // the screenshot: of example:phase:step, by default abc:0:10
  const [ex, ph, st] = (process.argv[3] || "abc:0:10").split(":");
  await open(ex);
  await phase(Number(ph));
  await slide(Number(st));
  if (process.argv[2]) {
    const shot = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: true });
    writeFileSync(process.argv[2], Buffer.from(shot.data, "base64"));
  }
  check(errors.length === 0, `no errors in the page: ${errors.join("; ")}`);
} catch (e) {
  failed++;
  console.log(`FAIL ${e.message}`);
}
console.log(`${checked} browser checks, ${failed ? failed + " failed" : "all passed"}`);
quit(failed ? 1 : 0);
