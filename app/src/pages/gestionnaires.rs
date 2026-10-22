//! Gestionnaires (Guide de campagne p. 89-91) : embauche, salaire quotidien, affectations.

use crate::app::App;
use crate::{theme, widgets};
use campagne::etat::Etat as EtatCampagne;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::Montant;
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct Etat {
    reference: Option<String>,
    nom: String,
    salaire: String,
    localite: Option<IdLocalite>,
    ajout_affectation: Option<(IdGestionnaire, Option<Affectation>)>,
}

fn libelle_affectation(a: &Affectation, etat: &EtatCampagne) -> String {
    match a {
        Affectation::Batiment(b) => etat
            .batiments
            .get(b)
            .map(|x| format!("🏛 {}", x.nom))
            .unwrap_or(b.to_string()),
        Affectation::Organisation(o) => etat
            .organisations
            .get(o)
            .map(|x| format!("⚔ {}", x.nom))
            .unwrap_or(o.to_string()),
    }
}

fn modifie(g: &GestionnaireEmploye) -> Evenement {
    Evenement::GestionnaireModifie {
        id: g.id,
        nom: g.nom.clone(),
        salaire: g.salaire,
        localite: g.localite,
        affectations: g.affectations.clone(),
        actif: g.actif,
    }
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    let etat = c.etat(&app.r).clone();
    let nouvel_id = c.nouvel_id_gestionnaire();
    let r = app.r.clone();
    widgets::titre_page(
        ui,
        "Gestionnaires",
        "Employés qualifiés qui gèrent bâtiments et organisations, payés chaque jour en po (Guide de campagne p. 89)",
    );
    let mut action: Option<(Evenement, String)> = None;
    let s = &mut app.ui_gestionnaires;

    // ---------- salaires par localité ----------
    let mut payeuses: Vec<_> = etat
        .localites
        .values()
        .filter(|l| etat.salaires_localite(l.id).pa > 0)
        .collect();
    payeuses.sort_by_key(|l| l.nom.clone());
    if !payeuses.is_empty() {
        widgets::titre_section(ui, "Salaires quotidiens");
        ui.horizontal_wrapped(|ui| {
            for l in payeuses {
                let jour = etat.salaires_localite(l.id);
                widgets::carte(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&l.nom).color(theme::OR_PALE).strong());
                        ui.horizontal(|ui| {
                            ui.label("Salaires :");
                            widgets::montant(ui, jour, 15.0);
                            ui.label("/ jour");
                        });
                        ui.horizontal(|ui| {
                            ui.label("Trésorerie :");
                            widgets::montant(ui, l.tresorerie, 15.0);
                        });
                        if l.tresorerie.est_negatif() {
                            ui.label(RichText::new("⚠ trésorerie négative").color(theme::ROUGE));
                        } else {
                            widgets::texte_doux(
                                ui,
                                format!(
                                    "≈ {} jours de salaires d'avance",
                                    l.tresorerie.pa / jour.pa
                                ),
                            );
                        }
                    });
                });
            }
        });
        ui.add_space(6.0);
    }

    // ---------- embauche ----------
    widgets::titre_section(ui, "Embaucher");
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        if etat.localites.is_empty() {
            widgets::texte_doux(ui, "Créez d'abord une localité.");
            return;
        }
        if s.localite.is_none() {
            s.localite = etat.localites.keys().next().copied();
        }
        egui::Grid::new("embauche")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Exemple du livre");
                let avant = s.reference.clone();
                egui::ComboBox::from_id_salt("ges_ref")
                    .selected_text(
                        s.reference
                            .as_ref()
                            .and_then(|x| r.gestionnaire(x))
                            .map(|g| format!("{} — {} po/jour", g.nom, g.salaire_po_jour))
                            .unwrap_or("aucun (personnalisé)".into()),
                    )
                    .width(320.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut s.reference, None, "aucun (personnalisé)");
                        for g in &r.gestionnaires {
                            ui.selectable_value(
                                &mut s.reference,
                                Some(g.id.clone()),
                                format!(
                                    "{} — {} po/jour (p. {})",
                                    g.nom,
                                    g.salaire_po_jour,
                                    g.source.page.unwrap_or(0)
                                ),
                            );
                        }
                    });
                if s.reference != avant {
                    if let Some(g) = s.reference.as_ref().and_then(|x| r.gestionnaire(x)) {
                        s.salaire = g.salaire_po_jour.to_string();
                        if s.nom.trim().is_empty() {
                            s.nom = g.nom.clone();
                        }
                    }
                }
                ui.end_row();
                ui.label("Nom");
                ui.add(
                    egui::TextEdit::singleline(&mut s.nom)
                        .hint_text("ex. Ameiko")
                        .desired_width(240.0),
                );
                ui.end_row();
                ui.label("Salaire par jour");
                widgets::champ_montant(ui, &mut s.salaire, 120.0);
                ui.end_row();
                ui.label("Payé par la trésorerie de");
                egui::ComboBox::from_id_salt("ges_loc")
                    .selected_text(
                        s.localite
                            .and_then(|l| etat.localites.get(&l))
                            .map(|l| l.nom.clone())
                            .unwrap_or_default(),
                    )
                    .show_ui(ui, |ui| {
                        for l in etat.localites.values() {
                            ui.selectable_value(&mut s.localite, Some(l.id), &l.nom);
                        }
                    });
                ui.end_row();
            });
        if let Some(g) = s.reference.as_ref().and_then(|x| r.gestionnaire(x)) {
            widgets::texte_doux(ui, format!("Compétences : {}", g.competences));
        }
        let salaire = Montant::depuis_texte(&s.salaire).filter(|m| !m.est_negatif());
        let ok = !s.nom.trim().is_empty() && salaire.is_some() && s.localite.is_some();
        if widgets::bouton_principal(ui, "Embaucher", ok) {
            action = Some((
                Evenement::GestionnaireEmbauche {
                    id: nouvel_id,
                    reference: s.reference.clone(),
                    nom: s.nom.trim().into(),
                    salaire: salaire.expect("valide"),
                    localite: s.localite.expect("choisie"),
                },
                format!("{} embauché(e)", s.nom.trim()),
            ));
            s.nom.clear();
        }
    });

    // ---------- liste ----------
    ui.add_space(6.0);
    widgets::titre_section(ui, "Gestionnaires employés");
    if etat.gestionnaires.is_empty() {
        widgets::texte_doux(ui, "Aucun gestionnaire pour l'instant.");
    }
    for g in etat.gestionnaires.values() {
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(&g.nom)
                        .size(17.0)
                        .color(theme::OR_PALE)
                        .strong(),
                );
                if let Some(x) = g.reference.as_ref().and_then(|x| r.gestionnaire(x)) {
                    widgets::texte_doux(
                        ui,
                        format!("{} (p. {})", x.nom, x.source.page.unwrap_or(0)),
                    );
                }
                widgets::montant(ui, etat.salaire_quotidien(g), 15.0);
                ui.label("/ jour");
                if !g.actif {
                    widgets::badge(ui, "inactif (non payé)", theme::TEXTE_DOUX);
                }
            });
            ui.horizontal_wrapped(|ui| {
                let mut x = g.clone();
                let mut change = false;
                change |= ui
                    .checkbox(&mut x.actif, "actif")
                    .on_hover_text("Un gestionnaire inactif n'est plus payé")
                    .changed();
                ui.label("Payé par");
                egui::ComboBox::from_id_salt(("ges_loc", g.id.0))
                    .selected_text(
                        etat.localites
                            .get(&x.localite)
                            .map(|l| l.nom.clone())
                            .unwrap_or_default(),
                    )
                    .show_ui(ui, |ui| {
                        for l in etat.localites.values() {
                            change |= ui.selectable_value(&mut x.localite, l.id, &l.nom).changed();
                        }
                    });
                if change {
                    action = Some((modifie(&x), format!("{} modifié(e)", g.nom)));
                }
                if ui.small_button("Renvoyer").clicked() {
                    action = Some((
                        Evenement::GestionnaireRenvoye { id: g.id },
                        format!("{} renvoyé(e)", g.nom),
                    ));
                }
            });
            // affectations
            ui.horizontal_wrapped(|ui| {
                ui.label("S'occupe de :");
                if g.affectations.is_empty() {
                    widgets::texte_doux(ui, "rien pour l'instant");
                }
                for a in &g.affectations {
                    widgets::badge(ui, &libelle_affectation(a, &etat), theme::OR);
                    if ui
                        .small_button("✖")
                        .on_hover_text("Retirer cette affectation")
                        .clicked()
                    {
                        let mut x = g.clone();
                        x.affectations.retain(|y| y != a);
                        action = Some((modifie(&x), format!("{} modifié(e)", g.nom)));
                    }
                }
            });
            ui.horizontal(|ui| {
                let mut choix = match s.ajout_affectation {
                    Some((id, a)) if id == g.id => a,
                    _ => None,
                };
                egui::ComboBox::from_id_salt(("ges_aff", g.id.0))
                    .selected_text(
                        choix
                            .map(|a| libelle_affectation(&a, &etat))
                            .unwrap_or("Ajouter un bâtiment ou une organisation…".into()),
                    )
                    .width(300.0)
                    .show_ui(ui, |ui| {
                        for b in etat.batiments.values() {
                            let a = Affectation::Batiment(b.id);
                            if !g.affectations.contains(&a) {
                                ui.selectable_value(
                                    &mut choix,
                                    Some(a),
                                    libelle_affectation(&a, &etat),
                                );
                            }
                        }
                        for o in etat.organisations.values() {
                            let a = Affectation::Organisation(o.id);
                            if !g.affectations.contains(&a) {
                                ui.selectable_value(
                                    &mut choix,
                                    Some(a),
                                    libelle_affectation(&a, &etat),
                                );
                            }
                        }
                    });
                if choix.is_some() {
                    s.ajout_affectation = Some((g.id, choix));
                }
                if ui
                    .add_enabled(choix.is_some(), egui::Button::new("Affecter"))
                    .clicked()
                {
                    let mut x = g.clone();
                    x.affectations.push(choix.expect("choisie"));
                    action = Some((modifie(&x), format!("{} affecté(e)", g.nom)));
                    s.ajout_affectation = None;
                }
            });
        });
        ui.add_space(4.0);
    }
    let unique = !etat.salaire_par_affectation;
    widgets::texte_doux(
        ui,
        if unique {
            "Un gestionnaire affecté à plusieurs établissements n'est payé qu'une fois par jour (PC-06 ; modifiable dans les Paramètres)."
        } else {
            "Réglage actif : un gestionnaire est payé une fois par affectation (modifiable dans les Paramètres)."
        },
    );

    if let Some((ev, msg)) = action {
        app.executer(ev, &msg);
    }
}
