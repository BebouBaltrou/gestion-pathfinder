//! Page Paramètres : règles personnalisées et calendrier de la campagne.

use campagne::calendrier::{DateGolarion, RegleBissextile, JOURS_SEMAINE, MOIS};
use campagne::evenements::Evenement;
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageParametres {
    pub campagne: String,
    pub reglages: Vec<Reglage>,
    pub calendrier: VueCalendrier,
}

/// Un réglage à cocher.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Reglage {
    pub id: String,
    pub titre: String,
    pub aide: String,
    pub actif: bool,
    /// « officiel » ou « personnalise » : statut de la règle telle qu'elle s'applique
    /// maintenant (`None` : réglage sans statut).
    pub statut: Option<String>,
    /// Point à confirmer lié (« PC-14 »).
    pub point: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct VueCalendrier {
    /// Jour de la semaine du jour 1 (0 = lunedi).
    pub jour_semaine: u8,
    pub jour: u8,
    pub mois: u8,
    pub annee: i32,
    /// « quatre » (PF2) ou « huit » (PF1).
    pub bissextile: String,
    pub jours_semaine: Vec<String>,
    pub noms_mois: Vec<String>,
    /// « Aujourd'hui : Jour 12 — … »
    pub apercu: String,
}

fn statut(actif_officiel: bool, actif: bool) -> Option<String> {
    // une règle « officielle quand cochée » devient personnalisée quand elle est décochée, et inversement
    let officiel = actif == actif_officiel;
    Some(if officiel { "officiel" } else { "personnalise" }.into())
}

