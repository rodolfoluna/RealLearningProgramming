//! App Profesor: identidad y llaves, grupos, importación y verificación de entregas, tablero.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rlp_core::bd_profesor::{BdProfesor, DetalleEntrega, FilaTablero, RegistroEntrega};
use rlp_core::crypto::ParametrosKdf;
use rlp_core::modelo::{ahora_ms, GrupoInfo};
use rlp_core::profesor::{ArchivoIdentidad, IdentidadProfesor, NuevoGrupo};
use rlp_core::reproduccion::{linea_de_tiempo, LineaDeTiempo};
use rlp_core::verificacion::{abrir_entrega, Contexto};
use serde::Serialize;
use tauri::{Manager, State};

type R<T> = Result<T, String>;

fn texto<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

struct Rutas {
    base: PathBuf,
    datos: PathBuf,
}

impl Rutas {
    fn identidad(&self) -> PathBuf {
        self.datos.join("identidad.json")
    }
}

struct Estado {
    rutas: Rutas,
    escribible: bool,
    identidad: Mutex<Option<IdentidadProfesor>>,
    bd: Mutex<Option<BdProfesor>>,
}

impl Estado {
    fn con_bd<T>(&self, f: impl FnOnce(&BdProfesor) -> rlp_core::Resultado<T>) -> R<T> {
        let guardia = self.bd.lock().map_err(texto)?;
        let bd = guardia.as_ref().ok_or("Primero desbloquea tus llaves.")?;
        f(bd).map_err(texto)
    }

    fn con_identidad<T>(&self, f: impl FnOnce(&IdentidadProfesor) -> R<T>) -> R<T> {
        let guardia = self.identidad.lock().map_err(texto)?;
        f(guardia.as_ref().ok_or("Primero desbloquea tus llaves.")?)
    }

    fn activar(&self, id: IdentidadProfesor) -> R<()> {
        let bd = BdProfesor::abrir(&self.rutas.datos.join("profesor.db")).map_err(texto)?;
        *self.bd.lock().map_err(texto)? = Some(bd);
        *self.identidad.lock().map_err(texto)? = Some(id);
        Ok(())
    }
}

fn leer_identidad(ruta: &Path) -> R<ArchivoIdentidad> {
    let bytes = fs::read(ruta).map_err(texto)?;
    serde_json::from_slice(&bytes)
        .map_err(|_| "No es un archivo de llaves del profesor.".to_string())
}

#[derive(Serialize)]
struct EstadoApp {
    version: String,
    carpeta: String,
    escribible: bool,
    tiene_identidad: bool,
    desbloqueado: bool,
    nombre: Option<String>,
    llave_cifrado: Option<String>,
    /// Fase de autoprueba (solo compilaciones de desarrollo, variable RLP_AUTOPRUEBA).
    autoprueba: Option<String>,
}

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
fn autoprueba_fin(app: tauri::AppHandle, resultado: serde_json::Value) -> R<()> {
    if fase_autoprueba().is_none() {
        return Err("No disponible.".into());
    }
    let ok = resultado
        .get("ok")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    println!("AUTOPRUEBA {}", resultado);
    app.exit(if ok { 0 } else { 1 });
    Ok(())
}

#[tauri::command]
fn estado_app(estado: State<Estado>) -> R<EstadoApp> {
    let archivo = leer_identidad(&estado.rutas.identidad()).ok();
    let desbloqueado = estado.identidad.lock().map_err(texto)?.is_some();
    Ok(EstadoApp {
        version: env!("CARGO_PKG_VERSION").into(),
        carpeta: estado.rutas.base.to_string_lossy().into_owned(),
        escribible: estado.escribible,
        tiene_identidad: archivo.is_some(),
        desbloqueado,
        nombre: archivo.as_ref().map(|a| a.nombre.clone()),
        llave_cifrado: archivo.map(|a| a.publica_cifrado),
        autoprueba: fase_autoprueba(),
    })
}

#[tauri::command]
fn crear_identidad(estado: State<Estado>, nombre: String, contrasena: String) -> R<()> {
    if estado.rutas.identidad().exists() {
        return Err("Ya existe una identidad en esta carpeta.".into());
    }
    let id = IdentidadProfesor::nueva(&nombre).map_err(texto)?;
    let archivo = id
        .a_archivo(&contrasena, &ParametrosKdf::estandar())
        .map_err(texto)?;
    fs::create_dir_all(&estado.rutas.datos).map_err(texto)?;
    fs::write(
        estado.rutas.identidad(),
        serde_json::to_vec_pretty(&archivo).map_err(texto)?,
    )
    .map_err(texto)?;
    estado.activar(id)
}

#[tauri::command]
fn desbloquear(estado: State<Estado>, contrasena: String) -> R<()> {
    let archivo = leer_identidad(&estado.rutas.identidad())?;
    let id = IdentidadProfesor::desde_archivo(&archivo, &contrasena).map_err(texto)?;
    estado.activar(id)
}

