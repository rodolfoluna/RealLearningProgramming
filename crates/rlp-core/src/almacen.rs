//! Almacenamiento local del alumno, sobre un [`Deposito`] (SQLite o memoria):
//!
//! - `meta`: datos públicos del perfil y envolturas de la llave de datos (no son secretos).
//! - `actividades`: estado actual de cada actividad, cifrado con la llave de datos.
//! - `eventos`: historial append-only, cifrado; hash y firma en claro para verificar la cadena.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::crypto::{cifrar, de_b64_32, descifrar, Llave};
use crate::deposito::{Deposito, FilaActividad, FilaEvento, Lote};
use crate::error::{Error, Resultado};
use crate::eventos::{firma_valida, hash_encadenado, Cabeza, Evento, EventoFirmado};
use crate::modelo::{Envolturas, EstadoActividad, GrupoFirmado, PerfilPublico};

pub const ARCHIVO_BD: &str = "alumno.db";

/// Metadatos del perfil (en claro).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetaPerfil {
    pub formato: u32,
    pub perfil: PerfilPublico,
    /// Identificador de esta instalación (su cadena de eventos).
    pub dispositivo: String,
    pub envolturas: Envolturas,
    #[serde(default)]
    pub grupo: Option<GrupoFirmado>,
}

pub struct Almacen {
    dep: Box<dyn Deposito>,
    dek: Zeroizing<Llave>,
    pub meta: MetaPerfil,
    cabezas: HashMap<String, (u64, [u8; 32])>,
    eventos: Vec<Evento>,
}

fn aad_evento(perfil: &str, dispositivo: &str, seq: u64) -> Vec<u8> {
    format!("ev|{perfil}|{dispositivo}|{seq}").into_bytes()
}

fn aad_actividad(perfil: &str, id: &str) -> Vec<u8> {
    format!("act|{perfil}|{id}").into_bytes()
}

/// Lee los metadatos sin necesidad de la contraseña.
pub fn meta_de(dep: &dyn Deposito) -> Resultado<MetaPerfil> {
    let valor = dep
        .meta("perfil")?
        .ok_or_else(|| Error::Formato("perfil sin metadatos".into()))?;
    Ok(serde_json::from_str(&valor)?)
}

/// Lee los metadatos de la base de datos de un perfil sin necesidad de la contraseña.
#[cfg(feature = "sqlite")]
pub fn leer_meta(ruta: &std::path::Path) -> Resultado<MetaPerfil> {
    meta_de(&crate::deposito::DepositoSqlite::leer(ruta)?)
}

impl Almacen {
    /// Perfil nuevo en un depósito vacío.
    pub fn crear(dep: Box<dyn Deposito>, meta: MetaPerfil, dek: Llave) -> Resultado<Self> {
        let mut a = Almacen {
            dep,
            dek: Zeroizing::new(dek),
            meta,
            cabezas: HashMap::new(),
            eventos: Vec::new(),
        };
        a.guardar_meta()?;
        Ok(a)
    }

    /// Abre un perfil con su llave de datos ya desenvuelta. Verifica y descifra todo el historial.
    pub fn abrir(dep: Box<dyn Deposito>, dek: Llave) -> Resultado<Self> {
        let meta = meta_de(dep.as_ref())?;
        let mut a = Almacen {
            dep,
            dek: Zeroizing::new(dek),
            meta,
            cabezas: HashMap::new(),
            eventos: Vec::new(),
        };
        a.cargar_eventos()?;
        Ok(a)
    }

    fn cargar_eventos(&mut self) -> Resultado<()> {
        let perfil = self.meta.perfil.perfil_id.clone();
        let mut eventos = Vec::new();
        let mut cabezas: HashMap<String, (u64, [u8; 32])> = HashMap::new();
        for fila in self.dep.eventos()? {
            let FilaEvento {
                dispositivo: disp,
                seq,
                datos,
                hash: hash_hex,
                ..
            } = fila;
            let json = descifrar(&self.dek, &aad_evento(&perfil, &disp, seq), &datos)?;
            let json =
                String::from_utf8(json).map_err(|_| Error::Alterado("evento no UTF-8".into()))?;
            let (seq_ant, hash_ant) = cabezas.get(&disp).copied().unwrap_or((0, [0u8; 32]));
            let hash = hash_encadenado(&hash_ant, &json);
            if seq != seq_ant + 1 || hex::encode(hash) != hash_hex {
                return Err(Error::Alterado(format!(
                    "historial local roto en {disp}#{seq}"
                )));
            }
            cabezas.insert(disp.clone(), (seq, hash));
            eventos.push(serde_json::from_str::<Evento>(&json)?);
        }
        self.cabezas = cabezas;
        self.eventos = eventos;
        Ok(())
    }

