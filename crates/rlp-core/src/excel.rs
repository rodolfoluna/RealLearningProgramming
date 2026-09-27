//! Exportación del avance de un grupo a Excel (.xlsx) para la App Profesor: hojas Resumen,
//! Actividades y Calificaciones (con los comentarios como notas).

use std::collections::BTreeMap;

use rust_xlsxwriter::{
    Color, Format, FormatAlign, FormatBorder, Note, Workbook, Worksheet, XlsxError,
};

use crate::bd_profesor::{BdProfesor, Calificacion, FilaTablero};
use crate::error::{Error, Resultado};
use crate::verificacion::Nivel;

fn error(e: XlsxError) -> Error {
    Error::Formato(format!("no se pudo crear el archivo de Excel: {e}"))
}

fn nivel(n: Nivel) -> &'static str {
    match n {
        Nivel::Verde => "Íntegra",
        Nivel::Amarillo => "Revisar",
        Nivel::Rojo => "Alterada",
    }
}

struct Formatos {
    cabeza: Format,
    cabeza_vertical: Format,
    verde: Format,
    amarillo: Format,
    rojo: Format,
    centro: Format,
    decimal: Format,
}

impl Formatos {
    fn nuevos() -> Self {
        let cabeza = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0xE3EBFB))
            .set_border(FormatBorder::Thin)
            .set_text_wrap()
            .set_align(FormatAlign::VerticalCenter);
        Formatos {
            cabeza_vertical: cabeza
                .clone()
                .set_rotation(90)
                .set_align(FormatAlign::Center),
            cabeza,
            verde: Format::new()
                .set_background_color(Color::RGB(0xD7F0DD))
                .set_align(FormatAlign::Center),
            amarillo: Format::new()
                .set_background_color(Color::RGB(0xFFF1C2))
                .set_align(FormatAlign::Center),
            rojo: Format::new()
                .set_background_color(Color::RGB(0xF9D6D5))
                .set_align(FormatAlign::Center),
            centro: Format::new().set_align(FormatAlign::Center),
            decimal: Format::new().set_num_format("0.0"),
        }
    }

    fn de_nivel(&self, n: Nivel) -> &Format {
        match n {
            Nivel::Verde => &self.verde,
            Nivel::Amarillo => &self.amarillo,
            Nivel::Rojo => &self.rojo,
        }
    }
}

/// Número de control y nombre en las dos primeras columnas, con esas columnas fijas.
fn encabezado_alumno(hoja: &mut Worksheet, f: &Formatos) -> Result<(), XlsxError> {
    hoja.write_string_with_format(0, 0, "Número de control", &f.cabeza)?;
    hoja.write_string_with_format(0, 1, "Nombre", &f.cabeza)?;
    hoja.set_column_width(0, 16)?;
    hoja.set_column_width(1, 30)?;
    hoja.set_freeze_panes(1, 2)?;
    Ok(())
}

fn alumno(hoja: &mut Worksheet, fila: u32, a: &FilaTablero) -> Result<(), XlsxError> {
    hoja.write_string(fila, 0, &a.numero_control)?;
    hoja.write_string(fila, 1, &a.nombre)?;
    Ok(())
}

/// Genera el libro de Excel con el avance del grupo (o de todos los alumnos si no hay grupo).
/// `actividades` son los pares (id, título) del curso, en orden.
pub fn exportar_xlsx(
    bd: &BdProfesor,
    grupo_id: Option<&str>,
    actividades: &[(String, String)],
) -> Resultado<Vec<u8>> {
    let filas = bd.tablero(grupo_id)?;
    let mut calificaciones: Vec<BTreeMap<String, Calificacion>> = Vec::new();
    for f in &filas {
        calificaciones.push(bd.calificaciones(&f.perfil_id)?);
    }
    construir(&filas, &calificaciones, actividades).map_err(error)
}

