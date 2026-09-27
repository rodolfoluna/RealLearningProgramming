//! Flujo completo: registro → edición → exportación → verificación del profesor, continuación en
//! otro dispositivo y detección de manipulaciones.

use std::collections::HashMap;
use std::io::{Cursor, Read, Write};

use rlp_core::alumno::{listar_perfiles, Secreto, SesionAlumno};
use rlp_core::bd_profesor::BdProfesor;
use rlp_core::crypto::{sha256_hex, ParametrosKdf};
use rlp_core::entrega::{self, Payload};
use rlp_core::modelo::{GrupoFirmado, Politicas};
use rlp_core::profesor::{IdentidadProfesor, NuevoGrupo};
use rlp_core::replay::LoteOps;
use rlp_core::verificacion::{abrir_entrega, Contexto, EntregaAbierta, Nivel};
use serde_json::json;
use tempfile::TempDir;

const INICIAL: &str = "# Escribe tu programa\n";

fn kdf() -> ParametrosKdf {
    ParametrosKdf::rapidos()
}

fn contrasena(c: &str) -> Secreto {
    Secreto::Contrasena {
        contrasena: c.into(),
    }
}

fn profesor_y_grupo() -> (IdentidadProfesor, GrupoFirmado) {
    let profe = IdentidadProfesor::nueva("Profa. Martínez").unwrap();
    let (grupo, _) = profe
        .crear_grupo(&NuevoGrupo {
            nombre: "Programación Básica 1A".into(),
            materia: "Fundamentos".into(),
            periodo: "2026-2".into(),
            regex_control: r"\d{8}".into(),
            politicas: Politicas::default(),
            coprofesores: vec![],
        })
        .unwrap();
    (profe, grupo)
}

/// Simula teclear `texto` al final del documento (una operación por carácter).
fn teclear(texto_actual: &str, texto: &str) -> LoteOps {
    let mut pos = texto_actual.encode_utf16().count();
    let ops = texto
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let op = (i as i64 * 180, pos, pos, c.to_string(), "t".to_string());
            pos += c.len_utf16();
            op
        })
        .collect();
    LoteOps {
        t0: rlp_core::modelo::ahora_ms(),
        ops,
    }
}

fn escribir(s: &mut SesionAlumno, act: &str, texto: &str) {
    let actual = s.abrir_actividad(act, INICIAL).unwrap().codigo;
    let lote = teclear(&actual, texto);
    let esperado = format!("{actual}{texto}");
    let r = s.guardar_edicion(act, &lote, &esperado).unwrap();
    assert!(r.ok, "el texto reproducido debe coincidir");
    assert_eq!(r.codigo, esperado);
}

fn ctx() -> Contexto {
    Contexto {
        codigos_iniciales: HashMap::from([
            ("u1-a1".to_string(), INICIAL.to_string()),
            ("u1-a2".to_string(), INICIAL.to_string()),
        ]),
        ..Default::default()
    }
}

fn nivel_de(e: &EntregaAbierta, id: &str) -> Nivel {
    e.reporte
        .checks
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.nivel)
        .unwrap_or(Nivel::Verde)
}

/// Nivel del check de firma según la llave con la que se compiló esta prueba: desarrollo →
/// amarillo; llave publicada y de confianza → verde; llave desconocida → rojo.
fn nivel_firma_esperado() -> Nivel {
    let app = rlp_core::llave_app::llave_app();
    match rlp_core::llave_app::buscar_confiable(&app.publica_b64()) {
        Some(l) if l.dev => Nivel::Amarillo,
        Some(_) => Nivel::Verde,
        None => Nivel::Rojo,
    }
}

/// Todas las revisiones en verde salvo la firma, que depende de la llave de la compilación.
fn integra(e: &EntregaAbierta) {
    for c in &e.reporte.checks {
        let esperado = match c.id.as_str() {
            "firma" => nivel_firma_esperado(),
            // Eventos firmados con una llave que no está en la lista de confianza.
            "cadena" if nivel_firma_esperado() == Nivel::Rojo => Nivel::Amarillo,
            _ => Nivel::Verde,
        };
        assert_eq!(c.nivel, esperado, "{}: {}", c.nombre, c.detalle);
    }
}

