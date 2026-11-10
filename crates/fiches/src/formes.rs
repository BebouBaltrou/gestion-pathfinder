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

/// Multiplicateur de la caractéristique pour une attaque de forme : principale ×1, secondaire ×½.
/// Point à confirmer : le texte du sort ne précise pas ce multiplicateur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Attaque {
    Principale,
    Secondaire,
}

/// Toucher et bonus de dégâts d'une attaque de forme, avec les stats courantes du lanceur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttaqueForme {
    pub toucher: i32,
    pub bonus_degats: i32,
}

/// Genre d'une attaque de la créature, d'après sa fiche : une attaque notée « secondaire » est à
/// ½ Force, les autres à plein. La fiche du Bestiaire saisie ne marque pas toujours l'attaque :
/// dans ce cas on lit « principale » (point à confirmer avec le livre).
pub fn genre_selon_fiche(texte_attaque: &str) -> Attaque {
    if texte_attaque.to_lowercase().contains("secondaire") {
        Attaque::Secondaire
    } else {
        Attaque::Principale
    }
}

/// Attaque de forme. `bba` : BBA de la fiche (inclut déjà le bonus de taille de la race).
/// `taille_race` / `taille_forme` : codes P, M, G… ; la taille de la forme remplace celle de la race.
/// `score` : caractéristique d'attaque courante (Force, ou Dextérité pour les attaques de finesse).
pub fn attaque_de_forme(
    bba: i32,
    taille_race: &str,
    taille_forme: &str,
    score: i32,
    genre: Attaque,
) -> AttaqueForme {
    use crate::caracteristiques::modificateur;
    use crate::compagnon::modificateur_de_taille;
    let ecart_taille = modificateur_de_taille(taille_forme) - modificateur_de_taille(taille_race);
    let m = modificateur(score);
    let bonus_degats = match genre {
        Attaque::Principale => m,
        Attaque::Secondaire => m.div_euclid(2),
    };
    AttaqueForme {
        toucher: bba + ecart_taille + m,
        bonus_degats,
    }
}

/// Caractéristique courante : score de la fiche plus le bonus de la forme sur cette caractéristique.
pub fn score_courant(score_fiche: i32, carac_abrege: &str, forme: &FormeAnimale) -> i32 {
    let bonus: i32 = forme
        .effets
        .iter()
        .filter(|e| {
            e.cible
                .eq_ignore_ascii_case(&format!("carac.{carac_abrege}"))
        })
        .map(|e| e.valeur)
        .sum();
    score_fiche + bonus
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn l_attaque_de_forme_suit_les_stats_courantes() {
        // BBA 5, Force 16 (+3), race M, forme M : toucher 5 + 0 + 3 ; dégâts +3
        let a = attaque_de_forme(5, "M", "M", 16, Attaque::Principale);
        assert_eq!((a.toucher, a.bonus_degats), (8, 3));
        // une potion ajoute +2 de Force : le score courant passe à 18 (+4)
        let b = attaque_de_forme(5, "M", "M", 18, Attaque::Principale);
        assert_eq!((b.toucher, b.bonus_degats), (9, 4));
    }

    #[test]
    fn la_taille_de_la_forme_remplace_celle_de_la_race() {
        // race P (+1 déjà dans la BBA), forme G (−1) : écart −2
        let a = attaque_de_forme(6, "P", "G", 10, Attaque::Principale);
        assert_eq!(a.toucher, 6 - 2);
    }

    #[test]
    fn la_dexterite_et_la_force_sont_prises_separement() {
        // attaque de finesse : Dextérité 18 (+4), Force 8 (−1)
        let fin = attaque_de_forme(4, "M", "M", 18, Attaque::Principale);
        let force = attaque_de_forme(4, "M", "M", 8, Attaque::Principale);
        assert_eq!(fin.toucher, 8);
        assert_eq!(force.toucher, 3);
    }

    #[test]
    fn l_attaque_secondaire_prend_la_moitie_arrondie_vers_le_bas() {
        // Force 15 (+2) : principale +2, secondaire +1 ; Force 9 (−1) : secondaire −1
        assert_eq!(
            attaque_de_forme(0, "M", "M", 15, Attaque::Secondaire).bonus_degats,
            1
        );
        assert_eq!(
            attaque_de_forme(0, "M", "M", 9, Attaque::Secondaire).bonus_degats,
            -1
        );
    }

    #[test]
    fn le_bonus_de_forme_modifie_le_score_courant() {
        let f = Referentiel::partage()
            .formes
            .iter()
            .find(|f| f.id == "beast_shape_i_animal_de_taille_m")
            .unwrap()
            .clone();
        // Forme M : +2 Force ; score 14 devient 16
        assert_eq!(score_courant(14, "for", &f), 16);
        assert_eq!(score_courant(14, "con", &f), 14);
    }

    #[test]
    fn le_genre_suit_la_fiche_de_la_creature() {
        assert_eq!(
            genre_selon_fiche("coup +16 (2d6+10) secondaire"),
            Attaque::Secondaire
        );
        assert_eq!(
            genre_selon_fiche("défenses, +16 (2d8+10)"),
            Attaque::Principale
        );
    }
}
