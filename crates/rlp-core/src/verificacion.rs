//! Verificación de entregas en la App Profesor.
//!
//! Cada revisión produce un "check" con nivel Verde / Amarillo / Rojo. El nivel de la entrega
//! es el peor de sus checks.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::crypto::de_b64_32;
use crate::entrega::{self, EntregaLeida, Manifiesto, Payload};
use crate::error::Resultado;
use crate::estadisticas::{calcular, Estadisticas};
use crate::eventos::{firma_valida, hash_encadenado, Cabeza, Evento};
use crate::llave_app::{llaves_confiables, LlaveConocida};
use crate::profesor::IdentidadProfesor;
use crate::replay::{reproducir, ritmo, segmentos, OpEdicion, Ritmo, Segmento};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Nivel {
    Verde,
    Amarillo,
    Rojo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub id: String,
    pub nombre: String,
    pub nivel: Nivel,
    pub detalle: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reporte {
    pub nivel: Nivel,
    pub checks: Vec<Check>,
    pub ritmo: Ritmo,
    pub ritmo_por_actividad: BTreeMap<String, Ritmo>,
}

impl Reporte {
    fn agregar(&mut self, id: &str, nombre: &str, nivel: Nivel, detalle: impl Into<String>) {
        self.checks.push(Check {
            id: id.into(),
            nombre: nombre.into(),
            nivel,
            detalle: detalle.into(),
        });
        if nivel > self.nivel {
            self.nivel = nivel;
        }
    }
}

/// Información adicional para verificar.
#[derive(Clone, Debug, Default)]
pub struct Contexto {
    /// Código inicial de cada actividad según el curso del profesor.
    pub codigos_iniciales: HashMap<String, String>,
    /// Cabezas de cadena vistas en entregas anteriores del mismo alumno.
    pub cabezas_previas: BTreeMap<String, Cabeza>,
    /// Momento actual (ms), para detectar fechas futuras.
    pub ahora: i64,
}

/// Entrega abierta por el profesor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntregaAbierta {
    pub manifiesto: Manifiesto,
    pub payload: Option<Payload>,
    pub reporte: Reporte,
    pub estadisticas: Estadisticas,
    pub archivo_sha256: String,
}

pub fn abrir_entrega(
    bytes: &[u8],
    identidad: &IdentidadProfesor,
    ctx: &Contexto,
) -> Resultado<EntregaAbierta> {
    let leida = entrega::leer(bytes)?;
    let mut reporte = Reporte {
        nivel: Nivel::Verde,
        checks: Vec::new(),
        ritmo: Ritmo::default(),
        ritmo_por_actividad: BTreeMap::new(),
    };
    verificar_firma(&leida, &mut reporte);
    let payload = match identidad.abrir_dek(
        &leida.manifiesto.envolturas.profesores,
        &leida.manifiesto.perfil_id,
    ) {
        Err(_) => {
            reporte.agregar(
                "descifrado",
                "Descifrado",
                Nivel::Rojo,
                "La entrega no está dirigida a este profesor (¿grupo equivocado o archivo alterado?).",
            );
            None
        }
        Ok(dek) => {
            match entrega::descifrar_payload(&dek, &leida.manifiesto.perfil_id, &leida.payload) {
                Ok(p) => {
                    reporte.agregar(
                        "descifrado",
                        "Descifrado",
                        Nivel::Verde,
                        "El contenido se descifró correctamente.",
                    );
                    Some(p)
                }
                Err(_) => {
                    reporte.agregar(
                        "descifrado",
                        "Descifrado",
                        Nivel::Rojo,
                        "El contenido cifrado fue modificado.",
                    );
                    None
                }
            }
        }
    };
    let mut estadisticas = Estadisticas::default();
    if let Some(p) = &payload {
        estadisticas = verificar_contenido(&leida.manifiesto, p, ctx, &mut reporte);
    }
    Ok(EntregaAbierta {
        manifiesto: leida.manifiesto.clone(),
        payload,
        reporte,
        estadisticas,
        archivo_sha256: leida.archivo_sha256,
    })
}

