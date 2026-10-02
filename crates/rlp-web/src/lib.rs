//! Núcleo de la App Alumno para el navegador (versión web/PWA).
//!
//! Hace lo mismo que los comandos de `apps/alumno/src-tauri/src/lib.rs`, pero sin archivos ni
//! carpetas: los archivos llegan y salen como base64, y cada perfil vive en memoria
//! (`DepositoMemoria`). Después de cada operación, la app pide el **diario** de cambios
//! (`diario`) y lo guarda en IndexedDB; para abrir un perfil le pasa su **instantánea**.
//!
//! Toda la interfaz es una función: `llamar(metodo, args_json) -> json`, que corre en un Web
//! Worker (`packages/nucleo-web`). Así el Worker es genérico y esto se prueba en Rust nativo.

use rlp_core::almacen::MetaPerfil;
use rlp_core::alumno::{perfiles_locales, PerfilesEnMemoria, Secreto, SesionAlumno};
use rlp_core::crypto::{b64, de_b64, ParametrosKdf};
use rlp_core::deposito::{DepositoMemoria, Lote};
use rlp_core::grupo::{desde_texto_qr, leer_grupo};
use rlp_core::modelo::{GrupoFirmado, GrupoInfo};
use rlp_core::replay::LoteOps;
use rlp_core::{Error, Resultado};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Cambios de un perfil que la app debe guardar.
#[derive(Debug, Serialize)]
pub struct Diario {
    pub perfil: String,
    pub lote: Lote,
}

pub struct Nucleo {
    sesion: Option<SesionAlumno>,
    /// Diarios de sesiones ya cerradas que la app aún no recoge.
    pendientes: Vec<Diario>,
    kdf: ParametrosKdf,
}

impl Default for Nucleo {
    fn default() -> Self {
        Self::con_kdf(ParametrosKdf::estandar())
    }
}

// ------------------------------------------------------------------ argumentos

#[derive(Deserialize)]
struct ConMetas {
    /// Metadatos de los perfiles que la app tiene guardados (meta "perfil" de cada uno).
    #[serde(default)]
    metas: Vec<MetaPerfil>,
}

#[derive(Deserialize)]
struct Registro {
    #[serde(default)]
    metas: Vec<MetaPerfil>,
    grupo: Option<GrupoFirmado>,
    numero_control: String,
    nombre: String,
    contrasena: String,
}

#[derive(Deserialize)]
struct Entrar {
    instantanea: Lote,
    secreto: Secreto,
}

#[derive(Deserialize)]
struct Restaurar {
    #[serde(default)]
    metas: Vec<MetaPerfil>,
    /// Entrega `.rlp` en base64.
    archivo: String,
    secreto: Secreto,
}

#[derive(Deserialize)]
struct Archivo {
    /// Contenido en base64.
    archivo: String,
}

#[derive(Deserialize)]
struct Texto {
    contenido: String,
}

#[derive(Deserialize)]
struct LeerAcceso {
    #[serde(default)]
    metas: Vec<MetaPerfil>,
    /// Contenido del `.rlpa` (texto).
    archivo: String,
}

#[derive(Deserialize)]
struct EntrarAcceso {
    acceso: String,
    temporal: String,
    nueva: String,
    /// Perfil guardado en este navegador…
    instantanea: Option<Lote>,
    /// …o, si no está, su último `.rlp` (base64).
    entrega: Option<String>,
    #[serde(default)]
    metas: Vec<MetaPerfil>,
}

#[derive(Deserialize)]
struct Actividad {
    id: String,
    codigo_inicial: String,
}

#[derive(Deserialize)]
struct Edicion {
    id: String,
    lote: LoteOps,
    texto: String,
}

#[derive(Deserialize)]
struct Pruebas {
    id: String,
    pasadas: u32,
    total: u32,
    puntos: u32,
}

#[derive(Deserialize)]
struct Respuesta {
    id: String,
    respuesta: String,
    correcta: bool,
    puntos: u32,
}

#[derive(Deserialize)]
struct Pista {
    id: String,
    numero: u32,
}

#[derive(Deserialize)]
struct EventoLibre {
    tipo: String,
    actividad: Option<String>,
    #[serde(default)]
    datos: Value,
}

#[derive(Deserialize)]
struct Contrasenas {
    actual: String,
    nueva: String,
}

fn leer<T: DeserializeOwned>(args: &str) -> Resultado<T> {
    serde_json::from_str(args).map_err(|e| Error::validacion(format!("Argumentos inválidos: {e}")))
}

fn valor<T: Serialize>(v: T) -> Resultado<Value> {
    Ok(serde_json::to_value(v)?)
}

fn grupo_e_info((grupo, info): (GrupoFirmado, GrupoInfo)) -> Resultado<Value> {
    Ok(json!({ "grupo": grupo, "info": info }))
}

