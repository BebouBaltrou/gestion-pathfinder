//! Essais de l'équipement, des états et des effets temporaires.

use crate::donnees::Referentiel;
use crate::equipement::Charge;
use crate::etats::EffetTemporaire;
use crate::fiche::tests::elfe_ensorceleuse;
use crate::journal::{EmplacementProtection, EvenementFiche, Journal};
use crate::progression::ReglesFiche;
use crate::{Carac, Effet, Probleme, TypeBonus};

fn equipee() -> crate::fiche::Fiche {
    // FOR 12 : charge légère jusqu'à 43 lb, intermédiaire jusqu'à 86 lb.
    let mut f = elfe_ensorceleuse();
    f.scores_base.insert(Carac::For, 12);
    f.equipement.armure = Some("Chainmail".into());
    f.equipement.bouclier = Some("Heavy steel".into());
    f
}

#[test]
fn le_catalogue_est_complet_et_traduit() {
    let r = Referentiel::core();
    let c = &r.equipement;
    assert_eq!(c.armures.len(), 21);
    assert_eq!(c.armes.len(), 87);
    assert_eq!(c.materiel.len(), 161);
    assert!(c
        .armures
        .iter()
        .all(|a| !a.nom.is_empty() && a.source.page.is_some()));
    assert!(c
        .armes
        .iter()
        .all(|a| !a.nom.is_empty() && a.source.page.is_some()));
    assert!(c
        .materiel
        .iter()
        .all(|a| !a.nom.is_empty() && a.source.page.is_some()));
    // Table 6-6 : cotte de mailles +6, DEX max +2, malus –5, échec 30 %.
    let m = c.armure("Chainmail").unwrap();
    assert_eq!(
        (m.bonus_armure, m.dex_max, m.malus_armure, m.echec_sorts_pct),
        (Some(6), Some(2), Some(-5), Some(30))
    );
    assert_eq!(
        c.arme("Longsword").unwrap().degats_moyens().as_deref(),
        Some("1d8")
    );
}

#[test]
fn la_table_de_charge_et_les_forces_elevees() {
    let c = &Referentiel::core().equipement.charge;
    assert_eq!(c.limites(10), Some((33.0, 66.0, 100.0)));
    assert_eq!(c.limites(29), Some((466.0, 933.0, 1400.0)));
    // FOR 30 : ligne 20 × 4.
    assert_eq!(c.limites(30), Some((532.0, 1064.0, 1600.0)));
    assert_eq!(c.charge(10, 33.0), Charge::Legere);
    assert_eq!(c.charge(10, 34.0), Charge::Intermediaire);
    assert_eq!(c.charge(10, 101.0), Charge::Surcharge);
    assert_eq!(c.vitesse_reduite(30), 20);
    assert_eq!(c.vitesse_reduite(20), 15);
    assert_eq!(c.vitesse_reduite(40), 30);
}

#[test]
fn armure_et_bouclier_changent_ca_competences_sorts_et_vitesse() {
    let r = Referentiel::core();
    let nu = elfe_ensorceleuse().calculer(&r).unwrap();
    assert_eq!(nu.ca.total, 13); // 10 + DEX +3
    let v = equipee().calculer(&r).unwrap();
    // DEX +3 plafonné à +2 par la cotte de mailles ; +6 armure, +2 bouclier.
    assert_eq!(v.ca.total, 20, "{}", v.ca.explication());
    assert_eq!(v.ca_contact.total, 12);
    assert_eq!(v.ca_pris_au_depourvu.total, 18);
    assert_eq!(v.equipement.echec_sorts_pct, 45);
    assert_eq!(v.vitesse, 20);
    // Malus d'armure –5 –2 sur Discrétion (DEX +3).
    assert_eq!(v.competences["stealth"].total, -4);
    // Ni armure ni bouclier maîtrisés : le malus d'armure s'applique aussi aux attaques.
    assert_eq!(v.equipement.non_maitrises.len(), 2);
    // FOR 12 (+1) : BBA 0 + 1 – 7.
    assert_eq!(
        v.attaque_melee.total,
        -6,
        "{}",
        v.attaque_melee.explication()
    );
}

