// Tipos que devuelve el núcleo en Rust (crates/rlp-core).

export type Nivel = "verde" | "amarillo" | "rojo";

export interface EstadoApp {
  version: string;
  carpeta: string;
  escribible: boolean;
  tiene_identidad: boolean;
  desbloqueado: boolean;
  nombre: string | null;
  llave_cifrado: string | null;
  autoprueba?: string | null;
}

export interface Politicas {
  pegado: "bloquear" | "propio";
  registrar_salidas: boolean;
}

export interface GrupoInfo {
  grupo_id: string;
  nombre: string;
  materia: string;
  periodo: string;
  profesor: string;
  llaves_cifrado: string[];
  politicas: Politicas;
  regex_control: string;
  creado: number;
}

export interface NuevoGrupo {
  nombre: string;
  materia: string;
  periodo: string;
  regex_control: string;
  politicas: Politicas;
  coprofesores: string[];
}

export interface Contadores {
  tiempo_ms: number;
  ejecuciones: number;
  errores: number;
  pruebas: number;
  pruebas_exitosas: number;
  copias: number;
  pegados_intentos: number;
  pegados_permitidos: number;
  inserciones_sospechosas: number;
  salidas: number;
  tiempo_fuera_ms: number;
  pistas: number;
  teclas: number;
  ejecutables: number;
}

export interface ResumenActividad {
  completada: boolean;
  pasadas: number;
  total: number;
  puntos: number;
  intentos: number;
  pistas: number;
  con_codigo: boolean;
}

export interface FilaTablero {
  entrega_id: number;
  perfil_id: string;
  grupo_id: string | null;
  numero_control: string;
  nombre: string;
  recibido: number;
  creado: number;
  nivel: Nivel;
  entregas: number;
  global: Contadores;
  actividades: Record<string, ResumenActividad>;
  alerta_identidad: boolean;
}

export interface Check {
  id: string;
  nombre: string;
  nivel: Nivel;
  detalle: string;
}

export interface Ritmo {
  tecleados: number;
  automaticos: number;
  deshacer_rehacer: number;
  pegados: number;
  otros: number;
  max_cps: number;
  rafagas: number;
}

export interface Reporte {
  nivel: Nivel;
  checks: Check[];
  ritmo: Ritmo;
  ritmo_por_actividad: Record<string, Ritmo>;
}

export interface EstadoActividad {
  codigo: string;
  completada: boolean;
  pasadas: number;
  total: number;
  puntos: number;
  intentos: number;
  respuesta: string | null;
  pistas: number;
  actualizado: number;
}

export interface Calificacion {
  calificacion: number | null;
  comentario: string;
  actualizado: number;
}

export interface EntradaHistorial {
  entrega_id: number;
  recibido: number;
  creado: number;
  nivel: Nivel;
}

export interface DetalleEntrega {
  fila: FilaTablero;
  manifiesto: { creado: number; app: { version: string; dev: boolean }; cabezas: Record<string, { seq: number }> };
  perfil: { perfil_id: string; numero_control: string; nombre: string; creado: number } | null;
  actividades: Record<string, EstadoActividad>;
  reporte: Reporte;
  estadisticas: { global: Contadores; por_actividad: Record<string, Contadores> };
  calificaciones: Record<string, Calificacion>;
  historial: EntradaHistorial[];
}

export interface RegistroEntrega {
  entrega_id: number;
  nueva: boolean;
  nombre: string;
  numero_control: string;
  nivel: Nivel;
}

export interface ResultadoImportacion {
  archivo: string;
  error: string | null;
  registro: RegistroEntrega | null;
}

/** Archivo creado por la App Profesor (.rlpr o .rlpa). */
export interface ArchivoCreado {
  ruta: string;
  alumnos: number;
  temporal: string | null;
}

/** Operación de edición con su momento: [t (epoch ms), desde, hasta, insertado, origen]. */
export type OpConTiempo = [number, number, number, string, string];

export interface Tramo {
  dispositivo: string;
  motivo: "inicio" | "reinicio" | "continuacion" | string;
  texto_inicial: string | null;
  ops: OpConTiempo[];
  t_inicio: number;
  t_fin: number;
}

export interface Marca {
  t: number;
  tipo: string;
  dispositivo: string;
  datos: Record<string, unknown> | null;
}

export interface LineaDeTiempo {
  actividad: string;
  tramos: Tramo[];
  marcas: Marca[];
  codigo_final: string;
  avisos: string[];
}

export function contadoresVacios(): Contadores {
  return {
    tiempo_ms: 0, ejecuciones: 0, errores: 0, pruebas: 0, pruebas_exitosas: 0, copias: 0, pegados_intentos: 0,
    pegados_permitidos: 0, inserciones_sospechosas: 0, salidas: 0, tiempo_fuera_ms: 0, pistas: 0, teclas: 0, ejecutables: 0,
  };
}