fn verificar_firma(leida: &EntregaLeida, reporte: &mut Reporte) {
    let confiables = llaves_confiables();
    let conocida = de_b64_32(&leida.manifiesto.app.llave)
        .ok()
        .and_then(|p| confiables.iter().find(|l| l.publica == p).cloned());
    if !leida.firma_valida() {
        reporte.agregar(
            "firma",
            "Firma de la app",
            Nivel::Rojo,
            "La firma no corresponde al contenido: el archivo fue modificado.",
        );
    } else {
        match conocida {
            None => reporte.agregar(
                "firma",
                "Firma de la app",
                Nivel::Rojo,
                "Firmado con una llave desconocida (no es una versión oficial de la App Alumno).",
            ),
            Some(LlaveConocida { dev: true, .. }) => reporte.agregar(
                "firma",
                "Firma de la app",
                Nivel::Amarillo,
                "Firmado con la llave de desarrollo: úsala solo para pruebas.",
            ),
            Some(l) => reporte.agregar(
                "firma",
                "Firma de la app",
                Nivel::Verde,
                format!("Firma válida ({}).", l.nombre),
            ),
        }
    }
    if leida.payload_integro() {
        reporte.agregar(
            "integridad",
            "Integridad del archivo",
            Nivel::Verde,
            "El contenido coincide con el manifiesto.",
        );
    } else {
        reporte.agregar(
            "integridad",
            "Integridad del archivo",
            Nivel::Rojo,
            "El contenido no coincide con el manifiesto.",
        );
    }
}