#[test]
fn la_charge_et_l_armure_ne_se_cumulent_pas() {
    let r = Referentiel::core();
    let mut f = elfe_ensorceleuse();
    f.scores_base.insert(Carac::For, 12); // léger ≤ 43 lb, intermédiaire ≤ 86 lb
    f.equipement.ajouter_objet("Torch", 50);
    let v = f.calculer(&r).unwrap();
    assert_eq!(
        v.equipement.charge,
        Charge::Intermediaire,
        "{} lb",
        v.equipement.poids_total
    );
    // Charge intermédiaire : malus –3, vitesse réduite.
    assert_eq!(v.equipement.malus_d_armure.0, -3);
    assert_eq!(v.vitesse, 20);
    // Une chemise de mailles (malus –2) : on prend le pire des deux, –3, sans additionner.
    f.equipement.armure = Some("Chain shirt".into());
    let v = f.calculer(&r).unwrap();
    assert_eq!(v.equipement.charge, Charge::Intermediaire);
    assert_eq!(v.equipement.malus_d_armure.0, -3);
    // Charge ignorée par la campagne : seule l'armure compte (–2, vitesse inchangée).
    f.equipement.ignorer_charge = true;
    let v = f.calculer(&r).unwrap();
    assert_eq!(v.equipement.charge, Charge::Legere);
    assert_eq!(v.equipement.malus_d_armure.0, -2);
    assert_eq!(v.vitesse, 30);
}

#[test]
fn les_armes_et_leurs_maitrises() {
    let r = Referentiel::core();
    let mut f = elfe_ensorceleuse();
    f.equipement.armes = vec!["Longsword".into(), "Heavy mace".into(), "Dagger".into()];
    let v = f.calculer(&r).unwrap();
    let par = |n: &str| v.armes.iter().find(|a| a.arme_en == n).unwrap();
    // Les elfes maîtrisent l'épée longue ; la dague et la masse lourde sont simples.
    assert!(par("Longsword").maitrise);
    assert!(par("Dagger").maitrise);
    assert!(par("Heavy mace").maitrise);
    assert_eq!(par("Longsword").attaque.total, -1); // BBA 0 + FOR –1
    assert_eq!(par("Longsword").des.as_deref(), Some("1d8"));
    assert_eq!(par("Longsword").bonus_degats.total, -1);
    // Une arme martiale non maîtrisée : –4.
    f.equipement.armes = vec!["Battleaxe".into()];
    let v = f.calculer(&r).unwrap();
    assert!(!v.armes[0].maitrise);
    assert_eq!(v.armes[0].attaque.total, -5);
    // Le don Martial Weapon Proficiency (battleaxe) la rend maîtrisée.
    f.niveaux[0]
        .dons
        .push("Martial Weapon Proficiency (battleaxe)".into());
    assert!(f.calculer(&r).unwrap().armes[0].maitrise);
}

