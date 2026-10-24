//! Le mode aventure dans la campagne : démarrage, événements, répercussion sur les fiches et les
//! jours, combat validé, annuler/rétablir, format de fichier.

use crate::evenements::Evenement;
use crate::modele::IdPersonnage;
use crate::Campagne;
use aventure::{EvenementAventure, Sante};
use domaine::Montant;
use fiches::journal::EvenementFiche;
use fiches::{Effet, TypeBonus};
use temps::{Duree, Pas};

fn campagne() -> (Campagne, regles::Referentiel, IdPersonnage) {
    let r = regles::referentiel_officiel();
    let mut c = Campagne::nouvelle("Test");
    let pj = c.nouvel_id_personnage();
    c.executer(
        Evenement::PersonnageCree {
            id: pj,
            nom: "Lyra".into(),
            compte_initial: Montant::po(100),
            prestige: None,
        },
        &r,
    )
    .unwrap();
    c.executer(
        Evenement::Fiche {
            pj,
            evenement: EvenementFiche::Creation(Box::new(fiches::exemple::elfe_ensorceleuse())),
        },
        &r,
    )
    .unwrap();
    (c, r, pj)
}

fn demarrer(c: &mut Campagne, r: &regles::Referentiel, heure: u32) {
    c.executer(
        Evenement::AventureDemarree {
            heure,
            minute: 0,
            survie: false,
            degats_massifs: false,
        },
        r,
    )
    .unwrap();
}

fn av(c: &mut Campagne, r: &regles::Referentiel, ev: EvenementAventure) {
    c.executer(Evenement::Aventure(ev), r).unwrap();
}

#[test]
fn contexte_de_l_ensorceleuse() {
    let (mut c, r, pj) = campagne();
    let ctx = c.etat(&r).contexte_aventure(pj).unwrap();
    assert_eq!(ctx.nom, "Lyra");
    assert_eq!(ctx.niveau, 1);
    assert!(ctx.pv_max > 0);
    assert_eq!(ctx.con_score, 10); // Con 12 − 2 (elfe)
    let inc = ctx.incantation.unwrap();
    assert_eq!(inc.classe_en, "sorcerer");
    assert!(inc.arcanique && inc.spontane);
    assert_eq!(inc.mod_incantation, 3); // Charisme 16
    assert!(inc.sorts_connus.contains(&"Mage Armor".to_string()));
    let niveau1 = inc
        .emplacements
        .iter()
        .find(|e| e.niveau_de_sort == 1)
        .unwrap();
    assert!(niveau1.restants >= 1 && niveau1.dd == 14); // 10 + 1 + 3
    assert!(!inc.metamagie_active);
}

#[test]
fn demarrer_et_suivre_le_temps() {
    let (mut c, r, pj) = campagne();
    // rien ne marche avant le démarrage
    assert!(c
        .executer(
            Evenement::Aventure(EvenementAventure::Entree { pj: pj.0 }),
            &r
        )
        .is_err());
    demarrer(&mut c, &r, 22);
    assert!(c
        .executer(
            Evenement::AventureDemarree {
                heure: 8,
                minute: 0,
                survie: false,
                degats_massifs: false
            },
            &r
        )
        .is_err());
    av(&mut c, &r, EvenementAventure::Entree { pj: pj.0 });
    // 22:00 + 1 h = 23:00, puis 1 h de plus : minuit franchi, un jour d'intermède passe
    av(&mut c, &r, EvenementAventure::Pas { pas: Pas::Heure1 });
    assert_eq!(c.etat(&r).jour, 1);
    av(&mut c, &r, EvenementAventure::Pas { pas: Pas::Heure1 });
    assert_eq!(c.etat(&r).jour, 2);
    assert_eq!(
        c.etat(&r)
            .aventure
            .as_ref()
            .unwrap()
            .horloge
            .maintenant
            .jour(),
        2
    );
    // annuler remet le jour et l'horloge
    c.annuler();
    assert_eq!(c.etat(&r).jour, 1);
    c.retablir();
    assert_eq!(c.etat(&r).jour, 2);
}

