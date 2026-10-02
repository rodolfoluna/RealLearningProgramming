// Mientras un diálogo nativo (elegir archivo/carpeta) está abierto la ventana pierde el foco;
// esas "salidas" no deben contarse.
let abiertos = 0;
let hasta = 0;

export function enDialogoNativo(): boolean {
  return abiertos > 0 || Date.now() < hasta;
}

export async function conDialogo<T>(fn: () => Promise<T>): Promise<T> {
  abiertos++;
  try {
    return await fn();
  } finally {
    abiertos--;
    hasta = Date.now() + 800;
  }
}
