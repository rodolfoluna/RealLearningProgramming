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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let rutas = Rutas::resolver(app.handle());
            let _ = std::fs::remove_dir_all(&rutas.temporal);
            app.manage(Estado {
                rutas,
                sesion: Mutex::new(None),
            });
            Ok(())
        })
        .on_window_event(|ventana, evento| {
            if let tauri::WindowEvent::CloseRequested { .. } = evento {
                let estado = ventana.state::<Estado>();
                let sesion = estado.sesion.lock().ok().and_then(|mut g| g.take());
                if let Some(s) = sesion {
                    let _ = s.cerrar();
                }
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
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la App Alumno");
}
