//! Núcleo de RealLearningProgramming.
//!
//! Compartido por la App Alumno (Windows, Android y web) y la App Profesor: cifrado de los
//! avances, historial verificable, formato de entrega y verificación. Sin la función `sqlite`
//! compila a WebAssembly (`wasm32-unknown-unknown`).

pub mod acceso;
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
pub mod retroalimentacion;

#[cfg(feature = "firmar")]
pub mod almacen;
#[cfg(feature = "firmar")]
pub mod alumno;
pub mod deposito;

#[cfg(feature = "sqlite")]
pub mod bd_profesor;
#[cfg(feature = "excel")]
pub mod excel;
pub mod profesor;
pub mod verificacion;

pub use error::{Error, Resultado};
