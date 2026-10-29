//! Essais des classes autres que l'ensorceleur (tables du Manuel des joueurs).

use crate::donnees::Referentiel;
use crate::fiche::{BonusPredilection, Fiche, NiveauPris, Probleme};
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
        style_de_combat: None,
        effets: vec![],
        sorts_connus: vec![],
        equipement: Default::default(),
        etats: vec![],
        temporaires: vec![],
        divinite: None,
        alignement: None,
        domaines: vec![],
        energie: None,
        pacte: None,
        compagnon: None,
        graces: vec![],
        talents_roublard: vec![],
        ennemis_jures: vec![],
        augmentations_ennemis: vec![],
        environnements: vec![],
        augmentations_environnements: vec![],
        pacte_du_chasseur: None,
        pouvoirs_rage: vec![],
        en_rage: false,
        rounds_de_rage: 0,
        fatigue_rage: None,
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

fn pretre_de(divinite: &str, alignement: &str, domaines: &[&str], niveau: u32) -> Fiche {
    let mut f = pretre(niveau, 16);
    f.divinite = Some(divinite.into());
    f.alignement = Some(alignement.into());
    f.domaines = domaines.iter().map(|d| d.to_string()).collect();
    f
}

#[test]
fn divinite_alignement_et_domaines_du_pretre() {
    let r = Referentiel::core();
    // Iomédae (LB) : Bien et Soleil sont ses domaines, alignement LB.
    let f = pretre_de("iomedae", "LB", &["bien", "soleil"], 1);
    assert!(
        f.valider(&r).iter().all(|p| p.est_incomplet()),
        "{:?}",
        f.valider(&r)
    );
    // Un domaine que le dieu n'offre pas.
    let g = pretre_de("iomedae", "LB", &["bien", "feu"], 1);
    assert!(g
        .valider(&r)
        .iter()
        .any(|p| matches!(p, crate::Probleme::DomaineHorsDivinite { .. })));
    // Un domaine d'alignement qui ne correspond pas (prêtre NN de Gozreh ne peut prendre Bien).
    let h = pretre_de("gozreh", "N", &["air", "bien"], 1);
    let pb = h.valider(&r);
    assert!(
        pb.iter()
            .any(|p| matches!(p, crate::Probleme::DomaineAlignement { .. })),
        "{pb:?}"
    );
    assert!(pb
        .iter()
        .any(|p| matches!(p, crate::Probleme::DomaineHorsDivinite { .. })));
    // Alignement trop éloigné du dieu : Iomédae (LB), prêtre CM.
    let i = pretre_de("iomedae", "CM", &["bien", "soleil"], 1);
    assert!(i
        .valider(&r)
        .iter()
        .any(|p| matches!(p, crate::Probleme::AlignementHorsDivinite { .. })));
    // Pas assez de domaines : fiche inachevée, pas fausse.
    let j = pretre_de("iomedae", "LB", &["bien"], 1);
    let pj = j.valider(&r);
    assert!(pj
        .iter()
        .any(|p| matches!(p, crate::Probleme::DomainesAChoisir { .. })));
    assert!(pj.iter().all(|p| p.est_incomplet()));
    // Prêtre Neutre sur l'axe Bien-Mal : il doit choisir l'énergie.
    let k = pretre_de("gozreh", "N", &["air", "eau"], 1);
    assert!(k.valider(&r).contains(&crate::Probleme::EnergieAChoisir));
}

#[test]
fn canalisation_et_domaines_calcules() {
    let r = Referentiel::core();
    // Niveau 5, Charisme 12 (+1), Sagesse 18 (+4) : bon prêtre.
    let mut f = pretre_de("iomedae", "LB", &["bien", "soleil"], 5);
    let v = f.calculer(&r).unwrap();
    let c = v.canalisation.as_ref().expect("canalisation");
    // 1d6 + 1d6 par deux niveaux après le premier : 3d6 au niveau 5 ; DD 10 + 5/2 + CHA +1 ; 3 + CHA fois par jour.
    assert_eq!(
        (c.positive, c.des, c.dd, c.utilisations),
        (Some(true), 3, 10 + 2 + 1, 4)
    );
    assert_eq!(v.domaines.len(), 2);
    let bien = &v.domaines[0];
    assert_eq!(bien.nom, "Bien");
    // Toucher du Bien : 3 + modificateur de Sagesse (+4) ; Lance du Bien : niveau 8 (inactif au niveau 5).
    assert!(bien.pouvoirs[0].actif);
    assert_eq!(bien.pouvoirs[0].utilisations, Some(7));
    assert!(!bien.pouvoirs[1].actif);
    // Sorts de domaine : niveau 3 accessible au niveau 5 (sorts de niveau 1 à 3), le 4e non.
    assert!(
        bien.sorts
            .iter()
            .find(|s| s.niveau == 3)
            .unwrap()
            .accessible
    );
    assert!(
        !bien
            .sorts
            .iter()
            .find(|s| s.niveau == 4)
            .unwrap()
            .accessible
    );
    // Mauvais prêtre : énergie négative. Neutre : selon son choix.
    let mal = pretre_de("asmodeus", "LM", &["feu", "mal"], 1);
    assert_eq!(
        mal.calculer(&r).unwrap().canalisation.unwrap().positive,
        Some(false)
    );
    let mut n = pretre_de("gozreh", "N", &["air", "eau"], 1);
    assert_eq!(n.calculer(&r).unwrap().canalisation.unwrap().positive, None);
    n.energie = Some("positive".into());
    assert_eq!(
        n.calculer(&r).unwrap().canalisation.unwrap().positive,
        Some(true)
    );
    // L'arme de prédilection du dieu est maniée (Iomédae : épée longue), pas les autres armes de guerre.
    f.equipement.armes = vec!["Longsword".into(), "Greatsword".into()];
    let w = f.calculer(&r).unwrap();
    let maitrise = |nom: &str| w.armes.iter().find(|a| a.arme_en == nom).unwrap().maitrise;
    assert!(maitrise("Longsword"));
    assert!(!maitrise("Greatsword"));
}

// ── Préparation des sorts, lancer, conversion spontanée ────────────────────────

use crate::journal::{EvenementFiche, Journal};
use crate::preparation::SortPrepare;
use crate::progression::ReglesFiche;

/// Les `n` premiers sorts de prêtre du niveau `niveau`, sans sort opposé à l'alignement Loyal Bon.
fn sorts_de_pretre(r: &Referentiel, niveau: u32, n: usize) -> Vec<String> {
    let mut v: Vec<&str> = r
        .sorts
        .iter()
        .filter(|s| s.niveau_pour("cleric") == Some(niveau))
        .filter(|s| {
            !s.descripteurs
                .iter()
                .any(|d| matches!(d.to_lowercase().as_str(), "evil" | "chaotic"))
        })
        .map(|s| s.nom_en.as_str())
        .collect();
    v.sort_unstable();
    v.into_iter().take(n).map(String::from).collect()
}

fn prep(niveau: u32, sort: &str, domaine: bool) -> SortPrepare {
    SortPrepare {
        niveau,
        sort: sort.into(),
        domaine,
    }
}

#[test]
fn preparer_lancer_et_se_reposer() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    // Niveau 3, Sagesse 18 (+4) : oraisons 4 ; niveau 1 : 2 + 1 = 3 (+1 domaine) ; niveau 2 : 1 + 1 = 2 (+1 domaine).
    let mut j = Journal::nouveau(pretre_de("iomedae", "LB", &["bien", "soleil"], 3));
    let ajouter = |j: &mut Journal, e| j.ajouter(e, &r, &regles);
    let n1 = sorts_de_pretre(&r, 1, 4);
    let n2 = sorts_de_pretre(&r, 2, 3);
    let oraisons = sorts_de_pretre(&r, 0, 4);
    let mut sorts: Vec<SortPrepare> = oraisons.iter().map(|s| prep(0, s, false)).collect();
    sorts.extend(n1[..3].iter().map(|s| prep(1, s, false)));
    sorts.extend(n2[..2].iter().map(|s| prep(2, s, false)));
    // Emplacements de domaine : un sort de niveau 1 du domaine du Bien, un de niveau 2 du Soleil.
    sorts.push(prep(1, "Protection from Evil", true));
    sorts.push(prep(2, "Heat Metal", true));
    let prepare = |sorts: Vec<SortPrepare>| EvenementFiche::Preparation {
        classe: "pretre".into(),
        sorts,
    };

    // Trop de sorts au niveau 1 (4 normaux pour 3 permis).
    let mut trop = sorts.clone();
    trop.push(prep(1, &n1[3], false));
    assert!(ajouter(&mut j, prepare(trop)).is_err());
    // Un sort qui n'est pas dans la liste du prêtre, ou à un autre niveau.
    let mut faux = sorts.clone();
    faux[5] = prep(1, "Magic Missile", false);
    assert!(ajouter(&mut j, prepare(faux)).is_err());
    // Un sort de domaine qui n'appartient pas aux domaines choisis.
    let mut hors = sorts.clone();
    *hors.last_mut().unwrap() = prep(2, "Fire Trap", true);
    assert!(ajouter(&mut j, prepare(hors)).is_err());
    // Un sort Mauvais pour un prêtre Loyal Bon.
    let mauvais = r
        .sorts
        .iter()
        .find(|s| {
            s.niveau_pour("cleric") == Some(1)
                && s.descripteurs
                    .iter()
                    .any(|d| d.eq_ignore_ascii_case("evil"))
        })
        .unwrap();
    let mut opp = sorts.clone();
    opp[5] = prep(1, &mauvais.nom_en, false);
    assert!(ajouter(&mut j, prepare(opp)).is_err());

    // La bonne préparation passe.
    ajouter(&mut j, prepare(sorts.clone())).expect("préparation valide");
    let lancer = |niveau: u32, sort: &str| EvenementFiche::SortLance {
        classe: "pretre".into(),
        niveau_de_sort: niveau,
        sort: sort.into(),
    };
    // Un sort préparé se lance une fois ; un sort non préparé non ; le sort de domaine non plus deux fois.
    ajouter(&mut j, lancer(1, &n1[0])).expect("sort préparé");
    assert!(ajouter(&mut j, lancer(1, &n1[0])).is_err(), "déjà lancé");
    assert!(ajouter(&mut j, lancer(1, &n1[3])).is_err(), "non préparé");
    ajouter(&mut j, lancer(1, "Protection from Evil")).expect("sort de domaine");
    assert!(ajouter(&mut j, lancer(1, "Protection from Evil")).is_err());
    let e = j.etat(&r, &regles).unwrap();
    let restants = e.emplacements_restants(&r);
    let reste = |niv: u32| restants.iter().find(|x| x.1 == niv).unwrap();
    // Niveau 1 : 3 + 1 domaine = 4 emplacements, 2 lancés.
    assert_eq!((reste(1).2, reste(1).3), (2, 4));
    // Le repos rend tout, la préparation reste.
    ajouter(&mut j, EvenementFiche::Repos).unwrap();
    ajouter(&mut j, lancer(1, &n1[0])).expect("de nouveau lançable après le repos");
}