pub fn page_parametres(c: &mut campagne::Campagne, r: &Referentiel) -> PageParametres {
    let (par_pj, par_affectation, sans_usure) = {
        let e = c.etat(r);
        (
            e.limite_par_pj,
            e.salaire_par_affectation,
            e.sans_usure_avec_gestionnaire,
        )
    };
    let o = c.options.clone();
    let reglage = |id: &str,
                   titre: &str,
                   aide: &str,
                   actif: bool,
                   st: Option<String>,
                   point: Option<&str>| {
        Reglage {
            id: id.into(),
            titre: titre.into(),
            aide: aide.into(),
            actif,
            statut: st,
            point: point.map(String::from),
        }
    };
    let reglages = vec![
        reglage(
            "repartition_stricte",
            "Répartition stricte des bonus",
            "Chaque salle ou équipe donne tout son bonus à un seul type de capital par jour. \
             Désactivé : règle officielle, le bonus peut être réparti entre les types admis (Guide de campagne p. 94).",
            o.repartition_stricte,
            statut(false, o.repartition_stricte),
            None,
        ),
        reglage(
            "moitie_equipe_sans_batiment",
            "Équipes sans bâtiment : recettes divisées par 2",
            "Règle officielle (Guide de campagne p. 94), arrondi inférieur (PC-12). \
             Désactivé : l'équipe garde son bonus entier.",
            o.moitie_equipe_sans_batiment,
            statut(true, o.moitie_equipe_sans_batiment),
            None,
        ),
        reglage(
            "capitaux_produits_payants",
            "Faire payer le Coût produit des capitaux des bâtiments et organisations",
            "Activé : règle du livre, chaque Marchandise, Influence, Travail ou Magie obtenue aux tests de \
             capitaux se paie au Coût produit (Table 2-1 ; GC p. 94 « Le personnage doit tout de même payer les \
             coûts occasionnés par ce type d'acquisition de capitaux »). Désactivé (par défaut) : encaissés \
             gratuitement. Le travail du PJ lui-même restera payant (à venir avec les fiches de personnage).",
            o.capitaux_produits_payants,
            statut(true, o.capitaux_produits_payants),
            None,
        ),
        reglage(
            "delocalisation_jet_unique",
            "Délocalisation d'une organisation entière avec un seul jet",
            "Un seul jet de 1d6 pour la préparation de toutes les équipes de l'organisation. \
             Désactivé : règle officielle, 1d6 jours par groupe d'employés (Guide de campagne p. 95).",
            o.delocalisation_jet_unique,
            statut(false, o.delocalisation_jet_unique),
            None,
        ),
        reglage(
            "construction_parallele_autorisee",
            "Construction parallèle des salles d'un même étage",
            "Le livre ne dit rien sur la durée de travaux simultanés : option personnalisée.",
            o.construction_parallele_autorisee,
            statut(false, o.construction_parallele_autorisee),
            None,
        ),
        reglage(
            "production_pendant_amelioration",
            "Production pendant l'amélioration d'une salle",
            "Une salle en travaux d'amélioration continue de produire ses recettes.",
            o.production_pendant_amelioration,
            statut(false, o.production_pendant_amelioration),
            None,
        ),
        reglage(
            "sans_usure",
            "Aucune usure dans une localité qui a un gestionnaire payé",
            "Désactivé : règle officielle, un gestionnaire fait passer l'usure des capitaux de 1 tous les 7 jours \
             à 1 tous les 14 jours et évite le test de prestige au retour (GC p. 82 et 89). \
             Activé : aucune usure tant qu'un gestionnaire payé s'occupe de la localité. S'applique à partir d'aujourd'hui.",
            sans_usure,
            statut(false, sans_usure),
            None,
        ),
        reglage(
            "salaire_unique",
            "Salaire d'un gestionnaire compté une fois par jour",
            "Un gestionnaire affecté à plusieurs établissements n'est payé qu'une fois (PC-06). \
             Désactivé : il est payé une fois par affectation. S'applique à partir d'aujourd'hui (annulable).",
            !par_affectation,
            statut(false, !par_affectation),
            None,
        ),
        reglage(
            "limite_par_pj",
            "Limite des dépenses : une par PJ présent",
            "Désactivé (par défaut) : une seule limite par localité et par jour, tous chantiers confondus. \
             Activé : lecture du livre (« ce que votre personnage peut dépenser », GC p. 81), chaque PJ présent \
             ajoute sa limite ; sans PJ présent, une limite reste disponible pour les chantiers en cours. \
             Le changement s'applique à partir d'aujourd'hui et peut être annulé (Ctrl+Z).",
            par_pj,
            statut(true, par_pj),
            Some("PC-14"),
        ),
        reglage(
            "afficher_avertissements",
            "Afficher les avertissements",
            "Signale dans les calculs les données liées à un point à confirmer ou incohérentes (ex. Labyrinthe).",
            o.afficher_avertissements,
            None,
            None,
        ),
    ];
    let cal = &c.calendrier;
    let calendrier = VueCalendrier {
        jour_semaine: cal.jour_semaine_origine % 7,
        jour: cal.origine.jour,
        mois: cal.origine.mois,
        annee: cal.origine.annee,
        bissextile: match cal.bissextile {
            RegleBissextile::TousLes8Ans => "huit",
            _ => "quatre",
        }
        .into(),
        jours_semaine: JOURS_SEMAINE.iter().map(|j| j.to_string()).collect(),
        noms_mois: MOIS.iter().map(|(m, _)| m.to_string()).collect(),
        apercu: String::new(),
    };
    let apercu = format!("Aujourd'hui : {}", c.date_courante(r));
    PageParametres {
        campagne: c.nom.clone(),
        reglages,
        calendrier: VueCalendrier {
            apercu,
            ..calendrier
        },
    }
}

/// Conséquence d'un changement de réglage.
#[derive(Debug, Clone, PartialEq)]
pub enum ResultatReglage {
    /// Réglage enregistré dans le journal (annulable).
    Evenement(Box<Evenement>, String),
    /// Option modifiée directement dans la campagne : à enregistrer.
    Enregistre(String),
}

