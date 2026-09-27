//! Operaciones de la App Alumno sobre su perfil local.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zeroize::Zeroizing;

use crate::almacen::{leer_meta, Almacen, MetaPerfil, ARCHIVO_BD};
use crate::crypto::{
    abrir_con_secreto, aleatorio, codigo_recuperacion, de_b64_32, envolver_con_secreto,
    envolver_para, normalizar_codigo, Llave, ParametrosKdf,
};
use crate::entrega::{self, EntregaLeida};
use crate::error::{Error, Resultado};
use crate::estadisticas::{calcular, Estadisticas};
use crate::eventos::TIPOS_LIBRES;
use crate::grupo::{validar_numero_control, verificar_grupo};
use crate::modelo::{
    ahora_ms, nuevo_id, Envolturas, EstadoActividad, GrupoFirmado, GrupoInfo, PerfilPublico,
};
use crate::replay::{aplicar, hash_texto, LoteOps};

pub const LONGITUD_MINIMA_CONTRASENA: usize = 8;

/// Secreto con el que se abre un perfil o una entrega.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Secreto {
    Contrasena {
        contrasena: String,
    },
    /// Código de recuperación; se fija una contraseña nueva.
    Codigo {
        codigo: String,
        nueva_contrasena: String,
    },
}

fn aad_perfil(perfil_id: &str) -> Vec<u8> {
    format!("dek|{perfil_id}").into_bytes()
}

fn validar_contrasena(c: &str) -> Resultado<()> {
    if c.chars().count() < LONGITUD_MINIMA_CONTRASENA {
        return Err(Error::validacion(format!(
            "La contraseña debe tener al menos {LONGITUD_MINIMA_CONTRASENA} caracteres."
        )));
    }
    Ok(())
}

/// Desenvuelve la llave de datos con un secreto. Con código de recuperación también devuelve
/// la envoltura nueva para la contraseña indicada.
fn abrir_dek(
    envolturas: &Envolturas,
    perfil_id: &str,
    secreto: &Secreto,
    kdf: &ParametrosKdf,
) -> Resultado<(Zeroizing<Llave>, Option<Envolturas>)> {
    let aad = aad_perfil(perfil_id);
    match secreto {
        Secreto::Contrasena { contrasena } => Ok((
            abrir_con_secreto(&envolturas.contrasena, contrasena, &aad)?,
            None,
        )),
        Secreto::Codigo {
            codigo,
            nueva_contrasena,
        } => {
            validar_contrasena(nueva_contrasena)?;
            let dek =
                abrir_con_secreto(&envolturas.recuperacion, &normalizar_codigo(codigo), &aad)?;
            let mut nuevas = envolturas.clone();
            nuevas.contrasena = envolver_con_secreto(&dek, nueva_contrasena, kdf, &aad)?;
            Ok((dek, Some(nuevas)))
        }
    }
}

fn envolturas_profesor(
    dek: &Llave,
    grupo: &GrupoInfo,
    perfil_id: &str,
) -> Resultado<Vec<crate::crypto::EnvolturaPublica>> {
    grupo
        .llaves_cifrado
        .iter()
        .map(|l| Ok(envolver_para(dek, &de_b64_32(l)?, &aad_perfil(perfil_id))))
        .collect()
}

/// Resumen de un perfil para la pantalla de inicio.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PerfilLocal {
    pub perfil: PerfilPublico,
    pub carpeta: String,
    pub grupo: Option<String>,
}

pub fn listar_perfiles(dir_perfiles: &Path) -> Vec<PerfilLocal> {
    let mut v = Vec::new();
    let Ok(entradas) = fs::read_dir(dir_perfiles) else {
        return v;
    };
    for e in entradas.flatten() {
        let bd = e.path().join(ARCHIVO_BD);
        if let Ok(meta) = leer_meta(&bd) {
            let grupo = meta
                .grupo
                .as_ref()
                .and_then(|g| verificar_grupo(g).ok())
                .map(|g| g.nombre);
            v.push(PerfilLocal {
                perfil: meta.perfil,
                carpeta: e.path().to_string_lossy().into_owned(),
                grupo,
            });
        }
    }
    v.sort_by(|a, b| {
        a.perfil
            .nombre
            .to_lowercase()
            .cmp(&b.perfil.nombre.to_lowercase())
    });
    v
}

