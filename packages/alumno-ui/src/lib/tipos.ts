// Tipos que devuelve el núcleo en Rust (ver crates/rlp-core).

export interface PerfilPublico {
  perfil_id: string;
  numero_control: string;
  nombre: string;
  creado: number;
}

export interface PerfilLocal {
  perfil: PerfilPublico;
  carpeta: string;
  grupo: string | null;
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
  politicas: Politicas;
  regex_control: string;
  creado: number;
}

export interface EstadoApp {
  version: string;
  plataforma: string;
  carpeta_datos: string;
  escribible: boolean;
  grupo: GrupoInfo | null;
  perfiles: PerfilLocal[];
  puede_generar_exe: boolean;
  dev: boolean;
  autoprueba?: string | null;
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
  dispositivo: string;
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

export interface Estadisticas {
  global: Contadores;
  por_actividad: Record<string, Contadores>;
}

export interface NotaActividad {
  calificacion: number | null;
  comentario: string;
  actualizado: number;
}

/** Retroalimentación firmada del profesor (archivo .rlpr). */
export interface Retroalimentacion {
  profesor: string;
  creado: number;
  actividades: Record<string, NotaActividad>;
}

export interface EstadoAlumno {
  perfil: PerfilPublico;
  dispositivo: string;
  grupo: GrupoInfo | null;
  actividades: Record<string, EstadoActividad>;
  estadisticas: Estadisticas;
  retroalimentacion?: Retroalimentacion | null;
  /** Código de recuperación nuevo (tras entrar con un archivo de acceso); se muestra una vez. */
  codigo_nuevo?: string;
}

/** Datos de un archivo de acceso del profesor (.rlpa). */
export interface InfoAcceso {
  nombre: string;
  numero_control: string;
  profesor: string;
  perfil_local: boolean;
}

export interface ResultadoGuardado {
  ok: boolean;
  codigo: string;
}

export interface ResumenImportacion {
  eventos_nuevos: number;
  actividades_actualizadas: string[];
  dispositivos: string[];
  conflictos: string[];
}

export type Secreto =
  | { tipo: "contrasena"; contrasena: string }
  | { tipo: "codigo"; codigo: string; nueva_contrasena: string }
  | { tipo: "acceso"; archivo: string; temporal: string; nueva_contrasena: string };

export function contadoresVacios(): Contadores {
  return {
    tiempo_ms: 0,
    ejecuciones: 0,
    errores: 0,
    pruebas: 0,
    pruebas_exitosas: 0,
    copias: 0,
    pegados_intentos: 0,
    pegados_permitidos: 0,
    inserciones_sospechosas: 0,
    salidas: 0,
    tiempo_fuera_ms: 0,
    pistas: 0,
    teclas: 0,
    ejecutables: 0,
  };
}
