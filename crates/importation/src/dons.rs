//! Rapprochement des dons écrits en texte libre avec le catalogue du Manuel de base.
//!
//! Les noms français du catalogue sont des traductions non officielles
//! (`data/core/fr/dons_noms.json`) : un don non reconnu n'est jamais deviné, il est signalé.

use std::collections::BTreeMap;

/// Résultat du rapprochement d'une ligne de la feuille.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DonLu {
    /// Ligne telle qu'écrite sur la feuille.
    pub texte: String,
    /// Nom anglais du catalogue, si reconnu.
    pub nom_en: Option<String>,
    /// Précision éventuelle entre parenthèses (« Évocation »).
    pub precision: Option<String>,
    /// Reconnu par un nom de la feuille connu (et non par le catalogue de l'application).
    pub par_alias: bool,
}

/// Minuscules, sans accents ni ponctuation, pour comparer sans se soucier de la saisie.
pub(crate) fn normaliser(s: &str) -> String {
    let mut out = String::new();
    for c in s.to_lowercase().chars() {
        let c = match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'œ' => {
                out.push('o');
                'e'
            }
            c => c,
        };
        if c.is_alphanumeric() {
            out.push(c);
        } else if c.is_whitespace() || c == '\'' || c == '’' || c == '-' {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn noms_fr() -> BTreeMap<String, String> {
    let v: serde_json_lite::Noms =
        serde_json_lite::lire(include_str!("../../../data/core/fr/dons_noms.json"));
    v.0.into_iter()
        .map(|(en, fr)| (normaliser(&fr), en))
        .collect()
}

/// Lecture minimale du fichier de noms (évite d'ajouter une dépendance JSON ici).
mod serde_json_lite {
    pub struct Noms(pub Vec<(String, String)>);

    /// Extrait les paires `"Nom anglais": "Nom français"` de la section « noms ».
    pub fn lire(json: &str) -> Noms {
        let debut = json.find("\"noms\"").unwrap_or(0);
        let mut paires = Vec::new();
        for ligne in json[debut..].lines().skip(1) {
            let l = ligne.trim().trim_end_matches(',');
            if let Some((a, b)) = l.split_once("\": \"") {
                let a = a.trim_start_matches('"');
                let b = b.trim_end_matches('"');
                paires.push((a.replace("\\\"", "\""), b.replace("\\\"", "\"")));
            }
        }
        Noms(paires)
    }
}

/// Découpe le texte libre en lignes de dons : une ligne par don, tirets et puces retirés,
/// et on s'arrête à ce qui n'est manifestement pas un don (ligne vide, description longue).
pub fn lignes_de_dons(texte: &str) -> Vec<String> {
    texte
        .lines()
        .map(|l| {
            l.trim()
                .trim_start_matches(['-', '•', '*'])
                .trim()
                .to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn rapprocher(texte: &str) -> Vec<DonLu> {
    let catalogue = noms_fr();
    lignes_de_dons(texte)
        .into_iter()
        .map(|ligne| {
            let (nom, precision) = match ligne.split_once('(') {
                Some((n, p)) => (
                    n.trim().to_string(),
                    Some(p.trim_end_matches(')').trim().to_string()),
                ),
                None => (ligne.clone(), None),
            };
            let cle = normaliser(&nom);
            // Les noms de la feuille sont parfois abrégés ou légèrement différents : on accepte
            // une égalité, ou un nom du catalogue qui commence par le nom écrit (ou l'inverse),
            // seulement s'il n'y a qu'un candidat.
            let alias = ALIAS
                .iter()
                .find(|(fr, _)| {
                    let a = normaliser(fr);
                    cle == a || cle.starts_with(&format!("{a} "))
                })
                .map(|(_, en)| en.to_string());
            let par_alias = alias.is_some();
            let exact = alias.or_else(|| catalogue.get(&cle).cloned());
            let nom_en = exact.or_else(|| {
                let candidats: Vec<&String> = catalogue
                    .iter()
                    .filter(|(fr, _)| {
                        cle.len() >= 6 && (fr.starts_with(&cle) || cle.starts_with(fr.as_str()))
                    })
                    .map(|(_, en)| en)
                    .collect();
                (candidats.len() == 1).then(|| candidats[0].clone())
            });
            DonLu {
                texte: ligne,
                nom_en,
                precision,
                par_alias,
            }
        })
        .collect()
}

/// Variantes d'écriture vues sur des feuilles de joueurs (singulier, pluriel, faute d'accord) :
/// les noms officiels viennent du catalogue (`data/core/fr/dons_noms.json`, aligné sur le
/// Manuel des joueurs).
const ALIAS: &[(&str, &str)] = &[
    ("Création de baguette magique", "Craft Wand"),
    ("Efficacité des sorts accrues", "Spell Penetration"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnait_un_don_ecrit_a_la_main() {
        let dons =
            rapprocher("-Science du contresort\n- École renforcée (Évocation)\n-Truc inconnu");
        assert_eq!(dons.len(), 3);
        assert!(dons[2].nom_en.is_none());
        assert_eq!(dons[1].precision.as_deref(), Some("Évocation"));
    }

    #[test]
    fn normalise_les_accents() {
        assert_eq!(normaliser("École  Renforcée"), "ecole renforcee");
    }
}
