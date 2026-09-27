//! App Alumno: comandos que la interfaz invoca. Toda la lógica vive en `rlp-core`.

mod ejecutable;
mod rutas;

use std::path::PathBuf;
use std::sync::Mutex;

use rlp_core::alumno::{
    listar_perfiles, EstadoAlumno, PerfilLocal, ResultadoGuardado, ResumenImportacion, Secreto,
    SesionAlumno,
};
use rlp_core::crypto::ParametrosKdf;
use rlp_core::estadisticas::Estadisticas;
use rlp_core::grupo::{leer_grupo, verificar_grupo};
use rlp_core::modelo::{EstadoActividad, GrupoFirmado, GrupoInfo};
use rlp_core::replay::LoteOps;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{Emitter, Manager, State};

use rutas::Rutas;

type R<T> = Result<T, String>;

struct Estado {
    rutas: Rutas,
    sesion: Mutex<Option<SesionAlumno>>,
    cerrando: std::sync::atomic::AtomicBool,
}

impl Estado {
    fn cerrar_sesion(&self) {
        let sesion = self.sesion.lock().ok().and_then(|mut g| g.take());
        if let Some(s) = sesion {
            let _ = s.cerrar();
        }
    }
}

/// La interfaz ya guardó lo pendiente: se cierra la sesión y la app.
#[tauri::command]
fn cerrar_app(app: tauri::AppHandle, estado: State<Estado>) {
    estado.cerrar_sesion();
    app.exit(0);
}

fn texto<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

impl Estado {
    fn con_sesion<T>(&self, f: impl FnOnce(&mut SesionAlumno) -> rlp_core::Resultado<T>) -> R<T> {
        let mut guardia = self.sesion.lock().map_err(texto)?;
        let sesion = guardia.as_mut().ok_or("No hay una sesión iniciada.")?;
        f(sesion).map_err(texto)
    }

    fn grupo_app(&self) -> Option<(GrupoFirmado, GrupoInfo)> {
        let bytes = std::fs::read(self.rutas.archivo_grupo()).ok()?;
        leer_grupo(&bytes).ok()
    }
}

fn kdf() -> ParametrosKdf {
    ParametrosKdf::estandar()
}

#[derive(Serialize)]
struct EstadoApp {
    version: String,
    plataforma: String,
    carpeta_datos: String,
    escribible: bool,
    grupo: Option<GrupoInfo>,
    perfiles: Vec<PerfilLocal>,
    puede_generar_exe: bool,
    dev: bool,
    /// Fase de autoprueba (solo compilaciones de desarrollo, variable RLP_AUTOPRUEBA).
    autoprueba: Option<String>,
}

