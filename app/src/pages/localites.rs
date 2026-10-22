//! Localités : création, modification, soldes, injections, retraits, transferts, ajustements.

use crate::app::App;
use crate::{theme, widgets};
use campagne::evenements::Evenement;
use campagne::modele::{CategorieLocalite, IdLocalite, IdPersonnage, Localite, Lot};
use domaine::{CoutCapitaux, Montant, TypeCapital};
use eframe::egui::{self, RichText};

#[derive(Default, Clone, PartialEq)]
enum ModeCategorie {
    #[default]
    Officielle,
    Personnalisee,
}

#[derive(Default, Clone)]
pub struct Fiche {
    nom: String,
    genre: String,
    population: String,
    mode: ModeCategorie,
    categorie_officielle: String,
    categorie_perso_nom: String,
    categorie_perso_limite: String,
    notes: String,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Action {
    #[default]
    Injection,
    Retrait,
    Transfert,
    Ajustement,
}

#[derive(Default)]
pub struct Etat {
    pub selection: Option<IdLocalite>,
    creation: bool,
    fiche: Fiche,
    fiche_pour: Option<IdLocalite>,
    action: Action,
    pj: Option<IdPersonnage>,
    montant: String,
    destination: Option<IdLocalite>,
    ressource_po: bool,
    capital: Option<TypeCapital>,
    quantite: String,
    duree: String,
    ajout: [String; 4],
    retrait: [String; 4],
    motif: String,
}

impl Fiche {
    fn depuis(l: &Localite) -> Self {
        let mut f = Fiche {
            nom: l.nom.clone(),
            genre: l.genre.clone(),
            population: l.population.map(|p| p.to_string()).unwrap_or_default(),
            notes: l.notes.clone(),
            ..Fiche::default()
        };
        match &l.categorie {
            CategorieLocalite::Officielle { id } => {
                f.mode = ModeCategorie::Officielle;
                f.categorie_officielle = id.clone();
            }
            CategorieLocalite::Personnalisee {
                nom,
                limite_depense_jour,
            } => {
                f.mode = ModeCategorie::Personnalisee;
                f.categorie_perso_nom = nom.clone();
                f.categorie_perso_limite = limite_depense_jour.to_string();
            }
        }
        f
    }

    fn categorie(&self) -> Option<CategorieLocalite> {
        match self.mode {
            ModeCategorie::Officielle if !self.categorie_officielle.is_empty() => {
                Some(CategorieLocalite::Officielle {
                    id: self.categorie_officielle.clone(),
                })
            }
            ModeCategorie::Personnalisee => Some(CategorieLocalite::Personnalisee {
                nom: if self.categorie_perso_nom.trim().is_empty() {
                    "Personnalisée".into()
                } else {
                    self.categorie_perso_nom.trim().into()
                },
                limite_depense_jour: self.categorie_perso_limite.trim().parse().ok()?,
            }),
            _ => None,
        }
    }

