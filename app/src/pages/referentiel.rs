//! Consultation du référentiel officiel : chaque donnée avec sa source et son statut.

use crate::app::App;
use crate::{theme, widgets};
use domaine::fiches::{Admissibles, ElementModele, Modele, Taille, Unite};
use domaine::Statut;
use eframe::egui::{self, RichText};

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Onglet {
    #[default]
    Salles,
    Equipes,
    Extensions,
    Gestionnaires,
    Batiments,
    Organisations,
    Communautes,
    Capitaux,
}

#[derive(Default)]
pub struct Etat {
    onglet: Onglet,
    recherche: String,
    selection: Option<String>,
}

fn correspond(nom: &str, recherche: &str) -> bool {
    recherche.trim().is_empty()
        || nom
            .to_lowercase()
            .contains(&recherche.trim().to_lowercase())
}

fn texte_recettes(u: &Unite) -> String {
    match &u.recettes {
        None => "—".into(),
        Some(r) => match &r.admissibles {
            Admissibles::Liste { .. }
            | Admissibles::Toutes
            | Admissibles::TypesDuBatiment
            | Admissibles::SalleHote => r.texte.clone(),
        },
    }
}

fn texte_taille(t: &Option<Taille>) -> String {
    match t {
        Some(Taille::Cases { min, max }) => format!("{min}-{max} cases"),
        Some(Taille::Personnes { nombre }) => format!("{nombre} pers."),
        Some(Taille::Texte { texte }) => texte.clone(),
        None => "—".into(),
    }
}

fn statut(app: &App, pc: &Option<String>) -> Statut {
    match pc.as_deref().and_then(|c| app.r.point(c)) {
        Some(p) if p.statut == domaine::fiches::StatutPoint::Ouvert => Statut::AConfirmer,
        _ => Statut::Officiel,
    }
}

fn table_unites(app: &mut App, ui: &mut egui::Ui, unites: &[Unite], id: &str) {
    let rech = app.ui_referentiel.recherche.clone();
    egui::Grid::new(id)
        .striped(true)
        .num_columns(6)
        .spacing([14.0, 5.0])
        .show(ui, |ui| {
            for h in ["Nom", "Recettes", "Création", "Prix", "Durée", "Page"] {
                ui.label(RichText::new(h).color(theme::OR));
            }
            ui.end_row();
            for u in unites.iter().filter(|u| correspond(&u.nom, &rech)) {
                let choisi = app.ui_referentiel.selection.as_deref() == Some(&u.id);
                if ui
                    .selectable_label(choisi, RichText::new(&u.nom).color(theme::OR_PALE))
                    .clicked()
                {
                    app.ui_referentiel.selection = if choisi { None } else { Some(u.id.clone()) };
                }
                ui.label(texte_recettes(u));
                ui.label(
                    u.cout
                        .map(|c| c.to_string())
                        .unwrap_or("voir description".into()),
                );
                ui.label(u.prix_po.map(|p| format!("{p} po")).unwrap_or("—".into()));
                ui.label(
                    u.duree_jours
                        .map(|d| format!("{d} j"))
                        .unwrap_or("—".into()),
                );
                ui.label(u.source.page.map(|p| format!("p. {p}")).unwrap_or_default());
                ui.end_row();
            }
        });
}

