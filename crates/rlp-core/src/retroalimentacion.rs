//! Retroalimentación del profesor al alumno (archivo `.rlpr`): calificación y comentario por
//! actividad. Un solo archivo sirve para todo el grupo: la parte de cada alumno va cifrada con
//! SU llave de datos (nadie más la puede leer) y el archivo completo va firmado por el profesor.
//! La App Alumno solo acepta la firma del profesor que creó su grupo.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[cfg(feature = "sqlite")]
use crate::bd_profesor::BdProfesor;
use crate::crypto::{b64, cifrar, de_b64, de_b64_32, descifrar, verificar, Llave};
use crate::error::{Error, Resultado};
use crate::modelo::ahora_ms;
use crate::profesor::IdentidadProfesor;

pub const FORMATO: &str = "rlp-retroalimentacion";
pub const CONTEXTO_FIRMA: &[u8] = b"rlp-retro-v1";
pub const EXTENSION: &str = "rlpr";

/// Archivo tal como viaja: contenido JSON exacto + firma Ed25519 del profesor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchivoRetroalimentacion {
    pub contenido: String,
    pub firma: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Contenido {
    formato: String,
    #[serde(default)]
    grupo_id: Option<String>,
    profesor: String,
    llave_firma: String,
    creado: i64,
    /// perfil_id → retroalimentación cifrada con la llave de datos de ese alumno (base64).
    alumnos: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct NotaActividad {
    pub calificacion: Option<f64>,
    #[serde(default)]
    pub comentario: String,
    pub actualizado: i64,
}

/// Lo que recibe un alumno.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Retroalimentacion {
    pub profesor: String,
    pub creado: i64,
    pub actividades: BTreeMap<String, NotaActividad>,
}

fn aad(perfil_id: &str) -> Vec<u8> {
    format!("retro|{perfil_id}").into_bytes()
}

/// Arma el archivo para varios alumnos: `(perfil_id, llave de datos, notas)`.
pub fn crear(
    identidad: &IdentidadProfesor,
    grupo_id: Option<&str>,
    alumnos: &[(String, Llave, BTreeMap<String, NotaActividad>)],
) -> Resultado<Vec<u8>> {
    let creado = ahora_ms();
    let mut cifrados = BTreeMap::new();
    for (perfil_id, dek, actividades) in alumnos {
        let r = Retroalimentacion {
            profesor: identidad.nombre.clone(),
            creado,
            actividades: actividades.clone(),
        };
        let json = serde_json::to_vec(&r)?;
        cifrados.insert(perfil_id.clone(), b64(&cifrar(dek, &aad(perfil_id), &json)));
    }
    let contenido = serde_json::to_string(&Contenido {
        formato: FORMATO.into(),
        grupo_id: grupo_id.map(str::to_string),
        profesor: identidad.nombre.clone(),
        llave_firma: identidad.publica_firma(),
        creado,
        alumnos: cifrados,
    })?;
    let firma = identidad.firmar(CONTEXTO_FIRMA, contenido.as_bytes());
    Ok(serde_json::to_vec_pretty(&ArchivoRetroalimentacion {
        contenido,
        firma,
    })?)
}

/// Para la App Profesor: retroalimentación de todos los alumnos calificados de un grupo (o de
/// todos). Devuelve el archivo y cuántos alumnos incluye.
#[cfg(feature = "sqlite")]
pub fn crear_desde_bd(
    identidad: &IdentidadProfesor,
    bd: &BdProfesor,
    grupo_id: Option<&str>,
) -> Resultado<(Vec<u8>, usize)> {
    let mut alumnos = Vec::new();
    for fila in bd.tablero(grupo_id)? {
        let notas: BTreeMap<String, NotaActividad> = bd
            .calificaciones(&fila.perfil_id)?
            .into_iter()
            .map(|(id, c)| {
                (
                    id,
                    NotaActividad {
                        calificacion: c.calificacion,
                        comentario: c.comentario,
                        actualizado: c.actualizado,
                    },
                )
            })
            .collect();
        if notas.is_empty() {
            continue;
        }
        let Some(m) = bd.manifiesto(fila.entrega_id)? else {
            continue;
        };
        // Solo alumnos cuya entrega este profesor puede abrir.
        let Ok(dek) = identidad.abrir_dek(&m.envolturas.profesores, &m.perfil_id) else {
            continue;
        };
        alumnos.push((fila.perfil_id.clone(), *dek, notas));
    }
    if alumnos.is_empty() {
        return Err(Error::validacion(
            "Aún no has calificado ni comentado ninguna actividad de estos alumnos.",
        ));
    }
    let n = alumnos.len();
    Ok((crear(identidad, grupo_id, &alumnos)?, n))
}

/// Para la App Alumno: verifica la firma (debe ser la del profesor del grupo) y descifra la
/// parte de este alumno.
pub fn abrir(
    bytes: &[u8],
    perfil_id: &str,
    dek: &Llave,
    llave_firma_grupo: &str,
) -> Resultado<Retroalimentacion> {
    let archivo: ArchivoRetroalimentacion = serde_json::from_slice(bytes)
        .map_err(|_| Error::Formato("no es un archivo de retroalimentación".into()))?;
    let c: Contenido = serde_json::from_str(&archivo.contenido)
        .map_err(|_| Error::Formato("no es un archivo de retroalimentación".into()))?;
    if c.formato != FORMATO {
        return Err(Error::Formato(
            "no es un archivo de retroalimentación".into(),
        ));
    }
    if c.llave_firma != llave_firma_grupo {
        return Err(Error::validacion(
            "Esta retroalimentación no es del profesor de tu grupo.",
        ));
    }
    if !verificar(
        &de_b64_32(&c.llave_firma)?,
        CONTEXTO_FIRMA,
        archivo.contenido.as_bytes(),
        &archivo.firma,
    ) {
        return Err(Error::Alterado(
            "la firma de la retroalimentación no es válida".into(),
        ));
    }
    let cifrado = c
        .alumnos
        .get(perfil_id)
        .ok_or_else(|| Error::validacion("Este archivo no trae retroalimentación para ti."))?;
    let json = descifrar(dek, &aad(perfil_id), &de_b64(cifrado)?)
        .map_err(|_| Error::Alterado("tu parte de la retroalimentación no se pudo abrir".into()))?;
    let r: Retroalimentacion = serde_json::from_slice(&json)?;
    if r.creado != c.creado {
        return Err(Error::Alterado("retroalimentación inconsistente".into()));
    }
    Ok(r)
}
