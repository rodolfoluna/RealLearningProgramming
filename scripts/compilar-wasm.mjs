// Compila el núcleo de la versión web (crates/rlp-web) a WebAssembly y genera su envoltura de
// JavaScript en packages/nucleo-web/generado/. La app nativa (Windows y Android) no lo necesita.
//
// Requisitos (una sola vez):
//   rustup target add wasm32-unknown-unknown
//   cargo install wasm-bindgen-cli --version <la del Cargo.lock>
// Con RLP_CLAVE_APP_WEB definida, las entregas se firman con la llave web de producción.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const raiz = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const destino = join(raiz, "packages/nucleo-web/generado");
const objetivo = "wasm32-unknown-unknown";

function salir(mensaje) {
  console.error(`\n${mensaje}\n`);
  process.exit(1);
}

function correr(programa, argumentos) {
  execFileSync(programa, argumentos, { cwd: raiz, stdio: "inherit" });
}

function salida(programa, argumentos) {
  try {
    return execFileSync(programa, argumentos, { cwd: raiz, encoding: "utf8" });
  } catch {
    return null;
  }
}

// La CLI de wasm-bindgen debe ser exactamente la versión de la biblioteca en Cargo.lock.
const lock = readFileSync(join(raiz, "Cargo.lock"), "utf8");
const version = /name = "wasm-bindgen"\nversion = "([^"]+)"/.exec(lock)?.[1];
if (!version) salir("No encontré wasm-bindgen en Cargo.lock.");
const cli = salida("wasm-bindgen", ["--version"])?.trim();
if (!cli) salir(`Falta wasm-bindgen-cli. Instálalo con:\n  cargo install wasm-bindgen-cli --version ${version} --locked`);
if (!cli.endsWith(` ${version}`)) {
  salir(`Tu ${cli} no coincide con la biblioteca (${version}). Instala la misma versión:\n  cargo install wasm-bindgen-cli --version ${version} --locked --force`);
}
const targets = salida("rustup", ["target", "list", "--installed"]);
if (targets !== null && !targets.split(/\r?\n/).includes(objetivo)) {
  salir(`Falta el target de Rust. Instálalo con:\n  rustup target add ${objetivo}`);
}
if (!process.env.RLP_CLAVE_APP_WEB) {
  console.warn("Sin RLP_CLAVE_APP_WEB: las entregas de la versión web saldrán con la firma de desarrollo.");
}

correr("cargo", ["build", "-p", "rlp-web", "--target", objetivo, "--release"]);
rmSync(destino, { recursive: true, force: true });
mkdirSync(destino, { recursive: true });
const wasm = join(raiz, "target", objetivo, "release", "rlp_web.wasm");
correr("wasm-bindgen", ["--target", "web", "--out-dir", destino, "--out-name", "rlp_web", wasm]);

const final = join(destino, "rlp_web_bg.wasm");
const mb = (n) => `${(n / 1048576).toFixed(2)} MB`;
const bytes = readFileSync(final);
console.log(`Núcleo web → packages/nucleo-web/generado (${mb(statSync(final).size)}, ${mb(gzipSync(bytes).length)} con gzip)`);