    pub fn dek(&self) -> &Llave {
        &self.dek
    }

    fn aad_privado(&self, clave: &str) -> Vec<u8> {
        format!("privado|{}|{clave}", self.meta.perfil.perfil_id).into_bytes()
    }

    /// Guarda un dato cifrado con la llave de datos (p. ej. la retroalimentación del profesor).
    pub fn guardar_privado(&mut self, clave: &str, datos: &[u8]) -> Resultado<()> {
        let cifrado = crate::crypto::b64(&cifrar(&self.dek, &self.aad_privado(clave), datos));
        let mut lote = Lote::default();
        lote.meta.insert(format!("privado:{clave}"), cifrado);
        self.dep.escribir(lote)
    }

    pub fn leer_privado(&self, clave: &str) -> Resultado<Option<Vec<u8>>> {
        match self.dep.meta(&format!("privado:{clave}"))? {
            Some(v) => Ok(Some(descifrar(
                &self.dek,
                &self.aad_privado(clave),
                &crate::crypto::de_b64(&v)?,
            )?)),
            None => Ok(None),
        }
    }

    pub fn guardar_meta(&mut self) -> Resultado<()> {
        let mut lote = Lote::default();
        lote.meta
            .insert("perfil".into(), serde_json::to_string(&self.meta)?);
        self.dep.escribir(lote)
    }

    /// Cambios aún no guardados por la app (solo con el depósito en memoria de la versión web).
    pub fn tomar_diario(&mut self) -> Option<Lote> {
        self.dep.tomar_diario()
    }

    pub fn eventos(&self) -> &[Evento] {
        &self.eventos
    }

    pub fn cabezas(&self) -> BTreeMap<String, Cabeza> {
        self.cabezas
            .iter()
            .map(|(d, (seq, h))| {
                (
                    d.clone(),
                    Cabeza {
                        seq: *seq,
                        hash: hex::encode(h),
                    },
                )
            })
            .collect()
    }

    pub fn eventos_firmados(&self) -> Resultado<Vec<EventoFirmado>> {
        let perfil = &self.meta.perfil.perfil_id;
        let mut v = Vec::new();
        for f in self.dep.eventos()? {
            let json = String::from_utf8(descifrar(
                &self.dek,
                &aad_evento(perfil, &f.dispositivo, f.seq),
                &f.datos,
            )?)
            .map_err(|_| Error::Alterado("evento no UTF-8".into()))?;
            v.push(EventoFirmado {
                json,
                hash: f.hash,
                firma: f.firma,
                llave: f.llave,
            });
        }
        Ok(v)
    }

    fn fila_evento(dek: &Llave, perfil: &str, e: &Evento, ef: &EventoFirmado) -> FilaEvento {
        FilaEvento {
            dispositivo: e.dispositivo.clone(),
            seq: e.seq,
            datos: cifrar(
                dek,
                &aad_evento(perfil, &e.dispositivo, e.seq),
                ef.json.as_bytes(),
            ),
            hash: ef.hash.clone(),
            firma: ef.firma.clone(),
            llave: ef.llave.clone(),
        }
    }

    /// Agrega eventos a la cadena de ESTE dispositivo y, opcionalmente, actualiza actividades,
    /// todo en un solo lote.
    #[cfg(feature = "firmar")]
    pub fn agregar(
        &mut self,
        nuevos: Vec<(String, Option<String>, serde_json::Value)>,
        actividades: &[(&str, &EstadoActividad)],
    ) -> Resultado<Vec<Evento>> {
        let perfil = self.meta.perfil.perfil_id.clone();
        let disp = self.meta.dispositivo.clone();
        let (mut seq, mut hash) = self.cabezas.get(&disp).copied().unwrap_or((0, [0u8; 32]));
        let mut agregados = Vec::new();
        let mut lote = Lote::default();
        for (tipo, actividad, datos) in nuevos {
            seq += 1;
            let e = Evento {
                dispositivo: disp.clone(),
                seq,
                t: crate::modelo::ahora_ms(),
                tipo,
                actividad,
                datos,
            };
            let ef = crate::eventos::firmar_evento(&perfil, &hash, &e);
            lote.eventos
                .push(Self::fila_evento(&self.dek, &perfil, &e, &ef));
            hash = de_b64_hex(&ef.hash)?;
            agregados.push(e);
        }
        for (id, estado) in actividades {
            lote.actividades
                .push(Self::fila_actividad(&self.dek, &perfil, id, estado)?);
        }
        self.dep.escribir(lote)?;
        self.cabezas.insert(disp, (seq, hash));
        self.eventos.extend(agregados.iter().cloned());
        Ok(agregados)
    }

