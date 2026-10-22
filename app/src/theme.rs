//! Thème visuel : sombre, chaud, accents dorés, rouge pour les soldes négatifs.

use eframe::egui::{self, Color32, FontFamily, FontId, Stroke, TextStyle};

pub const OR: Color32 = Color32::from_rgb(214, 170, 92);
pub const OR_PALE: Color32 = Color32::from_rgb(236, 214, 168);
pub const TEXTE: Color32 = Color32::from_rgb(228, 222, 210);
pub const TEXTE_DOUX: Color32 = Color32::from_rgb(160, 152, 138);
pub const FOND: Color32 = Color32::from_rgb(24, 22, 20);
pub const PANNEAU: Color32 = Color32::from_rgb(32, 29, 26);
pub const CARTE: Color32 = Color32::from_rgb(42, 38, 34);
pub const BORDURE: Color32 = Color32::from_rgb(70, 62, 52);
pub const ROUGE: Color32 = Color32::from_rgb(226, 92, 80);
pub const VERT: Color32 = Color32::from_rgb(126, 190, 120);
pub const BLEU: Color32 = Color32::from_rgb(120, 170, 220);
pub const ORANGE: Color32 = Color32::from_rgb(230, 150, 70);

/// Couleurs associées aux ressources (po et capitaux).
pub fn couleur_ressource(r: domaine::Ressource) -> Color32 {
    use domaine::Ressource::*;
    match r {
        Po => OR,
        Marchandises => Color32::from_rgb(196, 140, 96),
        Influence => Color32::from_rgb(176, 132, 214),
        Travail => Color32::from_rgb(140, 176, 120),
        Magie => BLEU,
    }
}

pub fn appliquer(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = PANNEAU;
    v.window_fill = PANNEAU;
    v.extreme_bg_color = Color32::from_rgb(18, 16, 15);
    v.faint_bg_color = Color32::from_rgb(36, 33, 30);
    v.override_text_color = Some(TEXTE);
    v.hyperlink_color = OR;
    v.selection.bg_fill = Color32::from_rgb(110, 84, 40);
    v.selection.stroke = Stroke::new(1.0_f32, OR_PALE);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, BORDURE);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(52, 47, 41);
    v.widgets.inactive.bg_fill = Color32::from_rgb(52, 47, 41);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(74, 64, 50);
    v.widgets.hovered.bg_fill = Color32::from_rgb(74, 64, 50);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, OR);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(96, 78, 50);
    v.widgets.active.bg_fill = Color32::from_rgb(96, 78, 50);
    ctx.set_visuals(v);

    ctx.style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 4.0);
        s.spacing.interact_size.y = 24.0;
        s.text_styles.insert(
            TextStyle::Heading,
            FontId::new(24.0, FontFamily::Proportional),
        );
        s.text_styles
            .insert(TextStyle::Body, FontId::new(15.0, FontFamily::Proportional));
        s.text_styles.insert(
            TextStyle::Button,
            FontId::new(15.0, FontFamily::Proportional),
        );
        s.text_styles.insert(
            TextStyle::Small,
            FontId::new(12.5, FontFamily::Proportional),
        );
        s.text_styles.insert(
            TextStyle::Monospace,
            FontId::new(14.0, FontFamily::Monospace),
        );
    });
}