#[tauri::command]
fn restaurar_identidad(estado: State<Estado>, ruta: String, contrasena: String) -> R<()> {
    let archivo = leer_identidad(Path::new(&ruta))?;
    let id = IdentidadProfesor::desde_archivo(&archivo, &contrasena).map_err(texto)?;
    fs::create_dir_all(&estado.rutas.datos).map_err(texto)?;
    fs::write(
        estado.rutas.identidad(),
        serde_json::to_vec_pretty(&archivo).map_err(texto)?,
    )
    .map_err(texto)?;
    estado.activar(id)
}

#[tauri::command]
fn respaldar_identidad(estado: State<Estado>, carpeta: String) -> R<String> {
    let archivo = leer_identidad(&estado.rutas.identidad())?;
    let nombre: String = archivo
        .nombre
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();
    let destino = PathBuf::from(carpeta).join(format!("respaldo_llaves_{nombre}.rlpk"));
    fs::write(
        &destino,
        serde_json::to_vec_pretty(&archivo).map_err(texto)?,
    )
    .map_err(texto)?;
    Ok(destino.to_string_lossy().into_owned())
}

#[tauri::command]
fn bloquear(estado: State<Estado>) -> R<()> {
    *estado.identidad.lock().map_err(texto)? = None;
    *estado.bd.lock().map_err(texto)? = None;
    Ok(())
}

#[tauri::command]
fn crear_grupo(estado: State<Estado>, datos: NuevoGrupo) -> R<GrupoInfo> {
    let (grupo, info) = estado.con_identidad(|id| id.crear_grupo(&datos).map_err(texto))?;
    estado.con_bd(|bd| bd.guardar_grupo(&grupo, &info))?;
    Ok(info)
}

#[tauri::command]
fn grupos(estado: State<Estado>) -> R<Vec<GrupoInfo>> {
    estado.con_bd(|bd| bd.grupos())
}

fn archivo_grupo(estado: &Estado, grupo_id: &str) -> R<(Vec<u8>, GrupoInfo)> {
    let (g, info) = estado
        .con_bd(|bd| bd.grupo(grupo_id))?
        .ok_or("Grupo no encontrado.")?;
    Ok((serde_json::to_vec_pretty(&g).map_err(texto)?, info))
}

#[tauri::command]
fn exportar_grupo(estado: State<Estado>, grupo_id: String, carpeta: String) -> R<String> {
    let (bytes, info) = archivo_grupo(&estado, &grupo_id)?;
    let nombre: String = info
        .nombre
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let destino = PathBuf::from(carpeta).join(format!("grupo_{nombre}.rlpg"));
    fs::write(&destino, bytes).map_err(texto)?;
    Ok(destino.to_string_lossy().into_owned())
}

/// Copia el archivo de grupo dentro de una carpeta de la App Alumno (config/grupo.rlpg).
#[tauri::command]
fn instalar_grupo(estado: State<Estado>, grupo_id: String, carpeta_app: String) -> R<String> {
    let (bytes, _) = archivo_grupo(&estado, &grupo_id)?;
    let config = PathBuf::from(carpeta_app).join("config");
    fs::create_dir_all(&config).map_err(texto)?;
    let destino = config.join("grupo.rlpg");
    fs::write(&destino, bytes).map_err(texto)?;
    Ok(destino.to_string_lossy().into_owned())
}

#[derive(Serialize)]
struct ResultadoImportacion {
    archivo: String,
    error: Option<String>,
    registro: Option<RegistroEntrega>,
}

fn importar(
    estado: &Estado,
    rutas: Vec<PathBuf>,
    codigos_iniciales: HashMap<String, String>,
) -> R<Vec<ResultadoImportacion>> {
    let mut resultados = Vec::new();
    let mut ordenadas = rutas;
    ordenadas.sort();
    for ruta in ordenadas {
        let archivo = ruta
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let r = (|| -> R<RegistroEntrega> {
            let bytes = fs::read(&ruta).map_err(texto)?;
            let perfil_id = rlp_core::entrega::leer(&bytes)
                .map_err(texto)?
                .manifiesto
                .perfil_id;
            let ctx = Contexto {
                codigos_iniciales: codigos_iniciales.clone(),
                cabezas_previas: estado.con_bd(|bd| bd.cabezas_previas(&perfil_id))?,
                ahora: ahora_ms(),
            };
            let abierta =
                estado.con_identidad(|id| abrir_entrega(&bytes, id, &ctx).map_err(texto))?;
            estado.con_bd(|bd| bd.registrar_entrega(&abierta))
        })();
        match r {
            Ok(registro) => resultados.push(ResultadoImportacion {
                archivo,
                error: None,
                registro: Some(registro),
            }),
            Err(e) => resultados.push(ResultadoImportacion {
                archivo,
                error: Some(e),
                registro: None,
            }),
        }
    }
    Ok(resultados)
}