/// Autoprueba de extremo a extremo: solo existe en compilaciones de desarrollo.
fn fase_autoprueba() -> Option<String> {
    if cfg!(debug_assertions) {
        std::env::var("RLP_AUTOPRUEBA")
            .ok()
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

#[tauri::command]
fn autoprueba_fin(app: tauri::AppHandle, resultado: Value) -> R<()> {
    if fase_autoprueba().is_none() {
        return Err("No disponible.".into());
    }
    let ok = resultado
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    println!("AUTOPRUEBA {}", resultado);
    app.exit(if ok { 0 } else { 1 });
    Ok(())
}

#[tauri::command]
fn estado_app(estado: State<Estado>) -> R<EstadoApp> {
    let r = &estado.rutas;
    Ok(EstadoApp {
        version: env!("CARGO_PKG_VERSION").into(),
        plataforma: std::env::consts::OS.into(),
        carpeta_datos: r.base.to_string_lossy().into_owned(),
        escribible: r.escribible,
        grupo: estado.grupo_app().map(|g| g.1),
        perfiles: listar_perfiles(&r.perfiles),
        puede_generar_exe: ejecutable::disponible(r),
        dev: rlp_core::llave_app::llave_app().dev,
        autoprueba: fase_autoprueba(),
    })
}

#[tauri::command]
fn importar_grupo(estado: State<Estado>, ruta: String) -> R<GrupoInfo> {
    let bytes = std::fs::read(&ruta).map_err(texto)?;
    let (_, info) = leer_grupo(&bytes).map_err(texto)?;
    std::fs::create_dir_all(&estado.rutas.config).map_err(texto)?;
    std::fs::write(estado.rutas.archivo_grupo(), bytes).map_err(texto)?;
    Ok(info)
}

#[derive(Serialize)]
struct Registro {
    estado: EstadoAlumno,
    codigo: String,
}

#[tauri::command]
fn registrar(
    estado: State<Estado>,
    numero_control: String,
    nombre: String,
    contrasena: String,
) -> R<Registro> {
    let grupo = estado.grupo_app().map(|g| g.0);
    let (sesion, codigo) = SesionAlumno::registrar(
        &estado.rutas.perfiles,
        &numero_control,
        &nombre,
        &contrasena,
        grupo.as_ref(),
        kdf(),
    )
    .map_err(texto)?;
    let est = sesion.estado().map_err(texto)?;
    *estado.sesion.lock().map_err(texto)? = Some(sesion);
    Ok(Registro {
        estado: est,
        codigo,
    })
}

#[tauri::command]
fn iniciar_sesion(estado: State<Estado>, carpeta: String, secreto: Secreto) -> R<EstadoAlumno> {
    let carpeta = PathBuf::from(carpeta);
    if !estado.rutas.contiene(&carpeta) {
        return Err("Perfil no encontrado.".into());
    }
    let sesion = SesionAlumno::abrir(&carpeta, &secreto, kdf()).map_err(texto)?;
    let est = sesion.estado().map_err(texto)?;
    *estado.sesion.lock().map_err(texto)? = Some(sesion);
    Ok(est)
}

#[tauri::command]
fn restaurar(estado: State<Estado>, ruta: String, secreto: Secreto) -> R<EstadoAlumno> {
    let bytes = std::fs::read(&ruta).map_err(texto)?;
    let sesion =
        SesionAlumno::restaurar(&estado.rutas.perfiles, &bytes, &secreto, kdf()).map_err(texto)?;
    let est = sesion.estado().map_err(texto)?;
    *estado.sesion.lock().map_err(texto)? = Some(sesion);
    Ok(est)
}

#[tauri::command]
fn cerrar_sesion(estado: State<Estado>) -> R<()> {
    if let Some(s) = estado.sesion.lock().map_err(texto)?.take() {
        s.cerrar().map_err(texto)?;
    }
    Ok(())
}

#[tauri::command]
fn estado(estado: State<Estado>) -> R<EstadoAlumno> {
    estado.con_sesion(|s| s.estado())
}

#[tauri::command]
fn estadisticas(estado: State<Estado>) -> R<Estadisticas> {
    estado.con_sesion(|s| Ok(s.estadisticas()))
}

#[tauri::command]
fn abrir_actividad(
    estado: State<Estado>,
    id: String,
    codigo_inicial: String,
) -> R<EstadoActividad> {
    estado.con_sesion(|s| s.abrir_actividad(&id, &codigo_inicial))
}

#[tauri::command]
fn guardar_edicion(
    estado: State<Estado>,
    id: String,
    lote: LoteOps,
    texto: String,
) -> R<ResultadoGuardado> {
    estado.con_sesion(|s| s.guardar_edicion(&id, &lote, &texto))
}

#[tauri::command]
fn reiniciar_actividad(
    estado: State<Estado>,
    id: String,
    codigo_inicial: String,
) -> R<EstadoActividad> {
    estado.con_sesion(|s| s.reiniciar_actividad(&id, &codigo_inicial))
}

#[tauri::command]
fn registrar_pruebas(
    estado: State<Estado>,
    id: String,
    pasadas: u32,
    total: u32,
    puntos: u32,
) -> R<EstadoActividad> {
    estado.con_sesion(|s| s.registrar_pruebas(&id, pasadas, total, puntos))
}

#[tauri::command]
fn registrar_respuesta(
    estado: State<Estado>,
    id: String,
    respuesta: String,
    correcta: bool,
    puntos: u32,
) -> R<EstadoActividad> {
    estado.con_sesion(|s| s.registrar_respuesta(&id, &respuesta, correcta, puntos))
}

#[tauri::command]
fn registrar_pista(estado: State<Estado>, id: String, numero: u32) -> R<EstadoActividad> {
    estado.con_sesion(|s| s.registrar_pista(&id, numero))
}

#[tauri::command]
fn registrar_evento(
    estado: State<Estado>,
    tipo: String,
    actividad: Option<String>,
    datos: Value,
) -> R<()> {
    estado.con_sesion(|s| s.registrar_evento(&tipo, actividad.as_deref(), datos))
}

#[tauri::command]
fn exportar(estado: State<Estado>, carpeta: String) -> R<String> {
    let (nombre, bytes) = estado.con_sesion(|s| s.exportar())?;
    let destino = PathBuf::from(carpeta).join(nombre);
    std::fs::write(&destino, bytes)
        .map_err(|e| format!("No se pudo guardar en esa carpeta: {e}"))?;
    Ok(destino.to_string_lossy().into_owned())
}

#[tauri::command]
fn importar_avances(estado: State<Estado>, ruta: String) -> R<ResumenImportacion> {
    let bytes = std::fs::read(&ruta).map_err(texto)?;
    estado.con_sesion(|s| s.importar(&bytes))
}

#[tauri::command]
fn cambiar_contrasena(estado: State<Estado>, actual: String, nueva: String) -> R<()> {
    estado.con_sesion(|s| s.cambiar_contrasena(&actual, &nueva))
}

#[tauri::command]
fn unirse_grupo(estado: State<Estado>, ruta: String) -> R<GrupoInfo> {
    let bytes = std::fs::read(&ruta).map_err(texto)?;
    let (grupo, _) = leer_grupo(&bytes).map_err(texto)?;
    let info = estado.con_sesion(|s| s.unirse_grupo(&grupo))?;
    // Si la carpeta aún no tiene grupo, este queda como el de la app.
    if estado.grupo_app().is_none() {
        let _ = std::fs::create_dir_all(&estado.rutas.config);
        let _ = std::fs::write(estado.rutas.archivo_grupo(), &bytes);
    }
    let _ = verificar_grupo(&grupo);
    Ok(info)
}

#[tauri::command]
async fn generar_ejecutable(app: tauri::AppHandle, nombre: String, codigo: String) -> R<String> {
    let app2 = app.clone();
    let inicio = std::time::Instant::now();
    let nombre2 = nombre.clone();
    let resultado = tauri::async_runtime::spawn_blocking(move || {
        let estado = app2.state::<Estado>();
        let app3 = app2.clone();
        ejecutable::generar(&estado.rutas, &nombre2, &codigo, move |l| {
            let _ = app3.emit("ejecutable-progreso", l);
        })
    })
    .await
    .map_err(texto)?;
    let estado = app.state::<Estado>();
    let _ = estado.con_sesion(|s| {
        s.registrar_evento(
            "ejecutable",
            None,
            json!({ "nombre": nombre, "ok": resultado.is_ok(), "segundos": inicio.elapsed().as_secs() }),
        )
    });
    resultado.map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn abrir_carpeta(app: tauri::AppHandle, estado: State<Estado>, ruta: String) -> R<()> {
    use tauri_plugin_opener::OpenerExt;
    let ruta = PathBuf::from(ruta);
    if !ruta.is_dir() || !estado.rutas.contiene(&ruta) {
        return Err("Solo se pueden abrir carpetas de la app.".into());
    }
    app.opener()
        .open_path(ruta.to_string_lossy(), None::<&str>)
        .map_err(texto)
}

/// En compilaciones de desarrollo, reenvía errores y `console.*` del WebView a la terminal.
#[tauri::command]
fn entrada_enviar(puente: State<rlp_puente::Puente>, linea: String) {
    puente.enviar(linea);
}

#[tauri::command]
fn entrada_cancelar(puente: State<rlp_puente::Puente>) {
    puente.cancelar();
}

#[tauri::command]
fn consola_log(nivel: String, mensaje: String) {
    if cfg!(debug_assertions) {
        eprintln!("[webview {nivel}] {mensaje}");
    }
}

fn consola_depuracion<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("consola")
        .js_init_script(
            r#"(() => {
  const enviar = (nivel, args) => { try { window.__TAURI_INTERNALS__.invoke("consola_log", { nivel, mensaje: args.map(a => a instanceof Error ? a.stack || a.message : typeof a === "object" ? JSON.stringify(a) : String(a)).join(" ") }); } catch (_) {} };
  for (const n of ["log", "warn", "error"]) { const o = console[n].bind(console); console[n] = (...a) => { o(...a); enviar(n, a); }; }
  addEventListener("error", e => enviar("error", [e.message + " @ " + e.filename + ":" + e.lineno]));
  addEventListener("unhandledrejection", e => enviar("rechazo", [e.reason]));
})();"#
                .to_string(),
        )
        .on_page_load(|w, p| eprintln!("[webview] {:?} {}", p.event(), w.url().map(|u| u.to_string()).unwrap_or_default()))
        .build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut constructor = tauri::Builder::default();
    if cfg!(debug_assertions) {
        constructor = constructor.plugin(consola_depuracion());
    }
    constructor
        .manage(rlp_puente::Puente::default())
        .register_asynchronous_uri_scheme_protocol(
            rlp_puente::ESQUEMA,
            |ctx, peticion, responder| {
                ctx.app_handle()
                    .state::<rlp_puente::Puente>()
                    .atender(peticion, responder);
            },
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let rutas = Rutas::resolver(app.handle());
            let _ = std::fs::remove_dir_all(&rutas.temporal);
            app.manage(Estado {
                rutas,
                sesion: Mutex::new(None),
                cerrando: Default::default(),
            });
            Ok(())
        })
        .on_window_event(|ventana, evento| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = evento {
                // Primero se pide a la interfaz que guarde lo tecleado; si no responde, se cierra igual.
                let estado = ventana.state::<Estado>();
                if estado
                    .cerrando
                    .swap(true, std::sync::atomic::Ordering::SeqCst)
                {
                    estado.cerrar_sesion();
                    return;
                }
                api.prevent_close();
                let _ = ventana.emit("cerrando", ());
                let app = ventana.app_handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(3));
                    app.state::<Estado>().cerrar_sesion();
                    app.exit(0);
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            estado_app,
            importar_grupo,
            registrar,
            iniciar_sesion,
            restaurar,
            cerrar_sesion,
            estado,
            estadisticas,
            abrir_actividad,
            guardar_edicion,
            reiniciar_actividad,
            registrar_pruebas,
            registrar_respuesta,
            registrar_pista,
            registrar_evento,
            exportar,
            importar_avances,
            cambiar_contrasena,
            unirse_grupo,
            generar_ejecutable,
            abrir_carpeta,
            autoprueba_fin,
            cerrar_app,
            consola_log,
            entrada_enviar,
            entrada_cancelar,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la App Alumno");
}