#[test]
fn flujo_completo_y_verificacion() {
    let dir = TempDir::new().unwrap();
    let (profe, grupo) = profesor_y_grupo();
    let (mut s, codigo) = SesionAlumno::registrar(
        dir.path(),
        "21340001",
        "Ana López",
        "contraseña1",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    assert_eq!(codigo.len(), 24);
    escribir(&mut s, "u1-a1", "print('Hola')\n");
    s.registrar_evento(
        "copia",
        Some("u1-a1"),
        json!({"chars": 5, "origen": "editor"}),
    )
    .unwrap();
    s.registrar_evento(
        "pegado",
        Some("u1-a1"),
        json!({"chars": 40, "permitido": false, "via": "teclado"}),
    )
    .unwrap();
    s.registrar_evento("foco", Some("u1-a1"), json!({"estado": "perdido"}))
        .unwrap();
    s.registrar_pruebas("u1-a1", 2, 2, 10).unwrap();
    escribir(&mut s, "u1-a2", "x = 5\nprint(x * 2)\n");
    let est = s.estadisticas();
    assert_eq!(est.global.copias, 1);
    assert_eq!(est.global.pegados_intentos, 1);
    assert_eq!(est.global.salidas, 1);
    assert!(est.global.teclas > 20);

    let (nombre, bytes) = s.exportar().unwrap();
    assert!(nombre.starts_with("21340001_") && nombre.ends_with(".rlp"));

    let e = abrir_entrega(&bytes, &profe, &ctx()).unwrap();
    integra(&e);
    let p = e.payload.as_ref().unwrap();
    assert_eq!(p.perfil.nombre, "Ana López");
    assert!(p.actividades["u1-a1"].completada);
    assert_eq!(e.estadisticas.global.pegados_intentos, 1);

    // Base de datos del profesor.
    let bd = BdProfesor::abrir(&dir.path().join("profesor.db")).unwrap();
    let r = bd.registrar_entrega(&e).unwrap();
    assert!(r.nueva);
    assert!(
        !bd.registrar_entrega(&e).unwrap().nueva,
        "la misma entrega no se duplica"
    );
    let tablero = bd.tablero(None).unwrap();
    assert_eq!(tablero.len(), 1);
    assert!(tablero[0].actividades["u1-a1"].completada);
    bd.calificar(&p.perfil.perfil_id, "u1-a1", Some(10.0), "¡Bien!")
        .unwrap();
    let det = bd.detalle(r.entrega_id).unwrap().unwrap();
    assert_eq!(det.calificaciones["u1-a1"].calificacion, Some(10.0));
    let csv = bd
        .exportar_csv(None, &[("u1-a1".into(), "Hola mundo".into())])
        .unwrap();
    assert!(csv.contains("21340001") && csv.contains("Completada"));
}

#[test]
fn contrasenas_y_recuperacion() {
    let dir = TempDir::new().unwrap();
    let (_, grupo) = profesor_y_grupo();
    let (s, codigo) = SesionAlumno::registrar(
        dir.path(),
        "21340002",
        "Beto Ruiz",
        "secreto-beto",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    let carpeta = s.carpeta().to_path_buf();
    s.cerrar().unwrap();
    let (s2, _) = SesionAlumno::registrar(
        dir.path(),
        "21340003",
        "Carla Díaz",
        "secreto-carla",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    drop(s2);

    assert!(SesionAlumno::abrir(&carpeta, &contrasena("incorrecta"), kdf()).is_err());
    assert!(
        SesionAlumno::abrir(&carpeta, &contrasena("secreto-carla"), kdf()).is_err(),
        "otra alumna no abre"
    );
    assert!(SesionAlumno::abrir(&carpeta, &contrasena("secreto-beto"), kdf()).is_ok());

    // Código de recuperación (en minúsculas y sin guiones) fija una contraseña nueva.
    let rec = Secreto::Codigo {
        codigo: codigo.to_lowercase().replace('-', ""),
        nueva_contrasena: "nueva-clave-1".into(),
    };
    SesionAlumno::abrir(&carpeta, &rec, kdf()).unwrap();
    assert!(SesionAlumno::abrir(&carpeta, &contrasena("secreto-beto"), kdf()).is_err());
    let mut s = SesionAlumno::abrir(&carpeta, &contrasena("nueva-clave-1"), kdf()).unwrap();
    s.cambiar_contrasena("nueva-clave-1", "otra-clave-2")
        .unwrap();
    assert!(SesionAlumno::abrir(&carpeta, &contrasena("otra-clave-2"), kdf()).is_ok());

    // Validaciones de registro.
    assert!(SesionAlumno::registrar(
        dir.path(),
        "21340002",
        "Beto Ruiz",
        "12345678",
        Some(&grupo),
        kdf()
    )
    .is_err());
    assert!(
        SesionAlumno::registrar(dir.path(), "ABC", "Dani", "12345678", Some(&grupo), kdf())
            .is_err()
    );
    assert!(
        SesionAlumno::registrar(dir.path(), "21340009", "Dani", "corta", Some(&grupo), kdf())
            .is_err()
    );
    assert_eq!(listar_perfiles(dir.path()).len(), 2);
}

#[test]
fn continuar_en_otro_dispositivo_y_volver() {
    let pc = TempDir::new().unwrap();
    let cel = TempDir::new().unwrap();
    let (profe, grupo) = profesor_y_grupo();
    let (mut s, _) = SesionAlumno::registrar(
        pc.path(),
        "21340004",
        "Dora Paz",
        "clave-dora",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    escribir(&mut s, "u1-a1", "n = 3\n");
    let (_, archivo1) = s.exportar().unwrap();

    // En el "celular": restaurar con la contraseña y seguir escribiendo.
    let mut c =
        SesionAlumno::restaurar(cel.path(), &archivo1, &contrasena("clave-dora"), kdf()).unwrap();
    assert_eq!(c.perfil().numero_control, "21340004");
    assert_ne!(c.dispositivo(), s.dispositivo());
    escribir(&mut c, "u1-a1", "print(n * 2)\n");
    escribir(&mut c, "u1-a2", "print('desde el celular')\n");
    let (_, archivo2) = c.exportar().unwrap();
    let e2 = abrir_entrega(&archivo2, &profe, &ctx()).unwrap();
    integra(&e2);

    // De vuelta en la PC: importar lo del celular y seguir.
    std::thread::sleep(std::time::Duration::from_millis(5));
    let r = s.importar(&archivo2).unwrap();
    assert!(r.eventos_nuevos > 0);
    assert!(r.actividades_actualizadas.contains(&"u1-a1".to_string()));
    escribir(&mut s, "u1-a1", "# listo\n");
    let (_, archivo3) = s.exportar().unwrap();
    let e3 = abrir_entrega(&archivo3, &profe, &ctx()).unwrap();
    integra(&e3);
    let p = e3.payload.unwrap();
    assert_eq!(
        p.actividades["u1-a1"].codigo,
        format!("{INICIAL}n = 3\nprint(n * 2)\n# listo\n")
    );
    assert_eq!(
        e3.manifiesto.cabezas.len(),
        2,
        "historial de ambos dispositivos"
    );

    // Continuidad: la entrega 3 extiende la 2.
    let bd = BdProfesor::abrir(&pc.path().join("profesor.db")).unwrap();
    bd.registrar_entrega(&e2).unwrap();
    let ctx3 = Contexto {
        cabezas_previas: bd.cabezas_previas(&p.perfil.perfil_id).unwrap(),
        ..ctx()
    };
    let e3b = abrir_entrega(&archivo3, &profe, &ctx3).unwrap();
    assert_eq!(nivel_de(&e3b, "continuidad"), Nivel::Verde);
    // En cambio, una entrega vieja no extiende lo ya visto.
    bd.registrar_entrega(&e3b).unwrap();
    let ctx_viejo = Contexto {
        cabezas_previas: bd.cabezas_previas(&p.perfil.perfil_id).unwrap(),
        ..ctx()
    };
    let vieja = abrir_entrega(&archivo1, &profe, &ctx_viejo).unwrap();
    assert_eq!(nivel_de(&vieja, "continuidad"), Nivel::Rojo);

    // Restaurar con el código de recuperación también funciona.
    let otro = TempDir::new().unwrap();
    assert!(SesionAlumno::restaurar(otro.path(), &archivo3, &contrasena("mala"), kdf()).is_err());
}

// ------------------------------------------------------------------ manipulaciones

fn reescribir_zip(bytes: &[u8], cambiar: impl Fn(&str, Vec<u8>) -> Vec<u8>) -> Vec<u8> {
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).unwrap();
        let nombre = f.name().to_string();
        let mut datos = Vec::new();
        f.read_to_end(&mut datos).unwrap();
        out.start_file(&nombre, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&cambiar(&nombre, datos)).unwrap();
    }
    out.finish().unwrap().into_inner()
}

fn entrega_base() -> (TempDir, IdentidadProfesor, SesionAlumno, Vec<u8>) {
    let dir = TempDir::new().unwrap();
    let (profe, grupo) = profesor_y_grupo();
    let (mut s, _) = SesionAlumno::registrar(
        dir.path(),
        "21340005",
        "Elsa Rey",
        "clave-elsa",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    escribir(&mut s, "u1-a1", "print(1 + 1)\n");
    let (_, bytes) = s.exportar().unwrap();
    (dir, profe, s, bytes)
}

#[test]
fn byte_alterado_en_el_contenido() {
    let (_d, profe, _s, bytes) = entrega_base();
    let mala = reescribir_zip(&bytes, |n, mut d| {
        if n == "payload.bin" {
            let k = d.len() / 2;
            d[k] ^= 0x01;
        }
        d
    });
    let e = abrir_entrega(&mala, &profe, &ctx()).unwrap();
    assert_eq!(e.reporte.nivel, Nivel::Rojo);
    assert_eq!(nivel_de(&e, "integridad"), Nivel::Rojo);
    assert_eq!(nivel_de(&e, "descifrado"), Nivel::Rojo);
}

#[test]
fn manifiesto_editado_invalida_la_firma() {
    let (_d, profe, _s, bytes) = entrega_base();
    let mala = reescribir_zip(&bytes, |n, d| {
        if n == "manifiesto.json" {
            String::from_utf8(d)
                .unwrap()
                .replace("\"creado\": ", "\"creado\": 1")
                .into_bytes()
        } else {
            d
        }
    });
    let e = abrir_entrega(&mala, &profe, &ctx()).unwrap();
    assert_eq!(nivel_de(&e, "firma"), Nivel::Rojo);
}

/// Un alumno que conoce su contraseña descifra el contenido, cambia su código y vuelve a
/// cifrar y firmar (aquí con una llave propia: no tiene la llave de la app).
#[test]
fn recifrado_con_otra_llave_es_rechazado() {
    let (_d, profe, s, bytes) = entrega_base();
    let leida = entrega::leer(&bytes).unwrap();
    let carpeta = s.carpeta().to_path_buf();
    drop(s);
    let dek = {
        let meta = rlp_core::almacen::leer_meta(&carpeta.join("alumno.db")).unwrap();
        *rlp_core::crypto::abrir_con_secreto(
            &meta.envolturas.contrasena,
            "clave-elsa",
            format!("dek|{}", meta.perfil.perfil_id).as_bytes(),
        )
        .unwrap()
    };
    let mut p: Payload =
        entrega::descifrar_payload(&dek, &leida.manifiesto.perfil_id, &leida.payload).unwrap();
    p.actividades.get_mut("u1-a1").unwrap().codigo = "print('código copiado de internet')\n".into();
    let cifrado = entrega::cifrar_payload(&dek, &p).unwrap();
    let mut m = leida.manifiesto.clone();
    m.payload_sha256 = sha256_hex(&cifrado);
    let llave_propia = rlp_core::crypto::nueva_llave_firma();
    m.app.llave = rlp_core::crypto::b64(llave_propia.verifying_key().as_bytes());
    let mj = serde_json::to_string_pretty(&m).unwrap();
    let firma = rlp_core::crypto::firmar(&llave_propia, entrega::CONTEXTO_FIRMA, mj.as_bytes());
    let mala = entrega::escribir(&mj, &cifrado, &firma).unwrap();
    let e = abrir_entrega(&mala, &profe, &ctx()).unwrap();
    assert_eq!(nivel_de(&e, "firma"), Nivel::Rojo, "llave desconocida");
    assert_eq!(
        nivel_de(&e, "replay"),
        Nivel::Rojo,
        "el código no sale del historial"
    );
}

/// Incluso firmando con la llave de la app (p. ej. extraída del binario), cambiar el código sin
/// fabricar un historial de tecleo coherente se detecta al reproducir el historial.
#[test]
fn codigo_cambiado_sin_historial_se_detecta_aunque_la_firma_sea_valida() {
    let (_d, profe, s, bytes) = entrega_base();
    let leida = entrega::leer(&bytes).unwrap();
    let carpeta = s.carpeta().to_path_buf();
    drop(s);
    let meta = rlp_core::almacen::leer_meta(&carpeta.join("alumno.db")).unwrap();
    let dek = *rlp_core::crypto::abrir_con_secreto(
        &meta.envolturas.contrasena,
        "clave-elsa",
        format!("dek|{}", meta.perfil.perfil_id).as_bytes(),
    )
    .unwrap();
    let mut p: Payload =
        entrega::descifrar_payload(&dek, &leida.manifiesto.perfil_id, &leida.payload).unwrap();
    p.actividades.get_mut("u1-a1").unwrap().codigo = "print('solución de otro')\n".into();
    let actividades = p.actividades.clone();
    let mala = entrega::construir(
        &dek,
        &p.perfil,
        &leida.manifiesto.envolturas,
        p.grupo.as_ref(),
        leida.manifiesto.grupo_id.clone(),
        actividades,
        p.eventos_firmados().unwrap(),
        leida.manifiesto.cabezas.clone(),
    )
    .unwrap();
    let e = abrir_entrega(&mala, &profe, &ctx()).unwrap();
    assert_eq!(
        nivel_de(&e, "firma"),
        nivel_firma_esperado(),
        "firma válida de la app"
    );
    assert_eq!(nivel_de(&e, "replay"), Nivel::Rojo);
    assert_eq!(e.reporte.nivel, Nivel::Rojo);
}

#[test]
fn eventos_borrados_rompen_la_cadena() {
    let (_d, profe, s, bytes) = entrega_base();
    let leida = entrega::leer(&bytes).unwrap();
    let carpeta = s.carpeta().to_path_buf();
    drop(s);
    let meta = rlp_core::almacen::leer_meta(&carpeta.join("alumno.db")).unwrap();
    let dek = *rlp_core::crypto::abrir_con_secreto(
        &meta.envolturas.contrasena,
        "clave-elsa",
        format!("dek|{}", meta.perfil.perfil_id).as_bytes(),
    )
    .unwrap();
    let p: Payload =
        entrega::descifrar_payload(&dek, &leida.manifiesto.perfil_id, &leida.payload).unwrap();
    let mut eventos = p.eventos_firmados().unwrap();
    eventos.remove(2);
    let mala = entrega::construir(
        &dek,
        &p.perfil,
        &leida.manifiesto.envolturas,
        p.grupo.as_ref(),
        leida.manifiesto.grupo_id.clone(),
        p.actividades.clone(),
        eventos,
        leida.manifiesto.cabezas.clone(),
    )
    .unwrap();
    let e = abrir_entrega(&mala, &profe, &ctx()).unwrap();
    assert_eq!(nivel_de(&e, "cadena"), Nivel::Rojo);
}

#[test]
fn otro_profesor_no_puede_abrir() {
    let (_d, _profe, _s, bytes) = entrega_base();
    let intruso = IdentidadProfesor::nueva("Otro Profe").unwrap();
    let e = abrir_entrega(&bytes, &intruso, &ctx()).unwrap();
    assert!(e.payload.is_none());
    assert_eq!(nivel_de(&e, "descifrado"), Nivel::Rojo);
}

#[test]
fn identidad_del_profesor_y_grupo() {
    let (profe, grupo) = profesor_y_grupo();
    let archivo = profe.a_archivo("clave-del-profe", &kdf()).unwrap();
    assert!(IdentidadProfesor::desde_archivo(&archivo, "incorrecta").is_err());
    let recuperada = IdentidadProfesor::desde_archivo(&archivo, "clave-del-profe").unwrap();
    assert_eq!(recuperada.publica_cifrado(), profe.publica_cifrado());
    // Un grupo alterado no se acepta.
    let mut malo = grupo.clone();
    malo.contenido = malo.contenido.replace("bloquear", "propio");
    assert!(rlp_core::grupo::verificar_grupo(&malo).is_err());
    assert!(rlp_core::grupo::verificar_grupo(&grupo).is_ok());
}

#[test]
fn desincronizacion_del_editor_se_corrige() {
    let dir = TempDir::new().unwrap();
    let (mut s, _) =
        SesionAlumno::registrar(dir.path(), "X1", "Fer Gil", "clave-fer1", None, kdf()).unwrap();
    s.abrir_actividad("u1-a1", "abc").unwrap();
    let lote = LoteOps {
        t0: 0,
        ops: vec![(0, 3, 3, "d".into(), "t".into())],
    };
    let r = s.guardar_edicion("u1-a1", &lote, "abcX").unwrap();
    assert!(!r.ok);
    assert_eq!(r.codigo, "abcd");
    // Operación fuera de rango: no se guarda nada.
    let lote = LoteOps {
        t0: 0,
        ops: vec![(0, 99, 99, "d".into(), "t".into())],
    };
    let r = s.guardar_edicion("u1-a1", &lote, "").unwrap();
    assert!(!r.ok);
    assert_eq!(r.codigo, "abcd");
    // Eventos no permitidos desde la interfaz.
    assert!(s
        .registrar_evento("edicion", Some("u1-a1"), json!({}))
        .is_err());
    assert!(s
        .registrar_evento("base", Some("u1-a1"), json!({}))
        .is_err());
}

/// Un semestre de trabajo: miles de lotes de edición. Abrir, exportar y verificar debe ser rápido.
#[test]
fn historial_grande_se_verifica_rapido() {
    let dir = TempDir::new().unwrap();
    let (profe, grupo) = profesor_y_grupo();
    let (mut s, _) = SesionAlumno::registrar(
        dir.path(),
        "21340007",
        "Gil Soto",
        "clave-gil1",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    let mut texto = s.abrir_actividad("u1-a1", INICIAL).unwrap().codigo;
    for i in 0..3000 {
        let linea = format!("x{i} = {i}\n");
        let mut lote = teclear(&texto, &linea);
        lote.t0 += i as i64 * 3_000; // un lote cada 3 s, como al teclear de verdad
        texto.push_str(&linea);
        assert!(s.guardar_edicion("u1-a1", &lote, &texto).unwrap().ok);
    }
    let inicio = std::time::Instant::now();
    let (_, bytes) = s.exportar().unwrap();
    let e = abrir_entrega(&bytes, &profe, &ctx()).unwrap();
    let transcurrido = inicio.elapsed();
    integra(&e);
    assert!(
        transcurrido.as_secs() < 20,
        "exportar y verificar tardó {transcurrido:?}"
    );
    eprintln!(
        "3000 lotes: exportar + verificar en {transcurrido:?} ({} KB)",
        bytes.len() / 1024
    );
}
