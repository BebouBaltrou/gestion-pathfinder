//! Compagnon animal du druide (Manuel des joueurs, druide p. 41 à 44) : table 3-6, liste des
//! animaux et calcul de la feuille du compagnon à un niveau de druide donné.
//!
//! Les données sont dans `data/core/compagnons_animaux.json` ; rien n'est codé ici pour un
//! animal en particulier.

use crate::caracteristiques::{modificateur, Carac};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Une ligne de la table 3-6 : ce que le compagnon gagne à chaque niveau de druide.
#[derive(Debug, Clone, Deserialize)]
pub struct LigneCompagnon {
    pub niveau: u32,
    pub dv: u32,
    pub bba: i32,
    pub reflexes: i32,
    pub vigueur: i32,
    pub volonte: i32,
    pub competences: u32,
    pub dons: u32,
    pub armure_naturelle: i32,
    pub for_dex: i32,
    pub tours: u32,
    pub special: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CaracsAnimal {
    #[serde(rename = "for")]
    pub force: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub sag: i32,
    pub cha: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeveloppementAnimal {
    /// Niveau de druide auquel l'animal se développe (4 ou 7 selon l'animal).
    pub niveau: u32,
    pub taille: Option<String>,
    pub armure_naturelle: i32,
    pub attaques: Option<String>,
    /// Modificateurs de caractéristiques (`for`, `dex`, `con`).
    pub caracteristiques: BTreeMap<String, i32>,
    pub autres: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnimalCompagnon {
    pub id: String,
    pub nom: String,
    pub taille: String,
    pub vitesse: String,
    pub armure_naturelle: i32,
    pub attaques: String,
    pub caracteristiques: CaracsAnimal,
    pub autres: String,
    pub developpement: DeveloppementAnimal,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DonneesCompagnons {
    pub table: Vec<LigneCompagnon>,
    /// Texte du livre pour chaque capacité spéciale (nom en minuscules).
    pub textes: BTreeMap<String, String>,
    pub animaux: Vec<AnimalCompagnon>,
}

/// Choix du joueur pour son compagnon animal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compagnon {
    /// Identifiant de l'animal dans la liste du livre.
    pub animal: String,
    pub nom: String,
    /// Le livre permet d'ignorer le développement et d'ajouter à la place +2 en Dextérité et en
    /// Constitution.
    #[serde(default)]
    pub ignore_developpement: bool,
    /// Une caractéristique augmentée de 1 à chacun des niveaux 4, 9, 14 et 20 du druide.
    #[serde(default)]
    pub augmentations: Vec<Carac>,
}

/// Feuille calculée du compagnon à un niveau de druide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompagnonCalcule {
    pub animal: String,
    pub nom: String,
    pub taille: String,
    pub vitesse: String,
    pub dv: u32,
    /// Points de vie moyens : 4,5 par d8 (arrondi à l'inférieur) plus Constitution à chaque dé.
    pub pv: i32,
    pub bba: i32,
    /// Caractéristique, score, modificateur.
    pub caracteristiques: Vec<(Carac, i32, i32)>,
    pub armure_naturelle: i32,
    pub ca: i32,
    pub ca_contact: i32,
    pub ca_depourvu: i32,
    pub reflexes: i32,
    pub vigueur: i32,
    pub volonte: i32,
    pub attaques: String,
    /// Bonus à l'attaque : BBA + modificateur de Force + modificateur de taille.
    pub bonus_attaque: i32,
    pub competences: u32,
    pub dons: u32,
    pub tours: u32,
    pub capacites: Vec<String>,
    pub particularites: String,
    pub developpement_applique: bool,
    /// Augmentations de caractéristique déjà gagnées (niveaux 4, 9, 14, 20 de druide).
    pub augmentations_permises: u32,
}

/// Modificateur de taille à la CA et à l'attaque (P : +1, M : 0, G : −1…).
pub fn modificateur_de_taille(taille: &str) -> i32 {
    match taille {
        "Inf" | "I" => 8,
        "Min" => 4,
        "TP" => 2,
        "P" => 1,
        "G" => -1,
        "TG" => -2,
        "Gig" => -4,
        "C" => -8,
        _ => 0,
    }
}

/// Nombre d'augmentations de caractéristique gagnées par le compagnon au niveau `nc` de druide.
pub fn augmentations_permises(donnees: &DonneesCompagnons, nc: u32) -> u32 {
    donnees
        .table
        .iter()
        .filter(|l| l.niveau <= nc && l.special.contains("accroissement d'une caractéristique"))
        .count() as u32
}

impl Compagnon {
    pub fn calculer(&self, donnees: &DonneesCompagnons, nc: u32) -> Option<CompagnonCalcule> {
        let animal = donnees.animaux.iter().find(|a| a.id == self.animal)?;
        let ligne = donnees.table.get((nc as usize).checked_sub(1)?)?;
        let c = &animal.caracteristiques;
        let mut score: BTreeMap<Carac, i32> = [
            (Carac::For, c.force),
            (Carac::Dex, c.dex),
            (Carac::Con, c.con),
            (Carac::Int, c.int),
            (Carac::Sag, c.sag),
            (Carac::Cha, c.cha),
        ]
        .into_iter()
        .collect();
        let mut taille = animal.taille.clone();
        let mut na = animal.armure_naturelle + ligne.armure_naturelle;
        let mut attaques = animal.attaques.clone();
        let mut autres = animal.autres.clone();
        let d = &animal.developpement;
        let developpe = nc >= d.niveau;
        if developpe && self.ignore_developpement {
            *score.entry(Carac::Dex).or_default() += 2;
            *score.entry(Carac::Con).or_default() += 2;
        } else if developpe {
            for (k, v) in &d.caracteristiques {
                let carac = match k.as_str() {
                    "for" => Carac::For,
                    "dex" => Carac::Dex,
                    _ => Carac::Con,
                };
                *score.entry(carac).or_default() += v;
            }
            if let Some(t) = &d.taille {
                taille = t.clone();
            }
            na += d.armure_naturelle;
            if let Some(a) = &d.attaques {
                attaques = a.clone();
            }
            if !d.autres.is_empty() {
                autres = format!("{autres} ; {}", d.autres);
            }
        }
        // Bonus de Force et de Dextérité de la table.
        *score.entry(Carac::For).or_default() += ligne.for_dex;
        *score.entry(Carac::Dex).or_default() += ligne.for_dex;
        // Augmentations choisies (au plus autant que de niveaux 4, 9, 14, 20 atteints).
        let permises = augmentations_permises(donnees, nc);
        for carac in self.augmentations.iter().take(permises as usize) {
            *score.entry(*carac).or_default() += 1;
        }
        let m = |c: Carac| modificateur(score[&c]);
        let mod_taille = modificateur_de_taille(&taille);
        let dex = m(Carac::Dex);
        let ca = 10 + dex + mod_taille + na;
        Some(CompagnonCalcule {
            animal: animal.nom.clone(),
            nom: self.nom.clone(),
            taille,
            vitesse: animal.vitesse.clone(),
            dv: ligne.dv,
            pv: (ligne.dv as i32 * 9) / 2 + ligne.dv as i32 * m(Carac::Con),
            bba: ligne.bba,
            caracteristiques: Carac::TOUTES
                .into_iter()
                .map(|c| (c, score[&c], m(c)))
                .collect(),
            armure_naturelle: na,
            ca,
            ca_contact: 10 + dex + mod_taille,
            ca_depourvu: ca - dex.max(0),
            reflexes: ligne.reflexes + dex,
            vigueur: ligne.vigueur + m(Carac::Con),
            volonte: ligne.volonte + m(Carac::Sag),
            attaques,
            bonus_attaque: ligne.bba + m(Carac::For) + mod_taille,
            competences: ligne.competences,
            dons: ligne.dons,
            tours: ligne.tours,
            capacites: donnees
                .table
                .iter()
                .filter(|l| l.niveau <= nc && !l.special.is_empty())
                .map(|l| l.special.clone())
                .collect(),
            particularites: autres,
            developpement_applique: developpe,
            augmentations_permises: permises,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Referentiel;

    fn chien(nom: &str) -> Compagnon {
        Compagnon {
            animal: "chien".into(),
            nom: nom.into(),
            ignore_developpement: false,
            augmentations: vec![],
        }
    }

    #[test]
    fn la_table_3_6_et_les_animaux_viennent_du_livre() {
        let r = Referentiel::partage();
        let d = &r.compagnons;
        assert_eq!(d.table.len(), 20);
        assert_eq!(d.animaux.len(), 17);
        // Niveau 1 : 2 DV, BBA +1, jets 3/3/0, 2 rangs, 1 don, 1 tour ; niveau 20 : 16 DV, BBA +12.
        let n1 = &d.table[0];
        assert_eq!(
            (n1.dv, n1.bba, n1.reflexes, n1.vigueur, n1.volonte),
            (2, 1, 3, 3, 0)
        );
        assert_eq!((n1.competences, n1.dons, n1.tours), (2, 1, 1));
        let n20 = &d.table[19];
        assert_eq!(
            (n20.dv, n20.bba, n20.armure_naturelle, n20.for_dex),
            (16, 12, 12, 6)
        );
        assert!(d.textes.contains_key("lien"));
    }

    #[test]
    fn un_chien_au_niveau_1_puis_au_niveau_4() {
        let r = Referentiel::partage();
        let d = &r.compagnons;
        // Chien : taille P, CA +2 d'armure naturelle, For 13, Dex 17, Con 15, Int 2, Sag 12, Cha 6.
        let c1 = chien("Rex").calculer(d, 1).unwrap();
        assert_eq!((c1.dv, c1.bba), (2, 1));
        assert_eq!(c1.taille, "P");
        // CA = 10 + Dex +3 + taille +1 + armure naturelle 2 = 16.
        assert_eq!(c1.ca, 16);
        assert_eq!(c1.ca_contact, 14);
        // Jets : Réflexes 3 + Dex 3, Vigueur 3 + Con 2, Volonté 0 + Sag 1.
        assert_eq!((c1.reflexes, c1.vigueur, c1.volonte), (6, 5, 1));
        // PV : 2 dés, 4,5 de moyenne arrondi (9) + Con +2 par dé = 13.
        assert_eq!(c1.pv, 13);
        // Niveau 4 : développement (taille M, For +4, Dex −2, Con +2), 4 DV, bonus For/Dex +1.
        let c4 = chien("Rex").calculer(d, 4).unwrap();
        assert_eq!(c4.taille, "M");
        assert!(c4.developpement_applique);
        let sc = |c: &CompagnonCalcule, k: Carac| {
            c.caracteristiques.iter().find(|x| x.0 == k).unwrap().1
        };
        // For 13 + 4 + 1 (table) = 18 ; Dex 17 − 2 + 1 = 16 ; Con 15 + 2 = 17.
        assert_eq!(
            (
                sc(&c4, Carac::For),
                sc(&c4, Carac::Dex),
                sc(&c4, Carac::Con)
            ),
            (18, 16, 17)
        );
        assert_eq!(c4.dv, 4);
        assert_eq!(c4.armure_naturelle, 2 + 2);
        // Ignorer le développement : +2 Dex et +2 Con à la place, la taille reste P.
        let mut agile = chien("Rex");
        agile.ignore_developpement = true;
        let a4 = agile.calculer(d, 4).unwrap();
        assert_eq!(a4.taille, "P");
        assert_eq!(sc(&a4, Carac::Dex), 17 + 2 + 1);
        assert_eq!(sc(&a4, Carac::For), 13 + 1);
    }

    #[test]
    fn les_augmentations_de_caracteristique_suivent_les_niveaux() {
        let r = Referentiel::partage();
        let d = &r.compagnons;
        assert_eq!(augmentations_permises(d, 3), 0);
        assert_eq!(augmentations_permises(d, 4), 1);
        assert_eq!(augmentations_permises(d, 9), 2);
        assert_eq!(augmentations_permises(d, 20), 4);
        let mut c = chien("Rex");
        c.augmentations = vec![Carac::Sag, Carac::Sag];
        // Au niveau 4, une seule augmentation compte.
        let v = c.calculer(d, 4).unwrap();
        assert_eq!(
            v.caracteristiques
                .iter()
                .find(|x| x.0 == Carac::Sag)
                .unwrap()
                .1,
            13
        );
        let v = c.calculer(d, 9).unwrap();
        assert_eq!(
            v.caracteristiques
                .iter()
                .find(|x| x.0 == Carac::Sag)
                .unwrap()
                .1,
            14
        );
        // Animal inconnu.
        let mut x = chien("Rex");
        x.animal = "dragon".into();
        assert!(x.calculer(d, 4).is_none());
    }
}
