//! Primitivas criptográficas.
//!
//! - AES-256-GCM para cifrar datos (nonce aleatorio de 96 bits antepuesto al texto cifrado).
//! - Argon2id para derivar llaves de contraseñas y códigos de recuperación.
//! - X25519 + HKDF-SHA256 + AES-GCM para envolver la llave de datos hacia el profesor.
//! - Ed25519 para firmas (app, profesor).

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroizing;

use crate::error::{Error, Resultado};

pub type Llave = [u8; 32];

pub fn aleatorio<const N: usize>() -> [u8; N] {
    let mut b = [0u8; N];
    OsRng.fill_bytes(&mut b);
    b
}

pub fn b64(datos: &[u8]) -> String {
    STANDARD.encode(datos)
}

pub fn de_b64(texto: &str) -> Resultado<Vec<u8>> {
    STANDARD
        .decode(texto.trim())
        .map_err(|_| Error::Formato("base64 inválido".into()))
}

pub fn de_b64_32(texto: &str) -> Resultado<[u8; 32]> {
    de_b64(texto)?
        .try_into()
        .map_err(|_| Error::Formato("se esperaban 32 bytes".into()))
}

pub fn sha256(datos: &[u8]) -> [u8; 32] {
    Sha256::digest(datos).into()
}

pub fn sha256_hex(datos: &[u8]) -> String {
    hex::encode(sha256(datos))
}

// ------------------------------------------------------------------ AES-256-GCM

pub fn cifrar(llave: &Llave, aad: &[u8], datos: &[u8]) -> Vec<u8> {
    let cifrador = Aes256Gcm::new(llave.into());
    let nonce: [u8; 12] = aleatorio();
    let mut salida = nonce.to_vec();
    let ct = cifrador
        .encrypt(Nonce::from_slice(&nonce), Payload { msg: datos, aad })
        .expect("AES-GCM no falla al cifrar");
    salida.extend_from_slice(&ct);
    salida
}

pub fn descifrar(llave: &Llave, aad: &[u8], datos: &[u8]) -> Resultado<Vec<u8>> {
    if datos.len() < 12 + 16 {
        return Err(Error::Alterado("bloque cifrado incompleto".into()));
    }
    let (nonce, ct) = datos.split_at(12);
    Aes256Gcm::new(llave.into())
        .decrypt(Nonce::from_slice(nonce), Payload { msg: ct, aad })
        .map_err(|_| Error::Alterado("no se pudo descifrar".into()))
}

// ------------------------------------------------------------------ Argon2id

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParametrosKdf {
    /// Memoria en KiB.
    pub m: u32,
    /// Iteraciones.
    pub t: u32,
    /// Paralelismo.
    pub p: u32,
}

impl ParametrosKdf {
    /// 32 MiB, 3 pasadas: ~0.1–0.3 s en una PC de laboratorio o un celular de gama media.
    pub fn estandar() -> Self {
        Self {
            m: 32 * 1024,
            t: 3,
            p: 1,
        }
    }

    /// Solo para pruebas automáticas.
    pub fn rapidos() -> Self {
        Self { m: 256, t: 1, p: 1 }
    }
}

pub fn derivar(secreto: &[u8], sal: &[u8], p: &ParametrosKdf) -> Resultado<Zeroizing<Llave>> {
    let params = Params::new(p.m, p.t, p.p, Some(32))
        .map_err(|e| Error::Formato(format!("parámetros KDF: {e}")))?;
    let mut llave = Zeroizing::new([0u8; 32]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(secreto, sal, llave.as_mut())
        .map_err(|e| Error::Formato(format!("KDF: {e}")))?;
    Ok(llave)
}

/// Llave de datos envuelta con un secreto humano (contraseña o código de recuperación).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvolturaSecreto {
    pub sal: String,
    pub kdf: ParametrosKdf,
    pub llave: String,
}

pub fn envolver_con_secreto(
    dek: &Llave,
    secreto: &str,
    kdf: &ParametrosKdf,
    aad: &[u8],
) -> Resultado<EnvolturaSecreto> {
    let sal: [u8; 16] = aleatorio();
    let kek = derivar(secreto.as_bytes(), &sal, kdf)?;
    Ok(EnvolturaSecreto {
        sal: b64(&sal),
        kdf: kdf.clone(),
        llave: b64(&cifrar(&kek, aad, dek)),
    })
}

pub fn abrir_con_secreto(
    env: &EnvolturaSecreto,
    secreto: &str,
    aad: &[u8],
) -> Resultado<Zeroizing<Llave>> {
    let kek = derivar(secreto.as_bytes(), &de_b64(&env.sal)?, &env.kdf)?;
    let dek = descifrar(&kek, aad, &de_b64(&env.llave)?).map_err(|_| Error::Credenciales)?;
    let llave: Llave = dek
        .as_slice()
        .try_into()
        .map_err(|_| Error::Formato("llave de datos".into()))?;
    Ok(Zeroizing::new(llave))
}

// ------------------------------------------------------------------ X25519 (hacia el profesor)

/// Llave de datos envuelta para un destinatario X25519 (ECIES).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvolturaPublica {
    pub destinatario: String,
    pub efimera: String,
    pub llave: String,
}

fn kek_x25519(
    compartido: &[u8; 32],
    efimera: &[u8; 32],
    destinatario: &[u8; 32],
) -> Zeroizing<Llave> {
    let mut sal = Vec::with_capacity(64);
    sal.extend_from_slice(efimera);
    sal.extend_from_slice(destinatario);
    let hk = Hkdf::<Sha256>::new(Some(&sal), compartido);
    let mut kek = Zeroizing::new([0u8; 32]);
    hk.expand(b"rlp-envoltura-profesor-v1", kek.as_mut())
        .expect("longitud válida");
    kek
}

