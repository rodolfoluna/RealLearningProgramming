//! Archivos que el alumno elige con el diálogo del sistema. En escritorio son rutas normales; en
//! Android el selector devuelve URIs `content://`, que se abren con el plugin fs.

type R<T> = Result<T, String>;

pub fn leer(app: &tauri::AppHandle, ruta: &str) -> R<Vec<u8>> {
    #[cfg(target_os = "android")]
    if ruta.starts_with("content://") {
        use tauri_plugin_fs::{FilePath, FsExt};
        let url = tauri::Url::parse(ruta).map_err(|e| e.to_string())?;
        return app
            .fs()
            .read(FilePath::Url(url))
            .map_err(|e| format!("No se pudo leer el archivo: {e}"));
    }
    let _ = app;
    std::fs::read(ruta).map_err(|e| format!("No se pudo leer el archivo: {e}"))
}

pub fn leer_texto(app: &tauri::AppHandle, ruta: &str) -> R<String> {
    String::from_utf8(leer(app, ruta)?).map_err(|_| "El archivo no es de texto.".to_string())
}

pub fn escribir(app: &tauri::AppHandle, ruta: &str, datos: &[u8]) -> R<()> {
    #[cfg(target_os = "android")]
    if ruta.starts_with("content://") {
        use std::io::Write;
        use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};
        let url = tauri::Url::parse(ruta).map_err(|e| e.to_string())?;
        let mut opciones = OpenOptions::new();
        opciones.write(true).truncate(true);
        let mut archivo = app
            .fs()
            .open(FilePath::Url(url), opciones)
            .map_err(|e| format!("No se pudo guardar el archivo: {e}"))?;
        return archivo
            .write_all(datos)
            .map_err(|e| format!("No se pudo guardar el archivo: {e}"));
    }
    let _ = app;
    std::fs::write(ruta, datos).map_err(|e| format!("No se pudo guardar el archivo: {e}"))
}