/// Resultado de guardar un lote de ediciones.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResultadoGuardado {
    /// El texto reproducido coincide con el del editor.
    pub ok: bool,
    /// Texto vigente según el historial (el editor debe adoptarlo si `ok` es falso).
    pub codigo: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResumenImportacion {
    pub eventos_nuevos: usize,
    pub actividades_actualizadas: Vec<String>,
    pub dispositivos: Vec<String>,
    pub conflictos: Vec<String>,
}

/// Estado completo para la interfaz.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EstadoAlumno {
    pub perfil: PerfilPublico,
    pub dispositivo: String,
    pub grupo: Option<GrupoInfo>,
    pub actividades: BTreeMap<String, EstadoActividad>,
    pub estadisticas: Estadisticas,
}

pub struct SesionAlumno {
    almacen: Almacen,
    carpeta: PathBuf,
    kdf: ParametrosKdf,
}

impl SesionAlumno {
    /// Crea un perfil nuevo en `dir_perfiles/<perfil_id>/`. Devuelve la sesión y el código de
    /// recuperación (se muestra una sola vez).
    pub fn registrar(
        dir_perfiles: &Path,
        numero_control: &str,
        nombre: &str,
        contrasena: &str,
        grupo: Option<&GrupoFirmado>,
        kdf: ParametrosKdf,
    ) -> Resultado<(Self, String)> {
        let numero_control = numero_control.trim().to_string();
        let nombre = nombre.split_whitespace().collect::<Vec<_>>().join(" ");
        let info = grupo.map(verificar_grupo).transpose()?;
        validar_numero_control(
            info.as_ref()
                .map(|g| g.regex_control.as_str())
                .unwrap_or(""),
            &numero_control,
        )?;
        if nombre.chars().count() < 3 {
            return Err(Error::validacion("Escribe tu nombre completo."));
        }
        validar_contrasena(contrasena)?;
        if listar_perfiles(dir_perfiles)
            .iter()
            .any(|p| p.perfil.numero_control == numero_control)
        {
            return Err(Error::validacion(
                "Ya existe un perfil con ese número de control en esta carpeta. Inicia sesión o usa \"Recuperar mis trabajos\".",
            ));
        }
        let perfil = PerfilPublico {
            perfil_id: nuevo_id(),
            numero_control,
            nombre,
            creado: ahora_ms(),
        };
        let dek: Llave = aleatorio();
        let aad = aad_perfil(&perfil.perfil_id);
        let codigo = codigo_recuperacion();
        let envolturas = Envolturas {
            contrasena: envolver_con_secreto(&dek, contrasena, &kdf, &aad)?,
            recuperacion: envolver_con_secreto(&dek, &normalizar_codigo(&codigo), &kdf, &aad)?,
            profesores: match &info {
                Some(g) => envolturas_profesor(&dek, g, &perfil.perfil_id)?,
                None => Vec::new(),
            },
        };
        let carpeta = dir_perfiles.join(&perfil.perfil_id);
        fs::create_dir_all(&carpeta)?;
        let meta = MetaPerfil {
            formato: 1,
            perfil: perfil.clone(),
            dispositivo: nuevo_id(),
            envolturas,
            grupo: grupo.cloned(),
        };
        let mut almacen = Almacen::crear(&carpeta.join(ARCHIVO_BD), meta, dek)?;
        almacen.agregar(
            vec![
                (
                    "registro".into(),
                    None,
                    json!({
                        "numero_control": perfil.numero_control,
                        "nombre": perfil.nombre,
                        "grupo_id": info.as_ref().map(|g| g.grupo_id.clone()),
                    }),
                ),
                (
                    "sesion_inicio".into(),
                    None,
                    json!({ "version": env!("CARGO_PKG_VERSION") }),
                ),
            ],
            &[],
        )?;
        Ok((
            SesionAlumno {
                almacen,
                carpeta,
                kdf,
            },
            codigo,
        ))
    }

