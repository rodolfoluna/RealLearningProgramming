//! Llave de firma de la App Alumno.
//!
//! La App Alumno firma cada evento del historial y cada entrega con una llave Ed25519 incluida
//! en su binario. La llave privada de producción se inyecta al compilar con la variable
//! `RLP_CLAVE_APP` (semilla de 32 bytes en base64, guardada como secreto de CI) y nunca se sube
//! al repositorio. La versión web usa otra llave (`RLP_CLAVE_APP_WEB`), marcada `[web]` en
//! `llaves_app.txt`: su código se puede descargar, así que su firma no prueba nada por sí sola.
//! Sin esas variables se usa una llave de desarrollo pública que la App Profesor acepta, pero
//! marca como "firma de desarrollo".
//!
//! Limitación conocida: alguien con conocimientos de ingeniería inversa podría extraer la llave
//! del binario. Por eso la verificación también reproduce el historial de edición.

use crate::crypto::{de_b64_32, sha256};

/// Llave pública de confianza con la que se verifican entregas.
#[derive(Clone, Debug)]
pub struct LlaveConocida {
    pub publica: [u8; 32],
    pub nombre: String,
    pub dev: bool,
    /// Llave de la versión web (marca `[web]` en `llaves_app.txt`).
    pub web: bool,
}

const SEMILLA_DEV: &[u8] = b"rlp-llave-de-desarrollo-no-usar-en-produccion";

fn semilla_dev() -> [u8; 32] {
    sha256(SEMILLA_DEV)
}

pub fn publica_dev() -> [u8; 32] {
    ed25519_dalek::SigningKey::from_bytes(&semilla_dev())
        .verifying_key()
        .to_bytes()
}

/// Llaves públicas en las que confía la App Profesor: la de desarrollo, las listadas en
/// `llaves_app.txt` y la indicada al compilar con `RLP_CLAVE_APP_PUBLICA`.
pub fn llaves_confiables() -> Vec<LlaveConocida> {
    let mut llaves = vec![LlaveConocida {
        publica: publica_dev(),
        nombre: "desarrollo".into(),
        dev: true,
        web: false,
    }];
    agregar_lista(&mut llaves, include_str!("../llaves_app.txt"));
    if let Some(p) = option_env!("RLP_CLAVE_APP_PUBLICA") {
        agregar_lista(&mut llaves, &format!("{p} compilación actual"));
    }
    llaves
}

/// Agrega las llaves de un texto con el formato de `llaves_app.txt`.
fn agregar_lista(llaves: &mut Vec<LlaveConocida>, texto: &str) {
    for linea in texto.lines() {
        let linea = linea.trim();
        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }
        let mut partes = linea.splitn(2, char::is_whitespace);
        let llave = partes.next().unwrap_or_default();
        let nombre = partes.next().unwrap_or("versión publicada").trim();
        let Ok(publica) = de_b64_32(llave) else {
            continue;
        };
        if !llaves.iter().any(|l| l.publica == publica) {
            llaves.push(LlaveConocida {
                publica,
                nombre: nombre
                    .replace("[web]", "")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" "),
                dev: false,
                web: nombre.contains("[web]"),
            });
        }
    }
}

pub fn buscar_confiable(publica_b64: &str) -> Option<LlaveConocida> {
    let publica = de_b64_32(publica_b64).ok()?;
    llaves_confiables()
        .into_iter()
        .find(|l| l.publica == publica)
}

#[cfg(feature = "firmar")]
mod privada {
    use super::*;
    use crate::crypto::{b64, firmar};
    use ed25519_dalek::SigningKey;
    use std::sync::OnceLock;

    pub struct LlaveApp {
        llave: SigningKey,
        pub dev: bool,
    }

    impl LlaveApp {
        pub fn publica_b64(&self) -> String {
            b64(self.llave.verifying_key().as_bytes())
        }

        pub fn firmar(&self, contexto: &[u8], datos: &[u8]) -> String {
            firmar(&self.llave, contexto, datos)
        }
    }

    mod generado {
        include!(concat!(env!("OUT_DIR"), "/semilla_app.rs"));
    }

    // Debe coincidir con la máscara de build.rs.
    const MASCARA: [u8; 32] = [
        0x5a, 0x13, 0xc7, 0x2e, 0x91, 0x4b, 0x08, 0xf3, 0x66, 0xbd, 0x21, 0x7c, 0xe4, 0x39, 0x90,
        0x0f, 0x3a, 0xd5, 0x72, 0x18, 0xab, 0x4e, 0xc1, 0x67, 0x2d, 0x9f, 0x03, 0xb8, 0x55, 0xe2,
        0x7a, 0x14,
    ];

    fn semilla() -> ([u8; 32], bool) {
        match generado::SEMILLA_OFUSCADA {
            Some(ofuscada) => {
                let mut s = [0u8; 32];
                for i in 0..32 {
                    s[i] = ofuscada[i] ^ MASCARA[i];
                }
                (s, false)
            }
            None => (semilla_dev(), true),
        }
    }

    pub fn llave_app() -> &'static LlaveApp {
        static LLAVE: OnceLock<LlaveApp> = OnceLock::new();
        LLAVE.get_or_init(|| {
            let (s, dev) = semilla();
            LlaveApp {
                llave: SigningKey::from_bytes(&s),
                dev,
            }
        })
    }
}

#[cfg(feature = "firmar")]
pub use privada::{llave_app, LlaveApp};

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::crypto::b64;

    #[test]
    fn marca_web_en_la_lista() {
        let nativa = b64(&[1u8; 32]);
        let web = b64(&[2u8; 32]);
        let mut llaves = Vec::new();
        agregar_lista(
            &mut llaves,
            &format!("# comentario\n{nativa} v1 (producción)\n{web} v1 [web] (producción)\nbasura\n{nativa} repetida"),
        );
        assert_eq!(llaves.len(), 2);
        assert!(!llaves[0].web && llaves[1].web);
        assert_eq!(llaves[1].nombre, "v1 (producción)");
        // La lista publicada no tiene llaves web todavía mal marcadas como de desarrollo.
        assert!(llaves_confiables().iter().all(|l| !(l.dev && l.web)));
    }
}
