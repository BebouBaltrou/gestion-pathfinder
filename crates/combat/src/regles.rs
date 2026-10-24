//! Registre des règles optionnelles de combat (données dans `data/combat/regles_optionnelles.json`).
//! Chaque règle a une source ; on n'ajoute une règle que d'après un texte du livre ou une décision du joueur.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

const JSON: &str = include_str!("../../../data/combat/regles_optionnelles.json");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RegleOptionnelle {
    pub id: String,
    pub nom: String,
    pub description: String,
    pub source: String,
    pub par_defaut: bool,
}

#[derive(Deserialize)]
struct Fichier {
    regles: Vec<RegleOptionnelle>,
}

/// Règles optionnelles connues, dans l'ordre du fichier.
pub fn catalogue() -> &'static [RegleOptionnelle] {
    static T: OnceLock<Vec<RegleOptionnelle>> = OnceLock::new();
    T.get_or_init(|| {
        serde_json::from_str::<Fichier>(JSON)
            .expect("data/combat/regles_optionnelles.json est valide")
            .regles
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn les_degats_massifs_sont_au_registre_avec_leur_source() {
        let r = catalogue()
            .iter()
            .find(|r| r.id == "degats_massifs")
            .unwrap();
        assert!(r.source.contains("189") && !r.par_defaut);
    }
}
