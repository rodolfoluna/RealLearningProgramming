//! Archivo de acceso (`.rlpa`): el profesor ayuda a un alumno que olvidó su contraseña y su
//! código de recuperación. El profesor abre la llave de datos del alumno (desde su última
//! entrega), la envuelve con una contraseña temporal y firma el archivo. El alumno entra con
//! el archivo y la contraseña temporal, fija una contraseña nueva y recibe un código de
//! recuperación nuevo. La App Alumno solo acepta la firma del profesor de su grupo.

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::crypto::{
    abrir_con_secreto, aleatorio, de_b64_32, envolver_con_secreto, normalizar_codigo, verificar,
    EnvolturaSecreto, Llave, ParametrosKdf,
};
use crate::entrega::Manifiesto;
use crate::error::{Error, Resultado};
use crate::modelo::ahora_ms;
use crate::profesor::IdentidadProfesor;

pub const FORMATO: &str = "rlp-acceso";
pub const CONTEXTO_FIRMA: &[u8] = b"rlp-acceso-v1";
pub const EXTENSION: &str = "rlpa";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchivoAcceso {
    pub contenido: String,
    pub firma: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Acceso {
    pub formato: String,
    pub perfil_id: String,
    pub numero_control: String,
    pub nombre: String,
    pub profesor: String,
    /// Llave pública Ed25519 del profesor que firmó.
    pub llave_firma: String,
    pub creado: i64,
    /// Llave de datos del alumno envuelta con la contraseña temporal.
    pub envoltura: EnvolturaSecreto,
}

fn aad(perfil_id: &str) -> Vec<u8> {
    format!("dek|{perfil_id}").into_bytes()
}

/// Contraseña temporal fácil de dictar: `XXXX-XXXX-XXXX` (60 bits, protegida además con Argon2).
pub fn contrasena_temporal() -> String {
    const ALFABETO: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let bytes: [u8; 12] = aleatorio();
    let s: Vec<char> = bytes
        .iter()
        .map(|b| ALFABETO[(*b & 31) as usize] as char)
        .collect();
    s.chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Para el profesor: crea el archivo a partir de la última entrega del alumno. Devuelve el
/// archivo y la contraseña temporal que hay que darle al alumno.
pub fn crear(
    identidad: &IdentidadProfesor,
    manifiesto: &Manifiesto,
    numero_control: &str,
    nombre: &str,
    kdf: &ParametrosKdf,
) -> Resultado<(Vec<u8>, String)> {
    let dek = identidad.abrir_dek(&manifiesto.envolturas.profesores, &manifiesto.perfil_id)?;
    let temporal = contrasena_temporal();
    let acceso = Acceso {
        formato: FORMATO.into(),
        perfil_id: manifiesto.perfil_id.clone(),
        numero_control: numero_control.into(),
        nombre: nombre.into(),
        profesor: identidad.nombre.clone(),
        llave_firma: identidad.publica_firma(),
        creado: ahora_ms(),
        envoltura: envolver_con_secreto(
            &dek,
            &normalizar_codigo(&temporal),
            kdf,
            &aad(&manifiesto.perfil_id),
        )?,
    };
    let contenido = serde_json::to_string(&acceso)?;
    let firma = identidad.firmar(CONTEXTO_FIRMA, contenido.as_bytes());
    Ok((
        serde_json::to_vec_pretty(&ArchivoAcceso { contenido, firma })?,
        temporal,
    ))
}

/// Lee el archivo y comprueba su firma. Quien lo use debe comprobar además que `llave_firma`
/// sea la del profesor de su grupo.
pub fn leer(bytes: &[u8]) -> Resultado<Acceso> {
    let archivo: ArchivoAcceso = serde_json::from_slice(bytes)
        .map_err(|_| Error::Formato("no es un archivo de acceso".into()))?;
    let acceso: Acceso = serde_json::from_str(&archivo.contenido)
        .map_err(|_| Error::Formato("no es un archivo de acceso".into()))?;
    if acceso.formato != FORMATO {
        return Err(Error::Formato("no es un archivo de acceso".into()));
    }
    if !verificar(
        &de_b64_32(&acceso.llave_firma)?,
        CONTEXTO_FIRMA,
        archivo.contenido.as_bytes(),
        &archivo.firma,
    ) {
        return Err(Error::Alterado(
            "la firma del archivo de acceso no es válida".into(),
        ));
    }
    Ok(acceso)
}

/// Desenvuelve la llave de datos con la contraseña temporal.
pub fn abrir_dek(acceso: &Acceso, temporal: &str) -> Resultado<Zeroizing<Llave>> {
    abrir_con_secreto(
        &acceso.envoltura,
        &normalizar_codigo(temporal),
        &aad(&acceso.perfil_id),
    )
}
