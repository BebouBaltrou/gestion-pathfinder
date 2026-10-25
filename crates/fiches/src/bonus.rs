//! Agrégation des effets et empilement des bonus.

use crate::effet::{Cible, Effet, TypeBonus};
use std::collections::BTreeMap;

/// Une ligne de l'explication d'une valeur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terme {
    pub libelle: String,
    pub valeur: i32,
    /// `false` : le terme est affiché mais ne compte pas (bonus de même type moins élevé).
    pub compte: bool,
}

/// Valeur calculée et sa décomposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detail {
    pub total: i32,
    pub termes: Vec<Terme>,
}

impl Detail {
    /// Texte du type « 16 = +10 base +2 DEX +4 armure ».
    pub fn explication(&self) -> String {
        let mut s = format!("{} =", self.total);
        for t in self.termes.iter().filter(|t| t.compte) {
            s.push_str(&format!(" {:+} {}", t.valeur, t.libelle));
        }
        s
    }
}

/// Calcule une valeur pour `cible` : `base` (termes fixes déjà connus, ex. « base 10 »)
/// puis les effets visant la cible.
///
/// Règles (Core, « Types de bonus ») :
/// - les bonus de même type ne se cumulent pas : seul le plus élevé compte ;
/// - exceptions : esquive, circonstance et sans type se cumulent ;
/// - les malus (valeurs négatives) se cumulent toujours.
pub fn calculer(base: &[(&str, i32)], effets: &[Effet], cible: &Cible) -> Detail {
    let mut termes: Vec<Terme> = base
        .iter()
        .map(|(l, v)| Terme {
            libelle: (*l).to_string(),
            valeur: *v,
            compte: true,
        })
        .collect();

    let visees: Vec<&Effet> = effets.iter().filter(|e| &e.cible == cible).collect();

    // Meilleur bonus (le premier rencontré en cas d'égalité) pour chaque type non cumulable.
    let mut meilleur: BTreeMap<TypeBonus, usize> = BTreeMap::new();
    for (i, e) in visees.iter().enumerate() {
        if e.valeur > 0 && !e.type_bonus.cumulable() {
            match meilleur.get(&e.type_bonus) {
                Some(&j) if visees[j].valeur >= e.valeur => {}
                _ => {
                    meilleur.insert(e.type_bonus, i);
                }
            }
        }
    }

    for (i, e) in visees.iter().enumerate() {
        let compte =
            e.valeur <= 0 || e.type_bonus.cumulable() || meilleur.get(&e.type_bonus) == Some(&i);
        let libelle = if e.valeur < 0 {
            format!("{} (malus, {})", e.source, e.type_bonus.nom())
        } else if compte {
            format!("{} ({})", e.source, e.type_bonus.nom())
        } else {
            format!("{} ({}, ne se cumule pas)", e.source, e.type_bonus.nom())
        };
        termes.push(Terme {
            libelle,
            valeur: e.valeur,
            compte,
        });
    }

    let total = termes.iter().filter(|t| t.compte).map(|t| t.valeur).sum();
    Detail { total, termes }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca() -> Cible {
        Cible::new("ca")
    }

    #[test]
    fn meme_type_garde_le_plus_eleve() {
        let e = [
            Effet::new("ca", TypeBonus::Armure, 4, "Cotte de mailles"),
            Effet::new("ca", TypeBonus::Armure, 2, "Armure de mage"),
        ];
        let d = calculer(&[("base", 10)], &e, &ca());
        assert_eq!(d.total, 14);
        assert!(!d.termes[2].compte);
    }

    #[test]
    fn le_plus_eleve_gagne_meme_s_il_vient_apres() {
        let e = [
            Effet::new("ca", TypeBonus::Armure, 2, "Armure de mage"),
            Effet::new("ca", TypeBonus::Armure, 4, "Cotte de mailles"),
        ];
        assert_eq!(calculer(&[("base", 10)], &e, &ca()).total, 14);
    }

    #[test]
    fn egalite_ne_compte_qu_une_fois() {
        let e = [
            Effet::new("ca", TypeBonus::Naturel, 2, "A"),
            Effet::new("ca", TypeBonus::Naturel, 2, "B"),
        ];
        assert_eq!(calculer(&[("base", 10)], &e, &ca()).total, 12);
    }

    #[test]
    fn types_differents_se_cumulent() {
        let e = [
            Effet::new("ca", TypeBonus::Armure, 4, "Armure"),
            Effet::new("ca", TypeBonus::Bouclier, 2, "Bouclier"),
            Effet::new("ca", TypeBonus::Taille, 1, "Petite taille"),
        ];
        assert_eq!(calculer(&[("base", 10), ("DEX", 2)], &e, &ca()).total, 19);
    }

    #[test]
    fn esquive_circonstance_sans_type_se_cumulent() {
        for t in [
            TypeBonus::Esquive,
            TypeBonus::Circonstance,
            TypeBonus::SansType,
        ] {
            let e = [Effet::new("ca", t, 1, "A"), Effet::new("ca", t, 1, "B")];
            assert_eq!(calculer(&[("base", 10)], &e, &ca()).total, 12, "{t:?}");
        }
    }

    #[test]
    fn les_malus_se_cumulent_toujours() {
        let e = [
            Effet::new("ca", TypeBonus::Moral, -1, "Secoué"),
            Effet::new("ca", TypeBonus::Moral, -2, "Effrayé"),
            Effet::new("ca", TypeBonus::Moral, 1, "Bénédiction"),
        ];
        assert_eq!(calculer(&[("base", 10)], &e, &ca()).total, 8);
    }

    #[test]
    fn un_malus_ne_masque_pas_un_bonus_du_meme_type() {
        let e = [
            Effet::new("ca", TypeBonus::Armure, 4, "Armure"),
            Effet::new("ca", TypeBonus::Armure, -2, "Rouille"),
        ];
        assert_eq!(calculer(&[("base", 10)], &e, &ca()).total, 12);
    }

    #[test]
    fn ignore_les_effets_d_autres_cibles() {
        let e = [Effet::new("jds.vig", TypeBonus::Chance, 1, "Amulette")];
        assert_eq!(calculer(&[("base", 10)], &e, &ca()).total, 10);
    }

    #[test]
    fn explication_cite_les_sources() {
        let e = [Effet::new("ca", TypeBonus::Armure, 4, "Cotte de mailles")];
        let d = calculer(&[("base", 10), ("DEX", 2)], &e, &ca());
        assert_eq!(
            d.explication(),
            "16 = +10 base +2 DEX +4 Cotte de mailles (armure)"
        );
    }
}