    fn population(&self) -> Option<u32> {
        self.population.trim().replace(' ', "").parse().ok()
    }
}

/// Formulaire commun à la création et à la modification. Renvoie true si « Valider ».
fn formulaire(app: &App, f: &mut Fiche, ui: &mut egui::Ui, bouton: &str) -> bool {
    let mut valider = false;
    egui::Grid::new("fiche_localite")
        .num_columns(2)
        .spacing([12.0, 8.0])
        .show(ui, |ui| {
            ui.label("Nom");
            ui.add(egui::TextEdit::singleline(&mut f.nom).desired_width(280.0));
            ui.end_row();
            ui.label("Genre");
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut f.genre)
                        .hint_text("ville, village, lieu-dit…")
                        .desired_width(180.0),
                );
                for g in ["ville", "village", "lieu-dit"] {
                    if ui.small_button(g).clicked() {
                        f.genre = g.into();
                    }
                }
            });
            ui.end_row();
            ui.label("Population");
            ui.horizontal(|ui| {
                let pop = widgets::champ_entier(ui, &mut f.population, "facultatif", 120.0);
                if let Some(cat) = pop.and_then(|p| app.r.categorie_pour_population(p)) {
                    widgets::texte_doux(ui, format!("→ catégorie officielle : {}", cat.nom));
                    if f.mode == ModeCategorie::Officielle && ui.small_button("Appliquer").clicked()
                    {
                        f.categorie_officielle = cat.id.clone();
                    }
                }
            });
            ui.end_row();
            ui.label("Catégorie");
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut f.mode, ModeCategorie::Officielle, "Officielle");
                    ui.radio_value(&mut f.mode, ModeCategorie::Personnalisee, "Personnalisée");
                });
                match f.mode {
                    ModeCategorie::Officielle => {
                        let texte = app
                            .r
                            .communaute(&f.categorie_officielle)
                            .map(|c| format!("{} — {}/jour", c.nom, c.limite_depense_jour))
                            .unwrap_or_else(|| "Choisir…".into());
                        egui::ComboBox::from_id_salt("categorie_officielle")
                            .selected_text(texte)
                            .width(280.0)
                            .show_ui(ui, |ui| {
                                for c in &app.r.communautes {
                                    let pop = match c.population_max {
                                        Some(m) => format!("{} à {}", c.population_min, m),
                                        None => format!(
                                            "plus de {}",
                                            c.population_min.saturating_sub(1)
                                        ),
                                    };
                                    ui.selectable_value(
                                        &mut f.categorie_officielle,
                                        c.id.clone(),
                                        format!(
                                            "{} ({pop} hab.) — {}/jour",
                                            c.nom, c.limite_depense_jour
                                        ),
                                    );
                                }
                            });
                        widgets::texte_doux(ui, "Limite des dépenses : Guide de campagne p. 81");
                    }
                    ModeCategorie::Personnalisee => {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut f.categorie_perso_nom)
                                    .hint_text("nom")
                                    .desired_width(160.0),
                            );
                            ui.label("limite/jour");
                            widgets::champ_entier(
                                ui,
                                &mut f.categorie_perso_limite,
                                "unités",
                                60.0,
                            );
                        });
                        widgets::badge_statut(ui, domaine::Statut::Personnalise);
                    }
                }
            });
            ui.end_row();
            ui.label("Notes");
            ui.add(
                egui::TextEdit::multiline(&mut f.notes)
                    .desired_rows(2)
                    .desired_width(380.0),
            );
            ui.end_row();
            ui.label("");
            let ok = !f.nom.trim().is_empty() && f.categorie().is_some();
            valider = widgets::bouton_principal(ui, bouton, ok);
            ui.end_row();
        });
    valider
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    let etat = c.etat(&app.r).clone();
    let nouvel_id = c.nouvel_id_localite();
    let nouvel_id_transfert = c.nouvel_id_transfert();
    widgets::titre_page(
        ui,
        "Localités",
        "Villes, villages et lieux-dits où vos personnages possèdent des biens",
    );

    ui.columns(2, |cols| {
        // ---- liste ----
        let ui = &mut cols[0];
        ui.set_max_width(320.0);
        if widgets::bouton_principal(ui, "+ Nouvelle localité", true) {
            app.ui_localites.creation = true;
            app.ui_localites.selection = None;
            app.ui_localites.fiche = Fiche {
                mode: ModeCategorie::Officielle,
                ..Fiche::default()
            };
        }
        ui.add_space(6.0);
        for l in etat.localites.values() {
            let choisi = app.ui_localites.selection == Some(l.id);
            let reponse = widgets::carte(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let t = RichText::new(&l.nom).size(16.0).color(if choisi {
                        theme::OR
                    } else {
                        theme::OR_PALE
                    });
                    let r = ui.selectable_label(choisi, t);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        widgets::montant(ui, l.tresorerie, 14.0);
                    });
                    r
                })
                .inner
            });
            if reponse.clicked() {
                app.ui_localites.selection = Some(l.id);
                app.ui_localites.creation = false;
            }
        }

        // ---- détail ----
        let ui = &mut cols[1];
        if app.ui_localites.creation {
            widgets::carte(ui, |ui| {
                widgets::titre_section(ui, "Nouvelle localité");
                let mut f = std::mem::take(&mut app.ui_localites.fiche);
                if formulaire(app, &mut f, ui, "Créer la localité") {
                    let ev = Evenement::LocaliteCreee {
                        id: nouvel_id,
                        nom: f.nom.trim().into(),
                        genre: f.genre.trim().into(),
                        population: f.population(),
                        categorie: f.categorie().expect("validée"),
                        notes: f.notes.clone(),
                    };
                    if app.executer(ev, &format!("Localité « {} » créée", f.nom.trim())) {
                        app.ui_localites.creation = false;
                        app.ui_localites.selection = Some(nouvel_id);
                    }
                }
                app.ui_localites.fiche = f;
            });
            return;
        }
        let Some(l) = app
            .ui_localites
            .selection
            .and_then(|id| etat.localites.get(&id))
            .cloned()
        else {
            widgets::texte_doux(ui, "Sélectionnez une localité ou créez-en une.");
            return;
        };
        detail(app, ui, &etat, &l, nouvel_id_transfert);
    });
}

