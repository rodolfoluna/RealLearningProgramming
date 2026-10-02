// Escanea el QR del grupo con la cámara (versión web). Safari no tiene BarcodeDetector, así que se
// decodifica con jsQR sobre los cuadros del video.

import jsQR from "jsqr";

const ESTILO = `
.rlp-qr { position: fixed; inset: 0; z-index: 1000; display: flex; flex-direction: column;
  align-items: center; justify-content: center; gap: 1em; background: rgb(0 0 0 / 0.85); color: #fff;
  padding: 1em; text-align: center; }
.rlp-qr video { width: min(90vw, 480px); max-height: 60vh; border-radius: 12px; background: #000; }
.rlp-qr button { font: inherit; padding: 0.6em 1.4em; border-radius: 8px; border: 0; }
`;

export async function escanearQr(): Promise<string | null> {
  let flujo: MediaStream;
  try {
    flujo = await navigator.mediaDevices.getUserMedia({ video: { facingMode: "environment" }, audio: false });
  } catch {
    throw new Error("Permite el uso de la cámara para escanear el código del grupo.");
  }
  const capa = document.createElement("div");
  capa.className = "rlp-qr";
  capa.setAttribute("role", "dialog");
  capa.setAttribute("aria-label", "Escanear el QR del grupo");
  capa.innerHTML = `<style>${ESTILO}</style><p>Apunta la cámara al código QR que muestra tu profesor.</p>
    <video playsinline muted></video><button type="button">Cancelar</button>`;
  document.body.append(capa);
  const video = capa.querySelector("video")!;
  video.srcObject = flujo;
  await video.play().catch(() => undefined);
  const lienzo = document.createElement("canvas");
  const ctx = lienzo.getContext("2d", { willReadFrequently: true })!;

  return new Promise((resolver) => {
    let activo = true;
    const terminar = (texto: string | null) => {
      if (!activo) return;
      activo = false;
      flujo.getTracks().forEach((t) => t.stop());
      capa.remove();
      resolver(texto);
    };
    capa.querySelector("button")!.addEventListener("click", () => terminar(null));
    const cuadro = () => {
      if (!activo) return;
      if (video.readyState >= video.HAVE_ENOUGH_DATA && video.videoWidth) {
        lienzo.width = video.videoWidth;
        lienzo.height = video.videoHeight;
        ctx.drawImage(video, 0, 0);
        const imagen = ctx.getImageData(0, 0, lienzo.width, lienzo.height);
        const codigo = jsQR(imagen.data, imagen.width, imagen.height, { inversionAttempts: "dontInvert" });
        if (codigo?.data) return terminar(codigo.data);
      }
      requestAnimationFrame(cuadro);
    };
    requestAnimationFrame(cuadro);
  });
}
