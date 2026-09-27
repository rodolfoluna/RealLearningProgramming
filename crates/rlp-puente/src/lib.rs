//! Puente de entrada síncrona para el worker de Python.
//!
//! Con `SharedArrayBuffer` (WebView2 en Windows, Chromium) el worker espera la entrada con
//! `Atomics.wait`. Algunos WebView (WebKitGTK en Linux, ciertos Android) no lo ofrecen; entonces el
//! worker hace una petición XHR **síncrona** a este protocolo propio (`rlpentrada://`), que responde
//! cuando la interfaz entrega la línea escrita por el alumno (o al cancelar).
//!
//! Rutas: `/esperar` (bloquea hasta tener una línea) y `/dormir?ms=N` (para `time.sleep`).

use std::sync::Mutex;
use std::time::Duration;

use tauri::http::{Request, Response, StatusCode};
use tauri::UriSchemeResponder;

pub const ESQUEMA: &str = "rlpentrada";

#[derive(Default)]
pub struct Puente {
    estado: Mutex<Estado>,
}

#[derive(Default)]
struct Estado {
    esperando: Option<UriSchemeResponder>,
    linea: Option<String>,
}

fn respuesta(codigo: StatusCode, cuerpo: Vec<u8>) -> Response<Vec<u8>> {
    Response::builder()
        .status(codigo)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Access-Control-Allow-Origin", "*")
        .header("Cross-Origin-Resource-Policy", "cross-origin")
        .header("Cache-Control", "no-store")
        .body(cuerpo)
        .expect("respuesta válida")
}

impl Puente {
    /// Atiende una petición del protocolo `rlpentrada`.
    pub fn atender(&self, peticion: Request<Vec<u8>>, responder: UriSchemeResponder) {
        let ruta = peticion.uri().path().trim_end_matches('/').to_string();
        if ruta.ends_with("/dormir") {
            let ms = peticion
                .uri()
                .query()
                .and_then(|q| q.split('&').find_map(|p| p.strip_prefix("ms=")))
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0)
                .min(60_000);
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(ms));
                responder.respond(respuesta(StatusCode::OK, Vec::new()));
            });
            return;
        }
        if !ruta.ends_with("/esperar") {
            responder.respond(respuesta(StatusCode::NOT_FOUND, Vec::new()));
            return;
        }
        let mut e = self.estado.lock().expect("puente");
        if let Some(linea) = e.linea.take() {
            responder.respond(respuesta(StatusCode::OK, linea.into_bytes()));
        } else if let Some(anterior) = e.esperando.replace(responder) {
            // Un worker anterior quedó esperando (se reinició): se le responde fin de archivo.
            anterior.respond(respuesta(StatusCode::GONE, Vec::new()));
        }
    }

    /// Entrega la línea que escribió el alumno.
    pub fn enviar(&self, linea: String) {
        let mut e = self.estado.lock().expect("puente");
        match e.esperando.take() {
            Some(r) => r.respond(respuesta(StatusCode::OK, linea.into_bytes())),
            None => e.linea = Some(linea),
        }
    }

    /// Cancela la espera (el programa recibe fin de archivo).
    pub fn cancelar(&self) {
        let mut e = self.estado.lock().expect("puente");
        e.linea = None;
        if let Some(r) = e.esperando.take() {
            r.respond(respuesta(StatusCode::GONE, Vec::new()));
        }
    }
}
