//! Comprueba que la llave de firma con la que se compila la App Alumno sea de producción y que la
//! App Profesor confíe en ella (`llaves_app.txt` o `RLP_CLAVE_APP_PUBLICA`). CI lo ejecuta antes
//! de publicar una versión. Nunca muestra la semilla privada.
//!
//!   cargo run -p rlp-core --example verificar_llave_app        # RLP_CLAVE_APP (app nativa)
//!   cargo run -p rlp-core --example verificar_llave_app -- web # RLP_CLAVE_APP_WEB (versión web)
use std::process::exit;

use base64::Engine;

fn main() {
    let b64 = base64::engine::general_purpose::STANDARD;
    let web = std::env::args().nth(1).as_deref() == Some("web");
    let variable = if web {
        "RLP_CLAVE_APP_WEB"
    } else {
        "RLP_CLAVE_APP"
    };
    let semilla = std::env::var(variable).unwrap_or_default();
    if semilla.trim().is_empty() {
        eprintln!(
            "Falta el secreto {variable}: una versión publicada no puede firmar con la llave de desarrollo."
        );
        exit(1);
    }
    let Some(bytes) = b64
        .decode(semilla.trim())
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b).ok())
    else {
        eprintln!("{variable} debe ser la semilla de 32 bytes en base64.");
        exit(1);
    };
    let publica = b64.encode(
        ed25519_dalek::SigningKey::from_bytes(&bytes)
            .verifying_key()
            .to_bytes(),
    );
    match rlp_core::llave_app::buscar_confiable(&publica) {
        // La llave web no es secreta (su código se descarga): nunca debe firmar la app nativa.
        Some(llave) if !llave.dev && llave.web != web => {
            eprintln!(
                "La llave {publica} ({}) es la de la {}: no la uses en {variable}.",
                llave.nombre,
                if llave.web {
                    "versión web"
                } else {
                    "app nativa"
                }
            );
            exit(1);
        }
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
