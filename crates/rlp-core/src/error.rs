use thiserror::Error;

/// Errores del núcleo, con mensajes pensados para mostrarse al usuario.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Contraseña o código de recuperación incorrecto.")]
    Credenciales,
    #[error("El archivo está dañado o fue modificado fuera de la app ({0}).")]
    Alterado(String),
    #[error("Formato no reconocido: {0}")]
    Formato(String),
    #[error("{0}")]
    Validacion(String),
    #[cfg(feature = "sqlite")]
    #[error("Error de base de datos: {0}")]
    Bd(#[from] rusqlite::Error),
    /// Fallo del lugar donde se guardan los datos (p. ej. un depósito inconsistente).
    #[error("Error al guardar los datos: {0}")]
    Almacenamiento(String),
    #[error("Error de archivo: {0}")]
    Io(#[from] std::io::Error),
    #[error("Datos inválidos: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Error en el archivo comprimido: {0}")]
    Zip(#[from] zip::result::ZipError),
}

pub type Resultado<T> = Result<T, Error>;

impl Error {
    pub fn validacion(mensaje: impl Into<String>) -> Self {
        Error::Validacion(mensaje.into())
    }
}