#[test]
fn conversion_spontanee_en_soins() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut j = Journal::nouveau(pretre_de("iomedae", "LB", &["bien", "soleil"], 5));
    let ajouter = |j: &mut Journal, e| j.ajouter(e, &r, &regles);
    let mut sorts: Vec<SortPrepare> = sorts_de_pretre(&r, 2, 1)
        .iter()
        .map(|s| prep(2, s, false))
        .collect();
    sorts.push(prep(1, "Protection from Evil", true));
    ajouter(
        &mut j,
        EvenementFiche::Preparation {
            classe: "pretre".into(),
            sorts,
        },
    )
    .unwrap();
    let convertir = |niveau: u32, sort: &str| EvenementFiche::SortConverti {
        classe: "pretre".into(),
        niveau_sacrifie: niveau,
        sort: sort.into(),
    };
    // Soins sur un emplacement de niveau 2 : modérés (niveau 2) ou légers (niveau 1) ; pas graves (niveau 3).
    assert!(ajouter(&mut j, convertir(2, "Cure Serious Wounds")).is_err());
    // Un prêtre Bon ne convertit pas en blessures.
    assert!(ajouter(&mut j, convertir(2, "Inflict Moderate Wounds")).is_err());
    // On ne sacrifie pas un sort de domaine (ni une oraison) : au niveau 1 il n'y a que le sort de domaine.
    assert!(ajouter(&mut j, convertir(1, "Cure Light Wounds")).is_err());
    ajouter(&mut j, convertir(2, "Cure Light Wounds")).expect("conversion permise");
    // Plus de sort préparé à sacrifier au niveau 2.
    assert!(ajouter(&mut j, convertir(2, "Cure Moderate Wounds")).is_err());
}

#[test]
fn le_mauvais_pretre_convertit_en_blessures() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut j = Journal::nouveau(pretre_de("asmodeus", "LM", &["feu", "mal"], 5));
    let ajouter = |j: &mut Journal, e| j.ajouter(e, &r, &regles);
    let sort2 = r
        .sorts
        .iter()
        .filter(|s| s.niveau_pour("cleric") == Some(2))
        .filter(|s| {
            !s.descripteurs
                .iter()
                .any(|d| matches!(d.to_lowercase().as_str(), "good" | "chaotic"))
        })
        .map(|s| s.nom_en.clone())
        .min()
        .unwrap();
    ajouter(
        &mut j,
        EvenementFiche::Preparation {
            classe: "pretre".into(),
            sorts: vec![prep(2, &sort2, false)],
        },
    )
    .unwrap();
    let convertir = |sort: &str| EvenementFiche::SortConverti {
        classe: "pretre".into(),
        niveau_sacrifie: 2,
        sort: sort.into(),
    };
    assert!(ajouter(&mut j, convertir("Cure Light Wounds")).is_err());
    ajouter(&mut j, convertir("Inflict Light Wounds")).expect("blessures");
}

#[test]
fn un_pretre_monte_du_niveau_1_au_niveau_11() {
    use crate::progression::{appliquer, montee_auto};
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut f = pretre_de("iomedae", "LB", &["bien", "soleil"], 1);
    f.niveaux[0].dons = vec!["Toughness".into(), "Alertness".into()];
    for _ in 2..=11 {
        let m = montee_auto(&r, &f, "pretre", 5);
        f = appliquer(&r, &regles, &f, &m, 20).expect("montée valide");
        // Les dons manquants viennent de montee_auto ; rien d'autre ne doit manquer.
        assert!(
            f.valider(&r)
                .iter()
                .all(|p| p.est_incomplet() || matches!(p, crate::Probleme::DonsAuNiveau { .. })),
            "{:?}",
            f.valider(&r)
        );
    }
    assert_eq!(f.niveau_de_personnage(), 11);
    let v = f.calculer(&r).unwrap();
    assert_eq!(v.bba.total, 8);
    // Niveau 11 : jets 7/3/7 (+ CON +2, DEX 0, SAG +4).
    assert_eq!(v.vigueur.total, 7 + 2);
    assert_eq!(v.reflexes.total, 3);
    assert_eq!(v.volonte.total, 7 + 4);
    // Canalisation 6d6 (niveau 11) ; Lance du Bien disponible au niveau 8, 2 fois par jour au niveau 12 seulement.
    assert_eq!(v.canalisation.as_ref().unwrap().des, 6);
    let bien = &v.domaines[0];
    assert!(bien.pouvoirs[1].actif);
    assert_eq!(bien.pouvoirs[1].utilisations, Some(1));
    // Sorts de niveau 6 accessibles (1 emplacement + 1 de domaine) ; pas de niveau 7.
    let inc = &v.incantateurs[0];
    assert!(inc
        .emplacements
        .iter()
        .any(|e| e.niveau_de_sort == 6 && e.domaine == 1));
    assert!(!inc.emplacements.iter().any(|e| e.niveau_de_sort == 7));
    assert!(
        bien.sorts
            .iter()
            .find(|s| s.niveau == 6)
            .unwrap()
            .accessible
    );
    assert!(
        !bien
            .sorts
            .iter()
            .find(|s| s.niveau == 7)
            .unwrap()
            .accessible
    );
}

#[test]
fn certains_domaines_ajoutent_des_competences_de_classe() {
    let r = Referentiel::core();
    let mut f = pretre_de("calistria", "CN", &["connaissance", "duperie"], 1);
    f.niveaux[0].rangs.insert("knowledge:history".into(), 1);
    f.niveaux[0].rangs.insert("stealth".into(), 1);
    f.niveaux[0].rangs.insert("climb".into(), 1);
    let v = f.calculer(&r).unwrap();
    let de_classe = |cle: &str| {
        v.competences[cle]
            .termes
            .iter()
            .any(|t| t.libelle == "compétence de classe")
    };
    // Connaissance : toutes les Connaissances ; Duperie : Bluff, Déguisement, Discrétion.
    assert!(de_classe("knowledge:history"));
    assert!(de_classe("stealth"));
    // Escalade n'est pas une compétence de classe du prêtre.
    assert!(!de_classe("climb"));
}

#[test]
fn le_druide_n_a_droit_qu_aux_armures_et_boucliers_sans_metal() {
    let r = Referentiel::core();
    let mut f = druide(1);
    f.equipement.armure = Some("Hide".into());
    f.equipement.bouclier = Some("Heavy wooden".into());
    assert!(f.problemes_equipement(&r).is_empty());
    f.equipement.armure = Some("Studded leather".into());
    f.equipement.bouclier = Some("Buckler".into());
    let p = f.problemes_equipement(&r);
    assert_eq!(
        p.iter()
            .filter(|x| matches!(x, crate::Probleme::ObjetInterdit { .. }))
            .count(),
        2,
        "{p:?}"
    );
    // Le prêtre, lui, porte la cotte de mailles.
    let mut g = pretre(1, 14);
    g.equipement.armure = Some("Chainmail".into());
    assert!(g.problemes_equipement(&r).is_empty());
    // Refusé aussi par le journal : l'armure interdite ne s'enfile pas.
    let regles = ReglesFiche::default();
    let mut j = Journal::nouveau(druide(1));
    let e = EvenementFiche::Equiper {
        emplacement: crate::journal::EmplacementProtection::Armure,
        objet: Some("Chainmail".into()),
    };
    assert!(j.ajouter(e, &r, &regles).is_err());
}

// ── Pouvoirs limités par jour ─────────────────────────────────────────────────

#[test]
fn les_pouvoirs_du_jour_se_comptent_et_se_rendent_au_repos() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    // Prêtre niveau 5, Charisme 12 (+1) : canalisation 4 fois par jour ; Toucher du Bien : 3 + SAG (+4) = 7.
    let mut j = Journal::nouveau(pretre_de("iomedae", "LB", &["bien", "soleil"], 5));
    let ajouter = |j: &mut Journal, e| j.ajouter(e, &r, &regles);
    let utiliser = |cle: &str| EvenementFiche::PouvoirUtilise { cle: cle.into() };
    let v = j.etat(&r, &regles).unwrap().fiche.calculer(&r).unwrap();
    let max = |cle: &str| v.pouvoirs.iter().find(|p| p.cle == cle).map(|p| p.max);
    assert_eq!(max("canalisation"), Some(Some(4)));
    assert_eq!(max("domaine:bien:0"), Some(Some(7)));
    // Lance du Bien : niveau 8, pas encore accordée au niveau 5.
    assert_eq!(max("domaine:bien:1"), None);
    for _ in 0..4 {
        ajouter(&mut j, utiliser("canalisation")).expect("utilisation permise");
    }
    assert!(
        ajouter(&mut j, utiliser("canalisation")).is_err(),
        "cinquième utilisation"
    );
    assert!(
        ajouter(&mut j, utiliser("domaine:bien:1")).is_err(),
        "pouvoir pas encore accordé"
    );
    assert!(ajouter(&mut j, utiliser("inconnu")).is_err());
    ajouter(&mut j, utiliser("domaine:bien:0")).expect("autre pouvoir");
    ajouter(&mut j, EvenementFiche::Repos).unwrap();
    ajouter(&mut j, utiliser("canalisation")).expect("rendu au repos");
}

