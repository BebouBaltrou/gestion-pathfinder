//! Points à confirmer et avertissements de cohérence du référentiel.

use crate::app::App;
use crate::{theme, widgets};
use domaine::fiches::StatutPoint;
use eframe::egui::{self, RichText};

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    widgets::titre_page(
        ui,
        "Points à confirmer",
        "Règles ambiguës ou incohérentes du livre, et la lecture appliquée par l'application",
    );
    let avertir = app
        .campagne
        .as_ref()
        .is_none_or(|c| c.options.afficher_avertissements);
    if !avertir {
        widgets::texte_doux(ui, "Les avertissements sont masqués dans les calculs (Paramètres), mais restent listés ici.");
    }

    let avertissements = regles::controles::controler(&app.r);
    if !avertissements.is_empty() {
        widgets::titre_section(ui, "Contrôles du référentiel");
        for a in &avertissements {
            ui.label(RichText::new(format!("⚠ {a}")).color(theme::ORANGE));
        }
        ui.add_space(8.0);
    }

    let ouverts = app
        .r
        .points_a_confirmer
        .iter()
        .filter(|p| p.statut == StatutPoint::Ouvert)
        .count();
    widgets::titre_section(
        ui,
        &format!(
            "{} points, dont {ouverts} ouvert(s)",
            app.r.points_a_confirmer.len()
        ),
    );
    for p in &app.r.points_a_confirmer {
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&p.code).color(theme::OR).strong());
                match p.statut {
                    StatutPoint::Ouvert => widgets::badge(ui, "ouvert", theme::ORANGE),
                    StatutPoint::Valide => widgets::badge(ui, "validé", theme::VERT),
                }
                widgets::texte_doux(ui, app.r.reference(&p.source));
            });
            ui.label(&p.question);
            ui.label(
                RichText::new(format!("Lecture appliquée : {}", p.lecture_appliquee))
                    .color(theme::OR_PALE),
            );
            if let Some(d) = &p.decision {
                widgets::texte_doux(ui, d);
            }
        });
        ui.add_space(4.0);
    }
}
