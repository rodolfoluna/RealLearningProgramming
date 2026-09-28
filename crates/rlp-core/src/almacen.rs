//! Almacenamiento local del alumno: un archivo SQLite por perfil.
//!
//! - `meta`: datos públicos del perfil y envolturas de la llave de datos (no son secretos).
//! - `actividades`: estado actual de cada actividad, cifrado con la llave de datos.
//! - `eventos`: historial append-only, cifrado; hash y firma en claro para verificar la cadena.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::crypto::{cifrar, de_b64_32, descifrar, Llave};
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
    conn: Connection,
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

const ESQUEMA: &str = "
CREATE TABLE IF NOT EXISTS meta (clave TEXT PRIMARY KEY, valor TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS actividades (id TEXT PRIMARY KEY, datos BLOB NOT NULL);
CREATE TABLE IF NOT EXISTS eventos (
    dispositivo TEXT NOT NULL,
    seq INTEGER NOT NULL,
    datos BLOB NOT NULL,
    hash TEXT NOT NULL,
    firma TEXT NOT NULL,
    llave TEXT NOT NULL,
    PRIMARY KEY (dispositivo, seq)
);
";

fn conectar(ruta: &Path) -> Resultado<Connection> {
    let conn = Connection::open(ruta)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.execute_batch(ESQUEMA)?;
    Ok(conn)
}

/// Lee los metadatos sin necesidad de la contraseña.
pub fn leer_meta(ruta: &Path) -> Resultado<MetaPerfil> {
    let conn = Connection::open_with_flags(ruta, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let valor: String =
        conn.query_row("SELECT valor FROM meta WHERE clave = 'perfil'", [], |r| {
            r.get(0)
        })?;
    Ok(serde_json::from_str(&valor)?)
}

impl Almacen {
    pub fn crear(ruta: &Path, meta: MetaPerfil, dek: Llave) -> Resultado<Self> {
        if ruta.exists() {
            return Err(Error::validacion("Ya existe un perfil en esa ubicación."));
        }
        let conn = conectar(ruta)?;
        let a = Almacen {
            conn,
            dek: Zeroizing::new(dek),
            meta,
            cabezas: HashMap::new(),
            eventos: Vec::new(),
        };
        a.guardar_meta()?;
        Ok(a)
    }

    /// Abre un perfil con su llave de datos ya desenvuelta. Verifica y descifra todo el historial.
    pub fn abrir(ruta: &Path, dek: Llave) -> Resultado<Self> {
        let conn = conectar(ruta)?;
        let valor: String =
            conn.query_row("SELECT valor FROM meta WHERE clave = 'perfil'", [], |r| {
                r.get(0)
            })?;
        let meta: MetaPerfil = serde_json::from_str(&valor)?;
        let mut a = Almacen {
            conn,
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
        let mut st = self.conn.prepare(
            "SELECT dispositivo, seq, datos, hash FROM eventos ORDER BY dispositivo, seq",
        )?;
        let filas = st.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)? as u64,
                r.get::<_, Vec<u8>>(2)?,
                r.get::<_, String>(3)?,
            ))
        })?;
        let mut eventos = Vec::new();
        let mut cabezas: HashMap<String, (u64, [u8; 32])> = HashMap::new();
        for fila in filas {
            let (disp, seq, datos, hash_hex) = fila?;
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
        drop(st);
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
    pub fn guardar_privado(&self, clave: &str, datos: &[u8]) -> Resultado<()> {
        let cifrado = crate::crypto::b64(&cifrar(&self.dek, &self.aad_privado(clave), datos));
        self.conn.execute(
            "INSERT INTO meta (clave, valor) VALUES (?1, ?2) ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![format!("privado:{clave}"), cifrado],
        )?;
        Ok(())
    }

    pub fn leer_privado(&self, clave: &str) -> Resultado<Option<Vec<u8>>> {
        let valor: Option<String> = self
            .conn
            .query_row(
                "SELECT valor FROM meta WHERE clave = ?1",
                params![format!("privado:{clave}")],
                |r| r.get(0),
            )
            .optional()?;
        match valor {
            Some(v) => Ok(Some(descifrar(
                &self.dek,
                &self.aad_privado(clave),
                &crate::crypto::de_b64(&v)?,
            )?)),
            None => Ok(None),
        }
    }

    pub fn guardar_meta(&self) -> Resultado<()> {
        self.conn.execute(
            "INSERT INTO meta (clave, valor) VALUES ('perfil', ?1) ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
            params![serde_json::to_string(&self.meta)?],
        )?;
        Ok(())
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
        let mut st = self
            .conn
            .prepare("SELECT dispositivo, seq, datos, hash, firma, llave FROM eventos ORDER BY dispositivo, seq")?;
        let filas = st.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)? as u64,
                r.get::<_, Vec<u8>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })?;
        let mut v = Vec::new();
        for f in filas {
            let (disp, seq, datos, hash, firma, llave) = f?;
            let json = String::from_utf8(descifrar(
                &self.dek,
                &aad_evento(perfil, &disp, seq),
                &datos,
            )?)
            .map_err(|_| Error::Alterado("evento no UTF-8".into()))?;
            v.push(EventoFirmado {
                json,
                hash,
                firma,
                llave,
            });
        }
        Ok(v)
    }

    fn insertar(
        tx: &rusqlite::Transaction,
        dek: &Llave,
        perfil: &str,
        e: &Evento,
        ef: &EventoFirmado,
    ) -> Resultado<()> {
        let datos = cifrar(
            dek,
            &aad_evento(perfil, &e.dispositivo, e.seq),
            ef.json.as_bytes(),
        );
        tx.execute(
            "INSERT INTO eventos (dispositivo, seq, datos, hash, firma, llave) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![e.dispositivo, e.seq as i64, datos, ef.hash, ef.firma, ef.llave],
        )?;
        Ok(())
    }

    /// Agrega eventos a la cadena de ESTE dispositivo y, opcionalmente, actualiza actividades,
    /// todo en una sola transacción.
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
        let tx = self.conn.transaction()?;
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
            Self::insertar(&tx, &self.dek, &perfil, &e, &ef)?;
            hash = de_b64_hex(&ef.hash)?;
            agregados.push(e);
        }
        for (id, estado) in actividades {
            Self::escribir_actividad(&tx, &self.dek, &perfil, id, estado)?;
        }
        tx.commit()?;
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
            let existente: Option<String> = self
                .conn
                .query_row(
                    "SELECT hash FROM eventos WHERE dispositivo = ?1 AND seq = ?2",
                    params![e.dispositivo, e.seq as i64],
                    |r| r.get(0),
                )
                .optional()?;
            return match existente {
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
        let tx = self.conn.transaction()?;
        Self::insertar(&tx, &self.dek, &perfil, &e, ef)?;
        tx.commit()?;
        self.cabezas.insert(e.dispositivo.clone(), (e.seq, hash));
        self.eventos.push(e);
        Ok(true)
    }

    fn escribir_actividad(
        tx: &rusqlite::Transaction,
        dek: &Llave,
        perfil: &str,
        id: &str,
        estado: &EstadoActividad,
    ) -> Resultado<()> {
        let datos = cifrar(
            dek,
            &aad_actividad(perfil, id),
            serde_json::to_string(estado)?.as_bytes(),
        );
        tx.execute(
            "INSERT INTO actividades (id, datos) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET datos = excluded.datos",
            params![id, datos],
        )?;
        Ok(())
    }

    pub fn guardar_actividad(&mut self, id: &str, estado: &EstadoActividad) -> Resultado<()> {
        let perfil = self.meta.perfil.perfil_id.clone();
        let tx = self.conn.transaction()?;
        Self::escribir_actividad(&tx, &self.dek, &perfil, id, estado)?;
        tx.commit()?;
        Ok(())
    }

    pub fn actividad(&self, id: &str) -> Resultado<Option<EstadoActividad>> {
        let datos: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT datos FROM actividades WHERE id = ?1",
                params![id],
                |r| r.get(0),
            )
            .optional()?;
        match datos {
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
        let mut st = self.conn.prepare("SELECT id, datos FROM actividades")?;
        let filas = st.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, Vec<u8>>(1)?))
        })?;
        let mut mapa = BTreeMap::new();
        for f in filas {
            let (id, d) = f?;
            let json = descifrar(
                &self.dek,
                &aad_actividad(&self.meta.perfil.perfil_id, &id),
                &d,
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
