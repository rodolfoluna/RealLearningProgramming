//! Genera el par de llaves con el que la App Alumno firma sus entregas.
//!
//! - La semilla privada va al secreto de CI `RLP_CLAVE_APP` (NO la subas al repositorio), o a
//!   `RLP_CLAVE_APP_WEB` si es la llave de la versión web.
//! - La llave pública se agrega a `crates/rlp-core/llaves_app.txt` para que la App Profesor
//!   confíe en ella (con la marca `[web]` si es la de la versión web).
fn main() {
    use base64::Engine;
    let semilla: [u8; 32] = rlp_core::crypto::aleatorio();
    let publica = ed25519_dalek::SigningKey::from_bytes(&semilla)
        .verifying_key()
        .to_bytes();
    let b64 = base64::engine::general_purpose::STANDARD;
    println!(
        "Semilla para RLP_CLAVE_APP o RLP_CLAVE_APP_WEB (secreto, no publicar): {}",
        b64.encode(semilla)
    );
    println!(
        "Llave pública (agregar a llaves_app.txt): {}",
        b64.encode(publica)
    );
}
