// Datos de demostración para desarrollar y probar la interfaz en un navegador.
// La verificación real (cifrado, firmas, historial) vive en Rust (crates/rlp-core).

import { actividadesDe, type Curso } from "@rlp/curso";
import cursoJson from "@rlp/curso/profesor.json";
import type { Backend } from "./backend";
import {
  contadoresVacios,
  type Calificacion,
  type Check,
  type DetalleEntrega,
  type FilaTablero,
  type GrupoInfo,
  type LineaDeTiempo,
  type Marca,
  type Nivel,
  type OpConTiempo,
  type ResumenActividad,
} from "./tipos";

const curso = cursoJson as Curso;

interface Alumno {
  nombre: string;
  control: string;
  avance: number;
  nivel: Nivel;
  pegados: number;
  copias: number;
  salidas: number;
}

const ALUMNOS: Alumno[] = [
  { nombre: "Ana López García", control: "21340001", avance: 0.9, nivel: "verde", pegados: 0, copias: 3, salidas: 1 },
  { nombre: "Bruno Méndez Ruiz", control: "21340002", avance: 0.55, nivel: "verde", pegados: 2, copias: 8, salidas: 4 },
  { nombre: "Carla Díaz Torres", control: "21340003", avance: 0.3, nivel: "amarillo", pegados: 11, copias: 20, salidas: 9 },
  { nombre: "Diego Pérez Solís", control: "21340004", avance: 0.75, nivel: "rojo", pegados: 1, copias: 2, salidas: 0 },
];

function checks(nivel: Nivel): Check[] {
  const base: Check[] = [
    { id: "firma", nombre: "Firma de la app", nivel: "verde", detalle: "Firma válida (versión publicada)." },
    { id: "integridad", nombre: "Integridad del archivo", nivel: "verde", detalle: "El contenido coincide con el manifiesto." },
    { id: "descifrado", nombre: "Descifrado", nivel: "verde", detalle: "El contenido se descifró correctamente." },
    { id: "cadena", nombre: "Historial", nivel: "verde", detalle: "812 eventos en 1 dispositivo(s), encadenados y firmados." },
    { id: "identidad", nombre: "Identidad", nivel: "verde", detalle: "Coincide con los datos del registro." },
    { id: "tiempos", nombre: "Fechas y horas", nivel: "verde", detalle: "Las fechas del historial son coherentes." },
    { id: "replay", nombre: "Historial de escritura", nivel: "verde", detalle: "El código de 30 actividad(es) se reconstruye exactamente tecla a tecla." },
  ];
  if (nivel === "amarillo") base[6] = { ...base[6], nivel: "amarillo", detalle: "'u3-cajero': 2 ráfaga(s) de escritura a más de 12 caracteres/s" };
  if (nivel === "rojo") base[6] = { ...base[6], nivel: "rojo", detalle: "'u3-calculadora': el código entregado NO coincide con lo escrito en la app" };
  return base;
}

/** Historial de demostración: teclea `codigo` con pausas, un error corregido y algunas marcas. */
function historialSimulado(actividad: string, codigo: string, pegados: number): LineaDeTiempo {
  let semilla = 7;
  const azar = () => ((semilla = (semilla * 1103515245 + 12345) % 2147483648) / 2147483648);
  const inicio = Date.now() - 2 * 86400_000;
  let t = inicio;
  let pos = 0;
  const ops: OpConTiempo[] = [];
  const marcas: Marca[] = [];
  [...codigo].forEach((c, i) => {
    t += 120 + Math.round(azar() * 260) + (c === "\n" ? 900 + Math.round(azar() * 2500) : 0);
    if (i === Math.floor(codigo.length / 2)) t += 5 * 60_000; // se fue un rato
    if (i === 12) {
      // Un error de dedo que corrige.
      ops.push([t, pos, pos, "x", "t"]);
      t += 400;
      ops.push([t, pos, pos + 1, "", "d"]);
      t += 300;
    }
    ops.push([t, pos, pos, c, "t"]);
    pos += c.length;
    if (i === Math.floor(codigo.length * 0.4)) marcas.push({ t, tipo: "ejecucion", dispositivo: "d1", datos: { estado: "error" } });
    if (pegados && i === Math.floor(codigo.length * 0.6)) marcas.push({ t, tipo: "pegado", dispositivo: "d1", datos: { permitido: false } });
    if (i === Math.floor(codigo.length * 0.7)) marcas.push({ t: t + 10, tipo: "foco", dispositivo: "d1", datos: { estado: "perdido" } });
  });
  marcas.push({ t: t + 2000, tipo: "prueba", dispositivo: "d1", datos: { pasadas: 3, total: 3 } });
  return {
    actividad,
    tramos: [{ dispositivo: "d1", motivo: "inicio", texto_inicial: "", ops, t_inicio: inicio, t_fin: t }],
    marcas,
    codigo_final: codigo,
    avisos: [],
  };
}

