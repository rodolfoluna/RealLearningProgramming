//! Formato de entrega `.rlp` (zip):
//!
//! - `manifiesto.json`: datos en claro (perfil, envolturas de la llave, cabezas de la cadena,
//!   hash del payload).
//! - `payload.bin`: JSON comprimido y cifrado con la llave de datos del alumno (AES-256-GCM).
//! - `firma.sig`: firma Ed25519 de la App Alumno sobre los bytes exactos del manifiesto.
//!
//! El mismo archivo sirve para entregar al profesor y para continuar en otro dispositivo.

use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};

use crate::crypto::de_b64_32;
use crate::crypto::{cifrar, descifrar, sha256_hex, verificar, Llave};
use crate::error::{Error, Resultado};
use crate::eventos::{Cabeza, EventoFirmado};
use crate::modelo::{Envolturas, EstadoActividad, GrupoFirmado, PerfilPublico};

pub const FORMATO: &str = "rlp-entrega";
pub const VERSION: u32 = 1;
pub const CONTEXTO_FIRMA: &[u8] = b"rlp-entrega-v1";
pub const EXTENSION: &str = "rlp";
const LIMITE_PAYLOAD: u64 = 256 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InfoApp {
    pub version: String,
    /// Llave pública Ed25519 (base64) de la app que firmó.
    pub llave: String,
    pub dev: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifiesto {
    pub formato: String,
    pub version: u32,
    pub perfil_id: String,
    pub creado: i64,
    pub app: InfoApp,
    #[serde(default)]
    pub grupo_id: Option<String>,
    pub envolturas: Envolturas,
    pub cabezas: BTreeMap<String, Cabeza>,
    pub payload_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventoExportado {
    pub json: String,
    pub hash: String,
    pub firma: String,
    /// Índice en `Payload::llaves_app`.
    pub llave: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Payload {
    pub perfil: PerfilPublico,
    #[serde(default)]
    pub grupo: Option<GrupoFirmado>,
    pub actividades: BTreeMap<String, EstadoActividad>,
    pub llaves_app: Vec<String>,
    pub eventos: Vec<EventoExportado>,
}

impl Payload {
    pub fn empaquetar_eventos(eventos: Vec<EventoFirmado>) -> (Vec<String>, Vec<EventoExportado>) {
        let mut llaves: Vec<String> = Vec::new();
        let exportados = eventos
            .into_iter()
            .map(|e| {
                let llave = match llaves.iter().position(|l| *l == e.llave) {
                    Some(i) => i,
                    None => {
                        llaves.push(e.llave.clone());
                        llaves.len() - 1
                    }
                };
                EventoExportado {
                    json: e.json,
                    hash: e.hash,
                    firma: e.firma,
                    llave,
                }
            })
            .collect();
        (llaves, exportados)
    }

    pub fn eventos_firmados(&self) -> Resultado<Vec<EventoFirmado>> {
        self.eventos
            .iter()
            .map(|e| {
                Ok(EventoFirmado {
                    json: e.json.clone(),
                    hash: e.hash.clone(),
                    firma: e.firma.clone(),
                    llave: self
                        .llaves_app
                        .get(e.llave)
                        .cloned()
                        .ok_or_else(|| Error::Formato("índice de llave inválido".into()))?,
                })
            })
            .collect()
    }
}

pub fn aad_payload(perfil_id: &str) -> Vec<u8> {
    format!("rlp-payload-v1|{perfil_id}").into_bytes()
}

pub fn cifrar_payload(dek: &Llave, payload: &Payload) -> Resultado<Vec<u8>> {
    let json = serde_json::to_vec(payload)?;
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
    enc.write_all(&json)?;
    Ok(cifrar(
        dek,
        &aad_payload(&payload.perfil.perfil_id),
        &enc.finish()?,
    ))
}

pub fn descifrar_payload(dek: &Llave, perfil_id: &str, cifrado: &[u8]) -> Resultado<Payload> {
    let comprimido = descifrar(dek, &aad_payload(perfil_id), cifrado)?;
    let mut json = Vec::new();
    DeflateDecoder::new(comprimido.as_slice())
        .take(LIMITE_PAYLOAD)
        .read_to_end(&mut json)?;
    Ok(serde_json::from_slice(&json)?)
}

/// Entrega leída del disco, aún sin descifrar.
#[derive(Clone, Debug)]
pub struct EntregaLeida {
    pub manifiesto: Manifiesto,
    pub manifiesto_json: String,
    pub payload: Vec<u8>,
    pub firma: String,
    pub archivo_sha256: String,
}

impl EntregaLeida {
    /// ¿La firma de la app corresponde a la llave declarada en el manifiesto?
    pub fn firma_valida(&self) -> bool {
        de_b64_32(&self.manifiesto.app.llave)
            .map(|p| {
                verificar(
                    &p,
                    CONTEXTO_FIRMA,
                    self.manifiesto_json.as_bytes(),
                    &self.firma,
                )
            })
            .unwrap_or(false)
    }

    pub fn payload_integro(&self) -> bool {
        sha256_hex(&self.payload) == self.manifiesto.payload_sha256
    }
}

pub fn escribir(manifiesto_json: &str, payload: &[u8], firma: &str) -> Resultado<Vec<u8>> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opciones =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("manifiesto.json", opciones)?;
    zip.write_all(manifiesto_json.as_bytes())?;
    zip.start_file("payload.bin", opciones)?;
    zip.write_all(payload)?;
    zip.start_file("firma.sig", opciones)?;
    zip.write_all(firma.as_bytes())?;
    Ok(zip.finish()?.into_inner())
}

pub fn leer(bytes: &[u8]) -> Resultado<EntregaLeida> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| Error::Formato("no es un archivo .rlp".into()))?;
    let mut leer_entrada = |nombre: &str| -> Resultado<Vec<u8>> {
        let archivo = zip
            .by_name(nombre)
            .map_err(|_| Error::Alterado(format!("falta {nombre}")))?;
        let mut v = Vec::new();
        archivo.take(LIMITE_PAYLOAD).read_to_end(&mut v)?;
        Ok(v)
    };
    let manifiesto_json = String::from_utf8(leer_entrada("manifiesto.json")?)
        .map_err(|_| Error::Alterado("manifiesto no UTF-8".into()))?;
    let payload = leer_entrada("payload.bin")?;
    let firma = String::from_utf8(leer_entrada("firma.sig")?)
        .map_err(|_| Error::Alterado("firma".into()))?;
    let manifiesto: Manifiesto = serde_json::from_str(&manifiesto_json)
        .map_err(|e| Error::Formato(format!("manifiesto: {e}")))?;
    if manifiesto.formato != FORMATO {
        return Err(Error::Formato(
            "no es una entrega de RealLearningProgramming".into(),
        ));
    }
    if manifiesto.version > VERSION {
        return Err(Error::Formato(
            "la entrega es de una versión más nueva de la app".into(),
        ));
    }
    Ok(EntregaLeida {
        manifiesto,
        manifiesto_json,
        payload,
        firma: firma.trim().to_string(),
        archivo_sha256: sha256_hex(bytes),
    })
}

