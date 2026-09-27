//! Carpetas de la app. En Windows todo vive junto al ejecutable (app portable);
//! en Android, en el almacenamiento privado de la app.

use std::fs;
use std::path::{Path, PathBuf};

pub struct Rutas {
    pub base: PathBuf,
    pub perfiles: PathBuf,
    pub config: PathBuf,
    pub ejecutables: PathBuf,
    pub runtime: PathBuf,
    pub temporal: PathBuf,
    pub escribible: bool,
}

impl Rutas {
    pub fn resolver(app: &tauri::AppHandle) -> Rutas {
        let base = std::env::var_os("RLP_CARPETA")
            .map(PathBuf::from)
            .or_else(|| carpeta_plataforma(app))
            .unwrap_or_else(|| PathBuf::from("."));
        let datos = base.join("datos");
        let rutas = Rutas {
            perfiles: datos.join("perfiles"),
            temporal: datos.join("tmp"),
            config: base.join("config"),
            ejecutables: base.join("mis_ejecutables"),
            runtime: base.join("runtime"),
            escribible: false,
            base,
        };
        let escribible = probar_escritura(&rutas.perfiles);
        Rutas {
            escribible,
            ..rutas
        }
    }

    pub fn archivo_grupo(&self) -> PathBuf {
        self.config.join("grupo.rlpg")
    }

    /// ¿`ruta` está dentro de la carpeta de la app? (evita que la interfaz toque otras rutas)
    pub fn contiene(&self, ruta: &Path) -> bool {
        match (ruta.canonicalize(), self.base.canonicalize()) {
            (Ok(r), Ok(b)) => r.starts_with(b),
            _ => false,
        }
    }
}

#[cfg(target_os = "android")]
fn carpeta_plataforma(app: &tauri::AppHandle) -> Option<PathBuf> {
    use tauri::Manager;
    app.path().app_data_dir().ok()
}

#[cfg(not(target_os = "android"))]
fn carpeta_plataforma(_app: &tauri::AppHandle) -> Option<PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(Path::to_path_buf)
}

fn probar_escritura(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let prueba = dir.join(".escritura");
    let ok = fs::write(&prueba, b"ok").is_ok();
    let _ = fs::remove_file(prueba);
    ok
}
