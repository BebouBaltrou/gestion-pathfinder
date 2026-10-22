// Pas de console noire derrière la fenêtre en version publiée sous Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod pages;
mod theme;
mod widgets;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Intermède — gestion Pathfinder")
            .with_inner_size([1360.0, 860.0])
            .with_min_inner_size([980.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Intermède — gestion Pathfinder",
        options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}