fn verificar_contenido(
    m: &Manifiesto,
    p: &Payload,
    ctx: &Contexto,
    reporte: &mut Reporte,
) -> Estadisticas {
    if p.perfil.perfil_id != m.perfil_id {
        reporte.agregar(
            "perfil",
            "Identidad",
            Nivel::Rojo,
            "El perfil del contenido no coincide con el del archivo.",
        );
    }

    // --- Cadena de eventos por dispositivo -------------------------------------------------
    let confiables = llaves_confiables();
    let firmados = match p.eventos_firmados() {
        Ok(v) => v,
        Err(e) => {
            reporte.agregar(
                "cadena",
                "Historial",
                Nivel::Rojo,
                format!("Historial ilegible: {e}"),
            );
            return Estadisticas::default();
        }
    };
    let mut por_disp: BTreeMap<String, Vec<(Evento, String)>> = BTreeMap::new();
    let mut problemas_cadena = Vec::new();
    let mut llave_no_confiable = false;
    for ef in &firmados {
        let e = match ef.evento() {
            Ok(e) => e,
            Err(_) => {
                problemas_cadena.push("evento ilegible".to_string());
                continue;
            }
        };
        match de_b64_32(&ef.llave) {
            Ok(pk) if confiables.iter().any(|l| l.publica == pk) => {}
            _ => llave_no_confiable = true,
        }
        por_disp
            .entry(e.dispositivo.clone())
            .or_default()
            .push((e, ef.hash.clone()));
    }
    // Recalcular hashes y firmas en orden.
    let mut firmas_invalidas = 0;
    for (disp, lista) in por_disp.iter_mut() {
        lista.sort_by_key(|(e, _)| e.seq);
        let mut anterior = [0u8; 32];
        let mut seq_esperado = 1;
        for (e, hash_declarado) in lista.iter() {
            let ef = firmados
                .iter()
                .find(|f| &f.hash == hash_declarado)
                .expect("existe");
            let h = hash_encadenado(&anterior, &ef.json);
            if e.seq != seq_esperado {
                problemas_cadena.push(format!(
                    "falta el evento {seq_esperado} del dispositivo {}",
                    corto(disp)
                ));
                break;
            }
            if hex::encode(h) != *hash_declarado {
                problemas_cadena.push(format!(
                    "el evento {} del dispositivo {} fue modificado",
                    e.seq,
                    corto(disp)
                ));
                break;
            }
            let ok = de_b64_32(&ef.llave)
                .map(|pk| firma_valida(&m.perfil_id, &h, &pk, &ef.firma))
                .unwrap_or(false);
            if !ok {
                firmas_invalidas += 1;
            }
            anterior = h;
            seq_esperado += 1;
        }
        let cabeza = m.cabezas.get(disp);
        let ultimo = lista.last().map(|(e, h)| (e.seq, h.clone()));
        if cabeza.map(|c| (c.seq, c.hash.clone())) != ultimo {
            problemas_cadena.push(format!(
                "la cabeza del dispositivo {} no coincide con el manifiesto",
                corto(disp)
            ));
        }
    }
    if m.cabezas.keys().any(|d| !por_disp.contains_key(d)) {
        problemas_cadena.push("faltan dispositivos declarados en el manifiesto".into());
    }
    if firmas_invalidas > 0 {
        problemas_cadena.push(format!("{firmas_invalidas} evento(s) con firma inválida"));
    }
    if problemas_cadena.is_empty() {
        let n: usize = por_disp.values().map(Vec::len).sum();
        let nivel = if llave_no_confiable {
            Nivel::Amarillo
        } else {
            Nivel::Verde
        };
        let extra = if llave_no_confiable {
            " Algunos eventos se firmaron con una llave no reconocida."
        } else {
            ""
        };
        reporte.agregar(
            "cadena",
            "Historial",
            nivel,
            format!(
                "{n} eventos en {} dispositivo(s), encadenados y firmados.{extra}",
                por_disp.len()
            ),
        );
    } else {
        reporte.agregar(
            "cadena",
            "Historial",
            Nivel::Rojo,
            problemas_cadena.join("; "),
        );
    }

    // --- Identidad declarada al registrarse -------------------------------------------------
    let registro = por_disp
        .values()
        .flatten()
        .find(|(e, _)| e.tipo == "registro");
    match registro {
        Some((e, _)) => {
            let nc = e
                .datos
                .get("numero_control")
                .and_then(Value::as_str)
                .unwrap_or("");
            let nombre = e.datos.get("nombre").and_then(Value::as_str).unwrap_or("");
            if nc != p.perfil.numero_control || nombre != p.perfil.nombre {
                reporte.agregar(
                    "identidad",
                    "Identidad",
                    Nivel::Rojo,
                    format!(
                        "Se registró como {nombre} ({nc}) pero la entrega dice {} ({}).",
                        p.perfil.nombre, p.perfil.numero_control
                    ),
                );
            } else {
                reporte.agregar(
                    "identidad",
                    "Identidad",
                    Nivel::Verde,
                    "Coincide con los datos del registro.",
                );
            }
        }
        None => reporte.agregar(
            "identidad",
            "Identidad",
            Nivel::Amarillo,
            "No se encontró el evento de registro.",
        ),
    }

    // --- Tiempos -------------------------------------------------------------------------------
    let ahora = if ctx.ahora > 0 {
        ctx.ahora
    } else {
        crate::modelo::ahora_ms()
    };
    let mut retrocesos = 0;
    let mut futuros = 0;
    for lista in por_disp.values() {
        for par in lista.windows(2) {
            if par[1].0.t + 5 * 60 * 1000 < par[0].0.t {
                retrocesos += 1;
            }
        }
        futuros += lista
            .iter()
            .filter(|(e, _)| e.t > ahora + 24 * 3600 * 1000)
            .count();
    }
    if retrocesos + futuros == 0 {
        reporte.agregar(
            "tiempos",
            "Fechas y horas",
            Nivel::Verde,
            "Las fechas del historial son coherentes.",
        );
    } else {
        reporte.agregar(
            "tiempos",
            "Fechas y horas",
            Nivel::Amarillo,
            format!("El reloj del equipo retrocedió {retrocesos} vez/veces y hay {futuros} evento(s) con fecha futura."),
        );
    }

    // --- Continuidad con entregas anteriores -------------------------------------------------
    if !ctx.cabezas_previas.is_empty() {
        let mut rotos = Vec::new();
        for (disp, previa) in &ctx.cabezas_previas {
            if let Some(lista) = por_disp.get(disp) {
                match lista.iter().find(|(e, _)| e.seq == previa.seq) {
                    Some((_, h)) if *h == previa.hash => {}
                    Some(_) => rotos.push(format!(
                        "el historial del dispositivo {} cambió desde la entrega anterior",
                        corto(disp)
                    )),
                    None => rotos.push(format!(
                        "el dispositivo {} tiene menos eventos que en la entrega anterior",
                        corto(disp)
                    )),
                }
            }
        }
        if rotos.is_empty() {
            reporte.agregar(
                "continuidad",
                "Continuidad",
                Nivel::Verde,
                "Continúa las entregas anteriores.",
            );
        } else {
            reporte.agregar("continuidad", "Continuidad", Nivel::Rojo, rotos.join("; "));
        }
    }

    // --- Replay del historial de edición ------------------------------------------------------
    let eventos: Vec<Evento> = por_disp
        .values()
        .flatten()
        .map(|(e, _)| e.clone())
        .collect();
    verificar_replay(&por_disp, p, ctx, reporte);

    calcular(&eventos)
}

fn corto(id: &str) -> &str {
    &id[..8.min(id.len())]
}

