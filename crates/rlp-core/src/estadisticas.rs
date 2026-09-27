//! Estadísticas calculadas a partir del historial (la única fuente de verdad).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::eventos::Evento;
use crate::replay::LoteOps;

/// Pausas más largas que esto no cuentan como tiempo de trabajo.
const PAUSA_MAXIMA_MS: i64 = 2 * 60 * 1000;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contadores {
    pub tiempo_ms: u64,
    pub ejecuciones: u32,
    pub errores: u32,
    pub pruebas: u32,
    pub pruebas_exitosas: u32,
    pub copias: u32,
    pub pegados_intentos: u32,
    pub pegados_permitidos: u32,
    pub inserciones_sospechosas: u32,
    pub salidas: u32,
    pub tiempo_fuera_ms: u64,
    pub pistas: u32,
    pub teclas: u32,
    pub ejecutables: u32,
}

impl Contadores {
    fn sumar(&mut self, e: &Evento) {
        let d = &e.datos;
        let b = |k: &str| d.get(k).and_then(Value::as_bool).unwrap_or(false);
        match e.tipo.as_str() {
            "ejecucion" => {
                self.ejecuciones += 1;
                if d.get("estado").and_then(Value::as_str) == Some("error") {
                    self.errores += 1;
                }
            }
            "prueba" => {
                self.pruebas += 1;
                let pasadas = d.get("pasadas").and_then(Value::as_u64).unwrap_or(0);
                let total = d.get("total").and_then(Value::as_u64).unwrap_or(0);
                if total > 0 && pasadas == total {
                    self.pruebas_exitosas += 1;
                }
            }
            "copia" => self.copias += 1,
            "pegado" => {
                self.pegados_intentos += 1;
                if b("permitido") {
                    self.pegados_permitidos += 1;
                }
            }
            "insercion_sospechosa" => self.inserciones_sospechosas += 1,
            "foco" => {
                if d.get("estado").and_then(Value::as_str) == Some("perdido") {
                    self.salidas += 1;
                } else if let Some(ms) = d.get("fuera_ms").and_then(Value::as_u64) {
                    self.tiempo_fuera_ms += ms;
                }
            }
            "pista" => self.pistas += 1,
            "ejecutable" => self.ejecutables += 1,
            "edicion" => {
                if let Ok(lote) = LoteOps::desde_valor(d) {
                    self.teclas += lote
                        .operaciones()
                        .iter()
                        .filter(|o| o.origen == 't')
                        .count() as u32;
                }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Estadisticas {
    pub global: Contadores,
    pub por_actividad: BTreeMap<String, Contadores>,
}

/// Calcula estadísticas. `eventos` puede mezclar dispositivos; el tiempo se mide dentro de
/// cada dispositivo entre eventos consecutivos.
pub fn calcular(eventos: &[Evento]) -> Estadisticas {
    let mut est = Estadisticas::default();
    let mut por_disp: BTreeMap<&str, Vec<&Evento>> = BTreeMap::new();
    for e in eventos {
        por_disp.entry(e.dispositivo.as_str()).or_default().push(e);
    }
    for (_, mut lista) in por_disp {
        lista.sort_by_key(|e| e.seq);
        let mut anterior: Option<&Evento> = None;
        for e in lista {
            est.global.sumar(e);
            if let Some(a) = &e.actividad {
                est.por_actividad.entry(a.clone()).or_default().sumar(e);
            }
            if let Some(prev) = anterior {
                let dt = e.t - prev.t;
                let en_sesion = e.tipo != "sesion_inicio";
                let estaba_fuera = prev.tipo == "foco"
                    && prev.datos.get("estado").and_then(Value::as_str) == Some("perdido");
                if en_sesion && !estaba_fuera && dt > 0 && dt < PAUSA_MAXIMA_MS {
                    if let Some(a) = &prev.actividad {
                        est.global.tiempo_ms += dt as u64;
                        est.por_actividad.entry(a.clone()).or_default().tiempo_ms += dt as u64;
                    }
                }
            }
            anterior = Some(e);
        }
    }
    est
}
