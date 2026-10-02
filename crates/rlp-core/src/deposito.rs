//! Dónde se guardan los datos de un perfil del alumno, separado de su lógica (`almacen.rs`).
//!
//! El depósito solo guarda filas ya cifradas: no ve contraseñas, llaves ni código en claro.
//! - [`DepositoSqlite`] (función `sqlite`): un archivo `alumno.db` por perfil (escritorio y Android).
//! - [`DepositoMemoria`]: todo en memoria, con un diario de cambios que la versión web guarda en
//!   IndexedDB después de cada operación y una instantánea para volver a cargarlo.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Resultado};

/// Evento del historial tal como se guarda: su JSON cifrado; hash y firma en claro.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilaEvento {
    pub dispositivo: String,
    pub seq: u64,
    #[serde(with = "bytes_b64")]
    pub datos: Vec<u8>,
    pub hash: String,
    pub firma: String,
    pub llave: String,
}

/// Estado de una actividad, cifrado.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilaActividad {
    pub id: String,
    #[serde(with = "bytes_b64")]
    pub datos: Vec<u8>,
}

/// Cambios que se escriben juntos (todo o nada). También sirve de instantánea completa de un
/// depósito y de diario de cambios pendientes.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lote {
    /// Pares clave → valor de la tabla `meta` (se reemplazan).
    #[serde(default)]
    pub meta: BTreeMap<String, String>,
    /// Actividades (se reemplazan).
    #[serde(default)]
    pub actividades: Vec<FilaActividad>,
    /// Eventos nuevos (no pueden existir ya).
    #[serde(default)]
    pub eventos: Vec<FilaEvento>,
}

impl Lote {
    pub fn vacio(&self) -> bool {
        self.meta.is_empty() && self.actividades.is_empty() && self.eventos.is_empty()
    }
}

pub trait Deposito: Send {
    fn meta(&self, clave: &str) -> Resultado<Option<String>>;
    /// Todos los eventos, ordenados por dispositivo y número de secuencia.
    fn eventos(&self) -> Resultado<Vec<FilaEvento>>;
    fn hash_evento(&self, dispositivo: &str, seq: u64) -> Resultado<Option<String>>;
    fn actividad(&self, id: &str) -> Resultado<Option<Vec<u8>>>;
    fn actividades(&self) -> Resultado<Vec<FilaActividad>>;
    /// Escribe el lote completo o nada.
    fn escribir(&mut self, lote: Lote) -> Resultado<()>;
    /// Cambios desde la última llamada, para depósitos que guarda la app (la versión web).
    fn tomar_diario(&mut self) -> Option<Lote> {
        None
    }
}

fn duplicado(dispositivo: &str, seq: u64) -> Error {
    Error::Almacenamiento(format!("el evento {dispositivo}#{seq} ya existe"))
}

// ---------------------------------------------------------------------------------- memoria

#[derive(Default)]
pub struct DepositoMemoria {
    meta: BTreeMap<String, String>,
    actividades: BTreeMap<String, Vec<u8>>,
    eventos: BTreeMap<(String, u64), FilaEvento>,
    diario_meta: BTreeMap<String, String>,
    diario_actividades: BTreeMap<String, Vec<u8>>,
    diario_eventos: Vec<FilaEvento>,
}

impl DepositoMemoria {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Carga una instantánea (lo que la app guardó); el diario empieza vacío.
    pub fn desde(instantanea: Lote) -> Resultado<Self> {
        let mut d = Self::nuevo();
        d.escribir(instantanea)?;
        d.tomar_diario();
        Ok(d)
    }

    /// Todo el contenido del depósito.
    pub fn instantanea(&self) -> Lote {
        Lote {
            meta: self.meta.clone(),
            actividades: self
                .actividades
                .iter()
                .map(|(id, datos)| FilaActividad {
                    id: id.clone(),
                    datos: datos.clone(),
                })
                .collect(),
            eventos: self.eventos.values().cloned().collect(),
        }
    }
}

impl Deposito for DepositoMemoria {
    fn meta(&self, clave: &str) -> Resultado<Option<String>> {
        Ok(self.meta.get(clave).cloned())
    }

    fn eventos(&self) -> Resultado<Vec<FilaEvento>> {
        Ok(self.eventos.values().cloned().collect())
    }

    fn hash_evento(&self, dispositivo: &str, seq: u64) -> Resultado<Option<String>> {
        Ok(self
            .eventos
            .get(&(dispositivo.to_string(), seq))
            .map(|e| e.hash.clone()))
    }