#[cfg(feature = "firmar")]
#[allow(clippy::too_many_arguments)]
pub fn construir(
    dek: &Llave,
    perfil: &PerfilPublico,
    envolturas: &Envolturas,
    grupo: Option<&GrupoFirmado>,
    grupo_id: Option<String>,
    actividades: BTreeMap<String, EstadoActividad>,
    eventos: Vec<EventoFirmado>,
    cabezas: BTreeMap<String, Cabeza>,
) -> Resultado<Vec<u8>> {
    let app = crate::llave_app::llave_app();
    let (llaves_app, eventos) = Payload::empaquetar_eventos(eventos);
    let payload = Payload {
        perfil: perfil.clone(),
        grupo: grupo.cloned(),
        actividades,
        llaves_app,
        eventos,
    };
    let cifrado = cifrar_payload(dek, &payload)?;
    let manifiesto = Manifiesto {
        formato: FORMATO.into(),
        version: VERSION,
        perfil_id: perfil.perfil_id.clone(),
        creado: crate::modelo::ahora_ms(),
        app: InfoApp {
            version: env!("CARGO_PKG_VERSION").into(),
            llave: app.publica_b64(),
            dev: app.dev,
        },
        grupo_id,
        envolturas: envolturas.clone(),
        cabezas,
        payload_sha256: sha256_hex(&cifrado),
    };
    let manifiesto_json = serde_json::to_string_pretty(&manifiesto)?;
    let firma = app.firmar(CONTEXTO_FIRMA, manifiesto_json.as_bytes());
    escribir(&manifiesto_json, &cifrado, &firma)
}
