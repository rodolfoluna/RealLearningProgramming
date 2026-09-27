//! Línea de tiempo de una actividad para el reproductor de la App Profesor: el código se vuelve
//! a escribir tecla a tecla a partir del historial firmado de la entrega, con marcas de lo que
//! pasó mientras tanto (intentos de pegar, copias, salidas de la ventana, ejecuciones…).

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entrega::Payload;
use crate::error::Resultado;
use crate::eventos::Evento;
use crate::replay::{reproducir, segmentos, Segmento};

/// Operación con su momento absoluto: `[t, desde, hasta, insertado, origen]` (posiciones UTF-16).
pub type OpConTiempo = (i64, usize, usize, String, char);

/// Tramo escrito en un dispositivo desde un punto de partida.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tramo {
    pub dispositivo: String,
    /// "inicio", "reinicio" o "continuacion".
    pub motivo: String,
    /// Texto desde el que empieza el tramo (`None` si no se pudo reconstruir).
    pub texto_inicial: Option<String>,
    pub ops: Vec<OpConTiempo>,
    pub t_inicio: i64,
    pub t_fin: i64,
}

/// Algo que pasó en la actividad y se muestra sobre la línea de tiempo.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Marca {
    pub t: i64,
    pub tipo: String,
    pub dispositivo: String,
    #[serde(default)]
    pub datos: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LineaDeTiempo {
    pub actividad: String,
    /// Tramos en orden cronológico.
    pub tramos: Vec<Tramo>,
    /// Marcas en orden cronológico.
    pub marcas: Vec<Marca>,
    /// Código guardado en la entrega (lo que debería quedar al final).
    pub codigo_final: String,
    /// Problemas al reconstruir (historial dañado): el reproductor muestra lo que sí pudo.
    pub avisos: Vec<String>,
}

/// Eventos que no se muestran como marcas: ya forman parte de los tramos.
const SIN_MARCA: &[&str] = &["edicion", "actividad_abierta"];

/// Arma la línea de tiempo de `actividad` a partir del contenido descifrado de una entrega.
pub fn linea_de_tiempo(p: &Payload, actividad: &str) -> Resultado<LineaDeTiempo> {
    let mut por_disp: BTreeMap<String, Vec<Evento>> = BTreeMap::new();
    for e in &p.eventos {
        let ev: Evento = serde_json::from_str(&e.json)?;
        por_disp.entry(ev.dispositivo.clone()).or_default().push(ev);
    }
    let mut avisos = Vec::new();
    let mut segs: Vec<Segmento> = Vec::new();
    let mut marcas = Vec::new();
    for lista in por_disp.values_mut() {
        lista.sort_by_key(|e| e.seq);
        match segmentos(lista) {
            Ok(s) => segs.extend(s.into_iter().filter(|s| s.actividad == actividad)),
            Err(e) => avisos.push(e),
        }
        for e in lista.iter() {
            if e.actividad.as_deref() != Some(actividad) || SIN_MARCA.contains(&e.tipo.as_str()) {
                continue;
            }
            let tipo = if e.tipo == "base" {
                // Solo interesa cuando el alumno reinició su código.
                match e.datos.get("motivo").and_then(Value::as_str) {
                    Some("reinicio") => "reinicio".to_string(),
                    _ => continue,
                }
            } else {
                e.tipo.clone()
            };
            marcas.push(Marca {
                t: e.t,
                tipo,
                dispositivo: e.dispositivo.clone(),
                datos: if e.tipo == "base" {
                    Value::Null
                } else {
                    e.datos.clone()
                },
            });
        }
    }
    segs.sort_by_key(|s| s.t_inicio);
    marcas.sort_by_key(|m| m.t);

    // Los tramos que continúan en otro dispositivo parten de un texto intermedio de otro tramo.
    let buscados: HashSet<String> = segs
        .iter()
        .filter(|s| s.base_texto.is_none())
        .map(|s| s.base_hash.clone())
        .collect();
    let mut textos: HashMap<String, String> = HashMap::new();
    let mut iniciales: Vec<Option<String>> = segs.iter().map(|s| s.base_texto.clone()).collect();
    let mut hechos = vec![false; segs.len()];
    loop {
        let mut avance = false;
        for (i, s) in segs.iter().enumerate() {
            if hechos[i] {
                continue;
            }
            if iniciales[i].is_none() {
                iniciales[i] = textos.get(&s.base_hash).cloned();
            }
            let Some(inicio) = &iniciales[i] else {
                continue;
            };
            hechos[i] = true;
            avance = true;
            match reproducir(inicio, &s.ops, &buscados) {
                Ok(r) => textos.extend(r.encontrados),
                Err(e) => avisos.push(format!(
                    "tramo del dispositivo {}: {e}",
                    corto(&s.dispositivo)
                )),
            }
        }
        if !avance {
            break;
        }
    }
    for (i, s) in segs.iter().enumerate() {
        if iniciales[i].is_none() {
            avisos.push(format!(
                "un tramo del dispositivo {} continúa desde un código que no aparece en el historial",
                corto(&s.dispositivo)
            ));
        }
    }

    let tramos = segs
        .into_iter()
        .zip(iniciales)
        .map(|(s, texto_inicial)| Tramo {
            ops: s
                .ops
                .into_iter()
                .map(|(t, op)| (t, op.desde, op.hasta, op.insertado, op.origen))
                .collect(),
            dispositivo: s.dispositivo,
            motivo: s.motivo,
            texto_inicial,
            t_inicio: s.t_inicio,
            t_fin: s.t_fin,
        })
        .collect();
    Ok(LineaDeTiempo {
        actividad: actividad.to_string(),
        tramos,
        marcas,
        codigo_final: p
            .actividades
            .get(actividad)
            .map(|a| a.codigo.clone())
            .unwrap_or_default(),
        avisos,
    })
}

fn corto(id: &str) -> &str {
    &id[..8.min(id.len())]
}
