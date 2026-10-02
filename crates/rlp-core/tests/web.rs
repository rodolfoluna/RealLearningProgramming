//! Perfiles de la versión web: el núcleo trabaja en memoria y la app guarda el diario de cambios
//! (en el navegador, en IndexedDB). Las entregas son las mismas que en escritorio y Android.
#![cfg(feature = "firmar")]

use std::collections::HashMap;

use rlp_core::almacen::MetaPerfil;
use rlp_core::alumno::{PerfilesEnMemoria, Secreto, SesionAlumno};
use rlp_core::crypto::ParametrosKdf;
use rlp_core::deposito::{Deposito, DepositoMemoria, Lote};
use rlp_core::modelo::{GrupoFirmado, Politicas};
use rlp_core::profesor::{IdentidadProfesor, NuevoGrupo};
use rlp_core::replay::LoteOps;
use rlp_core::verificacion::{abrir_entrega, Contexto, Nivel};

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
    let profe = IdentidadProfesor::nueva("Profa. Web").unwrap();
    let (grupo, _) = profe
        .crear_grupo(&NuevoGrupo {
            nombre: "Programación 1B".into(),
            materia: "Fundamentos".into(),
            periodo: "2026-2".into(),
            regex_control: r"\d{8}".into(),
            politicas: Politicas::default(),
            coprofesores: vec![],
        })
        .unwrap();
    (profe, grupo)
}

fn escribir(s: &mut SesionAlumno, act: &str, texto: &str) {
    let actual = s.abrir_actividad(act, INICIAL).unwrap().codigo;
    let mut pos = actual.encode_utf16().count();
    let ops = texto
        .chars()
        .enumerate()
        .map(|(i, c)| {
            let op = (i as i64 * 150, pos, pos, c.to_string(), "t".to_string());
            pos += c.len_utf16();
            op
        })
        .collect();
    let lote = LoteOps {
        t0: rlp_core::modelo::ahora_ms(),
        ops,
    };
    let r = s
        .guardar_edicion(act, &lote, &format!("{actual}{texto}"))
        .unwrap();
    assert!(r.ok);
}

/// Lo que hace la app web con IndexedDB: aplica cada diario sobre lo que ya tenía guardado.
#[derive(Default)]
struct Navegador {
    guardado: DepositoMemoria,
}

impl Navegador {
    fn guardar(&mut self, diario: Option<Lote>) {
        if let Some(lote) = diario {
            self.guardado.escribir(lote).unwrap();
        }
    }

    fn reabrir(&self) -> Box<dyn Deposito> {
        Box::new(DepositoMemoria::desde(self.guardado.instantanea()).unwrap())
    }