export function crearBackendSimulado(): Backend {
  const grupos: GrupoInfo[] = [
    {
      grupo_id: "g1",
      nombre: "Programación 1A",
      materia: "Fundamentos de Programación",
      periodo: "Ago–Dic 2026",
      profesor: "Profesor de prueba",
      llaves_cifrado: ["demo"],
      politicas: { pegado: "bloquear", registrar_salidas: true },
      regex_control: "\\d{8}",
      creado: Date.now(),
    },
  ];
  let desbloqueado = new URLSearchParams(location.search).has("desbloqueado");
  const calificaciones: Record<string, Record<string, Calificacion>> = {};
  const acts = actividadesDe(curso);

  const filas: FilaTablero[] = ALUMNOS.map((a, i) => {
    const hechas = Math.round(acts.length * a.avance);
    const actividades: Record<string, ResumenActividad> = {};
    acts.slice(0, hechas + 2).forEach((act, j) => {
      const completada = j < hechas;
      const total = act.pruebas?.length ?? 0;
      actividades[act.id] = { completada, pasadas: completada ? total : Math.max(0, total - 1), total, puntos: completada ? act.puntos : 0, intentos: 1 + (j % 3), pistas: j % 4 === 0 ? 1 : 0, con_codigo: act.tipo === "codigo" };
    });
    return {
      entrega_id: i + 1,
      perfil_id: `p${i + 1}`,
      grupo_id: "g1",
      numero_control: a.control,
      nombre: a.nombre,
      recibido: Date.now() - i * 3600_000,
      creado: Date.now() - i * 3600_000 - 600_000,
      nivel: a.nivel,
      entregas: 1 + (i % 3),
      global: { ...contadoresVacios(), tiempo_ms: (3 + hechas) * 9 * 60_000, ejecuciones: hechas * 6, errores: hechas * 2, pruebas: hechas * 2, copias: a.copias, pegados_intentos: a.pegados, salidas: a.salidas, pistas: Math.round(hechas / 3), teclas: hechas * 180 },
      actividades,
      alerta_identidad: false,
    };
  });

  return {
    estadoApp: async () => ({
      version: "0.1.0-navegador",
      carpeta: "(memoria del navegador)",
      escribible: true,
      tiene_identidad: true,
      desbloqueado,
      nombre: "Profesor de prueba",
      llave_cifrado: "demo",
    }),
    elegirArchivos: async () => ["simulado://entrega.rlp"],
    elegirCarpeta: async () => "simulado://carpeta",
    crearIdentidad: async () => void (desbloqueado = true),
    desbloquear: async (c) => {
      if (c !== "profesor1234") throw new Error("Contraseña o código de recuperación incorrecto.");
      desbloqueado = true;
    },
    restaurarIdentidad: async () => void (desbloqueado = true),
    respaldarIdentidad: async () => "simulado://carpeta/respaldo_llaves.rlpk",
    bloquear: async () => void (desbloqueado = false),
    crearGrupo: async (d) => {
      const g: GrupoInfo = { ...d, grupo_id: `g${grupos.length + 1}`, profesor: "Profesor de prueba", llaves_cifrado: ["demo"], creado: Date.now() };
      grupos.unshift(g);
      return g;
    },
    grupos: async () => grupos,
    exportarGrupo: async (id) => `simulado://carpeta/grupo_${id}.rlpg`,
    instalarGrupo: async () => "simulado://App Alumno/config/grupo.rlpg",
    // Cuadrícula de demostración (el QR real lo genera Rust con el grupo firmado).
    qrGrupo: async () =>
      `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 21 21" width="360" height="360" data-qr-demo><rect width="21" height="21" fill="#fff"/>${Array.from(
        { length: 120 },
        (_, i) => `<rect x="${(i * 7) % 21}" y="${Math.floor((i * 7) / 21) % 21}" width="1" height="1"/>`,
      ).join("")}</svg>`,
    importarEntregas: async () =>
      filas.map((f) => ({ archivo: `${f.numero_control}_20261015-1200.rlp`, error: null, registro: { entrega_id: f.entrega_id, nueva: true, nombre: f.nombre, numero_control: f.numero_control, nivel: f.nivel } })),
    importarCarpeta: async () => [{ archivo: "archivo_danado.rlp", error: "Formato no reconocido: no es un archivo .rlp", registro: null }],
    tablero: async () => filas,
    detalle: async (id) => {
      const base = filas.find((x) => x.entrega_id === id % 100)!;
      const f = { ...base, entrega_id: id };
      const actividades: DetalleEntrega["actividades"] = {};
      for (const [aid, r] of Object.entries(f.actividades)) {
        const act = acts.find((a) => a.id === aid)!;
        actividades[aid] = {
          codigo: act.tipo === "codigo" ? (r.completada ? act.solucion ?? "" : act.codigo_inicial ?? "") : "",
          completada: r.completada, pasadas: r.pasadas, total: r.total, puntos: r.puntos, intentos: r.intentos,
          respuesta: act.tipo !== "codigo" ? "(respuesta del alumno)" : null, pistas: r.pistas, actualizado: Date.now(),
        };
      }
      const por_actividad = Object.fromEntries(
        Object.keys(f.actividades).map((k, j) => [k, { ...contadoresVacios(), tiempo_ms: (4 + j) * 60_000, ejecuciones: 3 + (j % 5), pruebas: 1 + (j % 2), copias: j % 3, pegados_intentos: j === 2 ? f.global.pegados_intentos : 0, salidas: j % 4 === 0 ? 1 : 0 }]),
      );
      return {
        fila: f,
        manifiesto: { creado: f.creado, app: { version: "0.1.0", dev: false }, cabezas: { d1: { seq: 812 } } },
        perfil: { perfil_id: f.perfil_id, numero_control: f.numero_control, nombre: f.nombre, creado: Date.now() - 30 * 86400_000 },
        actividades,
        reporte: { nivel: f.nivel, checks: checks(f.nivel), ritmo: { tecleados: 5400, automaticos: 800, deshacer_rehacer: 120, pegados: 0, otros: 0, max_cps: f.nivel === "amarillo" ? 18.4 : 5.2, rafagas: f.nivel === "amarillo" ? 2 : 0 }, ritmo_por_actividad: {} },
        estadisticas: { global: f.global, por_actividad },
        calificaciones: calificaciones[f.perfil_id] ?? {},
        // Entregas anteriores: ids 100, 200… sobre el de la más reciente (ver `base` arriba).
        historial: Array.from({ length: f.entregas }, (_, k) => ({ entrega_id: f.entrega_id + 100 * k, recibido: f.recibido - k * 86400_000 * 7, creado: f.creado - k * 86400_000 * 7, nivel: f.nivel })),
      };
    },
    reproduccion: async (id, actividad) => {
      const f = filas.find((x) => x.entrega_id === id % 100)!;
      const act = acts.find((a) => a.id === actividad)!;
      const codigo = f.actividades[actividad]?.completada ? act.solucion ?? "" : act.codigo_inicial ?? "";
      return historialSimulado(actividad, codigo, f.global.pegados_intentos);
    },
    calificar: async (perfil, act, calificacion, comentario) => {
      (calificaciones[perfil] ??= {})[act] = { calificacion, comentario, actualizado: Date.now() };
    },
    exportarCsv: async () => "simulado://carpeta/avance.csv",
    exportarXlsx: async () => "simulado://carpeta/avance.xlsx",
    async exportarRetroalimentacion() {
      const alumnos = Object.values(calificaciones).filter((c) => Object.keys(c).length).length;
      if (!alumnos) throw new Error("Aún no has calificado ni comentado ninguna actividad de estos alumnos.");
      return { ruta: "simulado://carpeta/retroalimentacion_grupo.rlpr", alumnos, temporal: null };
    },
    async crearAcceso(id) {
      const f = filas.find((x) => x.entrega_id === id % 100)!;
      return { ruta: `simulado://carpeta/acceso_${f.numero_control}.rlpa`, alumnos: 1, temporal: "ABCD-EFGH-JKMN" };
    },
  };
}
