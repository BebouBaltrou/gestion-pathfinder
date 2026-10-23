use crate::app::{App, Page};
use crate::{theme, widgets};
use campagne::etat::Etat as EtatCampagne;
use campagne::modele::*;
use domaine::{CoutCapitaux, Montant, TypeCapital};
use eframe::egui::{self, RichText};
use regles::Referentiel;

#[derive(Default)]
pub struct Etat;

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    let etat = c.etat(&app.r).clone();
    let date = c.date_courante(&app.r);
    let calendrier = c.calendrier.clone();
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
                widgets::texte_doux(ui, "Employés");
                let (gens, a_venir) = etat.effectif(etat.equipes.values(), &app.r);
                ui.label(
                    RichText::new(format!("👥 {gens}"))
                        .size(22.0)
                        .color(theme::OR_PALE),
                );
                if a_venir > 0 {
                    widgets::texte_doux(ui, format!("+{a_venir} à venir"));
                }
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

    // ---------- alertes et travaux ----------
    let r = app.r.clone();
    if let Some(p) = alertes(ui, &etat, &r) {
        app.page = p;
    }
    if let Some(p) = travaux(ui, &etat, &r, &calendrier) {
        app.page = p;
    }

    // ---------- tests de prestige ----------
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
                        let (gens, _) = etat.effectif(
                            etat.equipes.values().filter(|e| e.localite == l.id),
                            &app.r,
                        );
                        widgets::texte_doux(
                            ui,
                            format!(
                                "{n_bat} bâtiment(s) · {n_org} organisation(s) · 👥 {gens} employé(s)"
                            ),
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

/// Une alerte : texte, couleur, page où agir.
struct Alerte {
    texte: String,
    grave: bool,
    page: Page,
}

fn alertes(ui: &mut egui::Ui, etat: &EtatCampagne, r: &Referentiel) -> Option<Page> {
    let mut v = Vec::new();
    for l in etat.localites.values() {
        if l.tresorerie.est_negatif() {
            v.push(Alerte {
                texte: format!("Trésorerie de {} négative : {}", l.nom, l.tresorerie),
                grave: true,
                page: Page::Localites,
            });
        }
        let salaires = etat.salaires_localite(l.id);
        if salaires.pa > 0 && !l.tresorerie.est_negatif() {
            let jours = l.tresorerie.pa / salaires.pa;
            if jours < 7 {
                v.push(Alerte {
                    texte: format!(
                        "{} : salaires couverts encore {jours} jour(s) seulement ({salaires}/jour)",
                        l.nom
                    ),
                    grave: false,
                    page: Page::Gestionnaires,
                });
            }
        }
        let absence = etat.absence.get(&l.id).copied().unwrap_or(0);
        if absence > 0 && etat.frequentees.contains(&l.id) && l.capitaux.unites() > 0 {
            if let Some(p) = etat.periode_usure(l.id) {
                let reste = p - absence % p;
                if reste <= 2 {
                    v.push(Alerte {
                        texte: format!(
                            "{} : usure des capitaux dans {reste} jour(s) (aucun PJ depuis {absence} j)",
                            l.nom
                        ),
                        grave: false,
                        page: Page::Personnages,
                    });
                }
            }
            if absence >= 23 && !etat.gestionnaire_paye(l.id) && etat.a_entreprise(l.id, r) {
                v.push(Alerte {
                    texte: format!(
                        "{} : aucun PJ depuis {absence} j ; à 30 j, un test de prestige sera demandé au retour",
                        l.nom
                    ),
                    grave: false,
                    page: Page::Gestionnaires,
                });
            }
        }
    }
    // chantiers et recrutements bloqués faute de capitaux en réserve
    for s in etat.salles.values() {
        if let StatutSalle::EnTravaux(ch) = &s.statut {
            let Some(b) = etat.batiments.get(&s.batiment) else {
                continue;
            };
            if bloque(ch, etat, b.localite) {
                v.push(Alerte {
                    texte: format!(
                        "{} — {} : réserves vides pour {}",
                        b.nom,
                        etat.nom_salle(s, r),
                        manque(ch, etat, b.localite)
                    ),
                    grave: false,
                    page: Page::Batiments,
                });
            }
        }
    }
    for e in etat.equipes.values() {
        if let StatutEquipe::EnRecrutement(ch) = &e.statut {
            if bloque(ch, etat, e.localite) {
                v.push(Alerte {
                    texte: format!(
                        "{} : réserves vides pour {}",
                        etat.nom_equipe(e, r),
                        manque(ch, etat, e.localite)
                    ),
                    grave: false,
                    page: Page::Organisations,
                });
            }
        }
    }
    if v.is_empty() {
        return None;
    }
    let mut aller = None;
    ui.add_space(8.0);
    widgets::titre_section(ui, "Alertes");
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        for a in &v {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(format!("⚠ {}", a.texte)).color(if a.grave {
                    theme::ROUGE
                } else {
                    theme::ORANGE
                }));
                if ui.small_button("Voir").clicked() {
                    aller = Some(a.page);
                }
            });
        }
    });
    aller
}

/// Le chantier a encore besoin de capitaux et aucun des types manquants n'est en réserve.
fn bloque(ch: &Chantier, etat: &EtatCampagne, l: IdLocalite) -> bool {
    let reste = ch.reste();
    let stock = etat
        .localites
        .get(&l)
        .map(|x| x.capitaux)
        .unwrap_or_default();
    reste.unites() > 0
        && TypeCapital::TOUS
            .iter()
            .all(|t| reste.get(*t) == 0 || stock.get(*t) == 0)
}