    /// Inserta un evento firmado de cualquier dispositivo (importación), verificando que
    /// continúe su cadena. Devuelve `Ok(false)` si ya existía idéntico.
    pub fn insertar_firmado(&mut self, ef: &EventoFirmado) -> Resultado<bool> {
        let perfil = self.meta.perfil.perfil_id.clone();
        let e = ef.evento()?;
        let (seq_ant, hash_ant) = self
            .cabezas
            .get(&e.dispositivo)
            .copied()
            .unwrap_or((0, [0u8; 32]));
        if e.seq <= seq_ant {
            return match self.dep.hash_evento(&e.dispositivo, e.seq)? {
                Some(h) if h == ef.hash => Ok(false),
                _ => Err(Error::Alterado(format!(
                    "historial en conflicto en {}#{}",
                    e.dispositivo, e.seq
                ))),
            };
        }
        let hash = hash_encadenado(&hash_ant, &ef.json);
        if e.seq != seq_ant + 1 || hex::encode(hash) != ef.hash {
            return Err(Error::Alterado(format!(
                "cadena incompleta en {}#{}",
                e.dispositivo, e.seq
            )));
        }
        if !firma_valida(&perfil, &hash, &de_b64_32(&ef.llave)?, &ef.firma) {
            return Err(Error::Alterado(format!(
                "firma inválida en {}#{}",
                e.dispositivo, e.seq
            )));
        }
        self.dep.escribir(Lote {
            eventos: vec![Self::fila_evento(&self.dek, &perfil, &e, ef)],
            ..Default::default()
        })?;
        self.cabezas.insert(e.dispositivo.clone(), (e.seq, hash));
        self.eventos.push(e);
        Ok(true)
    }

    fn fila_actividad(
        dek: &Llave,
        perfil: &str,
        id: &str,
        estado: &EstadoActividad,
    ) -> Resultado<FilaActividad> {
        Ok(FilaActividad {
            id: id.to_string(),
            datos: cifrar(
                dek,
                &aad_actividad(perfil, id),
                serde_json::to_string(estado)?.as_bytes(),
            ),
        })
    }

    pub fn guardar_actividad(&mut self, id: &str, estado: &EstadoActividad) -> Resultado<()> {
        let perfil = self.meta.perfil.perfil_id.clone();
        let fila = Self::fila_actividad(&self.dek, &perfil, id, estado)?;
        self.dep.escribir(Lote {
            actividades: vec![fila],
            ..Default::default()
        })
    }

    pub fn actividad(&self, id: &str) -> Resultado<Option<EstadoActividad>> {
        match self.dep.actividad(id)? {
            None => Ok(None),
            Some(d) => {
                let json = descifrar(
                    &self.dek,
                    &aad_actividad(&self.meta.perfil.perfil_id, id),
                    &d,
                )?;
                Ok(Some(serde_json::from_slice(&json)?))
            }
        }
    }

    pub fn actividades(&self) -> Resultado<BTreeMap<String, EstadoActividad>> {
        let mut mapa = BTreeMap::new();
        for FilaActividad { id, datos } in self.dep.actividades()? {
            let json = descifrar(
                &self.dek,
                &aad_actividad(&self.meta.perfil.perfil_id, &id),
                &datos,
            )?;
            mapa.insert(id, serde_json::from_slice(&json)?);
        }
        Ok(mapa)
    }
}

fn de_b64_hex(h: &str) -> Resultado<[u8; 32]> {
    hex::decode(h)
        .ok()
        .and_then(|v| v.try_into().ok())
        .ok_or_else(|| Error::Formato("hash inválido".into()))
}
