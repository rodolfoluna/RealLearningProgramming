// Tipos que cruzan la frontera con el núcleo en WebAssembly (crates/rlp-web).

/** Evento del historial tal como se guarda: JSON cifrado (base64); hash y firma en claro. */
export interface FilaEvento {
  dispositivo: string;
  seq: number;
  datos: string;
  hash: string;
  firma: string;
  llave: string;
}

export interface FilaActividad {
  id: string;
  datos: string;
}

/** Cambios de un perfil (o su contenido completo: la "instantánea"). Todo va cifrado. */
export interface Lote {
  meta: Record<string, string>;
  actividades: FilaActividad[];
  eventos: FilaEvento[];
}

export interface Diario {
  perfil: string;
  lote: Lote;
}

/** Metadatos públicos de un perfil (meta "perfil"): nombre, número de control, envolturas… */
export interface MetaPerfil {
  perfil: { perfil_id: string; numero_control: string; nombre: string; creado: number };
  [clave: string]: unknown;
}

/** Grupo firmado por el profesor, con sus datos ya verificados. */
export interface GrupoInstalado {
  grupo: unknown;
  info: { grupo_id: string; nombre: string; [clave: string]: unknown };
}

/** El módulo de rlp-web: una sola función. */
export interface NucleoWasm {
  llamar(metodo: string, args: string): string;
}
