//! El núcleo web visto como lo usa el Worker: llamadas JSON y diarios que se guardan aparte.

use std::collections::{BTreeMap, HashMap};

use rlp_core::crypto::{b64, de_b64, ParametrosKdf};
use rlp_core::deposito::{Deposito, DepositoMemoria, Lote};
use rlp_core::modelo::{GrupoFirmado, Politicas};
use rlp_core::profesor::{IdentidadProfesor, NuevoGrupo};
use rlp_core::verificacion::{abrir_entrega, Contexto, Nivel};
use rlp_web::Nucleo;
use serde_json::{json, Value};

const INICIAL: &str = "# Escribe tu programa\n";

/// Lo que hace `packages/nucleo-web` con IndexedDB, en memoria.
struct App {
    nucleo: Nucleo,
    guardado: BTreeMap<String, DepositoMemoria>,
}

impl App {
    fn nueva() -> Self {
        App {
            nucleo: Nucleo::con_kdf(ParametrosKdf::rapidos()),
            guardado: BTreeMap::new(),
        }
    }

    /// Llama y guarda el diario, como el Worker después de cada operación.
    fn llamar(&mut self, metodo: &str, args: Value) -> Result<Value, String> {
        let r = self.nucleo.llamar(metodo, &args.to_string());
        let diarios: Vec<Value> =
            serde_json::from_str(&self.nucleo.llamar("diario", "{}").unwrap()).unwrap();
        for d in diarios {
            let perfil = d["perfil"].as_str().unwrap().to_string();
            let lote: Lote = serde_json::from_value(d["lote"].clone()).unwrap();
            self.guardado
                .entry(perfil)
                .or_default()
                .escribir(lote)
                .unwrap();
        }
        r.map(|t| serde_json::from_str(&t).unwrap())
    }

    fn metas(&self) -> Value {
        let metas: Vec<Value> = self
            .guardado
            .values()
            .map(|d| serde_json::from_str(&d.meta("perfil").unwrap().unwrap()).unwrap())
            .collect();
        json!(metas)
    }

    fn instantanea(&self, perfil: &str) -> Value {
        serde_json::to_value(self.guardado[perfil].instantanea()).unwrap()
    }
}

fn profesor_y_grupo() -> (IdentidadProfesor, GrupoFirmado) {
    let profe = IdentidadProfesor::nueva("Profa. Web").unwrap();
    let (grupo, _) = profe
        .crear_grupo(&NuevoGrupo {
            nombre: "Programación 1C".into(),
            materia: "Fundamentos".into(),
            periodo: "2026-2".into(),
            regex_control: r"\d{8}".into(),
            politicas: Politicas::default(),
            coprofesores: vec![],
        })
        .unwrap();
    (profe, grupo)
}

fn teclear(app: &mut App, actividad: &str, texto: &str) -> String {
    let actual = app
        .llamar(
            "abrir_actividad",
            json!({ "id": actividad, "codigo_inicial": INICIAL }),
        )
        .unwrap()["codigo"]
        .as_str()
        .unwrap()
        .to_string();
    let mut pos = actual.encode_utf16().count();
    let ops: Vec<Value> = texto
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let op = json!([i * 120, pos, pos, c.to_string(), "t"]);
            pos += c.len_utf16();
            op
        })
        .collect();
    let esperado = format!("{actual}{texto}");
    let r = app
        .llamar(
            "guardar_edicion",
            json!({ "id": actividad, "lote": { "t0": rlp_core::modelo::ahora_ms(), "ops": ops }, "texto": esperado }),
        )
        .unwrap();
    assert_eq!(r["ok"], json!(true));
    esperado
}

fn contrasena(c: &str) -> Value {
    json!({ "tipo": "contrasena", "contrasena": c })
}

