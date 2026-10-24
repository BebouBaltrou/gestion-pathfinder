//! Essais des classes autres que l'ensorceleur (tables du Manuel des joueurs).

use crate::donnees::Referentiel;
use crate::fiche::{BonusPredilection, Fiche, NiveauPris};
use crate::Carac;
use std::collections::BTreeMap;

fn pretre(niveau: u32, sagesse: i32) -> Fiche {
    let scores = [
        (Carac::For, 12),
        (Carac::Dex, 10),
        (Carac::Con, 14),
        (Carac::Int, 10),
        (Carac::Sag, sagesse),
        (Carac::Cha, 12),
    ];
    Fiche {
        nom: "Test".into(),
        race: "humain".into(),
        bonus_racial_libre: Some(Carac::Sag),
        scores_base: scores.into_iter().collect(),
        dommages: BTreeMap::new(),
        drain: BTreeMap::new(),
        augmentations: vec![],
        niveaux: (0..niveau)
            .map(|i| NiveauPris {
                classe: "pretre".into(),
                de_vie_obtenu: if i == 0 { 8 } else { 5 },
                rangs: BTreeMap::new(),
                predilection: Some(BonusPredilection::PointDeVie),
                dons: vec![],
            })
            .collect(),
        classes_de_predilection: vec!["pretre".into()],
        lignage: None,
        connaissance_de_lignage: None,
        effets: vec![],
        sorts_connus: vec![],
        equipement: Default::default(),
        etats: vec![],
        temporaires: vec![],
    }
}

#[test]
fn la_classe_pretre_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("pretre").expect("prêtre");
    assert_eq!(c.de_vie, 8);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.as_ref().unwrap().prepare());
    assert_eq!(c.source.page, Some(67));
}

#[test]
fn table_3_13_du_pretre() {
    let r = Referentiel::core();
    let c = r.classe("pretre").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +0, jets +2/+0/+2, 3 oraisons, un sort de niveau 1 (+1 de domaine).
    assert_eq!(
        (l(1).bba.clone(), l(1).vigueur, l(1).reflexes, l(1).volonte),
        (vec![0], 2, 0, 2)
    );
    assert_eq!(l(1).oraisons, Some(3));
    assert_eq!(l(1).sorts_par_jour[0], Some(1));
    assert_eq!(l(1).sorts_par_jour[1], None);
    // Niveau 11 : BBA +8/+3, jets +7/+3/+7, 4+1 sorts de niveau 1 à 5, 1 sort de niveau 6.
    assert_eq!(l(11).bba, vec![8, 3]);
    assert_eq!((l(11).vigueur, l(11).reflexes, l(11).volonte), (7, 3, 7));
    assert_eq!(
        l(11).sorts_par_jour,
        [4, 4, 4, 3, 2, 1]
            .map(Some)
            .into_iter()
            .chain([None; 3])
            .collect::<Vec<_>>()
    );
    // Niveau 20 : BBA +15/+10/+5, jets +12/+6/+12, 4 sorts de chaque niveau.
    assert_eq!(l(20).bba, vec![15, 10, 5]);
    assert_eq!((l(20).vigueur, l(20).reflexes, l(20).volonte), (12, 6, 12));
    assert!(l(20).sorts_par_jour.iter().all(|s| *s == Some(4)));
}

#[test]
fn un_pretre_prepare_ses_sorts_et_gagne_un_emplacement_de_domaine() {
    let r = Referentiel::core();
    // Niveau 5, Sagesse 16 + 2 (humain) = 18, modificateur +4.
    let f = pretre(5, 16);
    assert!(
        f.valider(&r).iter().all(|p| p.est_incomplet()),
        "{:?}",
        f.valider(&r)
    );
    let v = f.calculer(&r).unwrap();
    assert_eq!(v.bba.total, 3);
    // Vigueur : 4 (table) + 2 (CON 14).
    assert_eq!(v.vigueur.total, 6);
    let inc = &v.incantateurs[0];
    assert_eq!(inc.classe, "pretre");
    let niv = |n: u32| {
        inc.emplacements
            .iter()
            .find(|e| e.niveau_de_sort == n)
            .unwrap()
    };
    // Oraisons : 4 par jour ; niveau 1 : 3 + 1 (Sagesse +4) ; niveau 3 : 1 + 1 ; un emplacement de domaine.
    assert_eq!(niv(0).base, Some(4));
    assert_eq!(niv(1).par_jour, Some(3 + 1));
    assert_eq!(niv(3).par_jour, Some(1 + 1));
    assert_eq!(niv(1).domaine, 1);
    assert_eq!(niv(0).domaine, 0);
    assert!(inc.emplacements.iter().all(|e| e.niveau_de_sort <= 3));
    // DD = 10 + niveau du sort + modificateur de Sagesse.
    assert_eq!(niv(2).dd, 10 + 2 + 4);
}

