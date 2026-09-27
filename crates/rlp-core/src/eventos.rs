//! Historial de eventos: cadena de hashes por dispositivo, firmada evento por evento.
//!
//! `hash_i = SHA-256(hash_{i-1} ‖ json_i)` con `hash_0 = 32 ceros`, y
//! `firma_i = Ed25519(llave_app, "rlp-evento-v1" ‖ perfil_id ‖ hash_i)`.
//! El JSON se guarda y exporta tal cual (bytes exactos), así el verificador no depende de una
//! serialización canónica.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::crypto::{sha256, verificar};

pub const CONTEXTO_EVENTO: &[u8] = b"rlp-evento-v1";

/// Tipos de evento que la interfaz puede registrar directamente.
pub const TIPOS_LIBRES: &[&str] = &[
    "actividad_abierta",
    "ejecucion",
    "prueba",
    "copia",
    "pegado",
    "insercion_sospechosa",
    "foco",
    "pista",
    "respuesta",
    "ejecutable",
    "leccion_abierta",
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Evento {
    pub dispositivo: String,
    pub seq: u64,
    /// Momento (epoch ms) según el reloj del dispositivo.
    pub t: i64,
    pub tipo: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actividad: Option<String>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub datos: Value,
}

/// Evento tal como se almacena y exporta.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventoFirmado {
    /// JSON exacto del evento.
    pub json: String,
    /// Hash de la cadena en hexadecimal.
    pub hash: String,
    /// Firma Ed25519 (base64).
    pub firma: String,
    /// Llave pública de la app que firmó (base64).
    pub llave: String,
}

impl EventoFirmado {
    pub fn evento(&self) -> serde_json::Result<Evento> {
        serde_json::from_str(&self.json)
    }
}

pub fn hash_encadenado(anterior: &[u8; 32], json: &str) -> [u8; 32] {
    let mut datos = Vec::with_capacity(32 + json.len());
    datos.extend_from_slice(anterior);
    datos.extend_from_slice(json.as_bytes());
    sha256(&datos)
}

pub fn mensaje_firma(perfil_id: &str, hash: &[u8; 32]) -> Vec<u8> {
    let mut m = Vec::with_capacity(perfil_id.len() + 33);
    m.extend_from_slice(perfil_id.as_bytes());
    m.push(0);
    m.extend_from_slice(hash);
    m
}

pub fn firma_valida(perfil_id: &str, hash: &[u8; 32], publica: &[u8; 32], firma: &str) -> bool {
    verificar(
        publica,
        CONTEXTO_EVENTO,
        &mensaje_firma(perfil_id, hash),
        firma,
    )
}

/// Cabeza de la cadena de un dispositivo.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Cabeza {
    pub seq: u64,
    pub hash: String,
}

#[cfg(feature = "firmar")]
pub fn firmar_evento(perfil_id: &str, anterior: &[u8; 32], evento: &Evento) -> EventoFirmado {
    let app = crate::llave_app::llave_app();
    let json = serde_json::to_string(evento).expect("un evento siempre se serializa");
    let hash = hash_encadenado(anterior, &json);
    EventoFirmado {
        firma: app.firmar(CONTEXTO_EVENTO, &mensaje_firma(perfil_id, &hash)),
        hash: hex::encode(hash),
        llave: app.publica_b64(),
        json,
    }
}