#[tauri::command]
fn importar_entregas(
    estado: State<Estado>,
    rutas: Vec<String>,
    codigos_iniciales: HashMap<String, String>,
) -> R<Vec<ResultadoImportacion>> {
    importar(
        &estado,
        rutas.into_iter().map(PathBuf::from).collect(),
        codigos_iniciales,
    )
}

/// Importa todas las entregas .rlp de una carpeta (y de sus subcarpetas directas).
#[tauri::command]
fn importar_carpeta(
    estado: State<Estado>,
    carpeta: String,
    codigos_iniciales: HashMap<String, String>,
) -> R<Vec<ResultadoImportacion>> {
    let mut rutas = Vec::new();
    let mut pendientes = vec![(PathBuf::from(carpeta), 0)];
    while let Some((dir, nivel)) = pendientes.pop() {
        for e in fs::read_dir(&dir).map_err(texto)?.flatten() {
            let p = e.path();
            if p.is_dir() && nivel == 0 {
                pendientes.push((p, 1));
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("rlp")) {
                rutas.push(p);
            }
        }
    }
    importar(&estado, rutas, codigos_iniciales)
}

#[tauri::command]
fn tablero(estado: State<Estado>, grupo_id: Option<String>) -> R<Vec<FilaTablero>> {
    estado.con_bd(|bd| bd.tablero(grupo_id.as_deref()))
}

#[tauri::command]
fn detalle(estado: State<Estado>, entrega_id: i64) -> R<DetalleEntrega> {
    estado
        .con_bd(|bd| bd.detalle(entrega_id))?
        .ok_or_else(|| "Entrega no encontrada.".into())
}

/// Línea de tiempo para reproducir cómo se escribió el código de una actividad.
#[tauri::command]
fn reproduccion(estado: State<Estado>, entrega_id: i64, actividad_id: String) -> R<LineaDeTiempo> {
    let payload = estado
        .con_bd(|bd| bd.payload(entrega_id))?
        .ok_or("Esta entrega no se pudo abrir: no hay historial que reproducir.")?;
    linea_de_tiempo(&payload, &actividad_id).map_err(texto)
}

#[tauri::command]
fn calificar(
    estado: State<Estado>,
    perfil_id: String,
    actividad_id: String,
    calificacion: Option<f64>,
    comentario: String,
) -> R<()> {
    estado.con_bd(|bd| bd.calificar(&perfil_id, &actividad_id, calificacion, &comentario))
}

#[tauri::command]
fn exportar_csv(
    estado: State<Estado>,
    grupo_id: Option<String>,
    actividades: Vec<(String, String)>,
    carpeta: String,
) -> R<String> {
    let csv = estado.con_bd(|bd| bd.exportar_csv(grupo_id.as_deref(), &actividades))?;
    let destino = PathBuf::from(carpeta).join(format!("avance_{}.csv", ahora_ms() / 1000));
    fs::write(&destino, csv).map_err(texto)?;
    Ok(destino.to_string_lossy().into_owned())
}

/// Libro de Excel con las hojas Resumen, Actividades y Calificaciones.
#[tauri::command]
fn exportar_xlsx(
    estado: State<Estado>,
    grupo_id: Option<String>,
    actividades: Vec<(String, String)>,
    carpeta: String,
) -> R<String> {
    let bytes = estado
        .con_bd(|bd| rlp_core::excel::exportar_xlsx(bd, grupo_id.as_deref(), &actividades))?;
    let destino = PathBuf::from(carpeta).join(format!("avance_{}.xlsx", ahora_ms() / 1000));
    fs::write(&destino, bytes).map_err(texto)?;
    Ok(destino.to_string_lossy().into_owned())
}

fn resolver_rutas() -> Rutas {
    let base = std::env::var_os("RLP_CARPETA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()?
                .parent()
                .map(Path::to_path_buf)
        })
        .unwrap_or_else(|| PathBuf::from("."));
    Rutas {
        datos: base.join("datos_profesor"),
        base,
    }
}

fn escribible(dir: &Path) -> bool {
    fs::create_dir_all(dir).is_ok() && {
        let p = dir.join(".escritura");
        let ok = fs::write(&p, b"ok").is_ok();
        let _ = fs::remove_file(p);
        ok
    }
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
            let rutas = resolver_rutas();
            let escribible = escribible(&rutas.datos);
            app.manage(Estado {
                rutas,
                escribible,
                identidad: Mutex::new(None),
                bd: Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            estado_app,
            crear_identidad,
            desbloquear,
            restaurar_identidad,
            respaldar_identidad,
            bloquear,
            crear_grupo,
            grupos,
            exportar_grupo,
            instalar_grupo,
            importar_entregas,
            importar_carpeta,
            tablero,
            detalle,
            reproduccion,
            calificar,
            exportar_csv,
            exportar_xlsx,
            autoprueba_fin,
            consola_log,
            entrada_enviar,
            entrada_cancelar,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la App Profesor");
}
