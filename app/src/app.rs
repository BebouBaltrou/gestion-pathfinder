//! Application : navigation, campagne ouverte, enregistrement automatique, messages.

use crate::{pages, theme, widgets};
use campagne::evenements::Evenement;
use campagne::Campagne;
use eframe::egui::{self, RichText};
use regles::Referentiel;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    TableauDeBord,
    Localites,
    Personnages,
    Historique,
    Referentiel,
    Points,
    Parametres,
    Campagnes,
}

impl Page {
    const NAVIGATION: [(Page, &'static str, &'static str); 8] = [
        (Page::TableauDeBord, "🏠", "Tableau de bord"),
        (Page::Localites, "🏰", "Localités"),
        (Page::Personnages, "👤", "Personnages"),
        (Page::Historique, "📜", "Historique"),
        (Page::Referentiel, "📖", "Référentiel"),
        (Page::Points, "❓", "Points à confirmer"),
        (Page::Parametres, "⚙", "Paramètres"),
        (Page::Campagnes, "🗂", "Campagnes"),
    ];
}

pub enum Message {
    Ok(String),
    Erreur(String),
}

pub struct App {
    pub r: Referentiel,
    pub dossier: PathBuf,
    pub campagne: Option<Campagne>,
    pub chemin: Option<PathBuf>,
    pub page: Page,
    pub message: Option<(Message, f64)>,
    pub ui_localites: pages::localites::Etat,
    pub ui_personnages: pages::personnages::Etat,
    pub ui_referentiel: pages::referentiel::Etat,
    pub ui_campagnes: pages::campagnes::Etat,
    pub ui_tableau: pages::tableau::Etat,
}

