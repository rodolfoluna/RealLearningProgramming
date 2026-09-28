//! Archivo de grupo `.rlpg`: configuración firmada por el profesor.

use regex::Regex;

use crate::crypto::{de_b64_32, verificar};
use crate::error::{Error, Resultado};
use crate::modelo::{GrupoFirmado, GrupoInfo};

pub const FORMATO: &str = "rlp-grupo";
pub const CONTEXTO_FIRMA: &[u8] = b"rlp-grupo-v1";
pub const EXTENSION: &str = "rlpg";
/// Prefijo del grupo codificado para un código QR.
pub const PREFIJO_QR: &str = "RLPG1:";
const LIMITE_QR: u64 = 64 * 1024;

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

/// Codifica el grupo firmado para mostrarlo como código QR: JSON comprimido en base64 URL.
pub fn a_texto_qr(g: &GrupoFirmado) -> Resultado<String> {
    use base64::Engine;
    use std::io::Write;
    let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::best());
    z.write_all(&serde_json::to_vec(g)?)?;
    let b64 = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(z.finish()?);
    Ok(format!("{PREFIJO_QR}{b64}"))
}

/// Lee el texto de un código QR de grupo y verifica su firma.
pub fn desde_texto_qr(texto: &str) -> Resultado<(GrupoFirmado, GrupoInfo)> {
    use base64::Engine;
    use std::io::Read;
    let datos = texto
        .trim()
        .strip_prefix(PREFIJO_QR)
        .ok_or_else(|| Error::Formato("ese código QR no es de un grupo de RLP".into()))?;
    let comprimido = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(datos)
        .map_err(|_| Error::Formato("código QR de grupo dañado".into()))?;
    let mut json = Vec::new();
    flate2::read::DeflateDecoder::new(comprimido.as_slice())
        .take(LIMITE_QR)
        .read_to_end(&mut json)
        .map_err(|_| Error::Formato("código QR de grupo dañado".into()))?;
    leer_grupo(&json)
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

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::modelo::Politicas;
    use crate::profesor::{IdentidadProfesor, NuevoGrupo};

    #[test]
    fn grupo_por_codigo_qr() {
        let profe = IdentidadProfesor::nueva("Profa. Ruiz").unwrap();
        let (g, info) = profe
            .crear_grupo(&NuevoGrupo {
                nombre: "Programación 1B".into(),
                materia: "Fundamentos".into(),
                periodo: "2026-2".into(),
                regex_control: r"\d{8}".into(),
                politicas: Politicas::default(),
                coprofesores: vec![],
            })
            .unwrap();
        let texto = a_texto_qr(&g).unwrap();
        assert!(texto.starts_with(PREFIJO_QR));
        // Cabe con holgura en un QR (versión 40-L admite 2 953 bytes).
        assert!(texto.len() < 1500, "{} caracteres", texto.len());
        let (g2, info2) = desde_texto_qr(&texto).unwrap();
        assert_eq!(g2, g);
        assert_eq!(info2, info);
        assert!(desde_texto_qr("hola").is_err());
        assert!(desde_texto_qr(&format!("{PREFIJO_QR}AAAA")).is_err());
    }
}