fn construir(
    filas: &[FilaTablero],
    calificaciones: &[BTreeMap<String, Calificacion>],
    actividades: &[(String, String)],
) -> Result<Vec<u8>, XlsxError> {
    let f = Formatos::nuevos();
    let mut libro = Workbook::new();
    let ultima = filas.len() as u32;

    // --- Resumen
    let hoja = libro.add_worksheet().set_name("Resumen")?;
    encabezado_alumno(hoja, &f)?;
    let columnas = [
        "Integridad",
        "Actividades completadas",
        "Puntos",
        "Promedio de calificaciones",
        "Tiempo (min)",
        "Ejecuciones",
        "Errores",
        "Copias",
        "Intentos de pegar",
        "Salidas de ventana",
        "Pistas",
        "Entregas",
    ];
    for (i, c) in columnas.iter().enumerate() {
        hoja.write_string_with_format(0, 2 + i as u16, *c, &f.cabeza)?;
        hoja.set_column_width(2 + i as u16, 13)?;
    }
    hoja.set_row_height(0, 32)?;
    for (i, (a, cal)) in filas.iter().zip(calificaciones).enumerate() {
        let r = 1 + i as u32;
        alumno(hoja, r, a)?;
        hoja.write_string_with_format(r, 2, nivel(a.nivel), f.de_nivel(a.nivel))?;
        let completadas = a.actividades.values().filter(|x| x.completada).count();
        let puntos: u32 = a.actividades.values().map(|x| x.puntos).sum();
        hoja.write_number(r, 3, completadas as f64)?;
        hoja.write_number(r, 4, puntos as f64)?;
        let notas: Vec<f64> = cal.values().filter_map(|c| c.calificacion).collect();
        if !notas.is_empty() {
            let promedio = notas.iter().sum::<f64>() / notas.len() as f64;
            hoja.write_number_with_format(r, 5, promedio, &f.decimal)?;
        }
        let g = &a.global;
        let numeros = [
            (g.tiempo_ms as f64 / 60000.0).round(),
            g.ejecuciones as f64,
            g.errores as f64,
            g.copias as f64,
            g.pegados_intentos as f64,
            g.salidas as f64,
            g.pistas as f64,
            a.entregas as f64,
        ];
        for (j, n) in numeros.iter().enumerate() {
            hoja.write_number(r, 6 + j as u16, *n)?;
        }
    }
    if ultima > 0 {
        hoja.autofilter(0, 0, ultima, 1 + columnas.len() as u16)?;
    }

    // --- Actividades: una columna por actividad.
    let hoja = libro.add_worksheet().set_name("Actividades")?;
    encabezado_alumno(hoja, &f)?;
    hoja.set_row_height(0, 150)?;
    for (j, (_, titulo)) in actividades.iter().enumerate() {
        let c = 2 + j as u16;
        hoja.write_string_with_format(0, c, titulo, &f.cabeza_vertical)?;
        hoja.set_column_width(c, 5)?;
    }
    for (i, a) in filas.iter().enumerate() {
        let r = 1 + i as u32;
        alumno(hoja, r, a)?;
        for (j, (id, _)) in actividades.iter().enumerate() {
            let c = 2 + j as u16;
            match a.actividades.get(id) {
                Some(x) if x.completada => hoja.write_string_with_format(r, c, "✓", &f.verde)?,
                Some(x) if x.total > 0 => hoja.write_string_with_format(
                    r,
                    c,
                    format!("{}/{}", x.pasadas, x.total),
                    &f.amarillo,
                )?,
                Some(_) => hoja.write_string_with_format(r, c, "…", &f.amarillo)?,
                None => hoja,
            };
        }
    }

    // --- Calificaciones: solo las actividades que el profesor ya calificó a alguien.
    let hoja = libro.add_worksheet().set_name("Calificaciones")?;
    encabezado_alumno(hoja, &f)?;
    let calificadas: Vec<&(String, String)> = actividades
        .iter()
        .filter(|(id, _)| calificaciones.iter().any(|c| c.contains_key(id)))
        .collect();
    if calificadas.is_empty() {
        hoja.write_string(
            1,
            0,
            "Aún no hay calificaciones capturadas en la App Profesor.",
        )?;
    } else {
        hoja.set_row_height(0, 60)?;
        for (j, (_, titulo)) in calificadas.iter().enumerate() {
            hoja.write_string_with_format(0, 2 + j as u16, titulo, &f.cabeza)?;
            hoja.set_column_width(2 + j as u16, 14)?;
        }
        let col_promedio = 2 + calificadas.len() as u16;
        hoja.write_string_with_format(0, col_promedio, "Promedio", &f.cabeza)?;
        for (i, (a, cal)) in filas.iter().zip(calificaciones).enumerate() {
            let r = 1 + i as u32;
            alumno(hoja, r, a)?;
            for (j, (id, _)) in calificadas.iter().enumerate() {
                let c = 2 + j as u16;
                let Some(x) = cal.get(id) else { continue };
                if let Some(n) = x.calificacion {
                    hoja.write_number_with_format(r, c, n, &f.centro)?;
                }
                if !x.comentario.trim().is_empty() {
                    hoja.insert_note(r, c, &Note::new(x.comentario.trim()).set_author("Profesor"))?;
                }
            }
            let rango = format!(
                "{}{}:{}{}",
                columna(2),
                r + 1,
                columna(col_promedio - 1),
                r + 1
            );
            hoja.write_formula_with_format(
                r,
                col_promedio,
                format!("=IFERROR(AVERAGE({rango}),\"\")").as_str(),
                &f.decimal,
            )?;
        }
    }

    libro.save_to_buffer()
}

/// Nombre de columna de Excel (0 → A, 26 → AA).
fn columna(mut n: u16) -> String {
    let mut s = Vec::new();
    loop {
        s.push(b'A' + (n % 26) as u8);
        if n < 26 {
            break;
        }
        n = n / 26 - 1;
    }
    s.reverse();
    String::from_utf8(s).unwrap_or_default()
}

#[cfg(test)]
mod pruebas {
    use super::columna;

    #[test]
    fn nombres_de_columna() {
        assert_eq!(columna(0), "A");
        assert_eq!(columna(25), "Z");
        assert_eq!(columna(26), "AA");
        assert_eq!(columna(27), "AB");
        assert_eq!(columna(701), "ZZ");
        assert_eq!(columna(702), "AAA");
    }
}
