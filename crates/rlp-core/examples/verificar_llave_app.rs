//! Comprueba que la llave de firma con la que se compila la App Alumno (`RLP_CLAVE_APP`) sea de
//! producción y que la App Profesor confíe en ella (`llaves_app.txt` o `RLP_CLAVE_APP_PUBLICA`).
//! CI lo ejecuta antes de publicar una versión. Nunca muestra la semilla privada.
use std::process::exit;

use base64::Engine;

fn main() {
    let b64 = base64::engine::general_purpose::STANDARD;
    let semilla = std::env::var("RLP_CLAVE_APP").unwrap_or_default();
    if semilla.trim().is_empty() {
        eprintln!(
            "Falta el secreto RLP_CLAVE_APP: una versión publicada no puede firmar con la llave de desarrollo."
        );
        exit(1);
    }
    let Some(bytes) = b64
        .decode(semilla.trim())
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b).ok())
    else {
        eprintln!("RLP_CLAVE_APP debe ser la semilla de 32 bytes en base64.");
        exit(1);
    };
    let publica = b64.encode(
        ed25519_dalek::SigningKey::from_bytes(&bytes)
            .verifying_key()
            .to_bytes(),
    );
    match rlp_core::llave_app::buscar_confiable(&publica) {
        Some(llave) if !llave.dev => {
            println!(
                "Llave de firma de producción reconocida: {publica} ({})",
                llave.nombre
            )
        }
        _ => {
            eprintln!(
                "La llave pública {publica} no está en crates/rlp-core/llaves_app.txt: la App Profesor \
                 marcaría las entregas como no confiables. Agrégala antes de publicar."
            );
            exit(1);
        }
    }
}