fn detail_unite(app: &App, ui: &mut egui::Ui, u: &Unite, liste: &[Unite]) {
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&u.nom)
                    .size(20.0)
                    .color(theme::OR_PALE)
                    .strong(),
            );
            widgets::badge_statut(ui, statut(app, &u.point_a_confirmer));
            if let Some(pc) = &u.point_a_confirmer {
                widgets::badge(ui, pc, theme::ORANGE);
            }
        });
        widgets::texte_doux(ui, format!("Source : {}", app.r.reference(&u.source)));
        egui::Grid::new(("detail", &u.id))
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.label("Recettes");
                ui.label(texte_recettes(u));
                ui.end_row();
                if let Some(a) = &u.avantages {
                    ui.label("Avantages");
                    ui.label(a);
                    ui.end_row();
                }
                ui.label("Création");
                ui.label(
                    u.cout
                        .map(|c| c.to_string())
                        .unwrap_or("voir description".into()),
                );
                ui.end_row();
                ui.label("Achat terminé");
                ui.label(u.prix_po.map(|p| format!("{p} po")).unwrap_or("—".into()));
                ui.end_row();
                ui.label("Durée");
                ui.label(
                    u.duree_jours
                        .map(|d| format!("{d} jours"))
                        .unwrap_or("—".into()),
                );
                ui.end_row();
                ui.label("Taille");
                ui.label(texte_taille(&u.taille));
                ui.end_row();
                let noms = |ids: &[String]| {
                    ids.iter()
                        .map(|i| {
                            liste
                                .iter()
                                .find(|x| &x.id == i)
                                .map(|x| x.nom.clone())
                                .unwrap_or(i.clone())
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                if !u.amelioration_de.is_empty() {
                    ui.label("Amélioration de");
                    ui.label(noms(&u.amelioration_de));
                    ui.end_row();
                }
                if !u.amelioration_en.is_empty() {
                    ui.label("Amélioration en");
                    ui.label(noms(&u.amelioration_en));
                    ui.end_row();
                }
            });
    });
}