fn detail(
    app: &mut App,
    ui: &mut egui::Ui,
    etat: &campagne::etat::Etat,
    l: &Localite,
    id_transfert: campagne::modele::IdTransfert,
) {
    // ---- soldes ----
    widgets::carte(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&l.nom)
                    .size(22.0)
                    .color(theme::OR_PALE)
                    .strong(),
            );
            widgets::texte_doux(ui, &l.genre);
            if let Some(p) = l.population {
                widgets::texte_doux(ui, format!("· {p} hab."));
            }
        });
        ui.horizontal(|ui| {
            widgets::texte_doux(ui, super::libelle_categorie(&l.categorie, &app.r));
            if matches!(l.categorie, CategorieLocalite::Personnalisee { .. }) {
                widgets::badge_statut(ui, domaine::Statut::Personnalise);
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Trésorerie");
            widgets::montant(ui, l.tresorerie, 22.0);
        });
        widgets::capitaux(ui, &l.capitaux, false);
        for t in etat
            .transferts
            .iter()
            .filter(|t| t.vers == l.id || t.de == l.id)
        {
            let (sens, autre) = if t.vers == l.id {
                ("arrive de", t.de)
            } else {
                ("part vers", t.vers)
            };
            let nom = etat
                .localites
                .get(&autre)
                .map(|x| x.nom.as_str())
                .unwrap_or("?");
            widgets::texte_doux(
                ui,
                format!(
                    "⏳ {} {sens} {nom} — arrivée le jour {}",
                    t.lot, t.jour_arrivee
                ),
            );
        }
        if !l.notes.is_empty() {
            widgets::texte_doux(ui, &l.notes);
        }
    });
    ui.add_space(8.0);

    // ---- opérations ----
    widgets::carte(ui, |ui| {
        widgets::titre_section(ui, "Opérations");
        let s = &mut app.ui_localites;
        ui.horizontal(|ui| {
            ui.selectable_value(&mut s.action, Action::Injection, "Injection depuis un PJ");
            ui.selectable_value(&mut s.action, Action::Retrait, "Retrait vers un PJ");
            ui.selectable_value(&mut s.action, Action::Transfert, "Transfert");
            ui.selectable_value(&mut s.action, Action::Ajustement, "Ajustement");
        });
        ui.separator();
        let mut ev: Option<(Evenement, String)> = None;
        match s.action {
            Action::Injection | Action::Retrait => {
                let injection = s.action == Action::Injection;
                ui.horizontal(|ui| {
                    ui.label("Personnage");
                    let texte =
                        s.pj.and_then(|id| etat.personnages.get(&id))
                            .map(|p| format!("{} ({})", p.nom, p.compte));
                    egui::ComboBox::from_id_salt("pj_operation")
                        .selected_text(texte.unwrap_or_else(|| "Choisir…".into()))
                        .show_ui(ui, |ui| {
                            for p in etat.personnages.values() {
                                ui.selectable_value(
                                    &mut s.pj,
                                    Some(p.id),
                                    format!("{} ({})", p.nom, p.compte),
                                );
                            }
                        });
                    ui.label("Montant");
                    let m = widgets::champ_montant(ui, &mut s.montant, 160.0);
                    let libelle = if injection { "Injecter" } else { "Retirer" };
                    if widgets::bouton_principal(
                        ui,
                        libelle,
                        m.is_some_and(|m| m.pa > 0) && s.pj.is_some(),
                    ) {
                        let (pj, montant) = (s.pj.expect("choisi"), m.expect("valide"));
                        ev = Some(if injection {
                            (
                                Evenement::InjectionPo {
                                    pj,
                                    localite: l.id,
                                    montant,
                                },
                                format!("{montant} injectés dans {}", l.nom),
                            )
                        } else {
                            (
                                Evenement::RetraitPo {
                                    localite: l.id,
                                    pj,
                                    montant,
                                },
                                format!("{montant} retirés de {}", l.nom),
                            )
                        });
                    }
                });
                if etat.personnages.is_empty() {
                    widgets::texte_doux(ui, "Créez d'abord un personnage (page Personnages).");
                }
            }
            Action::Transfert => {
                ui.horizontal(|ui| {
                    ui.label("Vers");
                    let texte = s
                        .destination
                        .and_then(|id| etat.localites.get(&id))
                        .map(|x| x.nom.clone());
                    egui::ComboBox::from_id_salt("destination")
                        .selected_text(texte.unwrap_or_else(|| "Choisir…".into()))
                        .show_ui(ui, |ui| {
                            for x in etat.localites.values().filter(|x| x.id != l.id) {
                                ui.selectable_value(&mut s.destination, Some(x.id), &x.nom);
                            }
                        });
                    ui.label("Quoi");
                    let texte = if s.ressource_po {
                        "po".to_string()
                    } else {
                        s.capital
                            .map(|c| c.to_string())
                            .unwrap_or("Choisir…".into())
                    };
                    egui::ComboBox::from_id_salt("ressource_transfert")
                        .selected_text(texte)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(s.ressource_po, "po").clicked() {
                                s.ressource_po = true;
                            }
                            for c in [
                                TypeCapital::Marchandises,
                                TypeCapital::Travail,
                                TypeCapital::Magie,
                            ] {
                                if ui
                                    .selectable_label(
                                        !s.ressource_po && s.capital == Some(c),
                                        c.to_string(),
                                    )
                                    .clicked()
                                {
                                    s.ressource_po = false;
                                    s.capital = Some(c);
                                }
                            }
                        });
                });
                ui.horizontal(|ui| {
                    let lot = if s.ressource_po {
                        ui.label("Montant");
                        widgets::champ_montant(ui, &mut s.montant, 140.0).map(Lot::Po)
                    } else {
                        ui.label("Quantité");
                        let q = widgets::champ_entier(ui, &mut s.quantite, "unités", 70.0);
                        s.capital.zip(q).map(|(c, q)| Lot::Capital(c, q))
                    };
                    ui.label("Durée (jours)");
                    let d = widgets::champ_entier(ui, &mut s.duree, "0 = immédiat", 90.0);
                    if widgets::bouton_principal(
                        ui,
                        "Envoyer",
                        lot.is_some() && d.is_some() && s.destination.is_some(),
                    ) {
                        let (lot, duree_jours, vers) = (
                            lot.expect("valide"),
                            d.expect("valide"),
                            s.destination.expect("choisie"),
                        );
                        ev = Some((
                            Evenement::TransfertEnvoye {
                                id: id_transfert,
                                de: l.id,
                                vers,
                                lot,
                                duree_jours,
                            },
                            format!("Transfert de {lot} envoyé"),
                        ));
                    }
                });
                widgets::texte_doux(ui, "L'Influence est attachée à sa communauté et ne se transfère pas (Guide de campagne p. 81).");
            }
            Action::Ajustement => {
                ui.horizontal(|ui| {
                    ui.label("po (±)");
                    widgets::champ_montant(ui, &mut s.montant, 140.0);
                });
                egui::Grid::new("ajustement")
                    .num_columns(5)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("");
                        for t in TypeCapital::TOUS {
                            ui.label(
                                RichText::new(t.to_string())
                                    .color(theme::couleur_ressource(t.into())),
                            );
                        }
                        ui.end_row();
                        ui.label("Ajouter");
                        for i in 0..4 {
                            widgets::champ_entier(ui, &mut s.ajout[i], "0", 60.0);
                        }
                        ui.end_row();
                        ui.label("Retirer");
                        for i in 0..4 {
                            widgets::champ_entier(ui, &mut s.retrait[i], "0", 60.0);
                        }
                        ui.end_row();
                    });
                ui.horizontal(|ui| {
                    ui.label("Motif");
                    ui.add(
                        egui::TextEdit::singleline(&mut s.motif)
                            .hint_text("récompense du MJ, correction…")
                            .desired_width(260.0),
                    );
                    let lire = |v: &[String; 4]| {
                        let n = |i: usize| v[i].trim().parse::<u32>().unwrap_or(0);
                        CoutCapitaux {
                            marchandises: n(0),
                            influence: n(1),
                            travail: n(2),
                            magie: n(3),
                        }
                    };
                    let delta = if s.montant.trim().is_empty() {
                        Some(Montant::ZERO)
                    } else {
                        Montant::depuis_texte(&s.montant)
                    };
                    let (ajout, retrait) = (lire(&s.ajout), lire(&s.retrait));
                    let utile =
                        delta.is_some_and(|d| d.pa != 0) || ajout.unites() + retrait.unites() > 0;
                    if widgets::bouton_principal(
                        ui,
                        "Appliquer",
                        utile && delta.is_some() && !s.motif.trim().is_empty(),
                    ) {
                        ev = Some((
                            Evenement::AjustementLocalite {
                                localite: l.id,
                                delta_po: delta.expect("valide"),
                                ajout,
                                retrait,
                                motif: s.motif.trim().into(),
                            },
                            format!("Ajustement de {} enregistré", l.nom),
                        ));
                    }
                });
            }
        }
        if let Some((e, msg)) = ev {
            if app.executer(e, &msg) {
                let s = &mut app.ui_localites;
                s.montant.clear();
                s.quantite.clear();
                s.motif.clear();
                s.ajout = Default::default();
                s.retrait = Default::default();
            }
        }
    });
    ui.add_space(8.0);

    // ---- modification ----
    egui::CollapsingHeader::new(RichText::new("Modifier la localité").color(theme::OR)).show(
        ui,
        |ui| {
            if app.ui_localites.fiche_pour != Some(l.id) {
                app.ui_localites.fiche = Fiche::depuis(l);
                app.ui_localites.fiche_pour = Some(l.id);
            }
            let mut f = std::mem::take(&mut app.ui_localites.fiche);
            if formulaire(app, &mut f, ui, "Enregistrer les modifications") {
                let ev = Evenement::LocaliteModifiee {
                    id: l.id,
                    nom: f.nom.trim().into(),
                    genre: f.genre.trim().into(),
                    population: f.population(),
                    categorie: f.categorie().expect("validée"),
                    notes: f.notes.clone(),
                };
                app.executer(ev, "Localité modifiée");
            }
            app.ui_localites.fiche = f;
        },
    );
}
