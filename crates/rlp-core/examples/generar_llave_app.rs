//! Genera el par de llaves con el que la App Alumno firma sus entregas.
//!
//! - La semilla privada va al secreto de CI `RLP_CLAVE_APP` (NO la subas al repositorio).
//! - La llave pública se agrega a `crates/rlp-core/llaves_app.txt` para que la App Profesor
//!   confíe en ella.
fn main() {
    use base64::Engine;
    let semilla: [u8; 32] = rlp_core::crypto::aleatorio();
    let publica = ed25519_dalek::SigningKey::from_bytes(&semilla)
        .verifying_key()
        .to_bytes();
    let b64 = base64::engine::general_purpose::STANDARD;
    println!(
        "RLP_CLAVE_APP (secreto, no publicar): {}",
        b64.encode(semilla)
    );
    println!(
        "Llave pública (agregar a llaves_app.txt): {}",
        b64.encode(publica)
    );
}
