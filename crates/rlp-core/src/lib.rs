//! Núcleo de RealLearningProgramming.
//!
//! Compartido por la App Alumno y la App Profesor (Windows ahora, Android después):
//! cifrado de los avances, historial verificable, formato de entrega y verificación.

pub mod crypto;
pub mod entrega;
pub mod error;
pub mod estadisticas;
pub mod eventos;
pub mod grupo;
pub mod llave_app;
pub mod modelo;
pub mod replay;
pub mod reproduccion;

#[cfg(feature = "firmar")]
pub mod almacen;
#[cfg(feature = "firmar")]
pub mod alumno;

pub mod bd_profesor;
#[cfg(feature = "excel")]
pub mod excel;
pub mod profesor;
pub mod verificacion;

pub use error::{Error, Resultado};