#[test]
fn un_jour_d_intermede_fait_avancer_l_horloge_d_aventure() {
    let (mut c, r, pj) = campagne();
    demarrer(&mut c, &r, 8);
    av(&mut c, &r, EvenementAventure::Entree { pj: pj.0 });
    av(
        &mut c,
        &r,
        EvenementAventure::EffetAjoute {
            pj: pj.0,
            nom: "Bénédiction".into(),
            source: "sort".into(),
            duree: Duree::Heures(2),
            effets: vec![Effet::new("attaque", TypeBonus::Moral, 1, "Bénédiction")],
            etats: vec![],
            soin_par_round: 0,
            protections: vec![],
        },
    );
    c.executer(Evenement::JoursAvances { nombre: 1 }, &r)
        .unwrap();
    let e = c.etat(&r);
    assert_eq!(e.jour, 2);
    let a = e.aventure.as_ref().unwrap();
    assert_eq!(a.horloge.maintenant.jour(), 2);
    assert!(a.effets.actifs.is_empty()); // l'effet de 2 h a expiré
                                         // la fiche n'a plus l'effet
    assert!(e.fiche_de(pj).unwrap().fiche.temporaires.is_empty());
}

#[test]
fn effets_etats_et_sante_arrivent_sur_la_fiche() {
    let (mut c, r, pj) = campagne();
    demarrer(&mut c, &r, 8);
    av(&mut c, &r, EvenementAventure::Entree { pj: pj.0 });
    let pv_max = c.etat(&r).contexte_aventure(pj).unwrap().pv_max;
    // un effet de +4 à la CA, valeur typée lue par la fiche
    av(
        &mut c,
        &r,
        EvenementAventure::EffetAjoute {
            pj: pj.0,
            nom: "Armure du mage".into(),
            source: "sort".into(),
            duree: Duree::Heures(1),
            effets: vec![Effet::new("ca", TypeBonus::Armure, 4, "Armure du mage")],
            etats: vec![],
            soin_par_round: 0,
            protections: vec![],
        },
    );
    let ca_avant = {
        let v = c
            .etat(&r)
            .fiche_de(pj)
            .unwrap()
            .fiche
            .calculer(fiches::Referentiel::partage())
            .unwrap();
        v.ca.total
    };
    assert!(ca_avant >= 14); // 10 + Dex +2 + armure +4 (au moins)
                             // tomber à 0 PV : états sur la fiche, puis dans le journal
    av(
        &mut c,
        &r,
        EvenementAventure::Degats {
            pj: pj.0,
            nombre: pv_max,
            letaux: true,
        },
    );
    let e = c.etat(&r);
    let fiche = e.fiche_de(pj).unwrap().fiche;
    assert!(fiche.etats.contains(&"disabled".to_string()));
    let a = e.aventure.as_ref().unwrap();
    let ctx = e.contexte_aventure(pj).unwrap();
    assert_eq!(
        a.pjs[&pj.0].vital.sante(ctx.pv_max, ctx.con_score),
        Sante::HorsDeCombat
    );
    // le soin ramène la fiche à la normale
    av(
        &mut c,
        &r,
        EvenementAventure::Soin {
            pj: pj.0,
            nombre: 10,
        },
    );
    assert!(!c
        .etat(&r)
        .fiche_de(pj)
        .unwrap()
        .fiche
        .etats
        .contains(&"disabled".to_string()));
}

#[test]
fn le_repos_rend_les_emplacements_de_sorts() {
    let (mut c, r, pj) = campagne();
    demarrer(&mut c, &r, 20);
    av(&mut c, &r, EvenementAventure::Entree { pj: pj.0 });
    c.executer(
        Evenement::Fiche {
            pj,
            evenement: EvenementFiche::SortLance {
                classe: "ensorceleur".into(),
                niveau_de_sort: 1,
                sort: "Magic Missile".into(),
            },
        },
        &r,
    )
    .unwrap();
    let restants = |c: &mut Campagne| {
        c.etat(&r)
            .contexte_aventure(pj)
            .unwrap()
            .incantation
            .unwrap()
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == 1)
            .unwrap()
            .restants
    };
    let avant = restants(&mut c);
    av(
        &mut c,
        &r,
        EvenementAventure::Repos {
            pjs: vec![pj.0],
            heures: 8,
            alitement_complet: false,
            interrompu: false,
        },
    );
    assert_eq!(restants(&mut c), avant + 1);
    // 20:00 + 8 h : minuit franchi pendant le repos
    assert_eq!(c.etat(&r).jour, 2);
}

