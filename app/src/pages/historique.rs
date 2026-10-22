//! Historique : journal des événements, du plus récent au plus ancien.

use crate::app::App;
use crate::{theme, widgets};
use eframe::egui::{self, RichText};

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    widgets::titre_page(
        ui,
        "Historique",
        "Toutes les opérations, dans l'ordre. Annuler (Ctrl+Z) retire la dernière ; Rétablir (Ctrl+Y) la remet.",
    );
    let annulees: Vec<_> = c.journal[c.curseur..].iter().rev().cloned().collect();
    let actives: Vec<_> = c.historique().cloned().collect();
    let cal = c.calendrier.clone();

    if !annulees.is_empty() {
        widgets::titre_section(ui, "Annulées (peuvent être rétablies)");
        for e in &annulees {
            let d = c.decrire(&e.evenement, &app.r);
            ui.label(
                RichText::new(format!("#{} · jour {} · {d}", e.numero, e.jour))
                    .color(theme::TEXTE_DOUX)
                    .strikethrough(),
            );
        }
        ui.add_space(8.0);
    }

    widgets::titre_section(ui, &format!("Journal ({} opérations)", actives.len()));
    if actives.is_empty() {
        widgets::texte_doux(ui, "Rien pour l'instant.");
    }
    egui::Grid::new("journal")
        .striped(true)
        .num_columns(3)
        .spacing([16.0, 6.0])
        .show(ui, |ui| {
            ui.label(RichText::new("N°").color(theme::OR));
            ui.label(RichText::new("Date").color(theme::OR));
            ui.label(RichText::new("Opération").color(theme::OR));
            ui.end_row();
            for e in &actives {
                let d = c.decrire(&e.evenement, &app.r);
                ui.label(RichText::new(format!("#{}", e.numero)).color(theme::TEXTE_DOUX));
                ui.label(format!("Jour {} — {}", e.jour, cal.libelle(e.jour)));
                ui.label(d);
                ui.end_row();
            }
        });
}