pub fn envolver_para(dek: &Llave, destinatario: &[u8; 32], aad: &[u8]) -> EnvolturaPublica {
    let efimera = StaticSecret::random_from_rng(OsRng);
    let pub_efimera = PublicKey::from(&efimera);
    let compartido = efimera.diffie_hellman(&PublicKey::from(*destinatario));
    let kek = kek_x25519(compartido.as_bytes(), pub_efimera.as_bytes(), destinatario);
    EnvolturaPublica {
        destinatario: b64(destinatario),
        efimera: b64(pub_efimera.as_bytes()),
        llave: b64(&cifrar(&kek, aad, dek)),
    }
}

pub fn abrir_envoltura(
    env: &EnvolturaPublica,
    secreto: &StaticSecret,
    aad: &[u8],
) -> Resultado<Zeroizing<Llave>> {
    let destinatario = PublicKey::from(secreto);
    if b64(destinatario.as_bytes()) != env.destinatario {
        return Err(Error::Credenciales);
    }
    let efimera = de_b64_32(&env.efimera)?;
    let compartido = secreto.diffie_hellman(&PublicKey::from(efimera));
    let kek = kek_x25519(compartido.as_bytes(), &efimera, destinatario.as_bytes());
    let dek = descifrar(&kek, aad, &de_b64(&env.llave)?).map_err(|_| Error::Credenciales)?;
    let llave: Llave = dek
        .as_slice()
        .try_into()
        .map_err(|_| Error::Formato("llave de datos".into()))?;
    Ok(Zeroizing::new(llave))
}

// ------------------------------------------------------------------ Ed25519

pub fn firmar(llave: &SigningKey, contexto: &[u8], datos: &[u8]) -> String {
    let mut mensaje = Vec::with_capacity(contexto.len() + datos.len());
    mensaje.extend_from_slice(contexto);
    mensaje.extend_from_slice(datos);
    b64(&llave.sign(&mensaje).to_bytes())
}

pub fn verificar(publica: &[u8; 32], contexto: &[u8], datos: &[u8], firma_b64: &str) -> bool {
    let Ok(vk) = VerifyingKey::from_bytes(publica) else {
        return false;
    };
    let Ok(bytes) = de_b64(firma_b64) else {
        return false;
    };
    let Ok(firma) = Signature::from_slice(&bytes) else {
        return false;
    };
    let mut mensaje = Vec::with_capacity(contexto.len() + datos.len());
    mensaje.extend_from_slice(contexto);
    mensaje.extend_from_slice(datos);
    vk.verify(&mensaje, &firma).is_ok()
}

pub fn nueva_llave_firma() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

// ------------------------------------------------------------------ códigos de recuperación

const ALFABETO: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ"; // base32 de Crockford

/// Código de recuperación de 100 bits: `XXXX-XXXX-XXXX-XXXX-XXXX`.
pub fn codigo_recuperacion() -> String {
    let bytes: [u8; 20] = aleatorio();
    let simbolos: Vec<char> = bytes
        .iter()
        .map(|b| ALFABETO[(*b & 31) as usize] as char)
        .collect();
    simbolos
        .chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// Normaliza lo que el alumno teclea (minúsculas, guiones, O/0, I/L/1).
pub fn normalizar_codigo(codigo: &str) -> String {
    codigo
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| match c.to_ascii_uppercase() {
            'O' => '0',
            'I' | 'L' => '1',
            'U' => 'V',
            otro => otro,
        })
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn aead_ida_y_vuelta_y_alteracion() {
        let k: Llave = aleatorio();
        let mut c = cifrar(&k, b"aad", b"hola");
        assert_eq!(descifrar(&k, b"aad", &c).unwrap(), b"hola");
        assert!(descifrar(&k, b"otro", &c).is_err());
        let n = c.len();
        c[n - 1] ^= 1;
        assert!(descifrar(&k, b"aad", &c).is_err());
    }

    #[test]
    fn envoltura_con_contrasena() {
        let dek: Llave = aleatorio();
        let env =
            envolver_con_secreto(&dek, "secreta123", &ParametrosKdf::rapidos(), b"p1").unwrap();
        assert_eq!(*abrir_con_secreto(&env, "secreta123", b"p1").unwrap(), dek);
        assert!(matches!(
            abrir_con_secreto(&env, "otra", b"p1"),
            Err(Error::Credenciales)
        ));
    }

    #[test]
    fn envoltura_para_profesor() {
        let dek: Llave = aleatorio();
        let profe = StaticSecret::random_from_rng(OsRng);
        let otro = StaticSecret::random_from_rng(OsRng);
        let env = envolver_para(&dek, PublicKey::from(&profe).as_bytes(), b"p1");
        assert_eq!(*abrir_envoltura(&env, &profe, b"p1").unwrap(), dek);
        assert!(abrir_envoltura(&env, &otro, b"p1").is_err());
    }

    #[test]
    fn firmas() {
        let k = nueva_llave_firma();
        let f = firmar(&k, b"ctx", b"datos");
        let p = k.verifying_key().to_bytes();
        assert!(verificar(&p, b"ctx", b"datos", &f));
        assert!(!verificar(&p, b"ctx", b"datoz", &f));
        assert!(!verificar(&p, b"otro", b"datos", &f));
    }

    #[test]
    fn codigos() {
        let c = codigo_recuperacion();
        assert_eq!(c.len(), 24);
        assert_eq!(normalizar_codigo(&c.to_lowercase()), c.replace('-', ""));
        assert_eq!(normalizar_codigo("ab-O1-il"), "AB0111");
    }
}
