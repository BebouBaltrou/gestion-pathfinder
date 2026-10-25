//! Fiche d'exemple valide (elfe ensorceleuse de niveau 1, lignage draconique), pour les essais
//! et les démonstrations.

use crate::caracteristiques::Carac;
use crate::fiche::{BonusPredilection, Fiche, NiveauPris};
use std::collections::BTreeMap;

/// Achat de 15 points : FOR 8, DEX 14, CON 12, INT 10, SAG 10, CHA 16 ; elfe : DEX+2 INT+2 CON-2.
pub fn elfe_ensorceleuse() -> Fiche {
    // Achat de 15 points : FOR 8, DEX 14, CON 12, INT 10, SAG 10, CHA 16 ; elfe : DEX+2 INT+2 CON-2.
    let scores = [
        (Carac::For, 8),
        (Carac::Dex, 14),
        (Carac::Con, 12),
        (Carac::Int, 10),
        (Carac::Sag, 10),
        (Carac::Cha, 16),
    ];
    let mut rangs = BTreeMap::new();
    rangs.insert("spellcraft".to_string(), 1);
    rangs.insert("bluff".to_string(), 1);
    rangs.insert("knowledge:arcana".to_string(), 1);
    Fiche {
        nom: "Lyra".into(),
        race: "elfe".into(),
        bonus_racial_libre: None,
        scores_base: scores.into_iter().collect(),
        dommages: BTreeMap::new(),
        drain: BTreeMap::new(),
        augmentations: vec![],
        niveaux: vec![NiveauPris {
            classe: "ensorceleur".into(),
            de_vie_obtenu: 6,
            rangs,
            predilection: Some(BonusPredilection::PointDeVie),
            dons: vec!["Toughness".into()],
        }],
        classes_de_predilection: vec!["ensorceleur".into()],
        lignage: Some("draconic".into()),
        connaissance_de_lignage: None,
        effets: vec![],
        equipement: Default::default(),
        etats: vec![],
        temporaires: vec![],
        divinite: None,
        alignement: None,
        domaines: vec![],
        energie: None,
        pacte: None,
        sorts_connus: [
            "Acid Splash",
            "Detect Magic",
            "Light",
            "Mage Hand",
            "Magic Missile",
            "Mage Armor",
        ]
        .map(String::from)
        .to_vec(),
    }
}

/// Fiche d'exemple d'un prêtre humain de niveau 3 (Iomédae, Loyal Bon, domaines du Bien et du Soleil),
/// pour les essais et les démonstrations.
pub fn pretre_d_iomedae() -> Fiche {
    let scores = [
        (Carac::For, 12),
        (Carac::Dex, 10),
        (Carac::Con, 14),
        (Carac::Int, 10),
        (Carac::Sag, 16),
        (Carac::Cha, 12),
    ];
    let niveau = |i: u32| NiveauPris {
        classe: "pretre".into(),
        de_vie_obtenu: if i == 0 { 8 } else { 5 },
        rangs: BTreeMap::new(),
        predilection: Some(BonusPredilection::PointDeVie),
        dons: if i == 0 {
            vec!["Toughness".into(), "Alertness".into()]
        } else if i == 2 {
            vec!["Combat Casting".into()]
        } else {
            vec![]
        },
    };
    Fiche {
        nom: "Aleth".into(),
        race: "humain".into(),
        bonus_racial_libre: Some(Carac::Sag),
        scores_base: scores.into_iter().collect(),
        dommages: BTreeMap::new(),
        drain: BTreeMap::new(),
        augmentations: vec![],
        niveaux: (0..3).map(niveau).collect(),
        classes_de_predilection: vec!["pretre".into()],
        lignage: None,
        connaissance_de_lignage: None,
        effets: vec![],
        sorts_connus: vec![],
        equipement: Default::default(),
        etats: vec![],
        temporaires: vec![],
        divinite: Some("iomedae".into()),
        alignement: Some("LB".into()),
        domaines: vec!["bien".into(), "soleil".into()],
        energie: None,
        pacte: None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn json_de_la_fiche_exemple() {
        let j = serde_json::to_string(&super::elfe_ensorceleuse()).unwrap();
        println!("JSON>>{j}");
        let ev = crate::journal::EvenementFiche::Equiper {
            emplacement: crate::journal::EmplacementProtection::Armure,
            objet: Some("Chainmail".into()),
        };
        println!("EV>>{}", serde_json::to_string(&ev).unwrap());
        println!(
            "EV2>>{}",
            serde_json::to_string(&crate::journal::EvenementFiche::Repos).unwrap()
        );
        let back: crate::fiche::Fiche = serde_json::from_str(&j).unwrap();
        assert_eq!(back, super::elfe_ensorceleuse());
    }
}
