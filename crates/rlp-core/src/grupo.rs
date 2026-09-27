//! Archivo de grupo `.rlpg`: configuración firmada por el profesor.

use regex::Regex;

use crate::crypto::{de_b64_32, verificar};
use crate::error::{Error, Resultado};
use crate::modelo::{GrupoFirmado, GrupoInfo};

pub const FORMATO: &str = "rlp-grupo";
pub const CONTEXTO_FIRMA: &[u8] = b"rlp-grupo-v1";
pub const EXTENSION: &str = "rlpg";

/// Verifica la firma del grupo y devuelve su contenido.
pub fn verificar_grupo(g: &GrupoFirmado) -> Resultado<GrupoInfo> {
    let info: GrupoInfo = serde_json::from_str(&g.contenido)
        .map_err(|_| Error::Formato("archivo de grupo inválido".into()))?;
    if info.formato != FORMATO {
        return Err(Error::Formato(
            "no es un archivo de grupo de RealLearningProgramming".into(),
        ));
    }
    let llave = de_b64_32(&info.llave_firma)?;
    if !verificar(&llave, CONTEXTO_FIRMA, g.contenido.as_bytes(), &g.firma) {
        return Err(Error::Alterado("la firma del grupo no es válida".into()));
    }
    if info.llaves_cifrado.is_empty() {
        return Err(Error::Formato(
            "el grupo no tiene llaves de profesor".into(),
        ));
    }
    for l in &info.llaves_cifrado {
        de_b64_32(l)?;
    }
    Ok(info)
}

pub fn leer_grupo(bytes: &[u8]) -> Resultado<(GrupoFirmado, GrupoInfo)> {
    let g: GrupoFirmado = serde_json::from_slice(bytes)
        .map_err(|_| Error::Formato("archivo de grupo inválido".into()))?;
    let info = verificar_grupo(&g)?;
    Ok((g, info))
}

/// Valida un número de control con la expresión del grupo (vacía = cualquiera no vacío).
pub fn validar_numero_control(regex: &str, numero: &str) -> Resultado<()> {
    if numero.trim().is_empty() {
        return Err(Error::validacion("Escribe tu número de control."));
    }
    if regex.trim().is_empty() {
        return Ok(());
    }
    let re = Regex::new(&format!("^(?:{regex})$"))
        .map_err(|_| Error::validacion("La regla del número de control del grupo es inválida."))?;
    if re.is_match(numero.trim()) {
        Ok(())
    } else {
        Err(Error::validacion(
            "El número de control no tiene el formato que pidió tu profesor.",
        ))
    }
}
