//! Reproducción del historial de edición.
//!
//! Las operaciones vienen del editor (CodeMirror) con posiciones en unidades UTF-16, igual que
//! las cadenas de JavaScript; aquí el texto se maneja como `Vec<u16>` para reproducirlas
//! exactamente.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::crypto::sha256_hex;
use crate::error::{Error, Resultado};
use crate::eventos::Evento;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpEdicion {
    pub dt: i64,
    pub desde: usize,
    pub hasta: usize,
    pub insertado: String,
    /// t tecleo · i automático · d borrado · u deshacer · r rehacer · p pegado permitido · o otro
    pub origen: char,
}

/// Lote de operaciones tal como llega del editor: `{t0, ops: [[dt, desde, hasta, texto, origen]]}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoteOps {
    pub t0: i64,
    pub ops: Vec<(i64, usize, usize, String, String)>,
}

impl LoteOps {
    pub fn operaciones(&self) -> Vec<OpEdicion> {
        self.ops
            .iter()
            .map(|(dt, desde, hasta, insertado, origen)| OpEdicion {
                dt: *dt,
                desde: *desde,
                hasta: *hasta,
                insertado: insertado.clone(),
                origen: origen.chars().next().unwrap_or('o'),
            })
            .collect()
    }

    pub fn desde_valor(v: &Value) -> Resultado<Self> {
        Ok(serde_json::from_value(v.clone())?)
    }
}

pub fn hash_texto(texto: &str) -> String {
    sha256_hex(texto.as_bytes())
}

/// Texto editable en unidades UTF-16.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Texto(Vec<u16>);

impl Texto {
    pub fn nuevo(texto: &str) -> Self {
        Texto(texto.encode_utf16().collect())
    }

    pub fn aplicar(&mut self, op: &OpEdicion) -> Resultado<()> {
        if op.desde > op.hasta || op.hasta > self.0.len() {
            return Err(Error::Alterado(format!(
                "operación fuera de rango ({}..{} en texto de {})",
                op.desde,
                op.hasta,
                self.0.len()
            )));
        }
        self.0
            .splice(op.desde..op.hasta, op.insertado.encode_utf16());
        Ok(())
    }

    pub fn cadena(&self) -> Resultado<String> {
        String::from_utf16(&self.0).map_err(|_| Error::Alterado("texto con UTF-16 inválido".into()))
    }
}

/// Aplica operaciones y devuelve el texto final.
pub fn aplicar(inicio: &str, ops: &[OpEdicion]) -> Resultado<String> {
    let mut t = Texto::nuevo(inicio);
    for op in ops {
        t.aplicar(op)?;
    }
    t.cadena()
}

/// Tramo del historial de una actividad en un dispositivo: empieza con un evento `base` y
/// sigue con las operaciones de edición de ese dispositivo.
#[derive(Clone, Debug)]
pub struct Segmento {
    pub dispositivo: String,
    pub actividad: String,
    /// "inicio", "reinicio" o "continuacion".
    pub motivo: String,
    pub base_texto: Option<String>,
    pub base_hash: String,
    pub ops: Vec<(i64, OpEdicion)>,
    pub t_inicio: i64,
    pub t_fin: i64,
}

/// Divide los eventos de UN dispositivo (en orden de seq) en segmentos por actividad.
pub fn segmentos(eventos: &[Evento]) -> Result<Vec<Segmento>, String> {
    use std::collections::HashMap;
    let mut abiertos: HashMap<String, usize> = HashMap::new();
    let mut lista: Vec<Segmento> = Vec::new();
    for e in eventos {
        let Some(act) = e.actividad.clone() else {
            continue;
        };
        match e.tipo.as_str() {
            "base" => {
                let motivo = e
                    .datos
                    .get("motivo")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let base_texto = e
                    .datos
                    .get("texto")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let base_hash = e
                    .datos
                    .get("hash")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if let Some(t) = &base_texto {
                    if hash_texto(t) != base_hash {
                        return Err(format!(
                            "base de '{act}' con hash inconsistente (seq {})",
                            e.seq
                        ));
                    }
                } else if motivo != "continuacion" {
                    return Err(format!("base de '{act}' sin texto (seq {})", e.seq));
                }
                abiertos.insert(act.clone(), lista.len());
                lista.push(Segmento {
                    dispositivo: e.dispositivo.clone(),
                    actividad: act,
                    motivo,
                    base_texto,
                    base_hash,
                    ops: Vec::new(),
                    t_inicio: e.t,
                    t_fin: e.t,
                });
            }
            "edicion" => {
                let Some(&i) = abiertos.get(&act) else {
                    return Err(format!(
                        "edición de '{act}' sin evento base (seq {})",
                        e.seq
                    ));
                };
                let lote = LoteOps::desde_valor(&e.datos)
                    .map_err(|x| format!("lote inválido (seq {}): {x}", e.seq))?;
                let seg = &mut lista[i];
                for op in lote.operaciones() {
                    seg.ops.push((lote.t0 + op.dt, op));
                }
                seg.t_fin = e.t;
            }
            _ => {}
        }
    }
    Ok(lista)
}

/// Resultado de reproducir un segmento.
#[derive(Clone, Debug)]
pub struct Reproduccion {
    pub texto_final: String,
    pub hash_final: String,
    /// Textos intermedios cuyo hash estaba en `buscados` (puntos de continuación).
    pub encontrados: HashMap<String, String>,
}