#[test]
fn les_etats_modifient_les_valeurs() {
    let r = Referentiel::core();
    let base = elfe_ensorceleuse().calculer(&r).unwrap();
    let mut f = elfe_ensorceleuse();
    f.etats = vec!["shaken".into()];
    let v = f.calculer(&r).unwrap();
    assert_eq!(v.vigueur.total, base.vigueur.total - 2);
    assert_eq!(v.volonte.total, base.volonte.total - 2);
    assert_eq!(v.attaque_melee.total, base.attaque_melee.total - 2);
    assert_eq!(
        v.competences["bluff"].total,
        base.competences["bluff"].total - 2
    );
    // Fatigué : DEX –2 (16 → 14, modificateur +2).
    f.etats = vec!["fatigued".into()];
    let v = f.calculer(&r).unwrap();
    assert_eq!(v.modificateurs[&Carac::Dex], 2);
    // Épuisé : vitesse divisée par deux, arrondie à 5.
    f.etats = vec!["exhausted".into()];
    assert_eq!(f.calculer(&r).unwrap().vitesse, 15);
    // Pris au dépourvu : plus de bonus de DEX à la CA.
    f.etats = vec!["flat_footed".into()];
    assert_eq!(f.calculer(&r).unwrap().ca.total, 10);
    // Aveuglé : –2 à la CA et plus de DEX ; une note signale ce qui n'est pas calculé.
    f.etats = vec!["blinded".into()];
    let v = f.calculer(&r).unwrap();
    assert_eq!(v.ca.total, 8);
    assert_eq!(v.notes_etats.len(), 1);
    f.etats = vec!["n_importe_quoi".into()];
    assert!(f
        .problemes_equipement(&r)
        .contains(&Probleme::EtatInconnu("n_importe_quoi".into())));
}

#[test]
fn le_journal_gere_equipement_etats_et_duree() {
    let r = Referentiel::core();
    let regles = ReglesFiche::default();
    let mut j = Journal::nouveau(elfe_ensorceleuse());
    let ajouter = |j: &mut Journal, e| j.ajouter(e, &r, &regles);
    ajouter(
        &mut j,
        EvenementFiche::Equiper {
            emplacement: EmplacementProtection::Armure,
            objet: Some("Chainmail".into()),
        },
    )
    .unwrap();
    // Un bouclier n'est pas une armure.
    assert!(ajouter(
        &mut j,
        EvenementFiche::Equiper {
            emplacement: EmplacementProtection::Armure,
            objet: Some("Tower".into()),
        },
    )
    .is_err());
    // Objet inconnu refusé ; retrait de ce qu'on n'a pas refusé.
    assert!(ajouter(
        &mut j,
        EvenementFiche::ObjetAjoute {
            nom_en: "Épée vorpale".into(),
            quantite: 1
        }
    )
    .is_err());
    assert!(ajouter(
        &mut j,
        EvenementFiche::ObjetRetire {
            nom_en: "Torch".into(),
            quantite: 1
        }
    )
    .is_err());
    ajouter(
        &mut j,
        EvenementFiche::ObjetAjoute {
            nom_en: "Torch".into(),
            quantite: 2,
        },
    )
    .unwrap();
    ajouter(&mut j, EvenementFiche::EtatAjoute("shaken".into())).unwrap();
    // Force +4 (altération) pendant 3 rounds.
    let force = EffetTemporaire {
        nom: "Force de taureau".into(),
        effets: vec![Effet::new(
            "carac.for",
            TypeBonus::Alteration,
            4,
            "Force de taureau",
        )],
        rounds_restants: Some(3),
    };
    ajouter(&mut j, EvenementFiche::EffetTemporaireAjoute(force)).unwrap();
    let e = j.etat(&r, &regles).unwrap();
    assert_eq!(e.fiche.calculer(&r).unwrap().modificateurs[&Carac::For], 1);
    ajouter(&mut j, EvenementFiche::TempsEcoule { rounds: 2 }).unwrap();
    assert_eq!(j.etat(&r, &regles).unwrap().fiche.temporaires.len(), 1);
    ajouter(&mut j, EvenementFiche::TempsEcoule { rounds: 1 }).unwrap();
    let e = j.etat(&r, &regles).unwrap();
    assert!(e.fiche.temporaires.is_empty());
    assert_eq!(e.fiche.calculer(&r).unwrap().modificateurs[&Carac::For], -1);
    // Annuler le dernier événement rend l'effet (1 round restant).
    j.annuler_dernier();
    assert_eq!(
        j.etat(&r, &regles).unwrap().fiche.temporaires[0].rounds_restants,
        Some(1)
    );
    assert!(j.historique().iter().any(|l| l.contains("Armure portée")));
}