    /// Abre un perfil existente (carpeta del perfil).
    pub fn abrir(carpeta: &Path, secreto: &Secreto, kdf: ParametrosKdf) -> Resultado<Self> {
        let bd = carpeta.join(ARCHIVO_BD);
        let meta = leer_meta(&bd)?;
        let (dek, nuevas) = abrir_dek(&meta.envolturas, &meta.perfil.perfil_id, secreto, &kdf)?;
        let mut almacen = Almacen::abrir(&bd, *dek)?;
        let mut eventos = vec![(
            "sesion_inicio".to_string(),
            None,
            json!({ "version": env!("CARGO_PKG_VERSION") }),
        )];
        if let Some(n) = nuevas {
            almacen.meta.envolturas = n;
            almacen.guardar_meta()?;
            eventos.insert(0, ("recuperacion".into(), None, json!({})));
        }
        almacen.agregar(eventos, &[])?;
        Ok(SesionAlumno {
            almacen,
            carpeta: carpeta.to_path_buf(),
            kdf,
        })
    }

    /// Crea un perfil local a partir de una entrega `.rlp` (continuar en otro dispositivo).
    pub fn restaurar(
        dir_perfiles: &Path,
        archivo: &[u8],
        secreto: &Secreto,
        kdf: ParametrosKdf,
    ) -> Resultado<Self> {
        let leida = entrega::leer(archivo)?;
        if !leida.firma_valida() || !leida.payload_integro() {
            return Err(Error::Alterado(
                "la entrega no tiene una firma válida".into(),
            ));
        }
        let m = &leida.manifiesto;
        if listar_perfiles(dir_perfiles)
            .iter()
            .any(|p| p.perfil.perfil_id == m.perfil_id)
        {
            return Err(Error::validacion(
                "Este perfil ya existe en esta carpeta: inicia sesión y usa \"Importar avances\".",
            ));
        }
        let (dek, nuevas) = abrir_dek(&m.envolturas, &m.perfil_id, secreto, &kdf)?;
        let payload = entrega::descifrar_payload(&dek, &m.perfil_id, &leida.payload)?;
        if payload.perfil.perfil_id != m.perfil_id {
            return Err(Error::Alterado("perfil inconsistente".into()));
        }
        let carpeta = dir_perfiles.join(&m.perfil_id);
        fs::create_dir_all(&carpeta)?;
        let meta = MetaPerfil {
            formato: 1,
            perfil: payload.perfil.clone(),
            dispositivo: nuevo_id(),
            envolturas: nuevas.unwrap_or_else(|| m.envolturas.clone()),
            grupo: payload.grupo.clone(),
        };
        let almacen = Almacen::crear(&carpeta.join(ARCHIVO_BD), meta, *dek)?;
        let mut sesion = SesionAlumno {
            almacen,
            carpeta,
            kdf,
        };
        let resumen = sesion.fusionar(&leida, &payload)?;
        sesion.almacen.agregar(
            vec![
                ("restauracion".into(), None, json!({ "dispositivos": resumen.dispositivos, "eventos": resumen.eventos_nuevos })),
                ("sesion_inicio".into(), None, json!({ "version": env!("CARGO_PKG_VERSION") })),
            ],
            &[],
        )?;
        Ok(sesion)
    }

    pub fn perfil(&self) -> &PerfilPublico {
        &self.almacen.meta.perfil
    }

    pub fn carpeta(&self) -> &Path {
        &self.carpeta
    }

    pub fn dispositivo(&self) -> &str {
        &self.almacen.meta.dispositivo
    }

    pub fn grupo(&self) -> Option<GrupoInfo> {
        self.almacen
            .meta
            .grupo
            .as_ref()
            .and_then(|g| verificar_grupo(g).ok())
    }

    pub fn estadisticas(&self) -> Estadisticas {
        calcular(self.almacen.eventos())
    }

    pub fn estado(&self) -> Resultado<EstadoAlumno> {
        Ok(EstadoAlumno {
            perfil: self.perfil().clone(),
            dispositivo: self.dispositivo().to_string(),
            grupo: self.grupo(),
            actividades: self.almacen.actividades()?,
            estadisticas: self.estadisticas(),
        })
    }

    pub fn cerrar(mut self) -> Resultado<()> {
        self.almacen
            .agregar(vec![("sesion_fin".into(), None, json!({}))], &[])?;
        Ok(())
    }

