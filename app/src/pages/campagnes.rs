//! Campagnes : créer, ouvrir.

use crate::app::App;
use crate::{theme, widgets};
use eframe::egui::{self, RichText};
use std::path::Path;

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
    export(app, ui);
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
    ui.horizontal(|ui| {
        widgets::texte_doux(ui, format!("Dossier : {}", app.dossier.display()));
        if ui.small_button("Ouvrir le dossier").clicked() {
            let _ = std::fs::create_dir_all(&app.dossier);
            ouvrir_externe(&app.dossier);
        }
    });
}

/// Ouvre un fichier ou un dossier avec l'application par défaut du système.
pub fn ouvrir_externe(p: &Path) {
    #[cfg(windows)]
    let r = std::process::Command::new("explorer").arg(p).spawn();
    #[cfg(not(windows))]
    let r = std::process::Command::new("xdg-open").arg(p).spawn();
    let _ = r;
}

fn nom_fichier(nom: &str) -> String {
    nom.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn ecrire(app: &mut App, chemin: &Path, contenu: &str, quoi: &str, ouvrir: bool) {
    match std::fs::write(chemin, contenu) {
        Ok(()) => {
            app.ok(format!("{quoi} enregistré : {}", chemin.display()));
            if ouvrir {
                ouvrir_externe(chemin);
            }
        }
        Err(e) => app.erreur(format!("Écriture impossible ({}) : {e}", chemin.display())),
    }
}

fn export(app: &mut App, ui: &mut egui::Ui) {
    widgets::titre_section(ui, "Exporter et importer");
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        let nom = app.campagne.as_ref().map(|c| nom_fichier(&c.nom));
        ui.horizontal_wrapped(|ui| {
            let ouverte = nom.is_some();
            if ui
                .add_enabled(ouverte, egui::Button::new("💾 Exporter la campagne…"))
                .on_hover_text("Copie complète (fichier .json) à ranger ou à transmettre")
                .clicked()
            {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Exporter la campagne")
                    .set_file_name(format!("{}.json", nom.clone().unwrap_or_default()))
                    .add_filter("Campagne", &["json"])
                    .save_file()
                {
                    let c = app.campagne.clone().expect("ouverte");
                    match campagne::fichier::vers_json(&c) {
                        Ok(j) => ecrire(app, &p, &j, "Export", false),
                        Err(e) => app.erreur(format!("Export impossible : {e}")),
                    }
                }
            }
            if ui
                .add_enabled(ouverte, egui::Button::new("📄 Récapitulatif (HTML / PDF)…"))
                .on_hover_text(
                    "Page lisible : personnages, localités, bâtiments, organisations, gestionnaires, \
                     historique. Ouvrez-la dans le navigateur puis « Imprimer → PDF ».",
                )
                .clicked()
            {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Récapitulatif de la campagne")
                    .set_file_name(format!("{} - récapitulatif.html", nom.clone().unwrap_or_default()))
                    .add_filter("Page web", &["html"])
                    .save_file()
                {
                    let r = app.r.clone();
                    let h = campagne::rapport::html(app.campagne.as_mut().expect("ouverte"), &r);
                    ecrire(app, &p, &h, "Récapitulatif", true);
                }
            }
            if ui
                .add_enabled(ouverte, egui::Button::new("📊 Historique (tableur)…"))
                .on_hover_text("Fichier CSV lisible par Excel ou LibreOffice")
                .clicked()
            {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Historique de la campagne")
                    .set_file_name(format!("{} - historique.csv", nom.clone().unwrap_or_default()))
                    .add_filter("Tableur", &["csv"])
                    .save_file()
                {
                    let r = app.r.clone();
                    let csv = campagne::rapport::historique_csv(app.campagne.as_mut().expect("ouverte"), &r);
                    ecrire(app, &p, &csv, "Historique", false);
                }
            }
            ui.separator();
            if ui
                .button("📥 Importer une campagne…")
                .on_hover_text("Ajoute une campagne exportée (.json) à vos campagnes et l'ouvre")
                .clicked()
            {
                if let Some(p) = rfd::FileDialog::new()
                    .set_title("Importer une campagne")
                    .add_filter("Campagne", &["json"])
                    .pick_file()
                {
                    importer(app, &p);
                }
            }
        });
        if app.simulation.is_some() {
            widgets::texte_doux(
                ui,
                "En simulation, les exports portent sur la campagne simulée.",
            );
        }
    });
}

fn importer(app: &mut App, source: &Path) {
    let c = match campagne::fichier::charger(source) {
        Ok(c) => c,
        Err(e) => {
            app.erreur(format!("Import impossible : {e}"));
            return;
        }
    };
    let _ = std::fs::create_dir_all(&app.dossier);
    let base = nom_fichier(&c.nom);
    let mut cible = app.dossier.join(format!("{base}.json"));
    let mut n = 2;
    while cible.exists() {
        cible = app.dossier.join(format!("{base} ({n}).json"));
        n += 1;
    }
    match campagne::fichier::enregistrer(&c, &cible) {
        Ok(()) => app.ouvrir(cible),
        Err(e) => app.erreur(format!("Import impossible : {e}")),
    }
}