    fn actividad(&self, id: &str) -> Resultado<Option<Vec<u8>>> {
        Ok(self.actividades.get(id).cloned())
    }

    fn actividades(&self) -> Resultado<Vec<FilaActividad>> {
        Ok(self.instantanea().actividades)
    }

    fn escribir(&mut self, lote: Lote) -> Resultado<()> {
        // Primero se valida todo: si un evento ya existe, no se escribe nada.
        let mut claves = std::collections::BTreeSet::new();
        for e in &lote.eventos {
            let clave = (e.dispositivo.clone(), e.seq);
            if self.eventos.contains_key(&clave) || !claves.insert(clave) {
                return Err(duplicado(&e.dispositivo, e.seq));
            }
        }
        for (k, v) in lote.meta {
            self.diario_meta.insert(k.clone(), v.clone());
            self.meta.insert(k, v);
        }
        for a in lote.actividades {
            self.diario_actividades
                .insert(a.id.clone(), a.datos.clone());
            self.actividades.insert(a.id, a.datos);
        }
        for e in lote.eventos {
            self.diario_eventos.push(e.clone());
            self.eventos.insert((e.dispositivo.clone(), e.seq), e);
        }
        Ok(())
    }

    fn tomar_diario(&mut self) -> Option<Lote> {
        let lote = Lote {
            meta: std::mem::take(&mut self.diario_meta),
            actividades: std::mem::take(&mut self.diario_actividades)
                .into_iter()
                .map(|(id, datos)| FilaActividad { id, datos })
                .collect(),
            eventos: std::mem::take(&mut self.diario_eventos),
        };
        (!lote.vacio()).then_some(lote)
    }
}

// ---------------------------------------------------------------------------------- SQLite

#[cfg(feature = "sqlite")]
pub use sqlite::DepositoSqlite;

#[cfg(feature = "sqlite")]
mod sqlite {
    use std::path::Path;

    use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

    use super::*;

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

    pub struct DepositoSqlite {
        conn: Connection,
    }

    impl DepositoSqlite {
        /// Crea la base de datos de un perfil nuevo (falla si ya existe).
        pub fn crear(ruta: &Path) -> Resultado<Self> {
            if ruta.exists() {
                return Err(Error::validacion("Ya existe un perfil en esa ubicación."));
            }
            Self::conectar(ruta)
        }

        /// Abre la base de datos de un perfil existente.
        pub fn abrir(ruta: &Path) -> Resultado<Self> {
            if !ruta.is_file() {
                return Err(Error::validacion("No se encontró ese perfil."));
            }
            Self::conectar(ruta)
        }

        /// Solo lectura (listar perfiles sin abrirlos).
        pub fn leer(ruta: &Path) -> Resultado<Self> {
            Ok(Self {
                conn: Connection::open_with_flags(ruta, OpenFlags::SQLITE_OPEN_READ_ONLY)?,
            })
        }

        fn conectar(ruta: &Path) -> Resultado<Self> {
            let conn = Connection::open(ruta)?;
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.execute_batch(ESQUEMA)?;
            Ok(Self { conn })
        }
    }

    impl Deposito for DepositoSqlite {
        fn meta(&self, clave: &str) -> Resultado<Option<String>> {
            Ok(self
                .conn
                .query_row(
                    "SELECT valor FROM meta WHERE clave = ?1",
                    params![clave],
                    |r| r.get(0),
                )
                .optional()?)
        }

        fn eventos(&self) -> Resultado<Vec<FilaEvento>> {
            let mut st = self.conn.prepare(
                "SELECT dispositivo, seq, datos, hash, firma, llave FROM eventos ORDER BY dispositivo, seq",
            )?;
            let filas = st.query_map([], |r| {
                Ok(FilaEvento {
                    dispositivo: r.get(0)?,
                    seq: r.get::<_, i64>(1)? as u64,
                    datos: r.get(2)?,
                    hash: r.get(3)?,
                    firma: r.get(4)?,
                    llave: r.get(5)?,
                })
            })?;
            Ok(filas.collect::<Result<_, _>>()?)
        }

        fn hash_evento(&self, dispositivo: &str, seq: u64) -> Resultado<Option<String>> {
            Ok(self
                .conn
                .query_row(
                    "SELECT hash FROM eventos WHERE dispositivo = ?1 AND seq = ?2",
                    params![dispositivo, seq as i64],
                    |r| r.get(0),
                )
                .optional()?)
        }