    pub fn cambiar_contrasena(&mut self, actual: &str, nueva: &str) -> Resultado<()> {
        validar_contrasena(nueva)?;
        let perfil_id = self.perfil().perfil_id.clone();
        let aad = aad_perfil(&perfil_id);
        let dek = abrir_con_secreto(&self.almacen.meta.envolturas.contrasena, actual, &aad)?;
        self.almacen.meta.envolturas.contrasena =
            envolver_con_secreto(&dek, nueva, &self.kdf, &aad)?;
        self.almacen.guardar_meta()?;
        self.almacen
            .agregar(vec![("cambio_contrasena".into(), None, json!({}))], &[])?;
        Ok(())
    }

    /// Asocia el perfil a un grupo (si se registró en modo práctica o el profesor cambió).
    pub fn unirse_grupo(&mut self, grupo: &GrupoFirmado) -> Resultado<GrupoInfo> {
        let info = verificar_grupo(grupo)?;
        let perfil_id = self.perfil().perfil_id.clone();
        let nuevas = envolturas_profesor(self.almacen.dek(), &info, &perfil_id)?;
        let envs = &mut self.almacen.meta.envolturas.profesores;
        for n in nuevas {
            if !envs.iter().any(|e| e.destinatario == n.destinatario) {
                envs.push(n);
            }
        }
        self.almacen.meta.grupo = Some(grupo.clone());
        self.almacen.guardar_meta()?;
        self.almacen.agregar(
            vec![("grupo".into(), None, json!({ "grupo_id": info.grupo_id }))],
            &[],
        )?;
        Ok(info)
    }

    // ------------------------------------------------------------------ actividades

    /// Abre una actividad en el editor. Registra el punto de partida del historial si hace falta.
    pub fn abrir_actividad(
        &mut self,
        id: &str,
        codigo_inicial: &str,
    ) -> Resultado<EstadoActividad> {
        let disp = self.dispositivo().to_string();
        let mut eventos = Vec::new();
        // `dispositivo` vacío = la actividad existe (p. ej. pidió una pista) pero su código aún
        // no tiene punto de partida en el historial.
        let estado = match self
            .almacen
            .actividad(id)?
            .filter(|e| !e.dispositivo.is_empty())
        {
            None => {
                eventos.push((
                    "base".to_string(),
                    Some(id.to_string()),
                    json!({ "motivo": "inicio", "texto": codigo_inicial, "hash": hash_texto(codigo_inicial) }),
                ));
                let previo = self.almacen.actividad(id)?.unwrap_or_default();
                EstadoActividad {
                    codigo: codigo_inicial.to_string(),
                    actualizado: ahora_ms(),
                    dispositivo: disp.clone(),
                    ..previo
                }
            }
            Some(mut e) => {
                if e.dispositivo != disp {
                    eventos.push((
                        "base".to_string(),
                        Some(id.to_string()),
                        json!({ "motivo": "continuacion", "hash": hash_texto(&e.codigo) }),
                    ));
                    e.dispositivo = disp.clone();
                }
                e
            }
        };
        eventos.push((
            "actividad_abierta".into(),
            Some(id.to_string()),
            Value::Null,
        ));
        self.almacen.agregar(eventos, &[(id, &estado)])?;
        Ok(estado)
    }

    /// Guarda un lote de operaciones del editor. El texto se deriva del historial; si no
    /// coincide con el del editor, se devuelve el vigente para que el editor lo adopte.
    pub fn guardar_edicion(
        &mut self,
        id: &str,
        lote: &LoteOps,
        texto_editor: &str,
    ) -> Resultado<ResultadoGuardado> {
        let Some(mut estado) = self.almacen.actividad(id)? else {
            return Err(Error::validacion("La actividad no está abierta."));
        };
        if estado.dispositivo != self.dispositivo() {
            return Err(Error::validacion("La actividad no está abierta."));
        }
        let ops = lote.operaciones();
        let nuevo = match aplicar(&estado.codigo, &ops) {
            Ok(t) => t,
            Err(_) => {
                return Ok(ResultadoGuardado {
                    ok: false,
                    codigo: estado.codigo,
                })
            }
        };
        if ops.is_empty() {
            return Ok(ResultadoGuardado {
                ok: nuevo == texto_editor,
                codigo: nuevo,
            });
        }
        estado.codigo = nuevo.clone();
        estado.actualizado = ahora_ms();
        self.almacen.agregar(
            vec![(
                "edicion".into(),
                Some(id.into()),
                serde_json::to_value(lote)?,
            )],
            &[(id, &estado)],
        )?;
        Ok(ResultadoGuardado {
            ok: nuevo == texto_editor,
            codigo: nuevo,
        })
    }