#[test]
fn la_forme_animale_du_druide_suit_la_table() {
    let r = Referentiel::core();
    let utilisations = |niveau: u32| {
        let v = druide(niveau).calculer(&r).unwrap();
        v.pouvoirs
            .iter()
            .find(|p| p.cle == "forme_animale")
            .map(|p| p.max)
    };
    // Rien avant le niveau 4 ; 1 par jour au niveau 4, 2 au niveau 6, 8 au niveau 18, à volonté au niveau 20.
    assert_eq!(utilisations(3), None);
    assert_eq!(utilisations(4), Some(Some(1)));
    assert_eq!(utilisations(5), Some(Some(1)));
    assert_eq!(utilisations(6), Some(Some(2)));
    assert_eq!(utilisations(10), Some(Some(4)));
    assert_eq!(utilisations(18), Some(Some(8)));
    assert_eq!(utilisations(19), Some(Some(8)));
    assert_eq!(utilisations(20), Some(None));
}

// ── Pacte avec la nature du druide ────────────────────────────────────────────

#[test]
fn le_pacte_avec_la_nature_du_druide() {
    let r = Referentiel::core();
    // Sans choix : fiche inachevée, pas fausse.
    let f = druide(1);
    let pb = f.valider(&r);
    assert!(pb.contains(&crate::Probleme::PacteManquant));
    assert!(pb.iter().all(|p| p.est_incomplet()));
    // Pacte du domaine du Feu : un emplacement de domaine par niveau de sort, pouvoirs et sorts du domaine.
    let mut g = druide(5);
    g.pacte = Some("domaine".into());
    g.domaines = vec!["feu".into()];
    assert!(
        g.valider(&r).iter().all(|p| p.est_incomplet()),
        "{:?}",
        g.valider(&r)
    );
    let v = g.calculer(&r).unwrap();
    let inc = &v.incantateurs[0];
    assert!(inc
        .emplacements
        .iter()
        .filter(|e| e.niveau_de_sort >= 1)
        .all(|e| e.domaine == 1));
    assert_eq!(v.domaines.len(), 1);
    assert_eq!(v.domaines[0].nom, "Feu");
    // Le niveau de prêtre du domaine est le niveau de druide : Trait de feu (niveau 1) est actif.
    assert!(v.domaines[0].pouvoirs[0].actif);
    // Un domaine que le pacte n'offre pas (Bien, Mort…) est refusé.
    let mut h = druide(1);
    h.pacte = Some("domaine".into());
    h.domaines = vec!["bien".into()];
    assert!(h
        .valider(&r)
        .iter()
        .any(|p| matches!(p, crate::Probleme::DomaineHorsPacte { .. })));
    // Pacte du compagnon : pas de domaine, pas d'emplacement de domaine.
    let mut c = druide(1);
    c.pacte = Some("compagnon".into());
    assert!(c.valider(&r).iter().all(|p| p.est_incomplet()));
    let v = c.calculer(&r).unwrap();
    assert!(v.incantateurs[0]
        .emplacements
        .iter()
        .all(|e| e.domaine == 0));
    c.domaines = vec!["feu".into()];
    assert!(c
        .valider(&r)
        .iter()
        .any(|p| matches!(p, crate::Probleme::DomainesAChoisir { .. })));
    // Pacte inconnu.
    c.pacte = Some("autre".into());
    assert!(c
        .valider(&r)
        .iter()
        .any(|p| matches!(p, crate::Probleme::PacteInvalide(_))));
}

#[test]
fn le_druide_prend_une_forme_animale() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut j = Journal::nouveau(druide(4));
    let ajouter = |j: &mut Journal, e| j.ajouter(e, &r, &regles);
    let adopter = |id: &str| EvenementFiche::FormeAdoptee { forme: id.into() };
    let sans = j.etat(&r, &regles).unwrap().fiche.calculer(&r).unwrap();
    // Niveau 4 : animal de taille P ou M seulement ; pas de taille TG (niveau 8).
    assert!(ajouter(&mut j, adopter("beast_shape_iii_animal_de_taille_tg")).is_err());
    assert!(ajouter(&mut j, adopter("forme_inconnue")).is_err());
    // Animal de taille M : +2 Force (taille), +2 armure naturelle, pendant 1 heure par niveau (4 h = 2 400 rounds).
    ajouter(&mut j, adopter("beast_shape_i_animal_de_taille_m")).expect("forme permise");
    let e = j.etat(&r, &regles).unwrap();
    let v = e.fiche.calculer(&r).unwrap();
    assert_eq!(
        v.scores[&Carac::For].total,
        sans.scores[&Carac::For].total + 2
    );
    assert_eq!(v.ca.total, sans.ca.total + 2);
    let t = e
        .fiche
        .temporaires
        .iter()
        .find(|t| t.nom.starts_with("Forme animale"))
        .expect("effet temporaire");
    assert_eq!(t.rounds_restants, Some(4 * 600));
    // Une seule utilisation par jour au niveau 4 : pas de seconde transformation avant le repos.
    assert!(ajouter(&mut j, adopter("beast_shape_i_animal_de_taille_p")).is_err());
    // Redevenir normal rend les valeurs, mais pas l'utilisation du jour.
    ajouter(&mut j, EvenementFiche::FormeQuittee).unwrap();
    let v = j.etat(&r, &regles).unwrap().fiche.calculer(&r).unwrap();
    assert_eq!(v.scores[&Carac::For].total, sans.scores[&Carac::For].total);
    assert!(ajouter(&mut j, adopter("beast_shape_i_animal_de_taille_p")).is_err());
    ajouter(&mut j, EvenementFiche::Repos).unwrap();
    ajouter(&mut j, adopter("beast_shape_i_animal_de_taille_p")).expect("rendu au repos");
    // Un prêtre n'a pas de forme animale.
    let mut p = Journal::nouveau(pretre(4, 14));
    assert!(p
        .ajouter(adopter("beast_shape_i_animal_de_taille_m"), &r, &regles)
        .is_err());
}

#[test]
fn le_druide_avec_son_compagnon_animal() {
    use crate::compagnon::Compagnon;
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut f = druide(4);
    f.pacte = Some("compagnon".into());
    // Pacte du compagnon sans animal choisi : fiche inachevée.
    assert!(f.valider(&r).contains(&crate::Probleme::CompagnonManquant));
    f.compagnon = Some(Compagnon {
        animal: "loup".into(),
        nom: "Croc".into(),
        ignore_developpement: false,
        augmentations: vec![],
    });
    assert!(
        f.valider(&r).iter().all(|p| p.est_incomplet()),
        "{:?}",
        f.valider(&r)
    );
    let v = f.calculer(&r).unwrap();
    let c = v.compagnon.as_ref().expect("compagnon");
    // Loup : taille M, développement au niveau 7 ; au niveau 4 : 4 DV, BBA +3.
    assert_eq!(
        (c.animal.as_str(), c.taille.as_str(), c.dv, c.bba),
        ("Loup", "M", 4, 3)
    );
    assert!(!c.developpement_applique);
    assert_eq!(c.augmentations_permises, 1);
    // Le compagnon ne se calcule pas sans le pacte du compagnon.
    let mut sans = f.clone();
    sans.pacte = Some("domaine".into());
    sans.domaines = vec!["feu".into()];
    assert!(sans.calculer(&r).unwrap().compagnon.is_none());
    // Trop d'augmentations pour le niveau (une seule au niveau 4).
    let mut trop = f.clone();
    trop.compagnon.as_mut().unwrap().augmentations = vec![Carac::Sag, Carac::Con];
    assert!(trop
        .valider(&r)
        .iter()
        .any(|p| matches!(p, crate::Probleme::AugmentationsDeTrop { .. })));
    // Animal inconnu.
    let mut inconnu = f.clone();
    inconnu.compagnon.as_mut().unwrap().animal = "dragon".into();
    assert!(inconnu
        .valider(&r)
        .iter()
        .any(|p| matches!(p, crate::Probleme::CompagnonInconnu(_))));
    // Le journal modifie le compagnon (nom, augmentation) et refuse un choix invalide.
    let mut j = Journal::nouveau(f);
    let mut nouveau = Compagnon {
        animal: "loup".into(),
        nom: "Crocs".into(),
        ignore_developpement: false,
        augmentations: vec![Carac::Sag],
    };
    j.ajouter(
        EvenementFiche::CompagnonModifie(nouveau.clone()),
        &r,
        &regles,
    )
    .expect("choix valide");
    let e = j.etat(&r, &regles).unwrap();
    let c = e.fiche.calculer(&r).unwrap().compagnon.unwrap();
    assert_eq!(c.nom, "Crocs");
    assert_eq!(
        c.caracteristiques
            .iter()
            .find(|x| x.0 == Carac::Sag)
            .unwrap()
            .1,
        12 + 1
    );
    nouveau.augmentations = vec![Carac::Sag, Carac::For];
    assert!(j
        .ajouter(EvenementFiche::CompagnonModifie(nouveau), &r, &regles)
        .is_err());
}

