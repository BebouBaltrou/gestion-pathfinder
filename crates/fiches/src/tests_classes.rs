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