/// Applique un réglage coché ou décoché.
pub fn regler(
    c: &mut campagne::Campagne,
    id: &str,
    actif: bool,
) -> Result<ResultatReglage, String> {
    let o = &mut c.options;
    let message = |texte: &str| Ok(ResultatReglage::Enregistre(texte.into()));
    match id {
        "repartition_stricte" => o.repartition_stricte = actif,
        "moitie_equipe_sans_batiment" => o.moitie_equipe_sans_batiment = actif,
        "capitaux_produits_payants" => o.capitaux_produits_payants = actif,
        "delocalisation_jet_unique" => o.delocalisation_jet_unique = actif,
        "construction_parallele_autorisee" => o.construction_parallele_autorisee = actif,
        "production_pendant_amelioration" => o.production_pendant_amelioration = actif,
        "afficher_avertissements" => o.afficher_avertissements = actif,
        "sans_usure" => {
            return Ok(ResultatReglage::Evenement(
                Box::new(Evenement::ReglageUsure {
                    sans_usure_avec_gestionnaire: actif,
                }),
                if actif {
                    "Usure : aucune avec un gestionnaire payé"
                } else {
                    "Usure : règle officielle"
                }
                .into(),
            ))
        }
        "salaire_unique" => {
            return Ok(ResultatReglage::Evenement(
                Box::new(Evenement::ReglageSalaireGestionnaire {
                    par_affectation: !actif,
                }),
                if actif {
                    "Salaire des gestionnaires : une fois par jour"
                } else {
                    "Salaire des gestionnaires : une fois par affectation"
                }
                .into(),
            ))
        }
        "limite_par_pj" => {
            return Ok(ResultatReglage::Evenement(
                Box::new(Evenement::ReglageLimiteDepense { par_pj: actif }),
                if actif {
                    "Limite des dépenses : une par PJ présent"
                } else {
                    "Limite des dépenses : une par localité"
                }
                .into(),
            ))
        }
        _ => return Err("Réglage inconnu".into()),
    }
    message("Paramètre enregistré")
}

/// Calendrier : date du jour 1, jour de la semaine du jour 1, années bissextiles.
pub fn regler_calendrier(
    c: &mut campagne::Campagne,
    jour_semaine: u8,
    jour: u8,
    mois: u8,
    annee: i32,
    bissextile: &str,
) -> Result<(), String> {
    if jour_semaine > 6 {
        return Err("Jour de la semaine inconnu".into());
    }
    if !(1..=12).contains(&mois) {
        return Err("Mois inconnu".into());
    }
    if !(1..=9999).contains(&annee) {
        return Err("L'année doit être comprise entre 1 et 9999".into());
    }
    c.calendrier.bissextile = match bissextile {
        "huit" => RegleBissextile::TousLes8Ans,
        "quatre" => RegleBissextile::TousLes4Ans,
        _ => return Err("Règle d'années bissextiles inconnue".into()),
    };
    c.calendrier.jour_semaine_origine = jour_semaine;
    let max = c.calendrier.jours_du_mois(annee, mois);
    c.calendrier.origine = DateGolarion {
        annee,
        mois,
        jour: jour.clamp(1, max),
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use campagne::Campagne;

    #[test]
    fn reglages_et_calendrier() {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let p = page_parametres(&mut c, &r);
        assert_eq!(p.reglages.len(), 10);
        assert_eq!(p.campagne, "Test");
        // une option simple est modifiée directement
        let avant = c.options.production_pendant_amelioration;
        assert!(matches!(
            regler(&mut c, "production_pendant_amelioration", !avant),
            Ok(ResultatReglage::Enregistre(_))
        ));
        assert_eq!(c.options.production_pendant_amelioration, !avant);
        // un réglage journalisé est un événement, qui change l'état une fois exécuté
        let Ok(ResultatReglage::Evenement(ev, _)) = regler(&mut c, "limite_par_pj", true) else {
            panic!("événement attendu");
        };
        c.executer(*ev, &r).unwrap();
        let p = page_parametres(&mut c, &r);
        let limite = p.reglages.iter().find(|x| x.id == "limite_par_pj").unwrap();
        assert!(limite.actif);
        assert_eq!(limite.point.as_deref(), Some("PC-14"));
        assert!(regler(&mut c, "inconnu", true).is_err());
        // calendrier : le jour est ramené au dernier jour du mois
        regler_calendrier(&mut c, 2, 31, 2, 4712, "huit").unwrap();
        let cal = page_parametres(&mut c, &r).calendrier;
        assert_eq!(
            (cal.jour_semaine, cal.mois, cal.bissextile.as_str()),
            (2, 2, "huit")
        );
        assert!(cal.jour <= 29);
        assert!(regler_calendrier(&mut c, 7, 1, 1, 4712, "huit").is_err());
        assert!(regler_calendrier(&mut c, 0, 1, 13, 4712, "huit").is_err());
        assert!(regler_calendrier(&mut c, 0, 1, 1, 0, "huit").is_err());
        assert!(regler_calendrier(&mut c, 0, 1, 1, 4712, "x").is_err());
    }
}