fn table_modeles(app: &mut App, ui: &mut egui::Ui, modeles: &[Modele], id: &str, unites: &[Unite]) {
    let rech = app.ui_referentiel.recherche.clone();
    for m in modeles.iter().filter(|m| correspond(&m.nom, &rech)) {
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&m.nom)
                        .size(17.0)
                        .color(theme::OR_PALE)
                        .strong(),
                );
                widgets::badge_statut(ui, statut(app, &m.point_a_confirmer));
                if let Some(pc) = &m.point_a_confirmer {
                    widgets::badge(ui, pc, theme::ORANGE);
                }
                widgets::texte_doux(ui, format!("p. {}", m.source.page.unwrap_or(0)));
            });
            ui.horizontal(|ui| {
                ui.label(format!("{}  ({} po)", m.cout, m.prix_po));
            });
            let elements: Vec<String> = m
                .composition
                .iter()
                .map(|e| match e {
                    ElementModele::Unite { id, quantite } => {
                        let nom = unites
                            .iter()
                            .find(|u| &u.id == id)
                            .map(|u| u.nom.as_str())
                            .unwrap_or(id);
                        format!("{quantite} × {nom}")
                    }
                    ElementModele::Extension { id, quantite, sur } => {
                        format!("{quantite} × extension {id} ({sur})")
                    }
                })
                .collect();
            ui.label(RichText::new(elements.join(" · ")).color(theme::TEXTE));
            if m.point_a_confirmer.is_some() {
                widgets::texte_doux(ui, format!("Texte imprimé : {}", m.composition_imprimee));
            }
        });
        ui.add_space(3.0);
    }
    let _ = id;
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    widgets::titre_page(
        ui,
        "Référentiel",
        "Données officielles du Guide de campagne (VF d'Ultimate Campaign), avec leur page",
    );
    ui.horizontal_wrapped(|ui| {
        let s = &mut app.ui_referentiel;
        for (o, t) in [
            (Onglet::Salles, "Salles"),
            (Onglet::Equipes, "Équipes"),
            (Onglet::Extensions, "Extensions"),
            (Onglet::Gestionnaires, "Gestionnaires"),
            (Onglet::Batiments, "Modèles de bâtiments"),
            (Onglet::Organisations, "Modèles d'organisations"),
            (Onglet::Communautes, "Communautés"),
            (Onglet::Capitaux, "Capitaux"),
        ] {
            if ui.selectable_value(&mut s.onglet, o, t).clicked() {
                s.selection = None;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("🔎");
        ui.add(
            egui::TextEdit::singleline(&mut app.ui_referentiel.recherche)
                .hint_text("Rechercher par nom")
                .desired_width(260.0),
        );
    });
    ui.add_space(6.0);

    let r = app.r.clone();
    match app.ui_referentiel.onglet {
        o @ (Onglet::Salles | Onglet::Equipes | Onglet::Extensions) => {
            let liste = match o {
                Onglet::Salles => &r.salles,
                Onglet::Equipes => &r.equipes,
                _ => &r.extensions,
            };
            if let Some(u) = app
                .ui_referentiel
                .selection
                .as_ref()
                .and_then(|id| liste.iter().find(|u| &u.id == id))
            {
                detail_unite(app, ui, u, liste);
                ui.add_space(6.0);
            }
            table_unites(app, ui, liste, "table_unites");
        }
        Onglet::Gestionnaires => {
            widgets::texte_doux(
                ui,
                "Salaire payé chaque jour en po (Guide de campagne p. 89-92).",
            );
            let rech = app.ui_referentiel.recherche.clone();
            egui::Grid::new("gestionnaires")
                .striped(true)
                .num_columns(4)
                .spacing([14.0, 5.0])
                .show(ui, |ui| {
                    for h in ["Nom", "Salaire", "Compétences", "Page"] {
                        ui.label(RichText::new(h).color(theme::OR));
                    }
                    ui.end_row();
                    for g in r.gestionnaires.iter().filter(|g| correspond(&g.nom, &rech)) {
                        ui.label(RichText::new(&g.nom).color(theme::OR_PALE));
                        ui.label(format!("{} po/jour", g.salaire_po_jour));
                        ui.label(&g.competences);
                        ui.label(g.source.page.map(|p| format!("p. {p}")).unwrap_or_default());
                        ui.end_row();
                    }
                });
        }
        Onglet::Batiments => table_modeles(app, ui, &r.modeles_batiments, "bat", &r.salles),
        Onglet::Organisations => {
            table_modeles(app, ui, &r.modeles_organisations, "org", &r.equipes)
        }
        Onglet::Communautes => {
            egui::Grid::new("communautes")
                .striped(true)
                .num_columns(4)
                .spacing([14.0, 5.0])
                .show(ui, |ui| {
                    for h in [
                        "Catégorie",
                        "Population",
                        "Limite des dépenses / jour",
                        "Statut",
                    ] {
                        ui.label(RichText::new(h).color(theme::OR));
                    }
                    ui.end_row();
                    for c in &r.communautes {
                        ui.label(RichText::new(&c.nom).color(theme::OR_PALE));
                        ui.label(match c.population_max {
                            Some(m) => format!("{} à {}", c.population_min, m),
                            None => format!("plus de {}", c.population_min.saturating_sub(1)),
                        });
                        ui.label(format!("{} unités", c.limite_depense_jour));
                        ui.horizontal(|ui| {
                            widgets::badge_statut(ui, statut(app, &c.point_a_confirmer));
                            if let Some(pc) = &c.point_a_confirmer {
                                widgets::badge(ui, pc, theme::ORANGE);
                            }
                        });
                        ui.end_row();
                    }
                });
            widgets::texte_doux(ui, "Population : Guide du maître (profil d'une communauté). Limite : Guide de campagne p. 81.");
        }
        Onglet::Capitaux => {
            egui::Grid::new("capitaux")
                .striped(true)
                .num_columns(3)
                .spacing([14.0, 5.0])
                .show(ui, |ui| {
                    for h in ["Capital", "Coût d'achat", "Coût produit"] {
                        ui.label(RichText::new(h).color(theme::OR));
                    }
                    ui.end_row();
                    for v in &r.capitaux {
                        ui.label(
                            RichText::new(v.ressource.to_string())
                                .color(theme::couleur_ressource(v.ressource)),
                        );
                        ui.label(format!("{} po", v.cout_achat_po));
                        ui.label(format!("{} po", v.cout_produit_po));
                        ui.end_row();
                    }
                });
            widgets::texte_doux(ui, "Table 2-1 : Valeurs des capitaux (Guide de campagne p. 78). Produire coûte la moitié de l'achat.");
        }
    }
}