    /// Vuelve al código inicial de la actividad.
    pub fn reiniciar_actividad(
        &mut self,
        id: &str,
        codigo_inicial: &str,
    ) -> Resultado<EstadoActividad> {
        let mut estado = self.almacen.actividad(id)?.unwrap_or_default();
        estado.codigo = codigo_inicial.to_string();
        estado.dispositivo = self.dispositivo().to_string();
        estado.actualizado = ahora_ms();
        self.almacen.agregar(
            vec![(
                "base".into(),
                Some(id.into()),
                json!({ "motivo": "reinicio", "texto": codigo_inicial, "hash": hash_texto(codigo_inicial) }),
            )],
            &[(id, &estado)],
        )?;
        Ok(estado)
    }

    /// Registra el resultado de las pruebas automáticas del código actual.
    pub fn registrar_pruebas(
        &mut self,
        id: &str,
        pasadas: u32,
        total: u32,
        puntos: u32,
    ) -> Resultado<EstadoActividad> {
        let mut estado = self
            .almacen
            .actividad(id)?
            .ok_or_else(|| Error::validacion("La actividad no está abierta."))?;
        estado.pasadas = pasadas;
        estado.total = total;
        estado.intentos += 1;
        if total > 0 && pasadas == total && !estado.completada {
            estado.completada = true;
            estado.puntos = puntos;
        }
        estado.actualizado = ahora_ms();
        let datos =
            json!({ "pasadas": pasadas, "total": total, "hash": hash_texto(&estado.codigo) });
        self.almacen.agregar(
            vec![("prueba".into(), Some(id.into()), datos)],
            &[(id, &estado)],
        )?;
        Ok(estado)
    }

    /// Registra la respuesta de una actividad de opción múltiple o predicción.
    pub fn registrar_respuesta(
        &mut self,
        id: &str,
        respuesta: &str,
        correcta: bool,
        puntos: u32,
    ) -> Resultado<EstadoActividad> {
        let mut estado = self.almacen.actividad(id)?.unwrap_or_default();
        estado.respuesta = Some(respuesta.to_string());
        estado.intentos += 1;
        if correcta && !estado.completada {
            estado.completada = true;
            estado.puntos = puntos;
        }
        estado.actualizado = ahora_ms();
        self.almacen.agregar(
            vec![(
                "respuesta".into(),
                Some(id.into()),
                json!({ "correcta": correcta, "respuesta": respuesta }),
            )],
            &[(id, &estado)],
        )?;
        Ok(estado)
    }

    pub fn registrar_pista(&mut self, id: &str, numero: u32) -> Resultado<EstadoActividad> {
        let mut estado = self.almacen.actividad(id)?.unwrap_or_default();
        estado.pistas = estado.pistas.max(numero);
        self.almacen.agregar(
            vec![("pista".into(), Some(id.into()), json!({ "numero": numero }))],
            &[(id, &estado)],
        )?;
        Ok(estado)
    }

    /// Registra eventos informativos (copias, intentos de pegado, ejecuciones, foco...).
    pub fn registrar_evento(
        &mut self,
        tipo: &str,
        actividad: Option<&str>,
        datos: Value,
    ) -> Resultado<()> {
        if !TIPOS_LIBRES.contains(&tipo) || matches!(tipo, "prueba" | "respuesta" | "pista") {
            return Err(Error::validacion(format!(
                "Tipo de evento no permitido: {tipo}"
            )));
        }
        self.almacen.agregar(
            vec![(tipo.into(), actividad.map(str::to_string), datos)],
            &[],
        )?;
        Ok(())
    }

    // ------------------------------------------------------------------ entregas