    fn meta(&self) -> MetaPerfil {
        serde_json::from_str(&self.guardado.meta("perfil").unwrap().unwrap()).unwrap()
    }
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

fn sin_rojos(bytes: &[u8], profe: &IdentidadProfesor) -> rlp_core::entrega::Payload {
    let e = abrir_entrega(bytes, profe, &ctx()).unwrap();
    for c in &e.reporte.checks {
        assert_ne!(c.nivel, Nivel::Rojo, "{}: {}", c.nombre, c.detalle);
    }
    e.payload.unwrap()
}

#[test]
fn perfil_web_se_guarda_por_diario_y_se_vuelve_a_abrir() {
    let (profe, grupo) = profesor_y_grupo();
    let mut nav = Navegador::default();

    let (mut s, _codigo) = SesionAlumno::registrar_en(
        &PerfilesEnMemoria::default(),
        "21340100",
        "Elena Web",
        "clave-elena",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    assert_eq!(s.ubicacion(), s.perfil().perfil_id);
    nav.guardar(s.tomar_diario());
    escribir(&mut s, "u1-a1", "print('hola')\n");
    nav.guardar(s.tomar_diario());
    escribir(&mut s, "u1-a2", "x = 1\n");
    nav.guardar(s.cerrar().unwrap());

    // Ya existe: no se puede registrar otra vez el mismo número de control.
    let existentes = PerfilesEnMemoria {
        existentes: vec![nav.meta()],
    };
    assert!(SesionAlumno::registrar_en(
        &existentes,
        "21340100",
        "Elena Otra",
        "clave-elena",
        Some(&grupo),
        kdf()
    )
    .is_err());

    // Al volver a abrir la página: contraseña incorrecta, luego la buena.
    let id = nav.meta().perfil.perfil_id;
    assert!(
        SesionAlumno::abrir_en(id.clone(), nav.reabrir(), &contrasena("mala-clave"), kdf())
            .is_err()
    );
    let mut s =
        SesionAlumno::abrir_en(id, nav.reabrir(), &contrasena("clave-elena"), kdf()).unwrap();
    let estado = s.estado().unwrap();
    assert_eq!(
        estado.actividades["u1-a1"].codigo,
        format!("{INICIAL}print('hola')\n")
    );
    escribir(&mut s, "u1-a1", "print('otra vez')\n");
    nav.guardar(s.tomar_diario());

    // La entrega es la misma que en escritorio: el profesor la verifica.
    let (_, bytes) = s.exportar().unwrap();
    nav.guardar(s.tomar_diario());
    let p = sin_rojos(&bytes, &profe);
    assert_eq!(
        p.actividades["u1-a1"].codigo,
        format!("{INICIAL}print('hola')\nprint('otra vez')\n")
    );

    // Un diario perdido (la app no lo guardó) deja la copia guardada en un estado anterior pero
    // consistente: se abre sin problemas.
    escribir(&mut s, "u1-a2", "y = 2\n");
    let s2 = SesionAlumno::abrir_en(
        s.ubicacion().to_string(),
        nav.reabrir(),
        &contrasena("clave-elena"),
        kdf(),
    )
    .unwrap();
    assert_eq!(
        s2.estado().unwrap().actividades["u1-a2"].codigo,
        format!("{INICIAL}x = 1\n")
    );
    s2.cerrar().unwrap();
}

#[cfg(feature = "sqlite")]
#[test]
fn continuar_entre_la_web_y_el_escritorio() {
    let (profe, grupo) = profesor_y_grupo();
    let pc = tempfile::TempDir::new().unwrap();

    // Empieza en el escritorio…
    let (mut s, _) = SesionAlumno::registrar(
        pc.path(),
        "21340101",
        "Fabián Mixto",
        "clave-fabian",
        Some(&grupo),
        kdf(),
    )
    .unwrap();
    escribir(&mut s, "u1-a1", "a = 1\n");
    let (_, archivo1) = s.exportar().unwrap();

    // …sigue en el navegador…
    let mut nav = Navegador::default();
    let mut w = SesionAlumno::restaurar_en(
        &PerfilesEnMemoria::default(),
        &archivo1,
        &contrasena("clave-fabian"),
        kdf(),
    )
    .unwrap();
    escribir(&mut w, "u1-a1", "b = 2\n");
    let (_, archivo2) = w.exportar().unwrap();
    nav.guardar(w.cerrar().unwrap());
    sin_rojos(&archivo2, &profe);

    // …y vuelve al escritorio con todo el historial.
    std::thread::sleep(std::time::Duration::from_millis(5));
    let r = s.importar(&archivo2).unwrap();
    assert!(r.conflictos.is_empty(), "{:?}", r.conflictos);
    assert_eq!(
        s.estado().unwrap().actividades["u1-a1"].codigo,
        format!("{INICIAL}a = 1\nb = 2\n")
    );
    let (_, archivo3) = s.exportar().unwrap();
    let p = sin_rojos(&archivo3, &profe);
    assert!(!p.eventos_firmados().unwrap().is_empty());

    // La copia guardada en el navegador también importa lo nuevo del escritorio.
    let id = nav.meta().perfil.perfil_id;
    let mut w =
        SesionAlumno::abrir_en(id, nav.reabrir(), &contrasena("clave-fabian"), kdf()).unwrap();
    let r = w.importar(&archivo3).unwrap();
    assert!(r.eventos_nuevos > 0 && r.conflictos.is_empty());
    nav.guardar(w.tomar_diario());
}