#[test]
fn le_pretre_porte_armure_intermediaire_et_bouclier_sans_malus_d_incompetence() {
    let r = Referentiel::core();
    let mut f = pretre(1, 14);
    f.equipement.armure = Some("Chainmail".into());
    f.equipement.bouclier = Some("Heavy steel".into());
    let m = f.maitrises(&r);
    assert!(f.problemes_equipement(&r).is_empty());
    assert!(m.boucliers && !m.pavois);
    let v = f.calculer(&r).unwrap();
    assert!(v.equipement.non_maitrises.is_empty());
}

#[test]
fn les_competences_de_classe_du_pretre() {
    let r = Referentiel::core();
    let mut f = pretre(1, 14);
    f.niveaux[0].rangs.insert("diplomacy".into(), 1);
    f.niveaux[0].rangs.insert("bluff".into(), 1);
    let v = f.calculer(&r).unwrap();
    assert!(v.competences["diplomacy"]
        .termes
        .iter()
        .any(|t| t.libelle == "compétence de classe"));
    assert!(!v.competences["bluff"]
        .termes
        .iter()
        .any(|t| t.libelle == "compétence de classe"));
}

fn druide(niveau: u32) -> Fiche {
    let mut f = pretre(niveau, 16);
    for (i, n) in f.niveaux.iter_mut().enumerate() {
        n.classe = "druide".into();
        n.de_vie_obtenu = if i == 0 { 8 } else { 5 };
    }
    f.classes_de_predilection = vec!["druide".into()];
    f
}

#[test]
fn table_3_5_du_druide() {
    let r = Referentiel::core();
    let c = r.classe("druide").expect("druide");
    assert_eq!(
        (c.de_vie, c.rangs_par_niveau, c.source.page),
        (8, 4, Some(38))
    );
    let l = |n: usize| &c.progression[n - 1];
    assert_eq!(
        (l(1).bba.clone(), l(1).vigueur, l(1).reflexes, l(1).volonte),
        (vec![0], 2, 0, 2)
    );
    assert_eq!(l(1).oraisons, Some(3));
    assert_eq!(l(1).sorts_par_jour[0], Some(1));
    assert!(!l(1).domaine);
    assert_eq!(l(8).bba, vec![6, 1]);
    assert_eq!((l(8).vigueur, l(8).reflexes, l(8).volonte), (6, 2, 6));
    assert_eq!(l(20).bba, vec![15, 10, 5]);
    assert!(l(20).sorts_par_jour.iter().all(|s| *s == Some(4)));
    // Les sorts par jour du druide et du prêtre sont identiques (hors emplacement de domaine).
    let p = r.classe("pretre").unwrap();
    for n in 1..=20 {
        assert_eq!(
            c.progression[n - 1].sorts_par_jour,
            p.progression[n - 1].sorts_par_jour
        );
    }
}

#[test]
fn instinct_naturel_et_rangs_du_druide() {
    let r = Referentiel::core();
    let mut f = druide(4);
    f.niveaux[0].rangs.insert("survival".into(), 1);
    f.niveaux[0].rangs.insert("perception".into(), 1);
    let v = f.calculer(&r).unwrap();
    // Survie : 1 rang + SAG +4 (16 + 2 humain) + 3 (compétence de classe) + 2 (instinct naturel).
    assert_eq!(v.competences["survival"].total, 1 + 4 + 3 + 2);
    // Perception : pas d'instinct naturel.
    assert_eq!(v.competences["perception"].total, 1 + 4 + 3);
    // 4 + modificateur d'Intelligence (10 : 0) + 1 rang d'humain par niveau.
    assert_eq!(v.rangs[0].disponibles, 4 + 1);
}

#[test]
fn le_druide_ne_connait_que_ses_armes() {
    let r = Referentiel::core();
    let mut f = druide(1);
    f.equipement.armes = vec!["Scimitar".into(), "Longsword".into()];
    let v = f.calculer(&r).unwrap();
    let maitrise = |nom: &str| v.armes.iter().find(|a| a.arme_en == nom).unwrap().maitrise;
    assert!(maitrise("Scimitar"));
    assert!(!maitrise("Longsword"));
}