fn sin_sesion() -> Error {
    Error::validacion("No hay una sesión iniciada.")
}

impl Nucleo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Con otros parámetros de derivación (pruebas).
    pub fn con_kdf(kdf: ParametrosKdf) -> Self {
        Nucleo {
            sesion: None,
            pendientes: Vec::new(),
            kdf,
        }
    }

    /// Ejecuta una operación. `args` es un objeto JSON; devuelve JSON o el mensaje de error.
    pub fn llamar(&mut self, metodo: &str, args: &str) -> Result<String, String> {
        let args = if args.trim().is_empty() { "{}" } else { args };
        self.despachar(metodo, args)
            .and_then(|v| Ok(serde_json::to_string(&v)?))
            .map_err(|e| e.to_string())
    }

    fn sesion(&mut self) -> Resultado<&mut SesionAlumno> {
        self.sesion.as_mut().ok_or_else(sin_sesion)
    }

    /// Cierra la sesión abierta (si hay) y guarda su último diario para la app.
    fn cerrar(&mut self) -> Resultado<()> {
        if let Some(s) = self.sesion.take() {
            let perfil = s.perfil().perfil_id.clone();
            if let Some(lote) = s.cerrar()? {
                self.pendientes.push(Diario { perfil, lote });
            }
        }
        Ok(())
    }

    /// Deja abierta la sesión y devuelve su estado (con el código de recuperación nuevo, si hay).
    fn activar(&mut self, mut s: SesionAlumno) -> Resultado<Value> {
        let mut estado = s.estado()?;
        estado.codigo_nuevo = s.tomar_codigo_nuevo();
        self.sesion = Some(s);
        valor(estado)
    }

    fn despachar(&mut self, metodo: &str, args: &str) -> Resultado<Value> {
        let kdf = self.kdf.clone();
        match metodo {
            // ---------------------------------------------------------- sin sesión
            "version" => Ok(json!({
                "version": env!("CARGO_PKG_VERSION"),
                "dev": rlp_core::llave_app::llave_app().dev,
            })),
            "perfiles" => {
                let a: ConMetas = leer(args)?;
                valor(perfiles_locales(&PerfilesEnMemoria {
                    existentes: a.metas,
                }))
            }
            "leer_grupo" => {
                let a: Archivo = leer(args)?;
                grupo_e_info(leer_grupo(&de_b64(&a.archivo)?)?)
            }
            "grupo_qr" => {
                let a: Texto = leer(args)?;
                grupo_e_info(desde_texto_qr(&a.contenido)?)
            }
            "leer_acceso" => {
                let a: LeerAcceso = leer(args)?;
                let acceso = rlp_core::acceso::leer(a.archivo.as_bytes())?;
                Ok(json!({
                    "perfil_id": acceso.perfil_id,
                    "nombre": acceso.nombre,
                    "numero_control": acceso.numero_control,
                    "profesor": acceso.profesor,
                    "perfil_local": a.metas.iter().any(|m| m.perfil.perfil_id == acceso.perfil_id),
                }))
            }

            // ---------------------------------------------------------- abrir una sesión
            "registrar" => {
                let a: Registro = leer(args)?;
                self.cerrar()?;
                let (s, codigo) = SesionAlumno::registrar_en(
                    &PerfilesEnMemoria {
                        existentes: a.metas,
                    },
                    &a.numero_control,
                    &a.nombre,
                    &a.contrasena,
                    a.grupo.as_ref(),
                    kdf,
                )?;
                let estado = self.activar(s)?;
                Ok(json!({ "estado": estado, "codigo": codigo }))
            }
            "iniciar_sesion" => {
                let a: Entrar = leer(args)?;
                self.cerrar()?;
                let dep = DepositoMemoria::desde(a.instantanea)?;
                let ubicacion = rlp_core::almacen::meta_de(&dep)?.perfil.perfil_id;
                let s = SesionAlumno::abrir_en(ubicacion, Box::new(dep), &a.secreto, kdf)?;
                self.activar(s)
            }
            "restaurar" => {
                let a: Restaurar = leer(args)?;
                self.cerrar()?;
                let s = SesionAlumno::restaurar_en(
                    &PerfilesEnMemoria {
                        existentes: a.metas,
                    },
                    &de_b64(&a.archivo)?,
                    &a.secreto,
                    kdf,
                )?;
                self.activar(s)
            }
            "entrar_con_acceso" => {
                let a: EntrarAcceso = leer(args)?;
                self.cerrar()?;
                let secreto = Secreto::Acceso {
                    archivo: a.acceso,
                    temporal: a.temporal,
                    nueva_contrasena: a.nueva,
                };
                let s = match (a.instantanea, a.entrega) {
                    (Some(inst), _) => {
                        let dep = DepositoMemoria::desde(inst)?;
                        let ubicacion = rlp_core::almacen::meta_de(&dep)?.perfil.perfil_id;
                        SesionAlumno::abrir_en(ubicacion, Box::new(dep), &secreto, kdf)?
                    }
                    (None, Some(entrega)) => SesionAlumno::restaurar_en(
                        &PerfilesEnMemoria { existentes: a.metas },
                        &de_b64(&entrega)?,
                        &secreto,
                        kdf,
                    )?,
                    (None, None) => {
                        return Err(Error::validacion(
                            "Tu perfil no está en este navegador: elige también tu último archivo .rlp.",
                        ))
                    }
                };
                self.activar(s)
            }
            "cerrar_sesion" => {
                self.cerrar()?;
                Ok(Value::Null)
            }

            // ---------------------------------------------------------- con sesión
            "estado" => valor(self.sesion()?.estado()?),
            "estadisticas" => valor(self.sesion()?.estadisticas()),
            "abrir_actividad" => {
                let a: Actividad = leer(args)?;
                valor(self.sesion()?.abrir_actividad(&a.id, &a.codigo_inicial)?)
            }
            "guardar_edicion" => {
                let a: Edicion = leer(args)?;
                valor(self.sesion()?.guardar_edicion(&a.id, &a.lote, &a.texto)?)
            }
            "reiniciar_actividad" => {
                let a: Actividad = leer(args)?;
                valor(
                    self.sesion()?
                        .reiniciar_actividad(&a.id, &a.codigo_inicial)?,
                )
            }
            "registrar_pruebas" => {
                let a: Pruebas = leer(args)?;
                valor(
                    self.sesion()?
                        .registrar_pruebas(&a.id, a.pasadas, a.total, a.puntos)?,
                )
            }
            "registrar_respuesta" => {
                let a: Respuesta = leer(args)?;
                valor(self.sesion()?.registrar_respuesta(
                    &a.id,
                    &a.respuesta,
                    a.correcta,
                    a.puntos,
                )?)
            }
            "registrar_pista" => {
                let a: Pista = leer(args)?;
                valor(self.sesion()?.registrar_pista(&a.id, a.numero)?)
            }
            "registrar_evento" => {
                let a: EventoLibre = leer(args)?;
                self.sesion()?
                    .registrar_evento(&a.tipo, a.actividad.as_deref(), a.datos)?;
                Ok(Value::Null)
            }
            "exportar" => {
                let (nombre, bytes) = self.sesion()?.exportar()?;
                Ok(json!({ "nombre": nombre, "archivo": b64(&bytes) }))
            }
            "importar_avances" => {
                let a: Archivo = leer(args)?;
                valor(self.sesion()?.importar(&de_b64(&a.archivo)?)?)
            }
            "importar_retroalimentacion" => {
                let a: Archivo = leer(args)?;
                valor(
                    self.sesion()?
                        .importar_retroalimentacion(&de_b64(&a.archivo)?)?,
                )
            }
            "cambiar_contrasena" => {
                let a: Contrasenas = leer(args)?;
                self.sesion()?.cambiar_contrasena(&a.actual, &a.nueva)?;
                Ok(Value::Null)
            }
            "unirse_grupo" => {
                let a: Archivo = leer(args)?;
                let (grupo, _) = leer_grupo(&de_b64(&a.archivo)?)?;
                let info = self.sesion()?.unirse_grupo(&grupo)?;
                grupo_e_info((grupo, info))
            }
            "unirse_grupo_qr" => {
                let a: Texto = leer(args)?;
                let (grupo, _) = desde_texto_qr(&a.contenido)?;
                let info = self.sesion()?.unirse_grupo(&grupo)?;
                grupo_e_info((grupo, info))
            }

            // ---------------------------------------------------------- persistencia
            "diario" => {
                let mut diarios = std::mem::take(&mut self.pendientes);
                if let Some(s) = self.sesion.as_mut() {
                    if let Some(lote) = s.tomar_diario() {
                        diarios.push(Diario {
                            perfil: s.perfil().perfil_id.clone(),
                            lote,
                        });
                    }
                }
                valor(diarios)
            }
            _ => Err(Error::validacion(format!(
                "Operación desconocida: {metodo}"
            ))),
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod js {
    use wasm_bindgen::prelude::*;

    /// El núcleo para JavaScript: `new Nucleo().llamar("estado", "{}")`.
    #[wasm_bindgen(js_name = Nucleo)]
    pub struct NucleoJs(super::Nucleo);

    #[wasm_bindgen(js_class = Nucleo)]
    impl NucleoJs {
        #[wasm_bindgen(constructor)]
        pub fn new() -> NucleoJs {
            NucleoJs(super::Nucleo::new())
        }

        pub fn llamar(&mut self, metodo: &str, args: &str) -> Result<String, JsError> {
            self.0.llamar(metodo, args).map_err(|e| JsError::new(&e))
        }
    }

    impl Default for NucleoJs {
        fn default() -> Self {
            Self::new()
        }
    }
}