fn barbare(niveau: u32, constitution: i32) -> Fiche {
    let scores = [
        (Carac::For, 15),
        (Carac::Dex, 12),
        (Carac::Con, constitution),
        (Carac::Int, 10),
        (Carac::Sag, 10),
        (Carac::Cha, 8),
    ];
    Fiche {
        nom: "Test".into(),
        race: "humain".into(),
        bonus_racial_libre: Some(Carac::For),
        scores_base: scores.into_iter().collect(),
        dommages: BTreeMap::new(),
        drain: BTreeMap::new(),
        augmentations: vec![],
        niveaux: (0..niveau)
            .map(|i| NiveauPris {
                classe: "barbare".into(),
                de_vie_obtenu: if i == 0 { 12 } else { 7 },
                rangs: BTreeMap::new(),
                predilection: Some(BonusPredilection::PointDeVie),
                dons: vec![],
            })
            .collect(),
        classes_de_predilection: vec!["barbare".into()],
        lignage: None,
        connaissance_de_lignage: None,
        style_de_combat: None,
        effets: vec![],
        sorts_connus: vec![],
        equipement: Default::default(),
        etats: vec![],
        temporaires: vec![],
        divinite: None,
        alignement: None,
        domaines: vec![],
        energie: None,
        pacte: None,
        compagnon: None,
        graces: vec![],
        talents_roublard: vec![],
        ennemis_jures: vec![],
        augmentations_ennemis: vec![],
        environnements: vec![],
        augmentations_environnements: vec![],
        pacte_du_chasseur: None,
        pouvoirs_rage: vec![],
        en_rage: false,
        rounds_de_rage: 0,
        fatigue_rage: None,
    }
}

#[test]
fn la_classe_barbare_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("barbare").expect("barbare");
    assert_eq!(c.de_vie, 12);
    assert_eq!(c.rangs_par_niveau, 4);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.is_none());
    assert_eq!(c.source.page, Some(32));
}

#[test]
fn table_3_2_du_barbare() {
    let r = Referentiel::core();
    let c = r.classe("barbare").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +1, Réf +0, Vig +2, Vol +0.
    assert_eq!(l(1).bba, vec![1]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (0, 2, 0));
    // Niveau 6 : BBA +6/+1, Réf +2, Vig +5, Vol +2.
    assert_eq!(l(6).bba, vec![6, 1]);
    assert_eq!((l(6).reflexes, l(6).vigueur, l(6).volonte), (2, 5, 2));
    // Niveau 11 : BBA +11/+6/+1, Réf +3, Vig +7, Vol +3.
    assert_eq!(l(11).bba, vec![11, 6, 1]);
    assert_eq!((l(11).reflexes, l(11).vigueur, l(11).volonte), (3, 7, 3));
    // Niveau 20 : BBA +20/+15/+10/+5, Réf +6, Vig +12, Vol +6.
    assert_eq!(l(20).bba, vec![20, 15, 10, 5]);
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (6, 12, 6));
}

#[test]
fn la_rage_du_barbare_suit_4_plus_constitution_et_2_par_niveau() {
    let r = Referentiel::core();
    // Constitution 14 : modificateur +2.
    let rage = |niveau: u32| {
        barbare(niveau, 14)
            .calculer(&r)
            .unwrap()
            .pouvoirs
            .iter()
            .find(|p| p.cle == "rage")
            .and_then(|p| p.max)
    };
    assert_eq!(rage(1), Some(6));
    assert_eq!(rage(2), Some(8));
    assert_eq!(rage(5), Some(14));
}

#[test]
fn les_competences_de_classe_du_barbare() {
    let r = Referentiel::core();
    let mut f = barbare(1, 14);
    f.niveaux[0].rangs.insert("intimidate".into(), 1);
    f.niveaux[0].rangs.insert("bluff".into(), 1);
    let v = f.calculer(&r).unwrap();
    assert!(v.competences["intimidate"]
        .termes
        .iter()
        .any(|t| t.libelle == "compétence de classe"));
    assert!(!v.competences["bluff"]
        .termes
        .iter()
        .any(|t| t.libelle == "compétence de classe"));
}

#[test]
fn un_barbare_monte_du_niveau_1_au_niveau_5() {
    use crate::progression::{appliquer, montee_auto};
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut f = barbare(1, 14);
    f.niveaux[0].dons = vec!["Toughness".into(), "Alertness".into()];
    for _ in 2..=5 {
        let m = montee_auto(&r, &f, "barbare", 7);
        f = appliquer(&r, &regles, &f, &m, 20).expect("montée valide");
    }
    assert_eq!(f.niveau_de_personnage(), 5);
    assert_eq!(f.niveaux.len(), 5);
    let v = f.calculer(&r).unwrap();
    assert_eq!(
        v.pouvoirs
            .iter()
            .find(|p| p.cle == "rage")
            .and_then(|p| p.max),
        Some(14)
    );
}

fn barde(niveau: u32, charisme: i32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| {
        n.classe = "barde".into();
        n.de_vie_obtenu = 8;
    });
    f.classes_de_predilection = vec!["barde".into()];
    f.scores_base.insert(Carac::Cha, charisme);
    f
}

#[test]
fn la_classe_barde_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("barde").expect("barde");
    assert_eq!(c.de_vie, 8);
    assert_eq!(c.rangs_par_niveau, 6);
    assert_eq!(c.progression.len(), 20);
    assert!(!c.incantation.as_ref().unwrap().prepare());
    assert_eq!(c.source.page, Some(38));
}

#[test]
fn table_3_3_du_barde() {
    let r = Referentiel::core();
    let c = r.classe("barde").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +0, Réf +2, Vig +0, Vol +2 ; 1 sort de niveau 1.
    assert_eq!(l(1).bba, vec![0]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (2, 0, 2));
    assert_eq!(l(1).sorts_par_jour[0], Some(1));
    assert_eq!(l(1).sorts_connus, vec![Some(4), Some(2)]);
    // Niveau 11 : BBA +8/+3, Réf +7, Vig +3, Vol +7 ; 5/4/4/2 sorts.
    assert_eq!(l(11).bba, vec![8, 3]);
    assert_eq!((l(11).reflexes, l(11).vigueur, l(11).volonte), (7, 3, 7));
    assert_eq!(
        &l(11).sorts_par_jour[..4],
        &[Some(5), Some(4), Some(4), Some(2)]
    );
    // Niveau 20 : BBA +15/+10/+5, Réf +12, Vig +6, Vol +12 ; 5 sorts de chaque niveau 1 à 6.
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (12, 6, 12));
    assert!(l(20).sorts_par_jour.iter().take(6).all(|s| *s == Some(5)));
}

#[test]
fn la_representation_bardique_suit_4_plus_charisme() {
    let r = Referentiel::core();
    // Charisme 16 : modificateur +3.
    let rounds = |niveau: u32| {
        barde(niveau, 16)
            .calculer(&r)
            .unwrap()
            .pouvoirs
            .iter()
            .find(|p| p.cle == "representation")
            .and_then(|p| p.max)
    };
    assert_eq!(rounds(1), Some(7));
    assert_eq!(rounds(3), Some(11));
}

#[test]
fn le_savoir_du_barde_couvre_toutes_les_connaissances() {
    let r = Referentiel::core();
    let mut f = barde(1, 14);
    f.niveaux[0].rangs.insert("knowledge:arcana".into(), 1);
    f.niveaux[0].rangs.insert("knowledge:nature".into(), 1);
    f.niveaux[0].rangs.insert("bluff".into(), 1);
    f.niveaux[0].rangs.insert("survival".into(), 1);
    let v = f.calculer(&r).unwrap();
    let de_classe = |cle: &str| {
        v.competences[cle]
            .termes
            .iter()
            .any(|t| t.libelle == "compétence de classe")
    };
    assert!(de_classe("knowledge:arcana"));
    assert!(de_classe("knowledge:nature"));
    assert!(de_classe("bluff"));
    assert!(!de_classe("survival"));
}

fn guerrier(niveau: u32, force: i32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| {
        n.classe = "guerrier".into();
        n.de_vie_obtenu = 10;
    });
    f.classes_de_predilection = vec!["guerrier".into()];
    f.scores_base.insert(Carac::For, force);
    f
}

#[test]
fn la_classe_guerrier_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("guerrier").expect("guerrier");
    assert_eq!(c.de_vie, 10);
    assert_eq!(c.rangs_par_niveau, 2);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.is_none());
    assert_eq!(c.source.page, Some(55));
}

#[test]
fn table_3_9_du_guerrier() {
    let r = Referentiel::core();
    let c = r.classe("guerrier").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +1, Réf +0, Vig +2, Vol +0.
    assert_eq!(l(1).bba, vec![1]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (0, 2, 0));
    // Niveau 11 : BBA +11/+6/+1, Réf +3, Vig +7, Vol +3.
    assert_eq!(l(11).bba, vec![11, 6, 1]);
    assert_eq!((l(11).reflexes, l(11).vigueur, l(11).volonte), (3, 7, 3));
    // Niveau 20 : BBA +20/+15/+10/+5, Réf +6, Vig +12, Vol +6.
    assert_eq!(l(20).bba, vec![20, 15, 10, 5]);
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (6, 12, 6));
}

#[test]
fn le_guerrier_gagne_un_don_supplementaire_a_ses_niveaux_pairs() {
    let r = Referentiel::core();
    let f = guerrier(4, 14);
    // Niveau 1 : don du niveau 1 + don supplémentaire de guerrier + don de race humaine.
    assert_eq!(f.dons_attendus_a(&r, 0), 3);
    // Niveau 2 : aucun don (ni standard ni de classe).
    assert_eq!(f.dons_attendus_a(&r, 1), 0);
    // Niveau 4 : don supplémentaire de guerrier.
    assert_eq!(f.dons_attendus_a(&r, 3), 1);
}

#[test]
fn un_guerrier_monte_du_niveau_1_au_niveau_5() {
    use crate::progression::{appliquer, montee_auto};
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut f = guerrier(1, 16);
    f.niveaux[0].dons = vec!["Toughness".into(), "Alertness".into(), "Dodge".into()];
    for _ in 2..=5 {
        let m = montee_auto(&r, &f, "guerrier", 7);
        f = appliquer(&r, &regles, &f, &m, 20).expect("montée valide");
    }
    assert_eq!(f.niveau_de_personnage(), 5);
    assert_eq!(f.niveaux.len(), 5);
}

