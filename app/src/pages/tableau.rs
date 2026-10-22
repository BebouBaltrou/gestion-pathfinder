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

    // ---------- alertes ----------
    let mut resolution = None;
    for (l, dd) in &etat.prestige_en_attente {
        let nom = etat
            .localites
            .get(l)
            .map(|x| x.nom.clone())
            .unwrap_or_default();
        ui.add_space(6.0);
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(format!("⚠ Test de prestige à faire à {nom} : DD {dd}"))
                    .color(theme::ORANGE)
                    .strong(),
            );
            widgets::texte_doux(
                ui,
                "Retour après 30 jours ou plus sans gestionnaire : 1d20 + Prestige contre ce DD. \
                 En cas d'échec, les entreprises de la localité risquent d'échapper à votre contrôle (GC p. 82) ; \
                 l'application note le résultat, à vous d'en tirer les conséquences avec le MJ.",
            );
            ui.horizontal(|ui| {
                if ui.button("Réussi").clicked() {
                    resolution = Some((*l, true));
                }
                if ui.button("Raté").clicked() {
                    resolution = Some((*l, false));
                }
            });
        });
    }

    ui.add_space(10.0);
    widgets::titre_section(ui, "Localités");
    if etat.localites.is_empty() {
        widgets::texte_doux(ui, "Aucune localité pour l'instant.");
        if ui.button("Créer une localité").clicked() {
            app.page = Page::Localites;
        }
    }
    let mut ouvrir = None;
    let mut aller_recettes = false;
    ui.horizontal_wrapped(|ui| {
        for l in etat.localites.values() {
            widgets::carte(ui, |ui| {
                ui.set_width(480.0);
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
                        let salaires = etat.salaires_localite(l.id);
                        if salaires.pa > 0 {
                            widgets::texte_doux(ui, format!("(salaires : −{salaires}/jour)"));
                        }
                    });
                    let attente = etat.jours_sans_recettes(l.id);
                    if attente > 0 && etat.a_entreprise(l.id, &app.r) {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "🎲 {attente} jour(s) de recettes à percevoir"
                                ))
                                .color(theme::OR),
                            );
                            if ui.small_button("Percevoir").clicked() {
                                aller_recettes = true;
                            }
                        });
                    }
                    let absence = etat.absence.get(&l.id).copied().unwrap_or(0);
                    if absence > 0 && etat.frequentees.contains(&l.id) {
                        let usure = match etat.periode_usure(l.id) {
                            Some(p) => format!("prochaine usure dans {} j", p - absence % p),
                            None => "pas d'usure (gestionnaire)".to_string(),
                        };
                        widgets::texte_doux(ui, format!("Aucun PJ depuis {absence} j — {usure}"));
                    }
                    let n_bat = etat.batiments_localite(l.id).count();
                    let n_org = etat.organisations_localite(l.id).count();
                    if n_bat + n_org > 0 {
                        widgets::texte_doux(
                            ui,
                            format!("{n_bat} bâtiment(s) · {n_org} organisation(s)"),
                        );
                    }
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
    if aller_recettes {
        app.page = Page::Recettes;
    }
    if let Some((localite, reussi)) = resolution {
        app.executer(
            campagne::evenements::Evenement::TestPrestigeResolu { localite, reussi },
            if reussi {
                "Test de prestige réussi"
            } else {
                "Test de prestige raté : voyez avec le MJ ce qu'il advient des entreprises"
            },
        );
    }
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
