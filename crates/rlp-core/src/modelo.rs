//! Tipos compartidos entre la App Alumno, la App Profesor y el formato de entrega.

use serde::{Deserialize, Serialize};

use crate::crypto::{EnvolturaPublica, EnvolturaSecreto};

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
pub fn ahora_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// En el navegador `SystemTime` no existe: se usa `Date.now()`.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub fn ahora_ms() -> i64 {
    js_sys::Date::now() as i64
}

pub fn nuevo_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// Datos públicos del alumno (nombre y número de control no son secretos).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerfilPublico {
    pub perfil_id: String,
    pub numero_control: String,
    pub nombre: String,
    pub creado: i64,
}

/// Envolturas de la llave de datos (DEK) del alumno.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Envolturas {
    pub contrasena: EnvolturaSecreto,
    pub recuperacion: EnvolturaSecreto,
    #[serde(default)]
    pub profesores: Vec<EnvolturaPublica>,
}

/// Políticas que el profesor fija para su grupo.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Politicas {
    /// "bloquear" (predeterminado) o "propio" (permite pegar lo copiado del propio editor).
    pub pegado: String,
    /// Registrar cuando el alumno cambia a otra ventana durante una actividad.
    pub registrar_salidas: bool,
}

impl Default for Politicas {
    fn default() -> Self {
        Self {
            pegado: "bloquear".into(),
            registrar_salidas: true,
        }
    }
}

/// Configuración de un grupo, firmada por el profesor (archivo `.rlpg`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrupoInfo {
    pub formato: String,
    pub grupo_id: String,
    pub nombre: String,
    #[serde(default)]
    pub materia: String,
    #[serde(default)]
    pub periodo: String,
    pub profesor: String,
    /// Llaves públicas X25519 (base64) de los profesores que pueden abrir las entregas.
    pub llaves_cifrado: Vec<String>,
    /// Llave pública Ed25519 (base64) con la que se firmó este grupo.
    pub llave_firma: String,
    #[serde(default)]
    pub politicas: Politicas,
    /// Expresión regular para validar el número de control ("" = cualquiera).
    #[serde(default)]
    pub regex_control: String,
    pub creado: i64,
}

/// Grupo tal como viaja: contenido JSON exacto + firma Ed25519 del profesor.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GrupoFirmado {
    pub contenido: String,
    pub firma: String,
}

/// Estado actual de una actividad del alumno.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EstadoActividad {
    pub codigo: String,
    #[serde(default)]
    pub completada: bool,
    #[serde(default)]
    pub pasadas: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub puntos: u32,
    #[serde(default)]
    pub intentos: u32,
    /// Respuesta de actividades de opción múltiple o predicción.
    #[serde(default)]
    pub respuesta: Option<String>,
    #[serde(default)]
    pub pistas: u32,
    pub actualizado: i64,
    /// Dispositivo cuyo historial produjo el texto actual (para continuar el replay).
    #[serde(default)]
    pub dispositivo: String,
}
