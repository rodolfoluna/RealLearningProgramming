// Simulación en memoria del núcleo, para desarrollar y probar la interfaz en un navegador.
// No cifra ni firma nada: la lógica real vive en Rust (crates/rlp-core).

import { aplicarOperaciones } from "@rlp/editor";
import type { Backend } from "./backend";
import {
  contadoresVacios,
  type Contadores,
  type EstadoActividad,
  type EstadoAlumno,
  type Estadisticas,
  type GrupoInfo,
  type PerfilLocal,
  type Retroalimentacion,
} from "./tipos";

interface Evento {
  tipo: string;
  actividad: string | null;
  datos: Record<string, unknown>;
}

interface Perfil {
  local: PerfilLocal;
  contrasena: string;
  codigo: string;
  actividades: Record<string, EstadoActividad>;
  eventos: Evento[];
  retroalimentacion: Retroalimentacion | null;
}

/** Contraseña temporal de los archivos de acceso de demostración. */
const TEMPORAL_DEMO = "ABCD-EFGH-JKMN";

const GRUPO_DEMO: GrupoInfo = {
  grupo_id: "demo",
  nombre: "Grupo de demostración",
  materia: "Fundamentos de Programación",
  periodo: "2026-2",
  profesor: "Profesor de prueba",
  politicas: { pegado: "bloquear", registrar_salidas: true },
  regex_control: "",
  creado: Date.now(),
};

function sumar(c: Contadores, e: Evento) {
  const d = e.datos;
  switch (e.tipo) {
    case "ejecucion":
      c.ejecuciones++;
      if (d.estado === "error") c.errores++;
      break;
    case "prueba":
      c.pruebas++;
      if (d.total && d.pasadas === d.total) c.pruebas_exitosas++;
      break;
    case "copia":
      c.copias++;
      break;
    case "pegado":
      c.pegados_intentos++;
      if (d.permitido) c.pegados_permitidos++;
      break;
    case "insercion_sospechosa":
      c.inserciones_sospechosas++;
      break;
    case "foco":
      if (d.estado === "perdido") c.salidas++;
      break;
    case "pista":
      c.pistas++;
      break;
    case "edicion":
      c.teclas += ((d.ops as unknown[][]) ?? []).filter((o) => o[4] === "t").length;
      break;
    case "ejecutable":
      c.ejecutables++;
      break;
  }
}