fn moine(niveau: u32, sagesse: i32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| {
        n.classe = "moine".into();
        n.de_vie_obtenu = 8;
    });
    f.classes_de_predilection = vec!["moine".into()];
    f.scores_base.insert(Carac::Sag, sagesse);
    f
}

#[test]
fn la_classe_moine_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("moine").expect("moine");
    assert_eq!(c.de_vie, 8);
    assert_eq!(c.rangs_par_niveau, 4);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.is_none());
    assert_eq!(c.source.page, Some(65));
}

#[test]
fn table_3_11_du_moine() {
    let r = Referentiel::core();
    let c = r.classe("moine").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +0, jets +2/+2/+2.
    assert_eq!(l(1).bba, vec![0]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (2, 2, 2));
    // Niveau 8 : BBA +6/+1, jets +6/+6/+6.
    assert_eq!(l(8).bba, vec![6, 1]);
    assert_eq!((l(8).reflexes, l(8).vigueur, l(8).volonte), (6, 6, 6));
    // Niveau 20 : BBA +15/+10/+5, jets +12/+12/+12.
    assert_eq!(l(20).bba, vec![15, 10, 5]);
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (12, 12, 12));
}

#[test]
fn le_moine_gagne_des_dons_supplementaires_aux_niveaux_1_2_6_10() {
    let r = Referentiel::core();
    // Niveau 1 : don du niveau 1 + don de moine + don de race humaine.
    assert_eq!(moine(3, 14).dons_attendus_a(&r, 0), 3);
    // Niveau 2 : don de moine seulement.
    assert_eq!(moine(3, 14).dons_attendus_a(&r, 1), 1);
    // Niveau 6 : don de moine seulement (les dons standard tombent aux niveaux impairs).
    assert_eq!(moine(6, 14).dons_attendus_a(&r, 5), 1);
    // Niveau 4 : aucun don.
    assert_eq!(moine(4, 14).dons_attendus_a(&r, 3), 0);
}

fn paladin(niveau: u32, charisme: i32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| {
        n.classe = "paladin".into();
        n.de_vie_obtenu = 10;
    });
    f.classes_de_predilection = vec!["paladin".into()];
    f.scores_base.insert(Carac::Cha, charisme);
    f
}

fn pouvoir_du_paladin(f: &Fiche, cle: &str) -> Option<i32> {
    let r = Referentiel::core();
    f.calculer(&r)
        .unwrap()
        .pouvoirs
        .iter()
        .find(|p| p.cle == cle)
        .and_then(|p| p.max)
}

#[test]
fn la_classe_paladin_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("paladin").expect("paladin");
    assert_eq!(c.de_vie, 10);
    assert_eq!(c.rangs_par_niveau, 2);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.as_ref().unwrap().prepare());
    assert_eq!(c.source.page, Some(68));
}

#[test]
fn table_3_12_du_paladin() {
    let r = Referentiel::core();
    let c = r.classe("paladin").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +1, Réf +0, Vig +2, Vol +2 ; pas encore de sorts.
    assert_eq!(l(1).bba, vec![1]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (0, 2, 2));
    assert!(l(1).sorts_par_jour.is_empty());
    // Niveau 4 : un sort de niveau 1 uniquement grâce à la sagesse.
    assert_eq!(l(4).sorts_par_jour, vec![Some(0)]);
    // Niveau 11 : BBA +11/+6/+1, Réf +3, Vig +7, Vol +7 ; 2/1/1 sorts.
    assert_eq!(l(11).bba, vec![11, 6, 1]);
    assert_eq!((l(11).reflexes, l(11).vigueur, l(11).volonte), (3, 7, 7));
    assert_eq!(l(11).sorts_par_jour, vec![Some(2), Some(1), Some(1)]);
    // Niveau 20 : Réf +6, Vig +12, Vol +12 ; 4/4/3/3 sorts.
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (6, 12, 12));
    assert_eq!(l(20).sorts_par_jour.len(), 4);
}

#[test]
fn chatiment_et_imposition_des_mains_du_paladin() {
    // Châtiment : 1/jour au niveau 1, +1 tous les 3 niveaux (2 au niveau 4, 4 au niveau 10).
    assert_eq!(pouvoir_du_paladin(&paladin(1, 14), "chatiment"), Some(1));
    assert_eq!(pouvoir_du_paladin(&paladin(4, 14), "chatiment"), Some(2));
    assert_eq!(pouvoir_du_paladin(&paladin(10, 14), "chatiment"), Some(4));
    // Imposition des mains : moitié du niveau + Charisme (14 : +2). Niveau 2 : 1 + 2 = 3 ; niveau 5 : 2 + 2 = 4.
    assert_eq!(pouvoir_du_paladin(&paladin(2, 14), "imposition"), Some(3));
    assert_eq!(pouvoir_du_paladin(&paladin(5, 14), "imposition"), Some(4));
}

#[test]
fn le_paladin_monte_du_niveau_1_au_niveau_5() {
    use crate::progression::{appliquer, montee_auto};
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut f = paladin(1, 14);
    f.niveaux[0].dons = vec!["Toughness".into(), "Alertness".into()];
    for _ in 2..=5 {
        let m = montee_auto(&r, &f, "paladin", 7);
        f = appliquer(&r, &regles, &f, &m, 20).expect("montée valide");
    }
    assert_eq!(f.niveau_de_personnage(), 5);
    assert_eq!(pouvoir_du_paladin(&f, "imposition"), Some(4));
}

fn rodeur(niveau: u32, sagesse: i32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| {
        n.classe = "rodeur".into();
        n.de_vie_obtenu = 10;
    });
    f.classes_de_predilection = vec!["rodeur".into()];
    f.scores_base.insert(Carac::Sag, sagesse);
    f
}

#[test]
fn la_classe_rodeur_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("rodeur").expect("rôdeur");
    assert_eq!(c.de_vie, 10);
    assert_eq!(c.rangs_par_niveau, 6);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.as_ref().unwrap().prepare());
    assert_eq!(c.source.page, Some(82));
}

#[test]
fn table_3_15_du_rodeur() {
    let r = Referentiel::core();
    let c = r.classe("rodeur").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +1, Réf +2, Vig +2, Vol +0.
    assert_eq!(l(1).bba, vec![1]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (2, 2, 0));
    // Niveau 11 : BBA +11/+6/+1, Réf +7, Vig +7, Vol +3 ; 2/1/1 sorts.
    assert_eq!(l(11).bba, vec![11, 6, 1]);
    assert_eq!((l(11).reflexes, l(11).vigueur, l(11).volonte), (7, 7, 3));
    assert_eq!(l(11).sorts_par_jour, vec![Some(2), Some(1), Some(1)]);
    // Niveau 20 : Réf +12, Vig +12, Vol +6.
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (12, 12, 6));
}

#[test]
fn le_rodeur_gagne_un_don_de_style_de_combat_aux_niveaux_2_6_10_14_18() {
    let r = Referentiel::core();
    // Niveau 1 : don du niveau 1 + don de race humaine, pas de don de style.
    assert_eq!(rodeur(2, 14).dons_attendus_a(&r, 0), 2);
    // Niveau 2 : don de style de combat.
    assert_eq!(rodeur(2, 14).dons_attendus_a(&r, 1), 1);
    // Niveau 4 : aucun don.
    assert_eq!(rodeur(4, 14).dons_attendus_a(&r, 3), 0);
}

fn roublard(niveau: u32, dexterite: i32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| {
        n.classe = "roublard".into();
        n.de_vie_obtenu = 8;
    });
    f.classes_de_predilection = vec!["roublard".into()];
    f.scores_base.insert(Carac::Dex, dexterite);
    f
}

#[test]
fn la_classe_roublard_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("roublard").expect("roublard");
    assert_eq!(c.de_vie, 8);
    assert_eq!(c.rangs_par_niveau, 8);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.is_none());
    assert_eq!(c.source.page, Some(85));
}

#[test]
fn table_3_16_du_roublard() {
    let r = Referentiel::core();
    let c = r.classe("roublard").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +0, Réf +2, Vig +0, Vol +0.
    assert_eq!(l(1).bba, vec![0]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (2, 0, 0));
    // Niveau 11 : BBA +8/+3, Réf +7, Vig +3, Vol +3.
    assert_eq!(l(11).bba, vec![8, 3]);
    assert_eq!((l(11).reflexes, l(11).vigueur, l(11).volonte), (7, 3, 3));
    // Niveau 20 : BBA +15/+10/+5, Réf +12, Vig +6, Vol +6.
    assert_eq!(l(20).bba, vec![15, 10, 5]);
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (12, 6, 6));
}

#[test]
fn les_competences_de_classe_du_roublard() {
    let r = Referentiel::core();
    let mut f = roublard(1, 14);
    f.niveaux[0].rangs.insert("disable_device".into(), 1);
    f.niveaux[0].rangs.insert("survival".into(), 1);
    let v = f.calculer(&r).unwrap();
    let de_classe = |cle: &str| {
        v.competences[cle]
            .termes
            .iter()
            .any(|t| t.libelle == "compétence de classe")
    };
    assert!(de_classe("disable_device"));
    assert!(!de_classe("survival"));
}

#[test]
fn un_roublard_monte_du_niveau_1_au_niveau_5() {
    use crate::progression::{appliquer, montee_auto};
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut f = roublard(1, 16);
    f.niveaux[0].dons = vec!["Toughness".into(), "Alertness".into()];
    for _ in 2..=5 {
        let m = montee_auto(&r, &f, "roublard", 5);
        f = appliquer(&r, &regles, &f, &m, 20).expect("montée valide");
    }
    assert_eq!(f.niveau_de_personnage(), 5);
}

