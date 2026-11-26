// Builds everything the extension ships:  node build.mjs
//   server/qsys.wasm   qsys, compiled for WASI: it runs treatments, and
//                         with the argument `lsp` it is the language server
//   dist/extension.js     the extension and its one dependency, bundled
import { execSync } from "node:child_process";
import { copyFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { build } from "esbuild";

// fileURLToPath, not .pathname, so a folder name with a space or a dash
// that is not ASCII still works.
const here = fileURLToPath(new URL(".", import.meta.url));
const root = fileURLToPath(new URL("../..", import.meta.url));
const release = `${root}target/wasm32-wasip1/release`;

execSync("cargo build --release --target wasm32-wasip1", { cwd: root, stdio: "inherit" });
copyFileSync(`${release}/qsys.wasm`, `${here}server/qsys.wasm`);

await build({
  entryPoints: [`${here}src/extension.js`],
  outfile: `${here}dist/extension.js`,
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node18",
  external: ["vscode"],
  minify: true,
  logLevel: "warning",
});
console.log("server/qsys.wasm and dist/extension.js written");
