// Cliente (hilo principal) del worker de Python.

import {
  CONTROL,
  type ErrorPython,
  type EventosEjecucion,
  type MensajeAlWorker,
  type MensajeDelWorker,
  type Prueba,
  type ResultadoEjecucion,
  type ResultadoPrueba,
  type ResultadoPruebas,
} from "./tipos";

export interface OpcionesEjecutor {
  /** URL (terminada en "/") donde están pyodide.mjs, pyodide.asm.wasm y python_stdlib.zip. */
  indexURL: string;
  /** Tiempo máximo por prueba automática. */
  timeoutPruebaMs?: number;
  /** Tiempo de gracia tras interrumpir antes de terminar el worker a la fuerza. */
  graciaMs?: number;
  /** Permite inyectar el worker (pruebas). */
  crearWorker?: () => Worker;
}

type Operacion =
  | {
      tipo: "ejecutar";
      id: number;
      eventos: EventosEjecucion;
      resolver: (r: ResultadoEjecucion) => void;
    }
  | {
      tipo: "probar";
      id: number;
      pruebas: Prueba[];
      parciales: ResultadoPrueba[];
      alProgreso?: (r: ResultadoPrueba) => void;
      resolver: (r: ResultadoPruebas) => void;
    }
  | { tipo: "sintaxis"; id: number; resolver: (r: ErrorPython | null) => void };

const MENSAJE_TIEMPO = "Tu programa tardó demasiado. ¿Hay un ciclo que nunca termina?";

/**
 * Ejecuta código Python del alumno en un Web Worker con Pyodide.
 * Solo corre una operación a la vez.
 */
export class EjecutorPython {
  private worker?: Worker;
  private listo?: Promise<void>;
  private readonly control = new SharedArrayBuffer(CONTROL.DATOS + CONTROL.TAMANO_DATOS);
  private readonly interrupcion = new SharedArrayBuffer(1);
  private readonly ctrl = new Int32Array(this.control, 0, 4);
  private siguienteId = 1;
  private actual?: Operacion;
  private detenidoPorUsuario = false;
  private temporizadores: ReturnType<typeof setTimeout>[] = [];
  private cola: Promise<unknown> = Promise.resolve();

  version = "";
  esperandoEntrada = false;

  constructor(private readonly opciones: OpcionesEjecutor) {}

  /** ¿El entorno permite memoria compartida (necesaria para input())? */
  static disponible(): boolean {
    return typeof SharedArrayBuffer !== "undefined" && globalThis.crossOriginIsolated === true;
  }

  get ocupado(): boolean {
    return this.actual !== undefined;
  }

  iniciar(): Promise<void> {
    if (!this.listo) this.listo = this.crear();
    return this.listo;
  }

  private crear(): Promise<void> {
    return new Promise((resolver, rechazar) => {
      const w = this.opciones.crearWorker
        ? this.opciones.crearWorker()
        : new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
      this.worker = w;
      w.onmessage = (e: MessageEvent<MensajeDelWorker>) => {
        const m = e.data;
        if (m.tipo === "listo") {
          this.version = m.version;
          resolver();
        } else if (m.tipo === "error_inicio") {
          this.listo = undefined;
          rechazar(new Error(`No se pudo iniciar Python: ${m.mensaje}`));
        } else {
          this.alMensaje(m);
        }
      };
      w.onerror = (e) => {
        this.listo = undefined;
        rechazar(new Error(`No se pudo iniciar Python: ${e.message}`));
      };
      const mensaje: MensajeAlWorker = {
        tipo: "iniciar",
        indexURL: this.opciones.indexURL,
        control: this.control,
        interrupcion: this.interrupcion,
      };
      w.postMessage(mensaje);
    });
  }

  private alMensaje(m: MensajeDelWorker) {
    const op = this.actual;
    if (!op || !("id" in m) || m.id !== op.id) return;
    switch (m.tipo) {
      case "salida":
        if (op.tipo === "ejecutar") op.eventos.salida?.(m.texto, m.flujo);
        break;
      case "entrada":
        this.esperandoEntrada = true;
        if (op.tipo === "ejecutar") op.eventos.entradaSolicitada?.();
        break;
      case "limpiar":
        if (op.tipo === "ejecutar") op.eventos.limpiar?.();
        break;
      case "prueba_inicio":
        this.limpiarTemporizadores();
        if (m.indice >= 0 && op.tipo === "probar") this.armarTimeoutPrueba();
        break;
      case "prueba_fin":
        if (op.tipo === "probar") {
          const r = JSON.parse(m.resultado) as ResultadoPrueba;
          op.parciales.push(r);
          op.alProgreso?.(r);
        }
        break;
      case "fin":
        this.terminarOperacion(m.resultado, m.salida_excedida);
        break;
    }
  }

  private terminarOperacion(json: string, salidaExcedida?: boolean) {
    const op = this.actual;
    if (!op) return;
    this.limpiarTemporizadores();
    this.actual = undefined;
    this.esperandoEntrada = false;
    if (op.tipo === "ejecutar") {
      const r = JSON.parse(json) as ResultadoEjecucion;
      if (salidaExcedida) {
        r.salida_excedida = true;
        r.estado = "detenido";
      } else if (this.detenidoPorUsuario && r.estado !== "ok") {
        r.estado = "detenido";
        r.error = null;
      }
      op.resolver(r);
    } else if (op.tipo === "probar") {
      op.resolver(JSON.parse(json) as ResultadoPruebas);
    } else {
      op.resolver(JSON.parse(json) as ErrorPython | null);
    }
  }