fn magicien(niveau: u32, intelligence: i32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| {
        n.classe = "magicien".into();
        n.de_vie_obtenu = 6;
    });
    f.classes_de_predilection = vec!["magicien".into()];
    f.scores_base.insert(Carac::Int, intelligence);
    f
}

#[test]
fn la_classe_magicien_est_chargee_depuis_le_livre() {
    let r = Referentiel::core();
    let c = r.classe("magicien").expect("magicien");
    assert_eq!(c.de_vie, 6);
    assert_eq!(c.rangs_par_niveau, 2);
    assert_eq!(c.progression.len(), 20);
    assert!(c.incantation.as_ref().unwrap().prepare());
    assert_eq!(c.source.page, Some(60));
}

#[test]
fn table_3_10_du_magicien() {
    let r = Referentiel::core();
    let c = r.classe("magicien").unwrap();
    let l = |n: usize| &c.progression[n - 1];
    // Niveau 1 : BBA +0, Réf +0, Vig +0, Vol +2 ; 3 tours de magie, 1 sort de niveau 1.
    assert_eq!(l(1).bba, vec![0]);
    assert_eq!((l(1).reflexes, l(1).vigueur, l(1).volonte), (0, 0, 2));
    assert_eq!(l(1).oraisons, Some(3));
    assert_eq!(l(1).sorts_par_jour, vec![Some(1)]);
    // Niveau 11 : BBA +5, Réf +3, Vig +3, Vol +7 ; 4/4/4/3/2/1 sorts.
    assert_eq!(l(11).bba, vec![5]);
    assert_eq!((l(11).reflexes, l(11).vigueur, l(11).volonte), (3, 3, 7));
    assert_eq!(l(11).sorts_par_jour.len(), 6);
    // Niveau 20 : BBA +10/+5, Réf +6, Vig +6, Vol +12 ; 4 sorts de chaque niveau 1 à 9.
    assert_eq!((l(20).reflexes, l(20).vigueur, l(20).volonte), (6, 6, 12));
    assert!(l(20).sorts_par_jour.iter().all(|s| *s == Some(4)));
    assert_eq!(l(20).sorts_par_jour.len(), 9);
}

#[test]
fn le_magicien_gagne_un_don_supplementaire_au_niveau_5() {
    let r = Referentiel::core();
    // Niveau 1 : don du niveau 1 + don de race humaine.
    assert_eq!(magicien(1, 16).dons_attendus_a(&r, 0), 2);
    // Niveau 4 : aucun don.
    assert_eq!(magicien(4, 16).dons_attendus_a(&r, 3), 0);
    // Niveau 5 : don standard du niveau impair + don supplémentaire de magicien.
    assert_eq!(magicien(5, 16).dons_attendus_a(&r, 4), 2);
}

/// Fiche de test d'une classe donnée, sur le modèle du barbare (seul le nom de classe change).
fn de_classe(classe: &str, niveau: u32) -> Fiche {
    let mut f = barbare(niveau, 14);
    f.niveaux.iter_mut().for_each(|n| n.classe = classe.into());
    f.classes_de_predilection = vec![classe.into()];
    f
}

#[test]
fn le_paladin_choisit_une_grace_tous_les_3_niveaux_a_partir_du_3() {
    let r = Referentiel::core();
    let c = &r.choix;
    assert_eq!(
        [2, 3, 5, 6, 9, 12].map(|n| c.nombre_graces(n)),
        [0, 1, 1, 2, 3, 4]
    );
    let p = de_classe("paladin", 3).problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::GracesAChoisir {
        attendus: 1,
        choisis: 0
    }));
    let mut f = de_classe("paladin", 3);
    f.graces = vec!["Secoué".into()];
    assert!(!f
        .problemes_choix_de_classe(&r)
        .iter()
        .any(|p| matches!(p, Probleme::GracesAChoisir { .. })));
}

#[test]
fn une_grace_doit_etre_accessible_au_niveau_et_avoir_son_prerequis() {
    let r = Referentiel::core();
    let mut f = de_classe("paladin", 3);
    f.graces = vec!["Chancelant".into()];
    let p = f.problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::GraceNonDisponible {
        nom: "Chancelant".into(),
        niveau: 6
    }));

    let mut f = de_classe("paladin", 9);
    f.graces = vec!["Effrayé".into(), "Fatigué".into(), "Fiévreux".into()];
    let p = f.problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::GracePrerequisManquante {
        nom: "Effrayé".into(),
        requis: "Secoué".into()
    }));

    f.graces = vec!["Secoué".into(), "Effrayé".into(), "Fatigué".into()];
    assert!(f.problemes_choix_de_classe(&r).is_empty());

    f.graces = vec!["Nexistepas".into(), "Fatigué".into(), "Fiévreux".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::GraceInconnue("Nexistepas".into())));
}

#[test]
fn le_roublard_ne_peut_prendre_que_des_talents_du_livre() {
    let r = Referentiel::core();
    assert!(r.choix.talent_connu("Attaque sanglante*"));
    assert!(r.choix.talent_connu("Attaque sanglante"));
    assert!(r.choix.talent_connu("Esprit fuyant"));
    assert!(!r.choix.talent_connu("Talent inventé"));
    let mut f = de_classe("roublard", 4);
    f.talents_roublard = vec!["Talent inventé".into(), "Debout".into()];
    let p = f.problemes_choix_de_classe(&r);
    assert_eq!(p, vec![Probleme::TalentInconnu("Talent inventé".into())]);
}

#[test]
fn talents_du_roublard_nombre_doublons_maitre_et_prerequis() {
    let r = Referentiel::core();
    let c = &r.choix;
    assert_eq!(
        [1, 2, 3, 4, 10, 20].map(|n| c.nombre_talents(n)),
        [0, 1, 1, 2, 5, 10]
    );
    assert_eq!(
        [9, 10, 12, 20].map(|n| c.talents_de_maitre_permis(n)),
        [0, 1, 2, 6]
    );
    let mut f = de_classe("roublard", 2);
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::TalentsAChoisir {
            attendus: 1,
            choisis: 0
        }));
    f.talents_roublard = vec!["Debout".into()];
    assert!(f.problemes_choix_de_classe(&r).is_empty());
    // Doublon refusé, sauf Maîtrise des compétences.
    let mut f = de_classe("roublard", 4);
    f.talents_roublard = vec!["Debout".into(), "Debout".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::TalentEnDouble("Debout".into())));
    // Talent de maître avant le niveau 10 : refusé. Prérequis : Magie majeure demande Magie mineure.
    let mut f = de_classe("roublard", 4);
    f.talents_roublard = vec!["Esprit fuyant".into(), "Magie majeure".into()];
    let p = f.problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::TalentsDeMaitreTropNombreux {
        permis: 0,
        choisis: 1
    }));
    assert!(p.contains(&Probleme::TalentPrerequisManquant {
        nom: "Magie majeure".into(),
        requis: "Magie mineure".into()
    }));
}

#[test]
fn le_rodeur_choisit_un_ennemi_juré_aux_niveaux_1_5_10_15_20() {
    let r = Referentiel::core();
    let c = &r.choix;
    assert_eq!(
        [1, 4, 5, 10, 20].map(|n| c.nombre_ennemis_jures(n)),
        [1, 1, 2, 3, 5]
    );
    let p = de_classe("rodeur", 5).problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::EnnemisJuresAChoisir {
        attendus: 2,
        choisis: 0
    }));
    let mut f = de_classe("rodeur", 1);
    f.ennemis_jures = vec!["Mort-vivant".into()];
    assert!(f.problemes_choix_de_classe(&r).is_empty());
    f.ennemis_jures = vec!["Type inventé".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::EnnemiJureInconnu("Type inventé".into())));
}

#[test]
fn le_barbare_choisit_un_pouvoir_de_rage_au_niveau_2_puis_tous_les_2_niveaux() {
    use crate::choix_classe::PouvoirsRage;
    assert_eq!(
        [1, 2, 3, 4, 6, 20].map(PouvoirsRage::attendus),
        [0, 1, 1, 2, 3, 10]
    );
    let r = Referentiel::core();
    let p = de_classe("barbare", 2).problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::PouvoirsRageAChoisir {
        attendus: 1,
        choisis: 0
    }));
    let mut f = de_classe("barbare", 2);
    f.pouvoirs_rage = vec!["Bagarreur".into()];
    assert!(f.problemes_choix_de_classe(&r).is_empty());
}

#[test]
fn un_pouvoir_de_rage_doit_etre_de_niveau_et_avoir_son_prerequis() {
    let r = Referentiel::core();
    // Bagarreur majeur demande Bagarreur (niveau 1 de table) : pris sans Bagarreur, c'est un prérequis manquant.
    let mut f = de_classe("barbare", 2);
    f.pouvoirs_rage = vec!["Bagarreur majeur".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::PouvoirRagePrerequisManquant {
            nom: "Bagarreur majeur".into(),
            requis: "Bagarreur".into()
        }));
    // Coup redoutable est de niveau 12 : pas accessible au niveau 2.
    let mut f = de_classe("barbare", 2);
    f.pouvoirs_rage = vec!["Coup redoutable".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::PouvoirRageNonDisponible {
            nom: "Coup redoutable".into(),
            niveau: 12
        }));
    // Un pouvoir masqué (UC) n'est pas dans la liste : inconnu.
    let mut f = de_classe("barbare", 2);
    f.pouvoirs_rage = vec!["Coup handicapant".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::PouvoirRageInconnu("Coup handicapant".into())));
}

#[test]
fn les_pouvoirs_totemiques_ne_viennent_que_d_un_seul_groupe() {
    use crate::choix_classe::{groupe_rage, FamilleRage};
    assert_eq!(
        groupe_rage("Ailes totémiques draconiques"),
        Some((FamilleRage::Totem, "draconique".to_string()))
    );
    assert_eq!(groupe_rage("Bagarreur"), None);
    let r = Referentiel::core();
    let mut f = de_classe("barbare", 6);
    f.pouvoirs_rage = vec!["Animal totem mineur".into(), "Totem fiélon mineur".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::PouvoirRageGroupeDifferent {
            nom: "Totem fiélon mineur".into()
        }));
    let mut f = de_classe("barbare", 6);
    f.pouvoirs_rage = vec!["Animal totem mineur".into(), "Animal totem".into()];
    assert!(!f
        .problemes_choix_de_classe(&r)
        .iter()
        .any(|p| matches!(p, Probleme::PouvoirRageGroupeDifferent { .. })));
}