        fn actividad(&self, id: &str) -> Resultado<Option<Vec<u8>>> {
            Ok(self
                .conn
                .query_row(
                    "SELECT datos FROM actividades WHERE id = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .optional()?)
        }

        fn actividades(&self) -> Resultado<Vec<FilaActividad>> {
            let mut st = self.conn.prepare("SELECT id, datos FROM actividades")?;
            let filas = st.query_map([], |r| {
                Ok(FilaActividad {
                    id: r.get(0)?,
                    datos: r.get(1)?,
                })
            })?;
            Ok(filas.collect::<Result<_, _>>()?)
        }

        fn escribir(&mut self, lote: Lote) -> Resultado<()> {
            let tx = self.conn.transaction()?;
            for (clave, valor) in &lote.meta {
                tx.execute(
                    "INSERT INTO meta (clave, valor) VALUES (?1, ?2) ON CONFLICT(clave) DO UPDATE SET valor = excluded.valor",
                    params![clave, valor],
                )?;
            }
            for a in &lote.actividades {
                tx.execute(
                    "INSERT INTO actividades (id, datos) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET datos = excluded.datos",
                    params![a.id, a.datos],
                )?;
            }
            for e in &lote.eventos {
                tx.execute(
                    "INSERT INTO eventos (dispositivo, seq, datos, hash, firma, llave) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![e.dispositivo, e.seq as i64, e.datos, e.hash, e.firma, e.llave],
                )?;
            }
            tx.commit()?;
            Ok(())
        }
    }
}

/// Bytes como base64 en JSON (la versión web pasa los lotes como JSON).
mod bytes_b64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(datos: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&crate::crypto::b64(datos))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let texto = String::deserialize(d)?;
        crate::crypto::de_b64(&texto).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn evento(disp: &str, seq: u64) -> FilaEvento {
        FilaEvento {
            dispositivo: disp.into(),
            seq,
            datos: vec![seq as u8; 3],
            hash: format!("h{seq}"),
            firma: "f".into(),
            llave: "l".into(),
        }
    }

    #[test]
    fn memoria_todo_o_nada_y_diario() {
        let mut d = DepositoMemoria::nuevo();
        let mut lote = Lote::default();
        lote.meta.insert("perfil".into(), "{}".into());
        lote.eventos = vec![evento("a", 1), evento("a", 2)];
        lote.actividades = vec![FilaActividad {
            id: "x".into(),
            datos: vec![1],
        }];
        d.escribir(lote).unwrap();

        // Un evento repetido rechaza todo el lote.
        let malo = Lote {
            eventos: vec![evento("a", 3), evento("a", 2)],
            ..Default::default()
        };
        assert!(d.escribir(malo).is_err());
        assert_eq!(d.eventos().unwrap().len(), 2);
        assert_eq!(d.hash_evento("a", 2).unwrap().as_deref(), Some("h2"));

        // El diario junta los cambios y se vacía al tomarlo; la actividad queda con su último valor.
        d.escribir(Lote {
            actividades: vec![FilaActividad {
                id: "x".into(),
                datos: vec![2],
            }],
            ..Default::default()
        })
        .unwrap();
        let diario = d.tomar_diario().unwrap();
        assert_eq!(diario.eventos.len(), 2);
        assert_eq!(diario.actividades[0].datos, vec![2]);
        assert!(d.tomar_diario().is_none());

        // La instantánea, pasada por JSON, reconstruye el mismo depósito.
        let json = serde_json::to_string(&d.instantanea()).unwrap();
        let otro = DepositoMemoria::desde(serde_json::from_str(&json).unwrap()).unwrap();
        assert_eq!(otro.instantanea(), d.instantanea());
        assert_eq!(otro.actividad("x").unwrap(), Some(vec![2]));
    }

    #[cfg(feature = "sqlite")]
    #[test]
    fn sqlite_todo_o_nada() {
        let dir = tempfile::tempdir().unwrap();
        let ruta = dir.path().join("p.db");
        assert!(DepositoSqlite::abrir(&ruta).is_err());
        let mut d = DepositoSqlite::crear(&ruta).unwrap();
        d.escribir(Lote {
            eventos: vec![evento("a", 1)],
            ..Default::default()
        })
        .unwrap();
        let malo = Lote {
            actividades: vec![FilaActividad {
                id: "x".into(),
                datos: vec![1],
            }],
            eventos: vec![evento("a", 1)],
            ..Default::default()
        };
        assert!(d.escribir(malo).is_err());
        assert_eq!(d.actividad("x").unwrap(), None);
        assert!(DepositoSqlite::crear(&ruta).is_err());
    }
}
