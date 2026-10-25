//! Forme animale du druide (Manuel des joueurs, druide p. 41 ; sorts forme bestiale, corps
//! élémentaire et forme végétale) : les formes que le druide peut prendre à chaque niveau, avec
//! les bonus chiffrés du livre. Le reste (vitesses, attaques naturelles…) reste en texte.

use crate::effet::{Effet, TypeBonus};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct EffetForme {
    pub cible: String,
    #[serde(rename = "type")]
    pub type_bonus: TypeBonus,
    pub valeur: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormeAnimale {
    pub id: String,
    pub nom: String,
    /// `animal`, `élémentaire` ou `plante`.
    pub categorie: String,
    pub taille: Option<String>,
    /// Niveau de druide à partir duquel la forme est permise.
    pub niveau_druide: u32,
    pub sort_en: String,
    pub effets: Vec<EffetForme>,
    pub texte: String,
}

#[derive(Debug, Deserialize)]
pub struct FichierFormes {
    pub formes: Vec<FormeAnimale>,
}

/// Début du nom des effets temporaires créés par une forme.
pub const PREFIXE_FORME: &str = "Forme animale";

impl FormeAnimale {
    /// Nom de l'effet temporaire quand le druide est transformé.
    pub fn nom_effet(&self) -> String {
        format!("{PREFIXE_FORME} : {}", self.nom)
    }

    /// Effets chiffrés de la forme (bonus de taille ou d'altération, armure naturelle).
    pub fn effets_chiffres(&self) -> Vec<Effet> {
        let source = self.nom_effet();
        self.effets
            .iter()
            .map(|e| Effet::new(&e.cible, e.type_bonus, e.valeur, &source))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::Referentiel;

    #[test]
    fn les_formes_viennent_des_sorts_du_livre() {
        let r = Referentiel::partage();
        assert_eq!(r.formes.len(), 26);
        // Chaque forme existe au moins une fois par niveau d'accès : 4 (animal P/M), 6, 8, 10, 12.
        let niveaux: std::collections::BTreeSet<u32> =
            r.formes.iter().map(|f| f.niveau_druide).collect();
        assert_eq!(
            niveaux.into_iter().collect::<Vec<_>>(),
            vec![4, 6, 8, 10, 12]
        );
        // Forme bestiale I, animal de taille M : +2 Force (bonus de taille), +2 armure naturelle.
        let m = r.forme("beast_shape_i_animal_de_taille_m").unwrap();
        assert_eq!(m.niveau_druide, 4);
        let e = m.effets_chiffres();
        assert!(e.iter().any(|x| x.cible.0 == "carac.for" && x.valeur == 2));
        assert!(e.iter().any(|x| x.cible.0 == "ca" && x.valeur == 2));
        // Aucune forme animale ne donne de créature magique.
        assert!(r.formes.iter().all(|f| !f.nom.contains("magique")));
        // Toutes les formes ont au moins un bonus chiffré.
        assert!(r.formes.iter().all(|f| !f.effets.is_empty()));
    }
}
