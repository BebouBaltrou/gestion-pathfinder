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
        divinite: None,
        alignement: None,
        domaines: vec![],
        energie: None,
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
