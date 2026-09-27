//! "Generar ejecutable": empaqueta el programa del alumno con PyInstaller usando el Python
//! portátil incluido en `runtime/` (sin internet).

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;

use crate::rutas::Rutas;

/// Nombre seguro para el archivo.
pub fn limpiar_nombre(nombre: &str) -> Result<String, String> {
    let limpio: String = nombre
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect();
    if limpio.trim_matches('_').is_empty() {
        return Err("Escribe un nombre para tu programa (letras, números, guiones).".into());
    }
    Ok(limpio)
}

/// Intérprete de Python a usar: el portátil de la app o, en desarrollo, el del sistema.
pub fn python(rutas: &Rutas) -> Option<PathBuf> {
    let candidatos = if cfg!(windows) {
        vec![rutas.runtime.join("python.exe")]
    } else {
        vec![
            rutas.runtime.join("bin").join("python3"),
            rutas.runtime.join("python3"),
        ]
    };
    if let Some(p) = candidatos.into_iter().find(|p| p.exists()) {
        return Some(p);
    }
    if let Some(p) = std::env::var_os("RLP_PYTHON") {
        return Some(PathBuf::from(p));
    }
    if cfg!(debug_assertions) {
        return Some(PathBuf::from(if cfg!(windows) {
            "python"
        } else {
            "python3"
        }));
    }
    None
}

pub fn disponible(rutas: &Rutas) -> bool {
    !cfg!(target_os = "android") && python(rutas).is_some()
}

/// Genera el ejecutable y devuelve su ruta. `progreso` recibe cada línea del registro.
pub fn generar(
    rutas: &Rutas,
    nombre: &str,
    codigo: &str,
    progreso: impl Fn(String),
) -> Result<PathBuf, String> {
    let nombre = limpiar_nombre(nombre)?;
    let python = python(rutas).ok_or(
        "Esta instalación no incluye la carpeta 'runtime' necesaria para crear ejecutables.",
    )?;
    let trabajo = rutas
        .temporal
        .join(format!("exe-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&trabajo).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&rutas.ejecutables).map_err(|e| e.to_string())?;
    let resultado = construir(
        &python,
        &trabajo,
        &rutas.ejecutables,
        &nombre,
        codigo,
        &progreso,
    );
    let _ = std::fs::remove_dir_all(&trabajo);
    resultado
}

fn construir(
    python: &Path,
    trabajo: &Path,
    destino: &Path,
    nombre: &str,
    codigo: &str,
    progreso: &impl Fn(String),
) -> Result<PathBuf, String> {
    let fuente = trabajo.join(format!("{nombre}.py"));
    // Al terminar, el programa espera Enter para que la consola no se cierre de inmediato.
    let envoltura = format!(
        "{codigo}\n\ntry:\n    input(\"\\nPresiona Enter para salir...\")\nexcept EOFError:\n    pass\n"
    );
    std::fs::write(&fuente, envoltura).map_err(|e| e.to_string())?;
    progreso(format!("Preparando «{nombre}»…"));
    let mut cmd = Command::new(python);
    cmd.args([
        "-m",
        "PyInstaller",
        "--onefile",
        "--console",
        "--noconfirm",
        "--clean",
        "--name",
        nombre,
    ])
    .arg("--distpath")
    .arg(destino)
    .arg("--workpath")
    .arg(trabajo.join("build"))
    .arg("--specpath")
    .arg(trabajo)
    .arg(&fuente)
    .env("PYTHONUTF8", "1")
    .env("PYTHONDONTWRITEBYTECODE", "1")
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let mut hijo = cmd
        .spawn()
        .map_err(|e| format!("No se pudo iniciar PyInstaller: {e}"))?;
    let (tx, rx) = mpsc::channel::<String>();
    let mut hilos = Vec::new();
    for flujo in [
        hijo.stdout
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
        hijo.stderr
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
    ]
    .into_iter()
    .flatten()
    {
        let tx = tx.clone();
        hilos.push(std::thread::spawn(move || {
            for linea in BufReader::new(flujo).lines().map_while(Result::ok) {
                let _ = tx.send(linea);
            }
        }));
    }
    drop(tx);
    let mut ultimas: Vec<String> = Vec::new();
    for linea in rx {
        // Solo lo esencial: PyInstaller es muy verboso.
        let corta = linea.split(": ").skip(1).collect::<Vec<_>>().join(": ");
        let mostrar = if corta.is_empty() {
            linea.clone()
        } else {
            corta
        };
        if linea.contains("ERROR")
            || linea.contains("Building")
            || linea.contains("completed successfully")
            || linea.contains("Error")
        {
            progreso(mostrar);
        }
        ultimas.push(linea);
        if ultimas.len() > 30 {
            ultimas.remove(0);
        }
    }
    for h in hilos {
        let _ = h.join();
    }
    let estado = hijo.wait().map_err(|e| e.to_string())?;
    if !estado.success() {
        let detalle = ultimas.join("\n");
        if detalle.contains("No module named PyInstaller") {
            return Err("PyInstaller no está instalado en el runtime de la app.".into());
        }
        return Err(format!("PyInstaller terminó con error:\n{detalle}"));
    }
    let exe = destino.join(if cfg!(windows) {
        format!("{nombre}.exe")
    } else {
        nombre.to_string()
    });
    if !exe.exists() {
        return Err("PyInstaller terminó pero no se encontró el ejecutable.".into());
    }
    progreso("Listo.".into());
    Ok(exe)
}
