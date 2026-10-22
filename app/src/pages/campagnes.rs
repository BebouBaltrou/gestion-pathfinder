//! Campagnes : créer, ouvrir.

use crate::app::App;
use crate::{theme, widgets};
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct Etat {
    nom: String,
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    widgets::titre_page(
        ui,
        "Campagnes",
        "Chaque campagne a ses personnages, ses localités et son historique, enregistrés automatiquement",
    );
    widgets::carte(ui, |ui| {
        widgets::titre_section(ui, "Nouvelle campagne");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut app.ui_campagnes.nom)
                    .hint_text("ex. Kingmaker")
                    .desired_width(260.0),
            );
            let nom = app.ui_campagnes.nom.clone();
            if widgets::bouton_principal(ui, "Créer", !nom.trim().is_empty()) {
                app.creer_campagne(&nom);
                app.ui_campagnes.nom.clear();
            }
        });
    });
    ui.add_space(10.0);

    widgets::titre_section(ui, "Campagnes enregistrées");
    let liste = app.lister_campagnes();
    if liste.is_empty() {
        widgets::texte_doux(ui, "Aucune campagne pour l'instant.");
    }
    let mut ouvrir = None;
    for (nom, chemin) in liste {
        let ouverte = app.chemin.as_ref() == Some(&chemin);
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&nom)
                        .size(17.0)
                        .color(theme::OR_PALE)
                        .strong(),
                );
                if ouverte {
                    widgets::badge(ui, "ouverte", theme::VERT);
                } else if ui.button("Ouvrir").clicked() {
                    ouvrir = Some(chemin.clone());
                }
            });
            widgets::texte_doux(ui, chemin.display().to_string());
        });
        ui.add_space(4.0);
    }
    if let Some(p) = ouvrir {
        app.ouvrir(p);
    }
    ui.add_space(8.0);
    widgets::texte_doux(ui, format!("Dossier : {}", app.dossier.display()));
}
