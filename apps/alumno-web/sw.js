// Service worker de LP Alumno (versión web). Esta es la plantilla: al compilar, vite.config.ts
// la copia a dist/sw.js con la versión y la lista de archivos de esa compilación.
//
// - Guarda la app y Pyodide para usarla sin conexión (Pyodide en una caché aparte, que se conserva
//   entre versiones de la app mientras Pyodide no cambie).
// - Agrega COOP/COEP a cada respuesta: la página queda aislada (SharedArrayBuffer, que necesita
//   input()) aunque el hosting no permita encabezados, como GitHub Pages.
// - La primera instalación se activa sola; una versión nueva espera a que la página la active
//   (botón "Actualizar"), para no cambiar la app a mitad de un ejercicio.

const VERSION = "__VERSION__";
const CACHE_APP = `rlp-app-${VERSION}`;
const CACHE_PYODIDE = "rlp-pyodide-__VERSION_PYODIDE__";
const ARCHIVOS_APP = __ARCHIVOS_APP__;
const ARCHIVOS_PYODIDE = __ARCHIVOS_PYODIDE__;

self.addEventListener("install", (evento) => {
  evento.waitUntil(
    (async () => {
      const cache = await caches.open(CACHE_APP);
      await cache.addAll(ARCHIVOS_APP.map((ruta) => new Request(ruta, { cache: "reload" })));
      if (!self.registration.active) await self.skipWaiting();
    })(),
  );
});

self.addEventListener("activate", (evento) => {
  evento.waitUntil(
    (async () => {
      for (const nombre of await caches.keys()) {
        if (nombre.startsWith("rlp-") && nombre !== CACHE_APP && nombre !== CACHE_PYODIDE) await caches.delete(nombre);
      }
      await self.clients.claim();
    })(),
  );
});

self.addEventListener("message", (evento) => {
  if (evento.data?.tipo === "activar") void self.skipWaiting();
  if (evento.data?.tipo === "precargar") evento.waitUntil(precargarPyodide(evento.source));
});

/** Descarga Pyodide a su caché (en segundo plano) y avisa el avance a la página. */
async function precargarPyodide(cliente) {
  const cache = await caches.open(CACHE_PYODIDE);
  let hechos = 0;
  for (const ruta of ARCHIVOS_PYODIDE) {
    if (!(await cache.match(ruta))) {
      const respuesta = await fetch(ruta, { cache: "reload" });
      if (!respuesta.ok) throw new Error(`No se pudo descargar ${ruta}`);
      await cache.put(ruta, respuesta);
    }
    hechos++;
    cliente?.postMessage({ tipo: "precarga", hechos, total: ARCHIVOS_PYODIDE.length });
  }
}

/** Copia la respuesta con los encabezados que aíslan la página. */
function aislar(respuesta) {
  if (respuesta.status === 0 || respuesta.type === "opaqueredirect") return respuesta;
  const encabezados = new Headers(respuesta.headers);
  encabezados.set("Cross-Origin-Opener-Policy", "same-origin");
  encabezados.set("Cross-Origin-Embedder-Policy", "require-corp");
  encabezados.set("Cross-Origin-Resource-Policy", "same-origin");
  return new Response(respuesta.body, {
    status: respuesta.status,
    statusText: respuesta.statusText,
    headers: encabezados,
  });
}

self.addEventListener("fetch", (evento) => {
  const pedido = evento.request;
  const url = new URL(pedido.url);
  if (pedido.method !== "GET" || url.origin !== location.origin) return;
  evento.respondWith(
    (async () => {
      let respuesta = await caches.match(pedido, { ignoreSearch: true });
      // La página se pide como "./" (o con otra ruta): siempre es index.html.
      if (!respuesta && pedido.mode === "navigate") {
        respuesta = await caches.match(new URL("index.html", self.registration.scope).href);
      }
      if (!respuesta) {
        respuesta = await fetch(pedido);
        if (respuesta.ok && url.pathname.includes("/pyodide/")) {
          const cache = await caches.open(CACHE_PYODIDE);
          await cache.put(pedido, respuesta.clone());
        }
      }
      return aislar(respuesta);
    })(),
  );
});
