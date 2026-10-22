use crate::app::{App, Page};
use crate::{theme, widgets};
use domaine::{CoutCapitaux, Montant};
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct Etat;

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    let etat = c.etat(&app.r).clone();
    let date = c.date_courante(&app.r);
    widgets::titre_page(ui, "Tableau de bord", &date);

    let total_po: Montant = etat.localites.values().map(|l| l.tresorerie).sum();
    let total_pj: Montant = etat.personnages.values().map(|p| p.compte).sum();
    let total_cap = etat
        .localites
        .values()
        .fold(CoutCapitaux::default(), |a, l| a + l.capitaux);
    ui.horizontal(|ui| {
        widgets::carte(ui, |ui| {
            ui.vertical(|ui| {
                widgets::texte_doux(ui, "Trésoreries des localités");
                widgets::montant(ui, total_po, 22.0);
            });
        });
        widgets::carte(ui, |ui| {
            ui.vertical(|ui| {
                widgets::texte_doux(ui, "Comptes des personnages");
                widgets::montant(ui, total_pj, 22.0);
            });
        });
        widgets::carte(ui, |ui| {
            ui.vertical(|ui| {
                widgets::texte_doux(ui, "Capitaux en réserve");
                widgets::capitaux(ui, &total_cap, false);
            });
        });
        widgets::carte(ui, |ui| {
            ui.vertical(|ui| {
                widgets::texte_doux(ui, "Transferts en route");
                ui.label(
                    RichText::new(etat.transferts.len().to_string())
                        .size(22.0)
                        .color(theme::OR_PALE),
                );
            });
        });
    });

    ui.add_space(10.0);
    widgets::titre_section(ui, "Localités");
    if etat.localites.is_empty() {
        widgets::texte_doux(ui, "Aucune localité pour l'instant.");
        if ui.button("Créer une localité").clicked() {
            app.page = Page::Localites;
        }
    }
    let mut ouvrir = None;
    ui.horizontal_wrapped(|ui| {
        for l in etat.localites.values() {
            widgets::carte(ui, |ui| {
                ui.set_width(300.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&l.nom)
                                .size(18.0)
                                .color(theme::OR_PALE)
                                .strong(),
                        );
                        widgets::texte_doux(ui, &l.genre);
                    });
                    widgets::texte_doux(ui, super::libelle_categorie(&l.categorie, &app.r));
                    ui.horizontal(|ui| {
                        ui.label("Trésorerie :");
                        widgets::montant(ui, l.tresorerie, 17.0);
                    });
                    widgets::capitaux(ui, &l.capitaux, false);
                    let en_route: Vec<_> = etat.en_route_vers(l.id).collect();
                    for t in en_route {
                        widgets::texte_doux(
                            ui,
                            format!("⏳ {} arrive le jour {}", t.lot, t.jour_arrivee),
                        );
                    }
                    let pjs: Vec<_> = etat
                        .personnages
                        .values()
                        .filter(|p| p.localite == Some(l.id))
                        .map(|p| p.nom.as_str())
                        .collect();
                    if !pjs.is_empty() {
                        widgets::texte_doux(ui, format!("Présents : {}", pjs.join(", ")));
                    }
                    if ui.button("Ouvrir").clicked() {
                        ouvrir = Some(l.id);
                    }
                });
            });
        }
    });
    if let Some(id) = ouvrir {
        app.ui_localites.selection = Some(id);
        app.page = Page::Localites;
    }

    ui.add_space(10.0);
    widgets::titre_section(ui, "Personnages");
    if etat.personnages.is_empty() {
        widgets::texte_doux(ui, "Aucun personnage pour l'instant.");
        if ui.button("Créer un personnage").clicked() {
            app.page = Page::Personnages;
        }
    }
    ui.horizontal_wrapped(|ui| {
        for p in etat.personnages.values() {
            widgets::carte(ui, |ui| {
                ui.set_width(220.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&p.nom)
                            .size(17.0)
                            .color(theme::OR_PALE)
                            .strong(),
                    );
                    widgets::montant(ui, p.compte, 16.0);
                    let lieu = p
                        .localite
                        .and_then(|id| etat.localites.get(&id))
                        .map(|l| l.nom.clone())
                        .unwrap_or_else(|| "en voyage / hors localité".into());
                    widgets::texte_doux(ui, format!("📍 {lieu}"));
                });
            });
        }
    });
}
