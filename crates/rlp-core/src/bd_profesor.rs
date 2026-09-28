//! Base de datos local de la App Profesor: grupos, entregas importadas y calificaciones.

use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::entrega::{Manifiesto, Payload};
use crate::error::Resultado;
use crate::estadisticas::{Contadores, Estadisticas};
use crate::eventos::Cabeza;
use crate::grupo::verificar_grupo;
use crate::modelo::{ahora_ms, GrupoFirmado, GrupoInfo, PerfilPublico};
use crate::verificacion::{EntregaAbierta, Nivel, Reporte};

const ESQUEMA: &str = "
CREATE TABLE IF NOT EXISTS grupos (
    grupo_id TEXT PRIMARY KEY,
    contenido TEXT NOT NULL,
    firma TEXT NOT NULL,
    creado INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS entregas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    archivo_sha256 TEXT NOT NULL UNIQUE,
    perfil_id TEXT NOT NULL,
    grupo_id TEXT,
    numero_control TEXT NOT NULL,
    nombre TEXT NOT NULL,
    recibido INTEGER NOT NULL,
    creado INTEGER NOT NULL,
    nivel TEXT NOT NULL,
    reporte TEXT NOT NULL,
    estadisticas TEXT NOT NULL,
    manifiesto TEXT NOT NULL,
    payload TEXT
);
CREATE INDEX IF NOT EXISTS entregas_perfil ON entregas(perfil_id);
CREATE TABLE IF NOT EXISTS calificaciones (
    perfil_id TEXT NOT NULL,
    actividad_id TEXT NOT NULL,
    calificacion REAL,
    comentario TEXT NOT NULL DEFAULT '',
    actualizado INTEGER NOT NULL,
    PRIMARY KEY (perfil_id, actividad_id)
);
";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResumenActividad {
    pub completada: bool,
    pub pasadas: u32,
    pub total: u32,
    pub puntos: u32,
    pub intentos: u32,
    pub pistas: u32,
    pub con_codigo: bool,
}

