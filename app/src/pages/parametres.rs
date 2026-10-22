//! Paramètres de la campagne : règles personnalisées et calendrier.

use crate::app::App;
use crate::{theme, widgets};
use campagne::calendrier::{DateGolarion, RegleBissextile, JOURS_SEMAINE, MOIS};
use domaine::Statut;
use eframe::egui::{self, RichText};

fn option(
    ui: &mut egui::Ui,
    valeur: &mut bool,
    titre: &str,
    aide: &str,
    statut_si_active: Statut,
) -> bool {
    let mut change = false;
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            change = ui.checkbox(valeur, RichText::new(titre).strong()).changed();
            let s = if *valeur {
                statut_si_active
            } else {
                oppose(statut_si_active)
            };
            widgets::badge_statut(ui, s);
        });
        widgets::texte_doux(ui, aide);
    });
    change
}

fn oppose(s: Statut) -> Statut {
    match s {
        Statut::Officiel => Statut::Personnalise,
        _ => Statut::Officiel,
    }
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    widgets::titre_page(ui, "Paramètres", &format!("Campagne « {} »", c.nom));
    let mut change = false;
    let mut par_pj = c.etat(&app.r).limite_par_pj;
    let par_affectation = c.etat(&app.r).salaire_par_affectation;
    let mut nouveau_par_affectation: Option<bool> = None;
    let sans_usure_avant = c.etat(&app.r).sans_usure_avec_gestionnaire;
    let mut nouveau_sans_usure: Option<bool> = None;
    let par_pj_avant = par_pj;

    widgets::titre_section(ui, "Règles");
    let o = &mut c.options;
    change |= option(
        ui,
        &mut o.repartition_stricte,
        "Répartition stricte des bonus",
        "Chaque salle ou équipe donne tout son bonus à un seul type de capital par jour. \
         Désactivé : règle officielle, le bonus peut être réparti entre les types admis (Guide de campagne p. 94).",
        Statut::Personnalise,
    );
    change |= option(
        ui,
        &mut o.moitie_equipe_sans_batiment,
        "Équipes sans bâtiment : recettes divisées par 2",
        "Règle officielle (Guide de campagne p. 94), arrondi inférieur (PC-12). \
         Désactivé : l'équipe garde son bonus entier.",
        Statut::Officiel,
    );
    change |= option(
        ui,
        &mut o.delocalisation_jet_unique,
        "Délocalisation d'une organisation entière avec un seul jet",
        "Un seul jet de 1d6 pour la préparation de toutes les équipes de l'organisation. \
         Désactivé : règle officielle, 1d6 jours par groupe d'employés (Guide de campagne p. 95).",
        Statut::Personnalise,
    );
    change |= option(
        ui,
        &mut o.construction_parallele_autorisee,
        "Construction parallèle des salles d'un même étage",
        "Le livre ne dit rien sur la durée de travaux simultanés : option personnalisée.",
        Statut::Personnalise,
    );
    change |= option(
        ui,
        &mut o.production_pendant_amelioration,
        "Production pendant l'amélioration d'une salle",
        "Une salle en travaux d'amélioration continue de produire ses recettes.",
        Statut::Personnalise,
    );
    let mut sans_usure = sans_usure_avant;
    if option(
        ui,
        &mut sans_usure,
        "Aucune usure dans une localité qui a un gestionnaire payé",
        "Désactivé : règle officielle, un gestionnaire fait passer l'usure des capitaux de 1 tous les 7 jours \
         à 1 tous les 14 jours et évite le test de prestige au retour (GC p. 82 et 89). \
         Activé : aucune usure tant qu'un gestionnaire payé s'occupe de la localité. S'applique à partir d'aujourd'hui.",
        Statut::Personnalise,
    ) {
        nouveau_sans_usure = Some(sans_usure);
    }
    let mut salaire_unique = !par_affectation;
    if option(
        ui,
        &mut salaire_unique,
        "Salaire d'un gestionnaire compté une fois par jour",
        "Un gestionnaire affecté à plusieurs établissements n'est payé qu'une fois (PC-06). \
         Désactivé : il est payé une fois par affectation. S'applique à partir d'aujourd'hui (annulable).",
        Statut::Personnalise,
    ) {
        nouveau_par_affectation = Some(!salaire_unique);
    }
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut par_pj,
                RichText::new("Limite des dépenses : une par PJ présent").strong(),
            );
            widgets::badge_statut(
                ui,
                if par_pj {
                    Statut::Officiel
                } else {
                    Statut::Personnalise
                },
            );
            widgets::badge(ui, "PC-14", theme::VERT);
        });
        widgets::texte_doux(
            ui,
            "Désactivé (par défaut) : une seule limite par localité et par jour, tous chantiers confondus. \
             Activé : lecture du livre (« ce que votre personnage peut dépenser », GC p. 81), chaque PJ présent \
             ajoute sa limite ; sans PJ présent, une limite reste disponible pour les chantiers en cours. \
             Le changement s'applique à partir d'aujourd'hui et peut être annulé (Ctrl+Z).",
        );
    });
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        change |= ui
            .checkbox(
                &mut o.afficher_avertissements,
                RichText::new("Afficher les avertissements").strong(),
            )
            .changed();
        widgets::texte_doux(ui, "Signale dans les calculs les données liées à un point à confirmer ou incohérentes (ex. Labyrinthe).");
    });

    ui.add_space(10.0);
    widgets::titre_section(ui, "Calendrier");
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        let cal = &mut c.calendrier;
        let mut d: DateGolarion = cal.origine;
        ui.horizontal(|ui| {
            ui.label("Le jour 1 de la campagne est le");
            egui::ComboBox::from_id_salt("jour_semaine")
                .selected_text(JOURS_SEMAINE[cal.jour_semaine_origine as usize % 7])
                .show_ui(ui, |ui| {
                    for (i, j) in JOURS_SEMAINE.iter().enumerate() {
                        change |= ui
                            .selectable_value(&mut cal.jour_semaine_origine, i as u8, *j)
                            .changed();
                    }
                });
            change |= ui
                .add(egui::DragValue::new(&mut d.jour).range(1..=31))
                .changed();
            egui::ComboBox::from_id_salt("mois")
                .selected_text(MOIS[(d.mois - 1) as usize].0)
                .show_ui(ui, |ui| {
                    for (i, (m, _)) in MOIS.iter().enumerate() {
                        change |= ui
                            .selectable_value(&mut d.mois, (i + 1) as u8, *m)
                            .changed();
                    }
                });
            change |= ui
                .add(egui::DragValue::new(&mut d.annee).range(1..=9999))
                .changed();
            ui.label("AR");
        });
        let max = cal.jours_du_mois(d.annee, d.mois);
        if d.jour > max {
            d.jour = max;
        }
        cal.origine = d;
        ui.horizontal(|ui| {
            ui.label("Années bissextiles");
            let texte = match cal.bissextile {
                RegleBissextile::Aucune => "aucune",
                RegleBissextile::TousLes4Ans => "tous les 4 ans (PF2)",
                RegleBissextile::TousLes8Ans => "tous les 8 ans (PF1)",
            };
            egui::ComboBox::from_id_salt("bissextile")
                .selected_text(texte)
                .show_ui(ui, |ui| {
                    change |= ui
                        .selectable_value(
                            &mut cal.bissextile,
                            RegleBissextile::TousLes4Ans,
                            "tous les 4 ans (PF2)",
                        )
                        .changed();
                    change |= ui
                        .selectable_value(
                            &mut cal.bissextile,
                            RegleBissextile::TousLes8Ans,
                            "tous les 8 ans (PF1)",
                        )
                        .changed();
                });
            widgets::badge(ui, "PC-13", theme::VERT);
        });
        widgets::texte_doux(
            ui,
            "Mois et jours de la semaine : calendrier de Golarion (VF). Aucune source ne fixe le jour de la semaine \
             d'une date donnée : choisissez celui du jour 1.",
        );
    });
    let apercu = c.date_courante(&app.r);
    widgets::texte_doux(ui, format!("Aujourd'hui : {apercu}"));

    if change {
        app.enregistrer();
    }
    if let Some(sans_usure_avec_gestionnaire) = nouveau_sans_usure {
        app.executer(
            campagne::evenements::Evenement::ReglageUsure {
                sans_usure_avec_gestionnaire,
            },
            if sans_usure_avec_gestionnaire {
                "Usure : aucune avec un gestionnaire payé"
            } else {
                "Usure : règle officielle"
            },
        );
    }
    if let Some(par_affectation) = nouveau_par_affectation {
        app.executer(
            campagne::evenements::Evenement::ReglageSalaireGestionnaire { par_affectation },
            if par_affectation {
                "Salaire des gestionnaires : une fois par affectation"
            } else {
                "Salaire des gestionnaires : une fois par jour"
            },
        );
    }
    if par_pj != par_pj_avant {
        let msg = if par_pj {
            "Limite des dépenses : une par PJ présent"
        } else {
            "Limite des dépenses : une par localité"
        };
        app.executer(
            campagne::evenements::Evenement::ReglageLimiteDepense { par_pj },
            msg,
        );
    }
}