/// Dossier des campagnes : %APPDATA%\GestionPathfinder\campagnes sous Windows.
fn dossier_campagnes() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_DATA_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("GestionPathfinder").join("campagnes")
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::appliquer(&cc.egui_ctx);
        let mut app = App {
            r: regles::referentiel_officiel(),
            dossier: dossier_campagnes(),
            campagne: None,
            chemin: None,
            page: Page::Campagnes,
            message: None,
            ui_localites: Default::default(),
            ui_personnages: Default::default(),
            ui_referentiel: Default::default(),
            ui_campagnes: Default::default(),
            ui_tableau: Default::default(),
        };
        // Rouvre la campagne la plus récemment modifiée.
        if let Some(chemin) = app.lister_campagnes().into_iter().next().map(|(_, p)| p) {
            app.ouvrir(chemin);
        }
        app
    }

    /// Campagnes du dossier, la plus récente d'abord : (nom de fichier, chemin).
    pub fn lister_campagnes(&self) -> Vec<(String, PathBuf)> {
        let mut v: Vec<(std::time::SystemTime, String, PathBuf)> = std::fs::read_dir(&self.dossier)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "json"))
            .map(|p| {
                let t = std::fs::metadata(&p)
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH);
                let nom = p
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                (t, nom, p)
            })
            .collect();
        v.sort_by(|a, b| b.0.cmp(&a.0));
        v.into_iter().map(|(_, n, p)| (n, p)).collect()
    }

    pub fn ouvrir(&mut self, chemin: PathBuf) {
        match campagne::fichier::charger(&chemin) {
            Ok(c) => {
                self.ok(format!("Campagne « {} » ouverte", c.nom));
                self.campagne = Some(c);
                self.chemin = Some(chemin);
                self.page = Page::TableauDeBord;
            }
            Err(e) => self.erreur(format!("Impossible d'ouvrir {} : {e}", chemin.display())),
        }
    }

    pub fn creer_campagne(&mut self, nom: &str) {
        let nom = nom.trim();
        if nom.is_empty() {
            self.erreur("Donnez un nom à la campagne".into());
            return;
        }
        let fichier: String = nom
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let chemin = self.dossier.join(format!("{}.json", fichier.trim()));
        if chemin.exists() {
            self.erreur(format!("Une campagne « {nom} » existe déjà"));
            return;
        }
        self.campagne = Some(Campagne::nouvelle(nom));
        self.chemin = Some(chemin);
        self.enregistrer();
        self.page = Page::Parametres;
        self.ok(format!(
            "Campagne « {nom} » créée — réglez son calendrier puis créez vos localités"
        ));
    }

    pub fn enregistrer(&mut self) {
        if let (Some(c), Some(p)) = (&self.campagne, &self.chemin) {
            if let Err(e) = campagne::fichier::enregistrer(c, p) {
                self.erreur(format!("Enregistrement impossible : {e}"));
            }
        }
    }

    /// Exécute un événement sur la campagne ouverte, puis enregistre.
    pub fn executer(&mut self, ev: Evenement, succes: &str) -> bool {
        let Some(c) = self.campagne.as_mut() else {
            return false;
        };
        match c.executer(ev, &self.r) {
            Ok(consequences) => {
                let mut msg = succes.to_string();
                let etat = c.etat(&self.r).clone();
                for campagne::etat::Consequence::TransfertArrive(t) in consequences {
                    let nom = etat
                        .localites
                        .get(&t.vers)
                        .map(|l| l.nom.clone())
                        .unwrap_or_default();
                    msg.push_str(&format!(" — arrivée de {} à {nom}", t.lot));
                }
                self.ok(msg);
                self.enregistrer();
                true
            }
            Err(e) => {
                self.erreur(format!("Action refusée : {e}"));
                false
            }
        }
    }

    pub fn ok(&mut self, m: String) {
        self.message = Some((Message::Ok(m), 0.0));
    }
    pub fn erreur(&mut self, m: String) {
        self.message = Some((Message::Erreur(m), 0.0));
    }

    fn annuler(&mut self) {
        if let Some(c) = self.campagne.as_mut() {
            if let Some(e) = c.annuler().map(|e| e.evenement.clone()) {
                let d = c.decrire(&e, &self.r);
                self.ok(format!("Annulé : {d}"));
                self.enregistrer();
            }
        }
    }

    fn retablir(&mut self) {
        if let Some(c) = self.campagne.as_mut() {
            if let Some(e) = c.retablir().map(|e| e.evenement.clone()) {
                let d = c.decrire(&e, &self.r);
                self.ok(format!("Rétabli : {d}"));
                self.enregistrer();
            }
        }
    }

    fn barre_haut(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("barre_haut")
            .frame(
                egui::Frame::new()
                    .fill(theme::FOND)
                    .inner_margin(egui::Margin::symmetric(14, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("⚜ Intermède")
                            .color(theme::OR)
                            .size(22.0)
                            .strong(),
                    );
                    ui.add_space(16.0);
                    let Some(c) = self.campagne.as_mut() else {
                        widgets::texte_doux(ui, "Aucune campagne ouverte");
                        return;
                    };
                    ui.label(RichText::new(&c.nom).color(theme::OR_PALE).size(17.0));
                    ui.separator();
                    let date = c.date_courante(&self.r);
                    ui.label(RichText::new(date).color(theme::TEXTE));
                    let (pa, pr) = (c.peut_annuler(), c.peut_retablir());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(pr, egui::Button::new("Rétablir ↪"))
                            .on_hover_text("Ctrl+Y")
                            .clicked()
                        {
                            self.retablir();
                        }
                        if ui
                            .add_enabled(pa, egui::Button::new("↩ Annuler"))
                            .on_hover_text("Ctrl+Z")
                            .clicked()
                        {
                            self.annuler();
                        }
                        ui.separator();
                        if ui.button("+7 jours").clicked() {
                            self.executer(Evenement::JoursAvances { nombre: 7 }, "7 jours écoulés");
                        }
                        if widgets::bouton_principal(ui, "Jour suivant ▶", true) {
                            self.executer(Evenement::JoursAvances { nombre: 1 }, "Jour suivant");
                        }
                    });
                });
            });
    }

    fn navigation(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .resizable(false)
            .exact_width(212.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::FOND)
                    .inner_margin(egui::Margin::symmetric(10, 12)),
            )
            .show(ctx, |ui| {
                let ouverte = self.campagne.is_some();
                for (p, icone, nom) in Page::NAVIGATION {
                    let actif =
                        ouverte || matches!(p, Page::Campagnes | Page::Referentiel | Page::Points);
                    let choisi = self.page == p;
                    let texte =
                        RichText::new(format!("{icone}  {nom}"))
                            .size(16.0)
                            .color(if choisi {
                                theme::OR
                            } else if actif {
                                theme::TEXTE
                            } else {
                                theme::TEXTE_DOUX
                            });
                    let b = egui::Button::new(texte)
                        .fill(if choisi {
                            theme::CARTE
                        } else {
                            egui::Color32::TRANSPARENT
                        })
                        .min_size(egui::vec2(ui.available_width(), 34.0));
                    if ui.add_enabled(actif, b).clicked() {
                        self.page = p;
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    widgets::texte_doux(ui, "Règles : Guide de campagne (VF d'Ultimate Campaign)");
                });
            });
    }

    fn barre_bas(&mut self, ctx: &egui::Context) {
        let dt = ctx.input(|i| i.stable_dt) as f64;
        egui::TopBottomPanel::bottom("messages")
            .frame(
                egui::Frame::new()
                    .fill(theme::FOND)
                    .inner_margin(egui::Margin::symmetric(14, 6)),
            )
            .show(ctx, |ui| {
                let mut effacer = false;
                match &mut self.message {
                    Some((m, age)) => {
                        *age += dt;
                        let (t, c) = match m {
                            Message::Ok(t) => (format!("✔ {t}"), theme::VERT),
                            Message::Erreur(t) => (format!("✖ {t}"), theme::ROUGE),
                        };
                        ui.label(RichText::new(t).color(c));
                        if *age > 12.0 {
                            effacer = true;
                        }
                    }
                    None => {
                        let chemin = self
                            .chemin
                            .as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default();
                        widgets::texte_doux(
                            ui,
                            if chemin.is_empty() {
                                "Prêt".into()
                            } else {
                                format!("Enregistrement automatique : {chemin}")
                            },
                        );
                    }
                }
                if effacer {
                    self.message = None;
                }
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let (z, y) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::Z),
                i.modifiers.command && i.key_pressed(egui::Key::Y),
            )
        });
        if z {
            self.annuler();
        }
        if y {
            self.retablir();
        }
        if self.campagne.is_none()
            && !matches!(
                self.page,
                Page::Campagnes | Page::Referentiel | Page::Points
            )
        {
            self.page = Page::Campagnes;
        }

        self.barre_haut(ctx);
        self.navigation(ctx);
        self.barre_bas(ctx);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::PANNEAU)
                    .inner_margin(egui::Margin::same(18)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.page {
                        Page::TableauDeBord => pages::tableau::afficher(self, ui),
                        Page::Localites => pages::localites::afficher(self, ui),
                        Page::Personnages => pages::personnages::afficher(self, ui),
                        Page::Historique => pages::historique::afficher(self, ui),
                        Page::Referentiel => pages::referentiel::afficher(self, ui),
                        Page::Points => pages::points::afficher(self, ui),
                        Page::Parametres => pages::parametres::afficher(self, ui),
                        Page::Campagnes => pages::campagnes::afficher(self, ui),
                    });
            });
        if self.message.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }
    }
}
