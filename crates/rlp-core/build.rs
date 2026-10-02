// Inyecta la semilla de la llave de firma de la App Alumno (base64 de 32 bytes) ofuscada, para
// que no aparezca literal en el binario: RLP_CLAVE_APP en la app nativa y RLP_CLAVE_APP_WEB al
// compilar a WebAssembly. La semilla nativa nunca entra a la versión web, cuyo código cualquiera
// puede descargar.
use std::{env, fs, path::PathBuf};

use base64::Engine;

pub const MASCARA: [u8; 32] = [
    0x5a, 0x13, 0xc7, 0x2e, 0x91, 0x4b, 0x08, 0xf3, 0x66, 0xbd, 0x21, 0x7c, 0xe4, 0x39, 0x90, 0x0f,
    0x3a, 0xd5, 0x72, 0x18, 0xab, 0x4e, 0xc1, 0x67, 0x2d, 0x9f, 0x03, 0xb8, 0x55, 0xe2, 0x7a, 0x14,
];

fn main() {
    println!("cargo:rerun-if-env-changed=RLP_CLAVE_APP");
    println!("cargo:rerun-if-env-changed=RLP_CLAVE_APP_WEB");
    println!("cargo:rerun-if-changed=llaves_app.txt");
    let destino = PathBuf::from(env::var("OUT_DIR").unwrap()).join("semilla_app.rs");
    let variable = if env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        "RLP_CLAVE_APP_WEB"
    } else {
        "RLP_CLAVE_APP"
    };
    let semilla = env::var(variable)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(s.trim())
                .unwrap_or_else(|_| panic!("{variable} debe ser base64"));
            assert_eq!(bytes.len(), 32, "{variable} debe tener 32 bytes");
            bytes
        });
    let codigo = match semilla {
        Some(b) => {
            let ofuscada: Vec<String> = b
                .iter()
                .zip(MASCARA)
                .map(|(x, m)| format!("{:#04x}", x ^ m))
                .collect();
            format!(
                "pub const SEMILLA_OFUSCADA: Option<[u8; 32]> = Some([{}]);\n",
                ofuscada.join(", ")
            )
        }
        None => "pub const SEMILLA_OFUSCADA: Option<[u8; 32]> = None;\n".to_string(),
    };
    fs::write(destino, codigo).unwrap();
}
