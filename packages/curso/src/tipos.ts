// Estructura del curso compilado (scripts/compilar-curso.mjs genera el JSON).

import type { Prueba } from "@rlp/python-worker";

export type TipoActividad = "codigo" | "prediccion" | "opcion_multiple";

export interface Opcion {
  texto: string;
  correcta: boolean;
  explicacion?: string;
}

export interface Actividad {
  id: string;
  titulo: string;
  tipo: TipoActividad;
  /** 1 básica · 2 intermedia · 3 reto */
  dificultad: 1 | 2 | 3;
  puntos: number;
  /** Markdown. */
  enunciado: string;
  pistas: string[];
  /** codigo: código con el que empieza el editor. */
  codigo_inicial?: string;
  /** codigo: pruebas automáticas. */
  pruebas?: Prueba[];
  /** prediccion: programa cuya salida hay que predecir. */
  codigo?: string;
  /** prediccion: salida correcta. */
  salida_esperada?: string;
  /** opcion_multiple */
  opciones?: Opcion[];
  /** Explicación que se muestra al resolver. */
  explicacion?: string;
  /** Solo en el curso del profesor. */
  solucion?: string;
}

export interface Leccion {
  id: string;
  titulo: string;
  /** Markdown. */
  contenido: string;
  actividades: Actividad[];
}

export interface Unidad {
  id: string;
  numero: number;
  titulo: string;
  descripcion: string;
  objetivos: string[];
  lecciones: Leccion[];
}

export interface Curso {
  id: string;
  titulo: string;
  version: string;
  descripcion: string;
  unidades: Unidad[];
}

/** Todas las actividades en orden. */
export function actividadesDe(curso: Curso): Actividad[] {
  return curso.unidades.flatMap((u) => u.lecciones.flatMap((l) => l.actividades));
}

/** Busca una actividad y su ubicación. */
export function ubicar(curso: Curso, id: string): { unidad: Unidad; leccion: Leccion; actividad: Actividad } | null {
  for (const unidad of curso.unidades)
    for (const leccion of unidad.lecciones)
      for (const actividad of leccion.actividades) if (actividad.id === id) return { unidad, leccion, actividad };
  return null;
}