fn verificar_replay(
    por_disp: &BTreeMap<String, Vec<(Evento, String)>>,
    p: &Payload,
    ctx: &Contexto,
    reporte: &mut Reporte,
) {
    let mut todos: Vec<Segmento> = Vec::new();
    let mut errores = Vec::new();
    for lista in por_disp.values() {
        let eventos: Vec<Evento> = lista.iter().map(|(e, _)| e.clone()).collect();
        match segmentos(&eventos) {
            Ok(s) => todos.extend(s),
            Err(e) => errores.push(e),
        }
    }
    let mut por_actividad: BTreeMap<String, Vec<Segmento>> = BTreeMap::new();
    for s in todos {
        por_actividad
            .entry(s.actividad.clone())
            .or_default()
            .push(s);
    }
    let mut avisos = Vec::new();
    let mut revisadas = 0;
    let mut todas_ops: Vec<(i64, OpEdicion)> = Vec::new();
    for (act, segs) in &por_actividad {
        // Puntos de partida que necesitan las continuaciones (otro dispositivo siguió desde ahí).
        let buscados: HashSet<String> = segs
            .iter()
            .filter(|s| s.motivo == "continuacion")
            .map(|s| s.base_hash.clone())
            .collect();
        let mut textos: HashMap<String, String> = HashMap::new();
        let mut finales: Vec<(i64, String)> = Vec::new();
        let mut ops_act: Vec<(i64, OpEdicion)> = Vec::new();
        let mut pendientes: Vec<&Segmento> = Vec::new();
        let mut procesar = |s: &Segmento,
                            inicio: &str,
                            textos: &mut HashMap<String, String>,
                            errores: &mut Vec<String>| {
            match reproducir(inicio, &s.ops, &buscados) {
                Ok(r) => {
                    textos.extend(r.encontrados);
                    finales.push((s.t_fin, r.hash_final));
                    ops_act.extend(s.ops.iter().cloned());
                }
                Err(e) => errores.push(format!("'{act}': {e}")),
            }
        };
        for s in segs {
            match &s.base_texto {
                Some(inicio) => {
                    if let Some(oficial) = ctx.codigos_iniciales.get(act) {
                        if normalizar(oficial) != normalizar(inicio) {
                            avisos.push(format!(
                                "'{act}' empezó con un código inicial distinto al del curso"
                            ));
                        }
                    }
                    procesar(s, inicio, &mut textos, &mut errores);
                }
                None => pendientes.push(s),
            }
        }
        // Las continuaciones se resuelven cuando su punto de partida aparece en otro segmento.
        loop {
            let (listas, resto): (Vec<&Segmento>, Vec<&Segmento>) = pendientes
                .into_iter()
                .partition(|s| textos.contains_key(&s.base_hash));
            pendientes = resto;
            if listas.is_empty() {
                break;
            }
            for s in listas {
                let inicio = textos[&s.base_hash].clone();
                procesar(s, &inicio, &mut textos, &mut errores);
            }
        }
        for s in pendientes {
            errores.push(format!(
                "'{act}' continuó desde un código que no aparece en el historial (dispositivo {})",
                corto(&s.dispositivo)
            ));
        }
        // El código guardado debe ser el final del segmento más reciente.
        if let Some(estado) = p.actividades.get(act) {
            let hash_actual = crate::replay::hash_texto(&estado.codigo);
            finales.sort_by_key(|(t, _)| *t);
            match finales.last() {
                Some((_, h)) if *h == hash_actual => {}
                _ if finales.iter().any(|(_, h)| *h == hash_actual) => avisos.push(format!(
                    "'{act}': el código guardado corresponde a una versión anterior"
                )),
                _ => errores.push(format!(
                    "'{act}': el código entregado NO coincide con lo escrito en la app"
                )),
            }
        }
        revisadas += 1;
        let r = ritmo(&ops_act);
        if r.rafagas > 0 {
            avisos.push(format!(
                "'{act}': {} ráfaga(s) de escritura a más de {:.0} caracteres/s",
                r.rafagas,
                crate::replay::CPS_SOSPECHOSO
            ));
        }
        reporte.ritmo_por_actividad.insert(act.clone(), r);
        todas_ops.extend(ops_act);
    }
    // Actividades con código pero sin ningún historial.
    for (act, estado) in &p.actividades {
        if !estado.codigo.is_empty() && !por_actividad.contains_key(act) {
            errores.push(format!(
                "'{act}' tiene código pero ningún historial de edición"
            ));
        }
    }
    reporte.ritmo = ritmo(&todas_ops);
    if !errores.is_empty() {
        reporte.agregar(
            "replay",
            "Historial de escritura",
            Nivel::Rojo,
            errores.join("; "),
        );
    } else if !avisos.is_empty() {
        reporte.agregar(
            "replay",
            "Historial de escritura",
            Nivel::Amarillo,
            avisos.join("; "),
        );
    } else {
        reporte.agregar(
            "replay",
            "Historial de escritura",
            Nivel::Verde,
            format!(
                "El código de {revisadas} actividad(es) se reconstruye exactamente tecla a tecla."
            ),
        );
    }
}

fn normalizar(t: &str) -> String {
    t.replace("\r\n", "\n").trim_end().to_string()
}
