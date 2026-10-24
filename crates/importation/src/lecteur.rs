//! Lecture des champs d'un PDF à formulaire (AcroForm), sans aucune interprétation.

use lopdf::{Dictionary, Document, Object};
use std::collections::BTreeMap;
use std::path::Path;

/// Valeurs des champs, par nom complet (ex. `Compétences.4.0`). Une case à cocher vaut
/// `"oui"` ou `"non"`.
pub type Champs = BTreeMap<String, String>;

pub fn lire_champs(chemin: &Path) -> Result<Champs, String> {
    let doc = Document::load(chemin).map_err(|e| format!("PDF illisible : {e}"))?;
    lire_champs_document(&doc)
}

pub fn lire_champs_octets(octets: &[u8]) -> Result<Champs, String> {
    let doc = Document::load_mem(octets).map_err(|e| format!("PDF illisible : {e}"))?;
    lire_champs_document(&doc)
}

fn lire_champs_document(doc: &Document) -> Result<Champs, String> {
    let racine = doc
        .catalog()
        .map_err(|e| format!("PDF sans catalogue : {e}"))?;
    let acro = racine
        .get(b"AcroForm")
        .ok()
        .and_then(|o| resoudre(doc, o))
        .ok_or("ce PDF n'a pas de champs remplissables (formulaire absent)")?;
    let champs = acro
        .get(b"Fields")
        .ok()
        .and_then(|o| resoudre_tableau(doc, o))
        .ok_or("formulaire sans champs")?;
    let mut sortie = Champs::new();
    for c in champs {
        parcourir(doc, &c, "", &mut sortie, 0);
    }
    Ok(sortie)
}

fn resoudre<'a>(doc: &'a Document, o: &'a Object) -> Option<&'a Dictionary> {
    match o {
        Object::Reference(id) => doc.get_object(*id).ok()?.as_dict().ok(),
        Object::Dictionary(d) => Some(d),
        _ => None,
    }
}

fn resoudre_tableau(doc: &Document, o: &Object) -> Option<Vec<Object>> {
    match o {
        Object::Reference(id) => resoudre_tableau(doc, doc.get_object(*id).ok()?),
        Object::Array(a) => Some(a.clone()),
        _ => None,
    }
}

fn parcourir(doc: &Document, objet: &Object, prefixe: &str, sortie: &mut Champs, prof: usize) {
    if prof > 8 {
        return;
    }
    let Some(d) = resoudre(doc, objet) else {
        return;
    };
    let nom = d
        .get(b"T")
        .ok()
        .and_then(|t| texte(doc, t))
        .unwrap_or_default();
    let complet = match (prefixe.is_empty(), nom.is_empty()) {
        (_, true) => prefixe.to_string(),
        (true, false) => nom,
        (false, false) => format!("{prefixe}.{nom}"),
    };
    if let Some(enfants) = d.get(b"Kids").ok().and_then(|k| resoudre_tableau(doc, k)) {
        // Les enfants sans nom propre sont de simples « widgets » du même champ.
        let a_des_champs = enfants
            .iter()
            .filter_map(|e| resoudre(doc, e))
            .any(|e| e.has(b"T"));
        if a_des_champs {
            for e in &enfants {
                parcourir(doc, e, &complet, sortie, prof + 1);
            }
            return;
        }
    }
    if complet.is_empty() {
        return;
    }
    let valeur = match d.get(b"V") {
        Ok(Object::Name(n)) => Some(if n == b"Off" { "non" } else { "oui" }.to_string()),
        Ok(v) => texte(doc, v),
        Err(_) => None,
    };
    sortie.insert(complet, valeur.unwrap_or_default());
}

fn texte(doc: &Document, o: &Object) -> Option<String> {
    match o {
        Object::Reference(id) => texte(doc, doc.get_object(*id).ok()?),
        Object::String(octets, _) => Some(decoder(octets)),
        Object::Name(n) => Some(String::from_utf8_lossy(n).into_owned()),
        _ => None,
    }
}

fn decoder(octets: &[u8]) -> String {
    if octets.starts_with(&[0xFE, 0xFF]) {
        let u: Vec<u16> = octets[2..]
            .chunks(2)
            .filter(|c| c.len() == 2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&u)
    } else if let Ok(s) = std::str::from_utf8(octets) {
        s.to_string()
    } else {
        // PDFDocEncoding ~ Latin-1 pour les caractères accentués courants.
        octets.iter().map(|&b| b as char).collect()
    }
}
