/** Error de Python descrito en español por harness.py. */
export interface ErrorPython {
  tipo: string;
  mensaje: string;
  linea: number | null;
  columna: number | null;
  texto_linea: string;
  explicacion: string;
  traza: string[];
}

export type EstadoEjecucion = "ok" | "error" | "salida" | "detenido";

export interface ResultadoEjecucion {
  estado: EstadoEjecucion;
  error: ErrorPython | null;
  duracion_ms: number;
  /** El programa imprimió más de lo permitido y se detuvo. */
  salida_excedida?: boolean;
}

/** Prueba de una actividad (formato de actividad.yaml). */
export interface Prueba {
  nombre?: string;
  oculta?: boolean;
  // Prueba de entrada/salida (en pruebas de función: datos para los input() de la función)
  entrada?: string;
  /** Lo que el programa (o la función) debe mostrar. */
  salida?: string | string[];
  modo?: "contiene" | "exacta" | "normalizada" | "regex" | "termina";
  // Prueba de función
  funcion?: string;
  args?: unknown[];
  /** Argumentos con nombre. */
  kwargs?: Record<string, unknown>;
  /** Valor que debe devolver la función (sin él y con `salida`, no se revisa). */
  esperado?: unknown;
}

export interface ResultadoPrueba {
  indice: number;
  nombre: string;
  oculta: boolean;
  tipo: "io" | "funcion";
  paso: boolean;
  mensaje: string;
  tiempo_agotado?: boolean;
  error?: ErrorPython;
  // io
  entrada?: string;
  esperado?: string | string[];
  obtenido?: string;
  modo?: string;
  // funcion
  llamada?: string;
  salida_esperada?: string | string[];
  salida_obtenida?: string;
}

export interface ResultadoPruebas {
  pasadas: number;
  total: number;
  resultados: ResultadoPrueba[];
}

export interface EventosEjecucion {
  /** Texto que el programa escribió en stdout/stderr. */
  salida?: (texto: string, flujo: "out" | "err") => void;
  /** El programa ejecutó input() y espera una línea. */
  entradaSolicitada?: () => void;
  /** El programa pidió limpiar la pantalla (os.system("cls")). */
  limpiar?: () => void;
}

// ---------------------------------------------------------------- protocolo con el worker

export type MensajeAlWorker =
  | {
      tipo: "iniciar";
      indexURL: string;
      /** Memoria compartida (si el entorno la ofrece). */
      control?: SharedArrayBuffer;
      interrupcion?: SharedArrayBuffer;
      /** URL base del puente de entrada síncrona (sin memoria compartida). */
      puente?: string;
    }
  | { tipo: "ejecutar"; id: number; codigo: string }
  | { tipo: "probar"; id: number; codigo: string; pruebas: string }
  | { tipo: "sintaxis"; id: number; codigo: string };

export type MensajeDelWorker =
  | { tipo: "listo"; version: string }
  | { tipo: "error_inicio"; mensaje: string }
  | { tipo: "salida"; id: number; texto: string; flujo: "out" | "err" }
  | { tipo: "entrada"; id: number }
  | { tipo: "limpiar"; id: number }
  | { tipo: "prueba_inicio"; id: number; indice: number }
  | { tipo: "prueba_fin"; id: number; resultado: string }
  | { tipo: "fin"; id: number; resultado: string; salida_excedida?: boolean };

/** Distribución de la memoria compartida de control. */
export const CONTROL = {
  /** Int32 índice 0: estado de la entrada (0 esperando, 1 lista, 2 cancelada). */
  ESTADO_ENTRADA: 0,
  /** Int32 índice 1: longitud en bytes de la línea escrita. */
  LONGITUD: 1,
  /** Int32 índice 2: canal para dormir (time.sleep) interrumpible. */
  DORMIR: 2,
  /** Desplazamiento en bytes del búfer de datos. */
  DATOS: 16,
  TAMANO_DATOS: 64 * 1024,
} as const;