/// Reproduce `ops` desde `inicio`, guardando los estados intermedios cuyo hash esté en `buscados`.
pub fn reproducir(
    inicio: &str,
    ops: &[(i64, OpEdicion)],
    buscados: &HashSet<String>,
) -> Resultado<Reproduccion> {
    let mut t = Texto::nuevo(inicio);
    let mut encontrados = HashMap::new();
    let mut revisar = |texto: &str| {
        if !buscados.is_empty() {
            let h = hash_texto(texto);
            if buscados.contains(&h) {
                encontrados.insert(h, texto.to_string());
            }
        }
    };
    revisar(inicio);
    for (_, op) in ops {
        t.aplicar(op)?;
        if !buscados.is_empty() {
            if let Ok(s) = t.cadena() {
                revisar(&s);
            }
        }
    }
    let texto_final = t.cadena()?;
    Ok(Reproduccion {
        hash_final: hash_texto(&texto_final),
        texto_final,
        encontrados,
    })
}

/// Métricas del ritmo de escritura.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Ritmo {
    /// Caracteres tecleados.
    pub tecleados: u64,
    /// Caracteres insertados por otras vías (automáticos, deshacer/rehacer, pegado permitido, otros).
    pub automaticos: u64,
    pub deshacer_rehacer: u64,
    pub pegados: u64,
    pub otros: u64,
    /// Máximo de caracteres por segundo tecleados en una ventana de 5 s.
    pub max_cps: f64,
    /// Ventanas de 5 s con ritmo por encima de lo humanamente plausible.
    pub rafagas: u32,
}

pub const CPS_SOSPECHOSO: f64 = 12.0;

pub fn ritmo(ops: &[(i64, OpEdicion)]) -> Ritmo {
    let mut r = Ritmo::default();
    let mut tecleo: Vec<(i64, u64)> = Vec::new();
    for (t, op) in ops {
        let n = op.insertado.chars().filter(|c| !c.is_whitespace()).count() as u64;
        match op.origen {
            't' => {
                r.tecleados += n;
                if n > 0 {
                    tecleo.push((*t, n));
                }
            }
            'i' => r.automaticos += n,
            'u' | 'r' => r.deshacer_rehacer += n,
            'p' => r.pegados += n,
            'd' => {}
            _ => r.otros += n,
        }
    }
    tecleo.sort_by_key(|x| x.0);
    let ventana = 5_000;
    let mut inicio = 0;
    let mut suma = 0u64;
    let mut en_rafaga = false;
    for fin in 0..tecleo.len() {
        suma += tecleo[fin].1;
        while tecleo[fin].0 - tecleo[inicio].0 > ventana {
            suma -= tecleo[inicio].1;
            inicio += 1;
        }
        let cps = suma as f64 / (ventana as f64 / 1000.0);
        if cps > r.max_cps {
            r.max_cps = cps;
        }
        if cps > CPS_SOSPECHOSO {
            if !en_rafaga {
                r.rafagas += 1;
                en_rafaga = true;
            }
        } else {
            en_rafaga = false;
        }
    }
    r
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use proptest::prelude::*;

    fn op(desde: usize, hasta: usize, ins: &str) -> OpEdicion {
        OpEdicion {
            dt: 0,
            desde,
            hasta,
            insertado: ins.into(),
            origen: 't',
        }
    }

    #[test]
    fn utf16_con_acentos_y_emoji() {
        // "año 😀" en UTF-16: el emoji ocupa 2 unidades.
        let r = aplicar("año 😀", &[op(6, 6, "!"), op(0, 1, "A")]).unwrap();
        assert_eq!(r, "Año 😀!");
        assert!(aplicar("abc", &[op(2, 9, "x")]).is_err());
    }

    fn js_aplicar(texto: &str, desde: usize, hasta: usize, ins: &str) -> String {
        // Referencia: semántica de String.prototype.slice en JS.
        let u: Vec<u16> = texto.encode_utf16().collect();
        let mut v = u[..desde].to_vec();
        v.extend(ins.encode_utf16());
        v.extend_from_slice(&u[hasta..]);
        String::from_utf16_lossy(&v)
    }

    proptest! {
        #[test]
        fn coincide_con_semantica_js(base in "[a-zñá😀 \n]{0,20}", cambios in prop::collection::vec((0usize..30, 0usize..5, "[a-zé:() \n]{0,4}"), 0..20)) {
            let mut esperado = base.clone();
            let mut ops = Vec::new();
            for (a, largo, ins) in cambios {
                let n = esperado.encode_utf16().count();
                let desde = a.min(n);
                let hasta = (desde + largo).min(n);
                // No partir pares sustitutos (el editor nunca lo hace).
                let u: Vec<u16> = esperado.encode_utf16().collect();
                let bordes_ok = |i: usize| i == 0 || i == u.len() || !(0xDC00..=0xDFFF).contains(&u[i]);
                if !bordes_ok(desde) || !bordes_ok(hasta) { continue; }
                esperado = js_aplicar(&esperado, desde, hasta, &ins);
                ops.push(op(desde, hasta, &ins));
            }
            prop_assert_eq!(aplicar(&base, &ops).unwrap(), esperado);
        }
    }

    #[test]
    fn ritmo_detecta_rafagas() {
        let humano: Vec<(i64, OpEdicion)> = (0..100).map(|i| (i * 250, op(0, 0, "a"))).collect();
        let r = ritmo(&humano);
        assert_eq!(r.tecleados, 100);
        assert!(r.max_cps <= 5.0, "{}", r.max_cps);
        assert_eq!(r.rafagas, 0);
        let robot: Vec<(i64, OpEdicion)> = (0..200).map(|i| (i * 20, op(0, 0, "a"))).collect();
        let r = ritmo(&robot);
        assert!(r.max_cps > CPS_SOSPECHOSO);
        assert_eq!(r.rafagas, 1);
    }
}