#[test]
fn la_canalisation_du_paladin_puise_2_utilisations_d_imposition_des_mains() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut f = de_classe("paladin", 4);
    f.scores_base.insert(Carac::Cha, 14);
    f.graces = vec!["Secoué".into()];
    let mut j = Journal::nouveau(f);
    let v = j.etat(&r, &regles).unwrap().fiche.calculer(&r).unwrap();
    let imposition = v
        .pouvoirs
        .iter()
        .find(|p| p.cle == "imposition")
        .and_then(|p| p.max)
        .unwrap();
    assert!(imposition >= 2);
    let canaliser = || EvenementFiche::PouvoirUtilise {
        cle: "canalisation".into(),
    };
    for _ in 0..imposition / 2 {
        assert!(j.ajouter(canaliser(), &r, &regles).is_ok());
    }
    let e = j.etat(&r, &regles).unwrap();
    assert_eq!(
        e.pouvoirs_utilises.get("imposition"),
        Some(&((imposition / 2 * 2) as u32))
    );
    assert!(!e.pouvoirs_utilises.contains_key("canalisation"));
    // Réserve vide (ou une seule utilisation restante) : la canalisation est refusée.
    assert!(j.ajouter(canaliser(), &r, &regles).is_err());
}

#[test]
fn la_rage_donne_ses_bonus_puis_fatigue_le_barbare() {
    use crate::choix_classe::effets_de_rage;
    let v = |n: u32| {
        effets_de_rage(n)
            .iter()
            .map(|e| e.valeur)
            .collect::<Vec<_>>()
    };
    assert_eq!(v(1), vec![4, 4, 2, -2]);
    assert_eq!(v(11), vec![6, 6, 3, -2]);
    assert_eq!(v(20), vec![8, 8, 4, -2]);
    assert!(effets_de_rage(0).is_empty());

    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut j = Journal::nouveau(de_classe("barbare", 1));
    assert!(j
        .ajouter(EvenementFiche::RageCommencee, &r, &regles)
        .is_ok());
    let e = j.etat(&r, &regles).unwrap();
    assert!(e.fiche.en_rage);
    assert!(j.ajouter(EvenementFiche::RageTerminee, &r, &regles).is_ok());
    let e = j.etat(&r, &regles).unwrap();
    assert!(!e.fiche.en_rage);
    assert!(e.fiche.etats.contains(&"fatigued".to_string()));
    // Fatigué : pas de nouvelle rage.
    assert!(j
        .ajouter(EvenementFiche::RageCommencee, &r, &regles)
        .is_err());
}

#[test]
fn inspirations_du_barde_et_courage_du_guerrier() {
    let l = crate::choix_classe::capacites_chiffrees(&de_classe("barde", 11));
    assert!(l[0].starts_with("Inspiration vaillante : +3"));
    assert!(l[1].starts_with("Inspiration talentueuse : +4"));
    assert!(l[2].starts_with("Inspiration glorieuse : 1 cible(s)"));
    assert_eq!(l.len(), 3);
    let l = crate::choix_classe::capacites_chiffrees(&de_classe("guerrier", 10));
    assert!(l.contains(&"Courage : +3 aux jets de sauvegarde contre la terreur".to_string()));
}

#[test]
fn la_fatigue_apres_la_rage_dure_le_double_des_rounds_et_s_use_avec_le_temps() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut j = Journal::nouveau(de_classe("barbare", 1));
    let temps = |rounds| EvenementFiche::TempsEcoule { rounds };
    assert!(j
        .ajouter(EvenementFiche::RageCommencee, &r, &regles)
        .is_ok());
    assert!(j.ajouter(temps(3), &r, &regles).is_ok());
    assert_eq!(j.etat(&r, &regles).unwrap().fiche.rounds_de_rage, 3);
    assert!(j.ajouter(EvenementFiche::RageTerminee, &r, &regles).is_ok());
    let f = j.etat(&r, &regles).unwrap().fiche;
    assert_eq!(f.fatigue_rage, Some(6));
    assert!(f.etats.contains(&"fatigued".to_string()));
    assert!(j.ajouter(temps(4), &r, &regles).is_ok());
    assert_eq!(j.etat(&r, &regles).unwrap().fiche.fatigue_rage, Some(2));
    assert!(j.ajouter(temps(2), &r, &regles).is_ok());
    let f = j.etat(&r, &regles).unwrap().fiche;
    assert_eq!(f.fatigue_rage, None);
    assert!(!f.etats.contains(&"fatigued".to_string()));
    // Plus fatigué : il peut entrer en rage de nouveau.
    assert!(j
        .ajouter(EvenementFiche::RageCommencee, &r, &regles)
        .is_ok());
    // Au niveau 17, aucune fatigue.
    let mut j = Journal::nouveau(de_classe("barbare", 17));
    assert!(j
        .ajouter(EvenementFiche::RageCommencee, &r, &regles)
        .is_ok());
    assert!(j.ajouter(EvenementFiche::RageTerminee, &r, &regles).is_ok());
    let f = j.etat(&r, &regles).unwrap().fiche;
    assert_eq!(f.fatigue_rage, None);
    assert!(!f.etats.contains(&"fatigued".to_string()));
}

#[test]
fn chaque_nouvel_ennemi_juré_dès_le_niveau_5_s_accompagne_d_une_augmentation_de_2() {
    let r = Referentiel::core();
    // Niveau 5 : 2 ennemis et 1 augmentation.
    let mut f = de_classe("rodeur", 5);
    f.ennemis_jures = vec!["Mort-vivant".into(), "Dragon".into()];
    let p = f.problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::AugmentationsEnnemisAChoisir {
        attendus: 1,
        choisis: 0
    }));
    f.augmentations_ennemis = vec!["Mort-vivant".into()];
    // Dès le niveau 5, le rôdeur a aussi son premier environnement (niveau 3) et son pacte (niveau 4).
    f.environnements = vec!["Jungle".into()];
    f.pacte_du_chasseur = Some("compagnon animal".into());
    assert!(f.problemes_choix_de_classe(&r).is_empty());
    let l = crate::choix_classe::capacites_chiffrees(&f);
    assert!(l
        .iter()
        .any(|x| x.starts_with("Ennemi juré (Mort-vivant) : +4 ")));
    assert!(l
        .iter()
        .any(|x| x.starts_with("Ennemi juré (Dragon) : +2 ")));
    // Un ennemi pris deux fois est refusé ; l'augmentation vise un ennemi déjà choisi.
    f.ennemis_jures = vec!["Dragon".into(), "Dragon".into()];
    f.augmentations_ennemis = vec!["Vase".into()];
    let p = f.problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::EnnemiJureEnDouble("Dragon".into())));
    assert!(p.contains(&Probleme::EnnemiJureInconnu("Vase".into())));
    // Niveau 1 : un seul ennemi, pas d'augmentation.
    let mut f = de_classe("rodeur", 1);
    f.ennemis_jures = vec!["Animal".into()];
    assert!(f.problemes_choix_de_classe(&r).is_empty());
}

#[test]
fn roublard_et_moine_affichent_leurs_valeurs() {
    let l = crate::choix_classe::capacites_chiffrees(&de_classe("roublard", 9));
    assert!(l
        .iter()
        .any(|x| x.starts_with("Recherche des pièges : +4 ")));
    assert!(l.iter().any(|x| x.starts_with("Sens des pièges : +3 ")));
    let l = crate::choix_classe::capacites_chiffrees(&de_classe("moine", 10));
    assert!(l
        .iter()
        .any(|x| x.contains("fer froid et argent, armes Loyales")));
}

#[test]
fn pouvoirs_de_rage_repetables_et_effets_chiffres() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    // Pas rapides jusqu'à trois fois, pas quatre ; Bagarreur une seule fois.
    let mut f = de_classe("barbare", 8);
    f.pouvoirs_rage = vec!["Pas rapides".into(); 3];
    f.pouvoirs_rage.push("Bagarreur".into());
    assert!(!f
        .problemes_choix_de_classe(&r)
        .iter()
        .any(|p| matches!(p, Probleme::PouvoirRageEnDouble(_))));
    f.pouvoirs_rage.push("Pas rapides".into());
    f.pouvoirs_rage.push("Bagarreur".into());
    let p = f.problemes_choix_de_classe(&r);
    assert!(p.contains(&Probleme::PouvoirRageEnDouble("Pas rapides".into())));
    assert!(p.contains(&Probleme::PouvoirRageEnDouble("Bagarreur".into())));
    assert!(r
        .rage
        .encore_prenable("Pas rapides", &vec!["Pas rapides".to_string(); 2]));
    assert!(!r
        .rage
        .encore_prenable("Bagarreur", &["Bagarreur".to_string()]));

    // En rage : Pas rapides ajoute 1,50 m (5 pieds) par choix, Escalade en rage le niveau à Escalade.
    let mut f = de_classe("barbare", 6);
    f.pouvoirs_rage = vec![
        "Pas rapides".into(),
        "Pas rapides".into(),
        "Escalade en rage".into(),
    ];
    let mut j = Journal::nouveau(f);
    let hors = j.etat(&r, &regles).unwrap().fiche.calculer(&r).unwrap();
    assert!(j
        .ajouter(EvenementFiche::RageCommencee, &r, &regles)
        .is_ok());
    let en = j.etat(&r, &regles).unwrap().fiche.calculer(&r).unwrap();
    assert_eq!(en.vitesse, hors.vitesse + 10);
    let escalade = |v: &crate::fiche::Valeurs| v.competences.get("climb").map(|d| d.total);
    // +6 d'altération (niveau) et +2 de modificateur de Force (+4 de rage) : au moins +6.
    assert!(escalade(&en).unwrap() >= escalade(&hors).unwrap() + 6);
}