/// Fila del tablero: la entrega más reciente de un alumno.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilaTablero {
    pub entrega_id: i64,
    pub perfil_id: String,
    pub grupo_id: Option<String>,
    pub numero_control: String,
    pub nombre: String,
    pub recibido: i64,
    pub creado: i64,
    pub nivel: Nivel,
    pub entregas: u32,
    pub global: Contadores,
    pub actividades: BTreeMap<String, ResumenActividad>,
    /// El mismo perfil aparece con otro número de control o viceversa.
    pub alerta_identidad: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Calificacion {
    pub calificacion: Option<f64>,
    pub comentario: String,
    pub actualizado: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetalleEntrega {
    pub fila: FilaTablero,
    pub manifiesto: Manifiesto,
    pub perfil: Option<PerfilPublico>,
    pub actividades: BTreeMap<String, crate::modelo::EstadoActividad>,
    pub reporte: Reporte,
    pub estadisticas: Estadisticas,
    pub calificaciones: BTreeMap<String, Calificacion>,
    pub historial: Vec<EntradaHistorial>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntradaHistorial {
    pub entrega_id: i64,
    pub recibido: i64,
    pub creado: i64,
    pub nivel: Nivel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistroEntrega {
    pub entrega_id: i64,
    pub nueva: bool,
    pub nombre: String,
    pub numero_control: String,
    pub nivel: Nivel,
}

/// Fila del tablero + payload + JSON de reporte, manifiesto y estadísticas por actividad.
type FilaCompleta = (FilaTablero, Option<Payload>, String, String, String);

pub struct BdProfesor {
    conn: Connection,
}

fn nivel_de(s: &str) -> Nivel {
    match s {
        "verde" => Nivel::Verde,
        "amarillo" => Nivel::Amarillo,
        _ => Nivel::Rojo,
    }
}

fn texto_nivel(n: Nivel) -> &'static str {
    match n {
        Nivel::Verde => "verde",
        Nivel::Amarillo => "amarillo",
        Nivel::Rojo => "rojo",
    }
}

impl BdProfesor {
    pub fn abrir(ruta: &Path) -> Resultado<Self> {
        let conn = Connection::open(ruta)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(ESQUEMA)?;
        Ok(BdProfesor { conn })
    }

    pub fn guardar_grupo(&self, g: &GrupoFirmado, info: &GrupoInfo) -> Resultado<()> {
        self.conn.execute(
            "INSERT INTO grupos (grupo_id, contenido, firma, creado) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(grupo_id) DO UPDATE SET contenido = excluded.contenido, firma = excluded.firma",
            params![info.grupo_id, g.contenido, g.firma, info.creado],
        )?;
        Ok(())
    }

    pub fn grupos(&self) -> Resultado<Vec<GrupoInfo>> {
        let mut st = self
            .conn
            .prepare("SELECT contenido, firma FROM grupos ORDER BY creado DESC")?;
        let filas = st.query_map([], |r| {
            Ok(GrupoFirmado {
                contenido: r.get(0)?,
                firma: r.get(1)?,
            })
        })?;
        Ok(filas
            .filter_map(|f| f.ok())
            .filter_map(|g| verificar_grupo(&g).ok())
            .collect())
    }

    pub fn grupo(&self, grupo_id: &str) -> Resultado<Option<(GrupoFirmado, GrupoInfo)>> {
        let g = self
            .conn
            .query_row(
                "SELECT contenido, firma FROM grupos WHERE grupo_id = ?1",
                params![grupo_id],
                |r| {
                    Ok(GrupoFirmado {
                        contenido: r.get(0)?,
                        firma: r.get(1)?,
                    })
                },
            )
            .optional()?;
        Ok(g.and_then(|g| verificar_grupo(&g).ok().map(|i| (g, i))))
    }

    /// Cabezas de cadena más avanzadas vistas en entregas anteriores de un perfil.
    pub fn cabezas_previas(&self, perfil_id: &str) -> Resultado<BTreeMap<String, Cabeza>> {
        let mut st = self
            .conn
            .prepare("SELECT manifiesto FROM entregas WHERE perfil_id = ?1")?;
        let filas = st.query_map(params![perfil_id], |r| r.get::<_, String>(0))?;
        let mut cabezas: BTreeMap<String, Cabeza> = BTreeMap::new();
        for f in filas {
            if let Ok(m) = serde_json::from_str::<Manifiesto>(&f?) {
                for (d, c) in m.cabezas {
                    match cabezas.get(&d) {
                        Some(prev) if prev.seq >= c.seq => {}
                        _ => {
                            cabezas.insert(d, c);
                        }
                    }
                }
            }
        }
        Ok(cabezas)
    }

    pub fn registrar_entrega(&self, e: &EntregaAbierta) -> Resultado<RegistroEntrega> {
        let (nombre, numero_control) = e
            .payload
            .as_ref()
            .map(|p| (p.perfil.nombre.clone(), p.perfil.numero_control.clone()))
            .unwrap_or_else(|| ("(no se pudo abrir)".into(), "?".into()));
        if let Some(id) = self
            .conn
            .query_row(
                "SELECT id FROM entregas WHERE archivo_sha256 = ?1",
                params![e.archivo_sha256],
                |r| r.get::<_, i64>(0),
            )
            .optional()?
        {
            return Ok(RegistroEntrega {
                entrega_id: id,
                nueva: false,
                nombre,
                numero_control,
                nivel: e.reporte.nivel,
            });
        }
        self.conn.execute(
            "INSERT INTO entregas (archivo_sha256, perfil_id, grupo_id, numero_control, nombre, recibido, creado,
                                   nivel, reporte, estadisticas, manifiesto, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                e.archivo_sha256,
                e.manifiesto.perfil_id,
                e.manifiesto.grupo_id,
                numero_control,
                nombre,
                ahora_ms(),
                e.manifiesto.creado,
                texto_nivel(e.reporte.nivel),
                serde_json::to_string(&e.reporte)?,
                serde_json::to_string(&e.estadisticas)?,
                serde_json::to_string(&e.manifiesto)?,
                e.payload.as_ref().map(serde_json::to_string).transpose()?,
            ],
        )?;
        Ok(RegistroEntrega {
            entrega_id: self.conn.last_insert_rowid(),
            nueva: true,
            nombre,
            numero_control,
            nivel: e.reporte.nivel,
        })
    }

    fn fila(&self, id: i64) -> Resultado<Option<FilaCompleta>> {
        let fila = self
            .conn
            .query_row(
                "SELECT id, perfil_id, grupo_id, numero_control, nombre, recibido, creado, nivel, estadisticas,
                        payload, reporte, manifiesto
                 FROM entregas WHERE id = ?1",
                params![id],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, Option<String>>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, i64>(5)?,
                        r.get::<_, i64>(6)?,
                        r.get::<_, String>(7)?,
                        r.get::<_, String>(8)?,
                        r.get::<_, Option<String>>(9)?,
                        r.get::<_, String>(10)?,
                        r.get::<_, String>(11)?,
                    ))
                },
            )
            .optional()?;
        let Some((
            id,
            perfil_id,
            grupo_id,
            nc,
            nombre,
            recibido,
            creado,
            nivel,
            est,
            payload,
            reporte,
            manifiesto,
        )) = fila
        else {
            return Ok(None);
        };
        let est: Estadisticas = serde_json::from_str(&est).unwrap_or_default();
        let payload: Option<Payload> = payload.and_then(|p| serde_json::from_str(&p).ok());
        let actividades = payload
            .as_ref()
            .map(|p| {
                p.actividades
                    .iter()
                    .map(|(k, a)| {
                        (
                            k.clone(),
                            ResumenActividad {
                                completada: a.completada,
                                pasadas: a.pasadas,
                                total: a.total,
                                puntos: a.puntos,
                                intentos: a.intentos,
                                pistas: a.pistas,
                                con_codigo: !a.codigo.is_empty(),
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let entregas: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM entregas WHERE perfil_id = ?1",
            params![perfil_id],
            |r| r.get(0),
        )?;
        let conflictos: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM entregas WHERE (perfil_id = ?1 AND numero_control <> ?2 AND numero_control <> '?')
                                              OR (numero_control = ?2 AND perfil_id <> ?1)",
            params![perfil_id, nc],
            |r| r.get(0),
        )?;
        let fila = FilaTablero {
            entrega_id: id,
            perfil_id,
            grupo_id,
            numero_control: nc,
            nombre,
            recibido,
            creado,
            nivel: nivel_de(&nivel),
            entregas,
            global: est.global,
            actividades,
            alerta_identidad: conflictos > 0,
        };
        Ok(Some((
            fila,
            payload,
            reporte,
            manifiesto,
            serde_json::to_string(&est.por_actividad)?,
        )))
    }

    /// Entrega más reciente de cada alumno (opcionalmente de un grupo).
    pub fn tablero(&self, grupo_id: Option<&str>) -> Resultado<Vec<FilaTablero>> {
        let mut st = self.conn.prepare(
            "SELECT e.id FROM entregas e
             WHERE (?1 IS NULL OR e.grupo_id = ?1)
               AND e.id = (SELECT e2.id FROM entregas e2 WHERE e2.perfil_id = e.perfil_id
                           ORDER BY e2.creado DESC, e2.id DESC LIMIT 1)
             ORDER BY e.nombre COLLATE NOCASE",
        )?;
        let ids: Vec<i64> = st
            .query_map(params![grupo_id], |r| r.get(0))?
            .collect::<Result<_, _>>()?;
        let mut v = Vec::new();
        for id in ids {
            if let Some((f, ..)) = self.fila(id)? {
                v.push(f);
            }
        }
        Ok(v)
    }

    pub fn detalle(&self, entrega_id: i64) -> Resultado<Option<DetalleEntrega>> {
        let Some((fila, payload, reporte, manifiesto, por_act)) = self.fila(entrega_id)? else {
            return Ok(None);
        };
        let mut st = self
            .conn
            .prepare("SELECT id, recibido, creado, nivel FROM entregas WHERE perfil_id = ?1 ORDER BY creado DESC, id DESC")?;
        let historial = st
            .query_map(params![fila.perfil_id], |r| {
                Ok(EntradaHistorial {
                    entrega_id: r.get(0)?,
                    recibido: r.get(1)?,
                    creado: r.get(2)?,
                    nivel: nivel_de(&r.get::<_, String>(3)?),
                })
            })?
            .collect::<Result<_, _>>()?;
        Ok(Some(DetalleEntrega {
            calificaciones: self.calificaciones(&fila.perfil_id)?,
            estadisticas: Estadisticas {
                global: fila.global.clone(),
                por_actividad: serde_json::from_str(&por_act)?,
            },
            perfil: payload.as_ref().map(|p| p.perfil.clone()),
            actividades: payload.map(|p| p.actividades).unwrap_or_default(),
            reporte: serde_json::from_str(&reporte)?,
            manifiesto: serde_json::from_str(&manifiesto)?,
            historial,
            fila,
        }))
    }

    /// Manifiesto de una entrega (contiene las envolturas de la llave de datos del alumno).
    pub fn manifiesto(&self, entrega_id: i64) -> Resultado<Option<Manifiesto>> {
        let texto: Option<String> = self
            .conn
            .query_row(
                "SELECT manifiesto FROM entregas WHERE id = ?1",
                params![entrega_id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match texto {
            Some(t) => Some(serde_json::from_str(&t)?),
            None => None,
        })
    }

    /// Contenido descifrado de una entrega (para reproducir su historial).
    pub fn payload(&self, entrega_id: i64) -> Resultado<Option<Payload>> {
        let texto: Option<Option<String>> = self
            .conn
            .query_row(
                "SELECT payload FROM entregas WHERE id = ?1",
                params![entrega_id],
                |r| r.get(0),
            )
            .optional()?;
        Ok(match texto.flatten() {
            Some(t) => Some(serde_json::from_str(&t)?),
            None => None,
        })
    }

    pub fn calificar(
        &self,
        perfil_id: &str,
        actividad_id: &str,
        calificacion: Option<f64>,
        comentario: &str,
    ) -> Resultado<()> {
        self.conn.execute(
            "INSERT INTO calificaciones (perfil_id, actividad_id, calificacion, comentario, actualizado)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(perfil_id, actividad_id) DO UPDATE SET calificacion = excluded.calificacion,
                 comentario = excluded.comentario, actualizado = excluded.actualizado",
            params![perfil_id, actividad_id, calificacion, comentario, ahora_ms()],
        )?;
        Ok(())
    }

    pub fn calificaciones(&self, perfil_id: &str) -> Resultado<BTreeMap<String, Calificacion>> {
        let mut st = self
            .conn
            .prepare("SELECT actividad_id, calificacion, comentario, actualizado FROM calificaciones WHERE perfil_id = ?1")?;
        let filas = st.query_map(params![perfil_id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                Calificacion {
                    calificacion: r.get(1)?,
                    comentario: r.get(2)?,
                    actualizado: r.get(3)?,
                },
            ))
        })?;
        Ok(filas.collect::<Result<_, _>>()?)
    }

    /// CSV (separado por comas, UTF-8 con BOM para Excel) con avance y estadísticas del grupo.
    pub fn exportar_csv(
        &self,
        grupo_id: Option<&str>,
        actividades: &[(String, String)],
    ) -> Resultado<String> {
        let esc = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
        let mut cab = vec![
            "Número de control",
            "Nombre",
            "Integridad",
            "Actividades completadas",
            "Puntos",
            "Tiempo (min)",
            "Ejecuciones",
            "Errores",
            "Copias",
            "Intentos de pegar",
            "Salidas de ventana",
            "Pistas",
            "Entregas",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        cab.extend(actividades.iter().map(|(_, t)| t.clone()));
        let mut out = String::from("\u{feff}");
        out.push_str(&cab.iter().map(|c| esc(c)).collect::<Vec<_>>().join(","));
        out.push('\n');
        for f in self.tablero(grupo_id)? {
            let completadas = f.actividades.values().filter(|a| a.completada).count();
            let puntos: u32 = f.actividades.values().map(|a| a.puntos).sum();
            let g = &f.global;
            let mut celdas = vec![
                esc(&f.numero_control),
                esc(&f.nombre),
                esc(texto_nivel(f.nivel)),
                completadas.to_string(),
                puntos.to_string(),
                format!("{:.0}", g.tiempo_ms as f64 / 60000.0),
                g.ejecuciones.to_string(),
                g.errores.to_string(),
                g.copias.to_string(),
                g.pegados_intentos.to_string(),
                g.salidas.to_string(),
                g.pistas.to_string(),
                f.entregas.to_string(),
            ];
            for (id, _) in actividades {
                celdas.push(match f.actividades.get(id) {
                    Some(a) if a.completada => "Completada".into(),
                    Some(a) if a.total > 0 => format!("{}/{}", a.pasadas, a.total),
                    Some(_) => "En progreso".into(),
                    None => String::new(),
                });
            }
            out.push_str(&celdas.join(","));
            out.push('\n');
        }
        Ok(out)
    }
}