#[cfg(feature = "serde")]
#[test]
fn format_du_fichier() {
    let (mut c, r, pj) = campagne();
    let json = crate::fichier::vers_json(&c).unwrap();
    assert!(json.contains("\"format\": 1")); // sans aventure : l'ancienne application lit encore
    demarrer(&mut c, &r, 8);
    av(&mut c, &r, EvenementAventure::Entree { pj: pj.0 });
    let json = crate::fichier::vers_json(&c).unwrap();
    assert!(json.contains("\"format\": 2"));
    let mut relue = crate::fichier::depuis_json(&json).unwrap();
    assert!(relue.etat(&r).aventure.is_some());
    assert_eq!(relue.journal.len(), c.journal.len());
}

#[test]
fn un_combat_valide_entre_dans_le_journal() {
    use combat::{ActionCombat, Combat, ParamsLancer, TypeAction, Validation};
    let (mut c, r, pj) = campagne();
    demarrer(&mut c, &r, 8);
    av(&mut c, &r, EvenementAventure::Entree { pj: pj.0 });
    let jour_avant = c.etat(&r).jour;
    let (base, contextes) = {
        let e = c.etat(&r);
        (e.aventure.clone().unwrap(), e.contextes_aventure())
    };
    let ctx = |id: aventure::IdPj| contextes.get(&id).cloned();
    // le combat se joue sur une copie, hors du journal
    let mut combat = Combat::nouveau(&base, vec![pj.0], &ctx).unwrap();
    combat
        .appliquer(ActionCombat::Initiative { pj: pj.0, d20: 12 }, &ctx)
        .unwrap();
    combat
        .appliquer(ActionCombat::Demarrer { surprise: None }, &ctx)
        .unwrap();
    combat
        .appliquer(
            ActionCombat::Lancer(ParamsLancer {
                pj: pj.0,
                sort: "Mage Armor".into(),
                cibles: vec![pj.0],
                ..Default::default()
            }),
            &ctx,
        )
        .unwrap();
    combat
        .appliquer(
            ActionCombat::Aventure(EvenementAventure::Degats {
                pj: pj.0,
                nombre: 3,
                letaux: true,
            }),
            &ctx,
        )
        .unwrap();
    combat.appliquer(ActionCombat::TourSuivant, &ctx).unwrap();
    let rounds = combat.etat().rounds_joues;
    assert_eq!(rounds, 1);
    assert_eq!(
        c.journal.len(),
        4,
        "le combat n'a rien écrit dans le journal"
    );
    // validation : un seul événement dans le journal
    let res = combat.valider(&Validation::Tout, &[], &ctx).unwrap();
    c.executer(
        Evenement::CombatValide {
            aventure: Box::new(res.aventure),
            fiche: res.sortie.fiche,
            minuits: res.sortie.minuits,
            resume: format!("{rounds} round"),
        },
        &r,
    )
    .unwrap();
    assert_eq!(c.journal.len(), 5);
    let e = c.etat(&r);
    let a = e.aventure.as_ref().unwrap();
    assert!(!a.horloge.en_combat());
    assert_eq!(a.pjs[&pj.0].vital.degats, 3);
    assert_eq!(a.effets.actifs.len(), 1); // Armure du mage, 1 h/niveau
    assert_eq!(
        a.horloge.maintenant,
        temps::Instant::depuis(jour_avant as i64, 8, 0)
            .unwrap()
            .plus(1)
    );
    // la fiche a dépensé l'emplacement et porte l'effet
    let fiche = e.fiche_de(pj).unwrap();
    assert_eq!(fiche.fiche.temporaires.len(), 1);
    assert_eq!(
        *fiche
            .sorts_depenses
            .get(&("ensorceleur".to_string(), 1))
            .unwrap(),
        1
    );
    // annuler le combat valide défait tout d'un coup
    c.annuler();
    let e = c.etat(&r);
    assert!(e.fiche_de(pj).unwrap().fiche.temporaires.is_empty());
    assert!(e.fiche_de(pj).unwrap().sorts_depenses.is_empty());
    assert_eq!(e.aventure.as_ref().unwrap().pjs[&pj.0].vital.degats, 0);
    let _ = TypeAction::Simple;
}