fn manque(ch: &Chantier, etat: &EtatCampagne, l: IdLocalite) -> String {
    let reste = ch.reste();
    let stock = etat
        .localites
        .get(&l)
        .map(|x| x.capitaux)
        .unwrap_or_default();
    TypeCapital::TOUS
        .iter()
        .filter(|t| reste.get(**t) > 0 && stock.get(**t) == 0)
        .map(|t| format!("{} {t}", reste.get(*t)))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Chantiers, recrutements et délocalisations en cours, avec une fin estimée.
fn travaux(
    ui: &mut egui::Ui,
    etat: &EtatCampagne,
    r: &Referentiel,
    cal: &campagne::calendrier::Calendrier,
) -> Option<Page> {
    // (libellé, détail, avancement 0-1, jours restants estimés, page)
    let mut lignes: Vec<(String, String, f32, u32, Page)> = Vec::new();
    let avance = |ch: &Chantier| {
        let c = ch.depense.unites() as f32 / ch.cout.unites().max(1) as f32;
        let t = if ch.duree == 0 {
            1.0
        } else {
            ch.jours_ecoules as f32 / ch.duree as f32
        };
        c.min(t).min(1.0)
    };
    for s in etat.salles.values() {
        if let StatutSalle::EnTravaux(ch) = &s.statut {
            let Some(b) = etat.batiments.get(&s.batiment) else {
                continue;
            };
            let limite = etat.limite_depense(b.localite, r).unwrap_or(0);
            let quoi = match &ch.nature {
                NatureChantier::Construction => "construction",
                NatureChantier::Amelioration { .. } => "amélioration",
            };
            lignes.push((
                format!("🏛 {} — {}", b.nom, etat.nom_salle(s, r)),
                format!("{quoi}, reste {}", ch.reste()),
                avance(ch),
                ch.jours_restants_estimes(limite),
                Page::Batiments,
            ));
        }
        for x in &s.extensions {
            if let StatutExtension::EnTravaux(ch) = &x.statut {
                let Some(b) = etat.batiments.get(&s.batiment) else {
                    continue;
                };
                let limite = etat.limite_depense(b.localite, r).unwrap_or(0);
                let n = r
                    .extension(&x.reference)
                    .map(|u| u.nom.clone())
                    .unwrap_or(x.reference.clone());
                lignes.push((
                    format!("🏛 {} — {} : {n}", b.nom, etat.nom_salle(s, r)),
                    format!("extension, reste {}", ch.reste()),
                    avance(ch),
                    ch.jours_restants_estimes(limite),
                    Page::Batiments,
                ));
            }
        }
    }
    for e in etat.equipes.values() {
        let org = etat
            .organisations
            .get(&e.organisation)
            .map(|o| o.nom.clone())
            .unwrap_or_default();
        match &e.statut {
            StatutEquipe::EnRecrutement(ch) => {
                let limite = etat.limite_depense(e.localite, r).unwrap_or(0);
                let quoi = match &ch.nature {
                    NatureChantier::Construction => "recrutement",
                    NatureChantier::Amelioration { .. } => "amélioration",
                };
                lignes.push((
                    format!("⚔ {org} — {}", etat.nom_equipe(e, r)),
                    format!("{quoi}, reste {}", ch.reste()),
                    avance(ch),
                    ch.jours_restants_estimes(limite),
                    Page::Organisations,
                ));
            }
            StatutEquipe::EnDelocalisation(d) => {
                let vers = etat
                    .localites
                    .get(&d.vers)
                    .map(|l| l.nom.clone())
                    .unwrap_or_default();
                let total = (d.preparation + d.voyage).max(1);
                lignes.push((
                    format!("⚔ {org} — {}", etat.nom_equipe(e, r)),
                    format!("délocalisation vers {vers}"),
                    d.jours_ecoules as f32 / total as f32,
                    d.jours_restants(),
                    Page::Organisations,
                ));
            }
            _ => {}
        }
    }
    for t in &etat.transferts {
        let de = etat
            .localites
            .get(&t.de)
            .map(|l| l.nom.clone())
            .unwrap_or_default();
        let vers = etat
            .localites
            .get(&t.vers)
            .map(|l| l.nom.clone())
            .unwrap_or_default();
        lignes.push((
            format!("⏳ {} : {de} → {vers}", t.lot),
            "transfert".into(),
            0.0,
            t.jour_arrivee.saturating_sub(etat.jour),
            Page::Localites,
        ));
    }
    if lignes.is_empty() {
        return None;
    }
    lignes.sort_by_key(|l| l.3);
    let mut aller = None;
    ui.add_space(8.0);
    widgets::titre_section(ui, "En cours");
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::Grid::new("travaux")
            .num_columns(4)
            .striped(true)
            .spacing([14.0, 6.0])
            .show(ui, |ui| {
                for (nom, detail, a, reste, page) in &lignes {
                    if ui
                        .link(RichText::new(nom).color(theme::OR_PALE))
                        .on_hover_text("Ouvrir la page")
                        .clicked()
                    {
                        aller = Some(*page);
                    }
                    widgets::texte_doux(ui, detail);
                    ui.add(egui::ProgressBar::new(*a).desired_width(120.0));
                    if *reste == u32::MAX {
                        ui.label(RichText::new("bloqué (limite nulle)").color(theme::ROUGE));
                    } else {
                        ui.label(format!("≈ {reste} j — {}", cal.libelle(etat.jour + reste)));
                    }
                    ui.end_row();
                }
            });
        widgets::texte_doux(
            ui,
            "Fins estimées avec la limite de dépense et la durée ; elles supposent des réserves suffisantes.",
        );
    });
    aller
}
