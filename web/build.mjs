// Builds the files the playground needs:  node web/build.mjs
//   web/qsys.wasm       qsys, compiled for WASI
//   web/examples.json   the examples and the research texts, each a program and its data
import { execSync } from "node:child_process";
import { copyFileSync, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
copyFileSync(`${root}target/wasm32-wasip1/release/qsys.wasm`, `${root}web/qsys.wasm`);

// The order the playground lists them in: C70's examples in the order of the report.
const first = ["abc", "abc-inverse", "mot", "unstructure", "structure", "culdesac", "paraphrase"];
const read = (dir) =>
  readdirSync(`${root}${dir}`)
    .filter((f) => f.endsWith(".q") && existsSync(`${root}${dir}/${f.replace(/\.q$/, ".dat")}`))
    .map((f) => {
      const program = readFileSync(`${root}${dir}/${f}`, "utf8");
      const data = readFileSync(`${root}${dir}/${f.replace(/\.q$/, ".dat")}`, "utf8");
      const name = f.replace(/\.q$/, "");
      // the first comment that says where the text comes from, else the first
      const comments = [...program.matchAll(/\*\*\s*([^.]*)\./g)].map((m) => m[1].replace(/\s+/g, " ").trim());
      const title = comments.find((c) => /^QSYS/.test(c)) || comments.find((c) => /^C70/.test(c)) || comments[0] || name;
      return { name, file: `${dir}/${f}`, title, program, data, group: dir };
    });
const examples = read("examples").sort((a, b) => {
  const i = (x) => first.indexOf(x.name) + 1 || 99;
  return i(a) - i(b) || a.name.localeCompare(b.name);
});
const all = [...examples, ...read("research").sort((a, b) => a.name.localeCompare(b.name))];
writeFileSync(`${root}web/examples.json`, JSON.stringify(all));
console.log(`web/qsys.wasm, web/examples.json (${all.length} texts) written`);
