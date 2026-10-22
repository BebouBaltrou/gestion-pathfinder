//! Petits éléments d'interface réutilisés par les pages.

use crate::theme;
use domaine::{CoutCapitaux, Montant, Ressource, Statut, TypeCapital};
use eframe::egui::{self, Color32, RichText, Stroke};

/// Cadre « carte » avec bordure discrète.
pub fn carte<R>(ui: &mut egui::Ui, contenu: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(theme::CARTE)
        .stroke(Stroke::new(1.0_f32, theme::BORDURE))
        .corner_radius(6)
        .inner_margin(egui::Margin::same(12))
        .show(ui, contenu)
        .inner
}

pub fn titre_section(ui: &mut egui::Ui, texte: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(texte).color(theme::OR).size(17.0).strong());
    ui.add_space(2.0);
}

pub fn titre_page(ui: &mut egui::Ui, texte: &str, sous_titre: &str) {
    ui.label(
        RichText::new(texte)
            .color(theme::OR_PALE)
            .size(26.0)
            .strong(),
    );
    if !sous_titre.is_empty() {
        ui.label(RichText::new(sous_titre).color(theme::TEXTE_DOUX));
    }
    ui.add_space(8.0);
}

/// Montant en po ; rouge s'il est négatif.
pub fn montant(ui: &mut egui::Ui, m: Montant, taille: f32) -> egui::Response {
    let c = if m.est_negatif() {
        theme::ROUGE
    } else {
        theme::OR_PALE
    };
    ui.label(RichText::new(m.to_string()).color(c).size(taille).strong())
}

/// Pastille « 12 Marchandises » colorée.
pub fn pastille(ui: &mut egui::Ui, r: Ressource, valeur: String) {
    let c = theme::couleur_ressource(r);
    egui::Frame::new()
        .fill(c.gamma_multiply(0.18))
        .stroke(Stroke::new(1.0_f32, c.gamma_multiply(0.7)))
        .corner_radius(10)
        .inner_margin(egui::Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(format!("{valeur} {r}")).color(c));
        });
}

/// Rangée des quatre capitaux.
pub fn capitaux(ui: &mut egui::Ui, c: &CoutCapitaux, masquer_zeros: bool) {
    ui.horizontal_wrapped(|ui| {
        let mut rien = true;
        for t in TypeCapital::TOUS {
            if masquer_zeros && c.get(t) == 0 {
                continue;
            }
            rien = false;
            pastille(ui, t.into(), c.get(t).to_string());
        }
        if rien {
            ui.label(RichText::new("aucun capital").color(theme::TEXTE_DOUX));
        }
    });
}

/// Badge de statut : officiel / personnalisé / à confirmer.
pub fn badge_statut(ui: &mut egui::Ui, s: Statut) {
    let (texte, c) = match s {
        Statut::Officiel => ("officiel", theme::VERT),
        Statut::Personnalise => ("personnalisé", theme::BLEU),
        Statut::AConfirmer => ("à confirmer", theme::ORANGE),
    };
    badge(ui, texte, c);
}

pub fn badge(ui: &mut egui::Ui, texte: &str, c: Color32) {
    egui::Frame::new()
        .fill(c.gamma_multiply(0.15))
        .stroke(Stroke::new(1.0_f32, c.gamma_multiply(0.8)))
        .corner_radius(4)
        .inner_margin(egui::Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(RichText::new(texte).color(c).size(12.5));
        });
}

/// Champ de saisie de montant avec validation visuelle.
pub fn champ_montant(ui: &mut egui::Ui, texte: &mut String, largeur: f32) -> Option<Montant> {
    let invalide = Montant::depuis_texte(texte).is_none() && !texte.trim().is_empty();
    let mut edit = egui::TextEdit::singleline(texte)
        .hint_text("ex. 12, 12,5 ou 4 po 7 pa")
        .desired_width(largeur);
    if invalide {
        edit = edit.text_color(theme::ROUGE);
    }
    ui.add(edit)
        .on_hover_text("Saisie en po : « 12 », « 12,5 » (= 12 po 5 pa), « 4 po 7 pa » ou « 7 pa »");
    Montant::depuis_texte(texte)
}

/// Champ entier positif saisi comme texte (vide = None).
pub fn champ_entier(
    ui: &mut egui::Ui,
    texte: &mut String,
    indice: &str,
    largeur: f32,
) -> Option<u32> {
    let lire = |t: &str| t.trim().replace([' ', '\u{a0}'], "").parse::<u32>().ok();
    let invalide = lire(texte).is_none() && !texte.trim().is_empty();
    let mut edit = egui::TextEdit::singleline(texte)
        .hint_text(indice)
        .desired_width(largeur);
    if invalide {
        edit = edit.text_color(theme::ROUGE);
    }
    ui.add(edit);
    lire(texte)
}

pub fn texte_doux(ui: &mut egui::Ui, t: impl Into<String>) {
    ui.label(RichText::new(t.into()).color(theme::TEXTE_DOUX));
}

/// Bouton principal (doré).
pub fn bouton_principal(ui: &mut egui::Ui, texte: &str, actif: bool) -> bool {
    ui.add_enabled(
        actif,
        egui::Button::new(
            RichText::new(texte)
                .color(Color32::from_rgb(30, 24, 16))
                .strong(),
        )
        .fill(theme::OR),
    )
    .clicked()
}