#[test]
fn magie_mineure_et_majeure_demandent_10_et_11_d_intelligence() {
    let r = Referentiel::core();
    let mut f = de_classe("roublard", 4);
    f.talents_roublard = vec!["Magie mineure".into(), "Magie majeure".into()];
    f.scores_base.insert(Carac::Int, 10);
    f.bonus_racial_libre = Some(Carac::For);
    let p = f.problemes_choix_de_classe(&r);
    assert!(!p.iter().any(|x| matches!(x, Probleme::TalentIntelligenceInsuffisante { nom, .. } if nom == "Magie mineure")));
    assert!(p.contains(&Probleme::TalentIntelligenceInsuffisante {
        nom: "Magie majeure".into(),
        minimum: 11
    }));
    f.scores_base.insert(Carac::Int, 12);
    assert!(f.problemes_choix_de_classe(&r).is_empty());
    f.scores_base.insert(Carac::Int, 8);
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::TalentIntelligenceInsuffisante {
            nom: "Magie mineure".into(),
            minimum: 10
        }));
}

#[test]
fn les_textes_du_manuel_sont_ranges_par_capacite_de_classe() {
    let r = Referentiel::core();
    let texte = |classe: &str, cle: &str| {
        r.classe(classe)
            .and_then(|c| c.textes_capacites.get(cle))
            .map(String::as_str)
            .unwrap_or("")
    };
    assert!(texte("barbare", "rage de berserker").contains("4 + son modificateur de Constitution"));
    assert!(texte("barbare", "pouvoir de rage").contains("tous les deux niveaux"));
    assert!(texte("barde", "inspiration glorieuse").contains("2 dés de vie supplémentaires"));
    assert!(texte("barde", "inspiration héroïque").contains("bonus de moral de +4"));
    assert!(
        texte("guerrier", "entraînement aux armes").contains("groupe d’armes")
            || texte("guerrier", "entraînement aux armes").contains("groupe d'armes")
    );
    assert!(texte("moine", "réserve de ki").contains("moitié du niveau du moine"));
    assert!(texte("paladin", "grâce").contains("Secoué"));
    assert!(texte("rodeur", "environnement de prédilection").contains("+2"));
    assert!(texte("roublard", "coup de maître").contains("endormie"));
}

#[test]
fn grace_divine_bonus_a_la_ca_du_moine_et_deplacement_du_barbare() {
    let r = Referentiel::core();
    // Paladin niveau 2 : le bonus de Charisme s'ajoute à tous les jets de sauvegarde.
    let mut f = de_classe("paladin", 2);
    f.scores_base.insert(Carac::Cha, 10);
    let sans = f.calculer(&r).unwrap();
    f.scores_base.insert(Carac::Cha, 14);
    let avec = f.calculer(&r).unwrap();
    assert!(avec
        .vigueur
        .termes
        .iter()
        .any(|t| t.libelle.starts_with("Grâce divine") && t.valeur == 2));
    assert_eq!(avec.vigueur.total, sans.vigueur.total + 2);
    assert_eq!(avec.volonte.total, sans.volonte.total + 2);
    // Au niveau 1, pas de grâce divine.
    let mut f1 = de_classe("paladin", 1);
    f1.scores_base.insert(Carac::Cha, 14);
    assert!(!f1
        .calculer(&r)
        .unwrap()
        .vigueur
        .termes
        .iter()
        .any(|t| t.libelle == "Grâce divine"));

    // Moine niveau 4, Sagesse 14 (+2) : +2 de Sagesse et +1 de niveau à la CA et au DMD sans armure.
    let mut m = de_classe("moine", 4);
    m.scores_base.insert(Carac::Sag, 14);
    let v = m.calculer(&r).unwrap();
    assert!(v
        .ca
        .termes
        .iter()
        .any(|t| t.libelle.starts_with("Bonus à la CA (moine)") && t.valeur == 3));
    assert!(v
        .dmd
        .termes
        .iter()
        .any(|t| t.libelle.starts_with("Bonus à la CA (moine)") && t.valeur == 3));
    // Avec une armure, le bonus est perdu.
    m.equipement.armure = Some("Chainmail".into());
    let v = m.calculer(&r).unwrap();
    assert!(!v
        .ca
        .termes
        .iter()
        .any(|t| t.libelle.starts_with("Bonus à la CA (moine)")));

    // Barbare : +3 m (10 pieds) à la vitesse, perdus avec une armure lourde.
    let b = de_classe("barbare", 1);
    let v = b.calculer(&r).unwrap();
    assert_eq!(v.vitesse, 40);
    let mut lourd = de_classe("barbare", 1);
    lourd.equipement.armure = Some("Full Plate".into());
    let v = lourd.calculer(&r).unwrap();
    assert!(
        v.vitesse <= 30,
        "vitesse avec armure lourde : {}",
        v.vitesse
    );
    let mut inter = de_classe("barbare", 1);
    inter.equipement.armure = Some("Chainmail".into());
    // Armure intermédiaire : 40 − réduction d'armure = 30 pour une race de vitesse 30.
    assert_eq!(inter.calculer(&r).unwrap().vitesse, 30);
}

#[test]
fn entrainement_aux_armures_du_guerrier_et_valeurs_du_barbare_et_du_rodeur() {
    let r = Referentiel::core();
    // Cotte de mailles : sans entraînement, malus et plafond de DEX de l'armure ; au niveau 3, −1 et +1.
    let mut f1 = de_classe("guerrier", 2);
    f1.equipement.armure = Some("Chainmail".into());
    let v1 = f1.calculer(&r).unwrap();
    let mut f3 = de_classe("guerrier", 3);
    f3.equipement.armure = Some("Chainmail".into());
    let v3 = f3.calculer(&r).unwrap();
    assert_eq!(
        v3.equipement.malus_d_armure.0,
        (v1.equipement.malus_d_armure.0 + 1).min(0)
    );
    assert_eq!(
        v3.equipement.dex_max.as_ref().map(|d| d.0),
        v1.equipement.dex_max.as_ref().map(|d| d.0 + 1)
    );
    let l = crate::choix_classe::capacites_chiffrees(&f3);
    assert!(l
        .iter()
        .any(|x| x.starts_with("Entraînement aux armures : −1")));
    let l = crate::choix_classe::capacites_chiffrees(&de_classe("guerrier", 9));
    assert!(l
        .iter()
        .any(|x| x.starts_with("Entraînement aux armes : 2 groupe(s)")));
    // Barbare niveau 10 : sens des pièges +3, réduction de dégâts 2/— ; barbare 6 + roublard 3 se cumulent.
    let l = crate::choix_classe::capacites_chiffrees(&de_classe("barbare", 10));
    assert!(l.iter().any(|x| x.starts_with("Sens des pièges : +3")));
    assert!(l.contains(&"Réduction de dégâts : 2/—".to_string()));
    let l = crate::choix_classe::capacites_chiffrees(&de_classe("rodeur", 11));
    assert!(l.iter().any(|x| x.starts_with("Pistage : +5")));
    assert!(l
        .iter()
        .any(|x| x.starts_with("Proie du chasseur : bonus d'intuition de +2")));
}

#[test]
fn le_rodeur_choisit_ses_environnements_et_son_pacte_du_chasseur() {
    let r = Referentiel::core();
    let c = &r.choix;
    assert_eq!(
        [2, 3, 7, 8, 13, 18, 20].map(|n| c.nombre_environnements(n)),
        [0, 1, 1, 2, 3, 4, 4]
    );
    // Niveau 3 : un environnement, pas d'augmentation, pas encore de pacte.
    let mut f = de_classe("rodeur", 3);
    f.ennemis_jures = vec!["Animal".into()];
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::EnvironnementsAChoisir {
            attendus: 1,
            choisis: 0
        }));
    f.environnements = vec!["Jungle".into()];
    assert!(f.problemes_choix_de_classe(&r).is_empty());
    // Niveau 8 : deux environnements, une augmentation ; le pacte (niveau 4) est requis.
    let mut f = de_classe("rodeur", 8);
    f.ennemis_jures = vec!["Animal".into(), "Dragon".into()];
    f.augmentations_ennemis = vec!["Animal".into()];
    f.environnements = vec!["Jungle".into(), "Marais".into()];
    f.augmentations_environnements = vec!["Jungle".into()];
    let p = f.problemes_choix_de_classe(&r);
    assert_eq!(p, vec![Probleme::PacteDuChasseurManquant]);
    f.pacte_du_chasseur = Some("compagnon animal".into());
    assert!(f.problemes_choix_de_classe(&r).is_empty());
    f.pacte_du_chasseur = Some("autre".into());
    assert!(f
        .problemes_choix_de_classe(&r)
        .contains(&Probleme::PacteDuChasseurInconnu("autre".into())));
    let l = crate::choix_classe::capacites_chiffrees(&f);
    assert!(l
        .iter()
        .any(|x| x.starts_with("Environnement de prédilection (Jungle) : +4")));
    assert!(l
        .iter()
        .any(|x| x.starts_with("Environnement de prédilection (Marais) : +2")));
    // Le pacte ne se change pas une fois choisi.
    let mut j = Journal::nouveau(de_classe("rodeur", 4));
    let regles = ReglesFiche::default();
    assert!(j
        .ajouter(
            EvenementFiche::PacteDuChasseurChoisi("compagnon animal".into()),
            &r,
            &regles
        )
        .is_ok());
    assert!(j
        .ajouter(
            EvenementFiche::PacteDuChasseurChoisi("lien avec les compagnons de chasse".into()),
            &r,
            &regles
        )
        .is_err());
}