export function crearBackendSimulado(): Backend {
  const perfiles: Perfil[] = [];
  let grupo: GrupoInfo | null = new URLSearchParams(location.search).has("sin-grupo") ? null : GRUPO_DEMO;
  let actual: Perfil | null = null;

  const sesion = () => {
    if (!actual) throw new Error("No hay sesión iniciada.");
    return actual;
  };
  const registrar = (tipo: string, actividad: string | null, datos: Record<string, unknown> = {}) =>
    sesion().eventos.push({ tipo, actividad, datos });
  const estadisticas = (): Estadisticas => {
    const e: Estadisticas = { global: contadoresVacios(), por_actividad: {} };
    for (const ev of sesion().eventos) {
      sumar(e.global, ev);
      if (ev.actividad) sumar((e.por_actividad[ev.actividad] ??= contadoresVacios()), ev);
    }
    return e;
  };
  const estado = (): EstadoAlumno => ({
    perfil: sesion().local.perfil,
    dispositivo: "simulado",
    grupo,
    actividades: structuredClone(sesion().actividades),
    estadisticas: estadisticas(),
    retroalimentacion: structuredClone(sesion().retroalimentacion),
  });
  const act = (id: string): EstadoActividad =>
    (sesion().actividades[id] ??= {
      codigo: "",
      completada: false,
      pasadas: 0,
      total: 0,
      puntos: 0,
      intentos: 0,
      respuesta: null,
      pistas: 0,
      actualizado: Date.now(),
      dispositivo: "",
    });
  const retrasar = <T>(v: T) => new Promise<T>((r) => setTimeout(() => r(v), 30));

  return {
    estadoApp: async () => ({
      version: "0.2.1-navegador",
      plataforma: new URLSearchParams(location.search).has("android") ? "android" : "navegador",
      carpeta_datos: "(memoria del navegador)",
      escribible: true,
      grupo,
      perfiles: perfiles.map((p) => p.local),
      puede_generar_exe: true,
      dev: true,
    }),
    elegirArchivo: async () => "simulado://archivo",
    elegirCarpeta: async () => "simulado://carpeta",
    elegirDestino: async (_t, nombre) => `simulado://descargas/${nombre}`,
    escanearQr: async () => "RLPG1:demo",
    importarGrupoQr: async () => (grupo = GRUPO_DEMO),
    unirseGrupoQr: async () => (grupo = GRUPO_DEMO),
    exportarA: async () => `${sesion().local.perfil.numero_control}.rlp`,
    abrirCarpeta: async () => undefined,
    importarGrupo: async () => (grupo = GRUPO_DEMO),
    async registrar(numeroControl, nombre, contrasena) {
      if (!numeroControl.trim()) throw new Error("Escribe tu número de control.");
      if (nombre.trim().length < 3) throw new Error("Escribe tu nombre completo.");
      if (contrasena.length < 8) throw new Error("La contraseña debe tener al menos 8 caracteres.");
      if (perfiles.some((p) => p.local.perfil.numero_control === numeroControl.trim()))
        throw new Error("Ya existe un perfil con ese número de control en esta carpeta.");
      const id = `p${perfiles.length + 1}`;
      const p: Perfil = {
        local: {
          perfil: { perfil_id: id, numero_control: numeroControl.trim(), nombre: nombre.trim(), creado: Date.now() },
          carpeta: `simulado://${id}`,
          grupo: grupo?.nombre ?? null,
        },
        contrasena,
        codigo: "ABCD-EFGH-JKMN-PQRS-TVWX",
        actividades: {},
        eventos: [],
        retroalimentacion: null,
      };
      perfiles.push(p);
      actual = p;
      return retrasar({ estado: estado(), codigo: p.codigo });
    },
    async iniciarSesion(carpeta, secreto) {
      const p = perfiles.find((x) => x.local.carpeta === carpeta);
      if (!p) throw new Error("Perfil no encontrado.");
      if (secreto.tipo === "contrasena" && secreto.contrasena !== p.contrasena)
        throw new Error("Contraseña o código de recuperación incorrecto.");
      if (secreto.tipo === "codigo") {
        if (secreto.codigo.replace(/[^A-Z0-9]/gi, "").toUpperCase() !== p.codigo.replace(/-/g, ""))
          throw new Error("Contraseña o código de recuperación incorrecto.");
        p.contrasena = secreto.nueva_contrasena;
      }
      actual = p;
      return retrasar(estado());
    },
    restaurar: async () => {
      throw new Error("La restauración solo está disponible en la app instalada.");
    },
    cerrarSesion: async () => {
      actual = null;
    },
    estado: async () => estado(),
    estadisticas: async () => estadisticas(),
    async abrirActividad(id, codigoInicial) {
      const a = act(id);
      if (!a.dispositivo) {
        a.codigo = codigoInicial;
        a.dispositivo = "simulado";
      }
      registrar("actividad_abierta", id);
      return structuredClone(a);
    },
    async guardarEdicion(id, lote, texto) {
      const a = act(id);
      const nuevo = aplicarOperaciones(a.codigo, lote.ops);
      a.codigo = nuevo;
      a.actualizado = Date.now();
      registrar("edicion", id, { ops: lote.ops });
      return { ok: nuevo === texto, codigo: nuevo };
    },
    async reiniciarActividad(id, codigoInicial) {
      const a = act(id);
      a.codigo = codigoInicial;
      return structuredClone(a);
    },
    async registrarPruebas(id, pasadas, total, puntos) {
      const a = act(id);
      Object.assign(a, { pasadas, total, intentos: a.intentos + 1 });
      if (total > 0 && pasadas === total && !a.completada) Object.assign(a, { completada: true, puntos });
      registrar("prueba", id, { pasadas, total });
      return structuredClone(a);
    },
    async registrarRespuesta(id, respuesta, correcta, puntos) {
      const a = act(id);
      a.respuesta = respuesta;
      a.intentos++;
      if (correcta && !a.completada) Object.assign(a, { completada: true, puntos });
      registrar("respuesta", id, { correcta });
      return structuredClone(a);
    },
    async registrarPista(id, numero) {
      const a = act(id);
      a.pistas = Math.max(a.pistas, numero);
      registrar("pista", id, { numero });
      return structuredClone(a);
    },
    registrarEvento: async (tipo, actividad, datos) => {
      registrar(tipo, actividad, datos);
    },
    exportar: async () => `simulado://carpeta/${sesion().local.perfil.numero_control}.rlp`,
    importarAvances: async () => ({ eventos_nuevos: 0, actividades_actualizadas: [], dispositivos: [], conflictos: [] }),
    async importarRetroalimentacion() {
      if (!grupo) throw new Error("Tu perfil no está en un grupo: primero únete al grupo de tu profesor.");
      const r: Retroalimentacion = {
        profesor: grupo.profesor,
        creado: Date.now(),
        actividades: {
          "u0-hola-mundo": { calificacion: 10, comentario: "¡Excelente inicio!", actualizado: Date.now() },
          "u0-mensaje-bienvenida": { calificacion: 8, comentario: "Revisa los acentos del mensaje.", actualizado: Date.now() },
        },
      };
      sesion().retroalimentacion = r;
      registrar("retroalimentacion", null, { actividades: 2 });
      return structuredClone(r);
    },
    async leerAcceso() {
      const p = perfiles[0];
      return {
        nombre: p?.local.perfil.nombre ?? "Alumno de prueba",
        numero_control: p?.local.perfil.numero_control ?? "00000000",
        profesor: "Profesor de prueba",
        perfil_local: !!p,
      };
    },
    async entrarConAcceso(_ruta, temporal, nueva, rutaEntrega) {
      const p = perfiles[0];
      if (!p && !rutaEntrega) throw new Error("Tu perfil no está en esta carpeta: elige también tu último archivo .rlp.");
      if (!p) throw new Error("La restauración solo está disponible en la app instalada.");
      if (temporal.replace(/[^A-Z0-9]/gi, "").toUpperCase() !== TEMPORAL_DEMO.replace(/-/g, ""))
        throw new Error("Contraseña o código de recuperación incorrecto.");
      if (nueva.length < 8) throw new Error("La contraseña debe tener al menos 8 caracteres.");
      p.contrasena = nueva;
      p.codigo = "NUEV-OCOD-IGOR-ECUP-ERAR";
      actual = p;
      registrar("acceso_profesor", null);
      return retrasar({ ...estado(), codigo_nuevo: p.codigo });
    },
    cambiarContrasena: async (a, n) => {
      if (a !== sesion().contrasena) throw new Error("Contraseña o código de recuperación incorrecto.");
      sesion().contrasena = n;
    },
    unirseGrupo: async () => (grupo = GRUPO_DEMO),
    async generarEjecutable(nombre, _codigo, alProgreso) {
      for (const l of ["Preparando…", "Compilando…", "Listo."]) {
        alProgreso(l);
        await retrasar(null);
      }
      registrar("ejecutable", null, { nombre, ok: true });
      return `simulado://mis_ejecutables/${nombre}.exe`;
    },
  };
}