    /// Genera el archivo `.rlp` en memoria y el nombre sugerido.
    pub fn exportar(&mut self) -> Resultado<(String, Vec<u8>)> {
        self.almacen
            .agregar(vec![("exportacion".into(), None, json!({}))], &[])?;
        let grupo = self.grupo();
        let bytes = entrega::construir(
            self.almacen.dek(),
            self.perfil(),
            &self.almacen.meta.envolturas,
            self.almacen.meta.grupo.as_ref(),
            grupo.map(|g| g.grupo_id),
            self.almacen.actividades()?,
            self.almacen.eventos_firmados()?,
            self.almacen.cabezas(),
        )?;
        let fecha = chrono::Local::now().format("%Y%m%d-%H%M");
        let control: String = self
            .perfil()
            .numero_control
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect();
        Ok((format!("{control}_{fecha}.{}", entrega::EXTENSION), bytes))
    }

    /// Importa avances de otro dispositivo del mismo alumno.
    pub fn importar(&mut self, archivo: &[u8]) -> Resultado<ResumenImportacion> {
        let leida = entrega::leer(archivo)?;
        if leida.manifiesto.perfil_id != self.perfil().perfil_id {
            return Err(Error::validacion("Ese archivo es de otro alumno."));
        }
        if !leida.firma_valida() || !leida.payload_integro() {
            return Err(Error::Alterado(
                "la entrega no tiene una firma válida".into(),
            ));
        }
        let payload = entrega::descifrar_payload(
            self.almacen.dek(),
            &leida.manifiesto.perfil_id,
            &leida.payload,
        )?;
        let resumen = self.fusionar(&leida, &payload)?;
        self.almacen.agregar(
            vec![(
                "importacion".into(),
                None,
                json!({ "dispositivos": resumen.dispositivos, "eventos": resumen.eventos_nuevos, "conflictos": resumen.conflictos }),
            )],
            &[],
        )?;
        Ok(resumen)
    }

    fn fusionar(
        &mut self,
        _leida: &EntregaLeida,
        payload: &entrega::Payload,
    ) -> Resultado<ResumenImportacion> {
        let mut resumen = ResumenImportacion::default();
        let mut dispositivos = BTreeSet::new();
        let mut bloqueados = BTreeSet::new();
        let mut eventos = payload.eventos_firmados()?;
        let clave = |e: &crate::eventos::EventoFirmado| {
            e.evento()
                .map(|x| (x.dispositivo, x.seq))
                .unwrap_or_default()
        };
        eventos.sort_by_key(clave);
        for ef in eventos {
            let e = ef.evento()?;
            if bloqueados.contains(&e.dispositivo) {
                continue;
            }
            match self.almacen.insertar_firmado(&ef) {
                Ok(true) => {
                    resumen.eventos_nuevos += 1;
                    dispositivos.insert(e.dispositivo.clone());
                }
                Ok(false) => {}
                Err(err) => {
                    resumen.conflictos.push(format!(
                        "{}: {err}",
                        &e.dispositivo[..8.min(e.dispositivo.len())]
                    ));
                    bloqueados.insert(e.dispositivo.clone());
                }
            }
        }
        for (id, remoto) in &payload.actividades {
            let local = self.almacen.actividad(id)?;
            let reemplazar = match &local {
                None => true,
                Some(l) => remoto.actualizado > l.actualizado && remoto.codigo != l.codigo,
            };
            let mut nuevo = if reemplazar {
                remoto.clone()
            } else {
                local.clone().unwrap_or_default()
            };
            if let Some(l) = &local {
                nuevo.completada = l.completada || remoto.completada;
                nuevo.puntos = l.puntos.max(remoto.puntos);
                nuevo.intentos = l.intentos.max(remoto.intentos);
                nuevo.pistas = l.pistas.max(remoto.pistas);
            }
            if local.as_ref() != Some(&nuevo) {
                self.almacen.guardar_actividad(id, &nuevo)?;
                if reemplazar {
                    resumen.actividades_actualizadas.push(id.clone());
                }
            }
        }
        if self.almacen.meta.grupo.is_none() {
            if let Some(g) = &payload.grupo {
                self.almacen.meta.grupo = Some(g.clone());
                self.almacen.guardar_meta()?;
            }
        }
        resumen.dispositivos = dispositivos.into_iter().collect();
        Ok(resumen)
    }
}