  private limpiarTemporizadores() {
    this.temporizadores.forEach(clearTimeout);
    this.temporizadores = [];
  }

  private interrumpir() {
    new Uint8Array(this.interrupcion)[0] = 2;
    Atomics.store(this.ctrl, CONTROL.ESTADO_ENTRADA, 2);
    Atomics.notify(this.ctrl, CONTROL.ESTADO_ENTRADA);
    Atomics.notify(this.ctrl, CONTROL.DORMIR);
  }

  /** Programa la terminación forzada si el worker no responde a la interrupción. */
  private armarTerminacion() {
    this.temporizadores.push(
      setTimeout(() => this.forzarTerminacion(), this.opciones.graciaMs ?? 1500),
    );
  }

  private armarTimeoutPrueba() {
    this.temporizadores.push(
      setTimeout(() => {
        this.interrumpir();
        this.armarTerminacion();
      }, this.opciones.timeoutPruebaMs ?? 4000),
    );
  }

  private forzarTerminacion() {
    const op = this.actual;
    this.worker?.terminate();
    this.worker = undefined;
    this.listo = undefined;
    this.actual = undefined;
    this.esperandoEntrada = false;
    this.limpiarTemporizadores();
    void this.iniciar().catch(() => undefined);
    if (!op) return;
    if (op.tipo === "ejecutar") {
      op.resolver({ estado: "detenido", error: null, duracion_ms: 0 });
    } else if (op.tipo === "probar") {
      const resultados = [...op.parciales];
      for (let i = resultados.length; i < op.pruebas.length; i++) {
        const p = op.pruebas[i];
        resultados.push({
          indice: i,
          nombre: p.nombre ?? `Prueba ${i + 1}`,
          oculta: !!p.oculta,
          tipo: p.funcion ? "funcion" : "io",
          paso: false,
          tiempo_agotado: i === op.parciales.length && !this.detenidoPorUsuario,
          mensaje: i === op.parciales.length && !this.detenidoPorUsuario ? MENSAJE_TIEMPO : "No se ejecutó.",
        });
      }
      op.resolver({ pasadas: resultados.filter((r) => r.paso).length, total: resultados.length, resultados });
    } else {
      op.resolver(null);
    }
  }

  private encolar<T>(fn: () => Promise<T>): Promise<T> {
    const p = this.cola.then(fn, fn);
    this.cola = p.catch(() => undefined);
    return p;
  }

  private async lanzar(op: Operacion, mensaje: MensajeAlWorker) {
    await this.iniciar();
    this.detenidoPorUsuario = false;
    new Uint8Array(this.interrupcion)[0] = 0;
    Atomics.store(this.ctrl, CONTROL.ESTADO_ENTRADA, 0);
    this.actual = op;
    this.worker!.postMessage(mensaje);
  }

  /** Ejecuta un programa de forma interactiva (input() pide datos a la interfaz). */
  ejecutar(codigo: string, eventos: EventosEjecucion = {}): Promise<ResultadoEjecucion> {
    return this.encolar(
      () =>
        new Promise<ResultadoEjecucion>((resolver, rechazar) => {
          const id = this.siguienteId++;
          this.lanzar({ tipo: "ejecutar", id, eventos, resolver }, { tipo: "ejecutar", id, codigo }).catch(
            rechazar,
          );
        }),
    );
  }

  /** Entrega una línea al input() que está esperando. */
  enviarEntrada(linea: string): void {
    if (!this.esperandoEntrada) return;
    const bytes = new TextEncoder().encode(linea).slice(0, CONTROL.TAMANO_DATOS);
    new Uint8Array(this.control, CONTROL.DATOS, CONTROL.TAMANO_DATOS).set(bytes);
    Atomics.store(this.ctrl, CONTROL.LONGITUD, bytes.length);
    this.esperandoEntrada = false;
    Atomics.store(this.ctrl, CONTROL.ESTADO_ENTRADA, 1);
    Atomics.notify(this.ctrl, CONTROL.ESTADO_ENTRADA);
  }

  /** Detiene lo que se esté ejecutando (KeyboardInterrupt y, si no responde, termina el worker). */
  detener(): void {
    if (!this.actual) return;
    this.detenidoPorUsuario = true;
    this.limpiarTemporizadores();
    this.interrumpir();
    this.armarTerminacion();
  }

  /** Corre las pruebas automáticas de una actividad. */
  probar(
    codigo: string,
    pruebas: Prueba[],
    alProgreso?: (r: ResultadoPrueba) => void,
  ): Promise<ResultadoPruebas> {
    return this.encolar(
      () =>
        new Promise<ResultadoPruebas>((resolver, rechazar) => {
          const id = this.siguienteId++;
          this.lanzar(
            { tipo: "probar", id, pruebas, parciales: [], alProgreso, resolver },
            { tipo: "probar", id, codigo, pruebas: JSON.stringify(pruebas) },
          ).catch(rechazar);
        }),
    );
  }

  /** Revisa la sintaxis sin ejecutar. Si hay algo corriendo, no revisa (devuelve null). */
  async sintaxis(codigo: string): Promise<ErrorPython | null> {
    if (this.ocupado) return null;
    return this.encolar(
      () =>
        new Promise<ErrorPython | null>((resolver, rechazar) => {
          const id = this.siguienteId++;
          this.lanzar({ tipo: "sintaxis", id, resolver }, { tipo: "sintaxis", id, codigo }).catch(rechazar);
        }),
    );
  }

  destruir(): void {
    this.worker?.terminate();
    this.worker = undefined;
    this.listo = undefined;
  }
}