#[test]
fn registro_recarga_y_entrega_verificada() {
    let (profe, grupo) = profesor_y_grupo();
    let mut app = App::nueva();
    assert!(app.llamar("version", json!({})).unwrap()["version"].is_string());
    assert!(app
        .llamar("estado", json!({}))
        .unwrap_err()
        .contains("sesión"));

    let r = app
        .llamar(
            "registrar",
            json!({
                "metas": app.metas(),
                "grupo": grupo,
                "numero_control": "21340200",
                "nombre": "Gael Web",
                "contrasena": "clave-gael",
            }),
        )
        .unwrap();
    let perfil = r["estado"]["perfil"]["perfil_id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(r["codigo"].as_str().unwrap().len(), 24);
    assert!(
        app.guardado.contains_key(&perfil),
        "el registro ya quedó guardado"
    );
    let codigo = teclear(&mut app, "u1-a1", "print('hola')\n");

    // Exportar: la App Profesor la abre y no hay nada en rojo.
    let e = app.llamar("exportar", json!({})).unwrap();
    assert!(e["nombre"].as_str().unwrap().starts_with("21340200_"));
    let bytes = de_b64(e["archivo"].as_str().unwrap()).unwrap();
    let ctx = Contexto {
        codigos_iniciales: HashMap::from([("u1-a1".to_string(), INICIAL.to_string())]),
        ..Default::default()
    };
    let abierta = abrir_entrega(&bytes, &profe, &ctx).unwrap();
    for c in &abierta.reporte.checks {
        assert_ne!(c.nivel, Nivel::Rojo, "{}: {}", c.nombre, c.detalle);
    }
    app.llamar("cerrar_sesion", json!({})).unwrap();

    // "Recargar la página": un núcleo nuevo con lo guardado.
    let mut app2 = App {
        nucleo: Nucleo::con_kdf(ParametrosKdf::rapidos()),
        guardado: std::mem::take(&mut app.guardado),
    };
    let lista = app2
        .llamar("perfiles", json!({ "metas": app2.metas() }))
        .unwrap();
    assert_eq!(lista[0]["perfil"]["nombre"], json!("Gael Web"));
    assert_eq!(lista[0]["grupo"], json!("Programación 1C"));
    let err = app2
        .llamar(
            "iniciar_sesion",
            json!({ "instantanea": app2.instantanea(&perfil), "secreto": contrasena("mala-clave") }),
        )
        .unwrap_err();
    assert!(err.contains("incorrect"), "{err}");
    let estado = app2
        .llamar(
            "iniciar_sesion",
            json!({ "instantanea": app2.instantanea(&perfil), "secreto": contrasena("clave-gael") }),
        )
        .unwrap();
    assert_eq!(estado["actividades"]["u1-a1"]["codigo"], json!(codigo));

    // El mismo número de control no se registra dos veces en este navegador.
    let err = app2
        .llamar(
            "registrar",
            json!({ "metas": app2.metas(), "grupo": grupo, "numero_control": "21340200",
                    "nombre": "Otra Persona", "contrasena": "clave-otra" }),
        )
        .unwrap_err();
    assert!(err.contains("Ya existe"), "{err}");
}

#[test]
fn acceso_del_profesor_y_continuar_desde_un_rlp() {
    let (profe, grupo) = profesor_y_grupo();
    let mut app = App::nueva();
    let r = app
        .llamar(
            "registrar",
            json!({ "metas": [], "grupo": grupo, "numero_control": "21340201",
                    "nombre": "Hilda Web", "contrasena": "clave-olvidada" }),
        )
        .unwrap();
    let perfil = r["estado"]["perfil"]["perfil_id"]
        .as_str()
        .unwrap()
        .to_string();
    teclear(&mut app, "u1-a1", "x = 1\n");
    let rlp = app.llamar("exportar", json!({})).unwrap()["archivo"]
        .as_str()
        .unwrap()
        .to_string();
    app.llamar("cerrar_sesion", json!({})).unwrap();

    // El profesor crea el archivo de acceso desde la entrega.
    let entrega = abrir_entrega(&de_b64(&rlp).unwrap(), &profe, &Contexto::default()).unwrap();
    let (acceso, temporal) = rlp_core::acceso::crear(
        &profe,
        &entrega.manifiesto,
        "21340201",
        "Hilda Web",
        &ParametrosKdf::rapidos(),
    )
    .unwrap();
    let acceso = String::from_utf8(acceso).unwrap();
    let info = app
        .llamar(
            "leer_acceso",
            json!({ "metas": app.metas(), "archivo": acceso }),
        )
        .unwrap();
    assert_eq!(info["perfil_local"], json!(true));
    assert_eq!(info["perfil_id"], json!(perfil));

    // En este navegador (instantánea): contraseña nueva y código de recuperación nuevo.
    let estado = app
        .llamar(
            "entrar_con_acceso",
            json!({ "acceso": acceso, "temporal": temporal, "nueva": "clave-nueva-1",
                    "instantanea": app.instantanea(&perfil) }),
        )
        .unwrap();
    assert!(estado["codigo_nuevo"].is_string());
    app.llamar("cerrar_sesion", json!({})).unwrap();
    app.llamar(
        "iniciar_sesion",
        json!({ "instantanea": app.instantanea(&perfil), "secreto": contrasena("clave-nueva-1") }),
    )
    .unwrap();

    // En otro navegador: sin perfil local hace falta el .rlp.
    let mut otro = App::nueva();
    let err = otro
        .llamar(
            "entrar_con_acceso",
            json!({ "acceso": acceso, "temporal": temporal, "nueva": "clave-nueva-2" }),
        )
        .unwrap_err();
    assert!(err.contains(".rlp"), "{err}");
    let estado = otro
        .llamar(
            "restaurar",
            json!({ "metas": [], "archivo": rlp, "secreto": contrasena("clave-olvidada") }),
        )
        .unwrap();
    assert_eq!(estado["perfil"]["perfil_id"], json!(perfil));
    assert!(otro.guardado[&perfil].eventos().unwrap().len() > 3);

    // Grupo por QR y por archivo.
    let qr = rlp_core::grupo::a_texto_qr(&grupo).unwrap();
    let g = otro.llamar("grupo_qr", json!({ "contenido": qr })).unwrap();
    assert_eq!(g["info"]["nombre"], json!("Programación 1C"));
    let archivo_grupo = b64(&serde_json::to_vec(&grupo).unwrap());
    otro.llamar("unirse_grupo", json!({ "archivo": archivo_grupo }))
        .unwrap();
    assert!(otro
        .llamar("no_existe", json!({}))
        .unwrap_err()
        .contains("desconocida"));
}

#[test]
fn retroalimentacion_del_profesor_en_la_version_web() {
    let (profe, grupo) = profesor_y_grupo();
    let mut app = App::nueva();
    let r = app
        .llamar(
            "registrar",
            json!({ "metas": [], "grupo": grupo, "numero_control": "21340202",
                    "nombre": "Iván Web", "contrasena": "clave-ivan" }),
        )
        .unwrap();
    let perfil = r["estado"]["perfil"]["perfil_id"]
        .as_str()
        .unwrap()
        .to_string();
    teclear(&mut app, "u1-a1", "print(1)\n");
    let rlp = app.llamar("exportar", json!({})).unwrap()["archivo"]
        .as_str()
        .unwrap()
        .to_string();

    // El profesor abre la entrega web y le devuelve una calificación.
    let entrega = abrir_entrega(&de_b64(&rlp).unwrap(), &profe, &Contexto::default()).unwrap();
    let dek = profe
        .abrir_dek(&entrega.manifiesto.envolturas.profesores, &perfil)
        .unwrap();
    let notas = std::collections::BTreeMap::from([(
        "u1-a1".to_string(),
        rlp_core::retroalimentacion::NotaActividad {
            calificacion: Some(9.0),
            comentario: "Bien hecho en la web".into(),
            actualizado: rlp_core::modelo::ahora_ms(),
        },
    )]);
    let archivo = rlp_core::retroalimentacion::crear(
        &profe,
        Some(&entrega.manifiesto.grupo_id.clone().unwrap()),
        &[(perfil.clone(), *dek, notas)],
    )
    .unwrap();
    let retro = app
        .llamar(
            "importar_retroalimentacion",
            json!({ "archivo": b64(&archivo) }),
        )
        .unwrap();
    assert_eq!(retro["actividades"]["u1-a1"]["calificacion"], json!(9.0));
    app.llamar("cerrar_sesion", json!({})).unwrap();

    // Queda guardada (cifrada) en el navegador: se ve al volver a entrar.
    let estado = app
        .llamar(
            "iniciar_sesion",
            json!({ "instantanea": app.instantanea(&perfil), "secreto": contrasena("clave-ivan") }),
        )
        .unwrap();
    assert_eq!(
        estado["retroalimentacion"]["actividades"]["u1-a1"]["comentario"],
        json!("Bien hecho en la web")
    );
}
