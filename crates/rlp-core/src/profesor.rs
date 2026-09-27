//! Identidad del profesor: llaves X25519 (abrir entregas) y Ed25519 (firmar grupos).

use ed25519_dalek::SigningKey;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey, StaticSecret};
use zeroize::Zeroizing;

use crate::crypto::{
    abrir_con_secreto, abrir_envoltura, aleatorio, b64, cifrar, de_b64, de_b64_32, descifrar,
    envolver_con_secreto, firmar, EnvolturaPublica, EnvolturaSecreto, Llave, ParametrosKdf,
};
use crate::error::{Error, Resultado};
use crate::grupo;
use crate::modelo::{ahora_ms, nuevo_id, GrupoFirmado, GrupoInfo, Politicas};

pub const FORMATO_IDENTIDAD: &str = "rlp-identidad-profesor";

pub struct IdentidadProfesor {
    pub nombre: String,
    pub creado: i64,
    cifrado: StaticSecret,
    firma: SigningKey,
}

#[derive(Serialize, Deserialize)]
struct Secretos {
    cifrado: String,
    firma: String,
}

/// Identidad guardada en disco (y como respaldo), protegida con la contraseña del profesor.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchivoIdentidad {
    pub formato: String,
    pub nombre: String,
    pub creado: i64,
    pub publica_cifrado: String,
    pub publica_firma: String,
    pub envoltura: EnvolturaSecreto,
    pub secretos: String,
}

/// Datos para crear un grupo.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NuevoGrupo {
    pub nombre: String,
    #[serde(default)]
    pub materia: String,
    #[serde(default)]
    pub periodo: String,
    #[serde(default)]
    pub regex_control: String,
    #[serde(default)]
    pub politicas: Politicas,
    /// Llaves públicas X25519 de otros profesores que también podrán abrir las entregas.
    #[serde(default)]
    pub coprofesores: Vec<String>,
}

impl IdentidadProfesor {
    pub fn nueva(nombre: &str) -> Resultado<Self> {
        let nombre = nombre.trim();
        if nombre.chars().count() < 3 {
            return Err(Error::validacion("Escribe tu nombre."));
        }
        Ok(IdentidadProfesor {
            nombre: nombre.to_string(),
            creado: ahora_ms(),
            cifrado: StaticSecret::random_from_rng(OsRng),
            firma: SigningKey::generate(&mut OsRng),
        })
    }

    pub fn publica_cifrado(&self) -> String {
        b64(PublicKey::from(&self.cifrado).as_bytes())
    }

    pub fn publica_firma(&self) -> String {
        b64(self.firma.verifying_key().as_bytes())
    }

    pub fn a_archivo(&self, contrasena: &str, kdf: &ParametrosKdf) -> Resultado<ArchivoIdentidad> {
        if contrasena.chars().count() < 10 {
            return Err(Error::validacion(
                "La contraseña del profesor debe tener al menos 10 caracteres.",
            ));
        }
        let kek: Llave = aleatorio();
        let secretos = Zeroizing::new(serde_json::to_vec(&Secretos {
            cifrado: b64(self.cifrado.as_bytes()),
            firma: b64(self.firma.as_bytes()),
        })?);
        Ok(ArchivoIdentidad {
            formato: FORMATO_IDENTIDAD.into(),
            nombre: self.nombre.clone(),
            creado: self.creado,
            publica_cifrado: self.publica_cifrado(),
            publica_firma: self.publica_firma(),
            envoltura: envolver_con_secreto(&kek, contrasena, kdf, b"profesor")?,
            secretos: b64(&cifrar(&kek, b"profesor-secretos", &secretos)),
        })
    }

    pub fn desde_archivo(a: &ArchivoIdentidad, contrasena: &str) -> Resultado<Self> {
        if a.formato != FORMATO_IDENTIDAD {
            return Err(Error::Formato(
                "no es un archivo de llaves del profesor".into(),
            ));
        }
        let kek = abrir_con_secreto(&a.envoltura, contrasena, b"profesor")?;
        let json = Zeroizing::new(
            descifrar(&kek, b"profesor-secretos", &de_b64(&a.secretos)?)
                .map_err(|_| Error::Credenciales)?,
        );
        let s: Secretos = serde_json::from_slice(&json)?;
        let id = IdentidadProfesor {
            nombre: a.nombre.clone(),
            creado: a.creado,
            cifrado: StaticSecret::from(de_b64_32(&s.cifrado)?),
            firma: SigningKey::from_bytes(&de_b64_32(&s.firma)?),
        };
        if id.publica_cifrado() != a.publica_cifrado || id.publica_firma() != a.publica_firma {
            return Err(Error::Alterado("llaves inconsistentes".into()));
        }
        Ok(id)
    }

    /// Crea y firma la configuración de un grupo.
    pub fn crear_grupo(&self, datos: &NuevoGrupo) -> Resultado<(GrupoFirmado, GrupoInfo)> {
        if datos.nombre.trim().is_empty() {
            return Err(Error::validacion("Escribe el nombre del grupo."));
        }
        if !datos.regex_control.trim().is_empty()
            && regex::Regex::new(&datos.regex_control).is_err()
        {
            return Err(Error::validacion(
                "La regla del número de control no es una expresión regular válida.",
            ));
        }
        if !matches!(datos.politicas.pegado.as_str(), "bloquear" | "propio") {
            return Err(Error::validacion("Política de pegado inválida."));
        }
        let mut llaves = vec![self.publica_cifrado()];
        for c in &datos.coprofesores {
            de_b64_32(c).map_err(|_| Error::validacion("Llave de coprofesor inválida."))?;
            if !llaves.contains(c) {
                llaves.push(c.clone());
            }
        }
        let info = GrupoInfo {
            formato: grupo::FORMATO.into(),
            grupo_id: nuevo_id(),
            nombre: datos.nombre.trim().into(),
            materia: datos.materia.trim().into(),
            periodo: datos.periodo.trim().into(),
            profesor: self.nombre.clone(),
            llaves_cifrado: llaves,
            llave_firma: self.publica_firma(),
            politicas: datos.politicas.clone(),
            regex_control: datos.regex_control.trim().into(),
            creado: ahora_ms(),
        };
        let contenido = serde_json::to_string_pretty(&info)?;
        let firma = firmar(&self.firma, grupo::CONTEXTO_FIRMA, contenido.as_bytes());
        Ok((GrupoFirmado { contenido, firma }, info))
    }

    /// Desenvuelve la llave de datos de un alumno con cualquiera de las envolturas dirigidas a
    /// este profesor.
    pub fn abrir_dek(
        &self,
        envolturas: &[EnvolturaPublica],
        perfil_id: &str,
    ) -> Resultado<Zeroizing<Llave>> {
        let aad = format!("dek|{perfil_id}").into_bytes();
        let mia = self.publica_cifrado();
        envolturas
            .iter()
            .filter(|e| e.destinatario == mia)
            .find_map(|e| abrir_envoltura(e, &self.cifrado, &aad).ok())
            .ok_or(Error::Credenciales)
    }
}
