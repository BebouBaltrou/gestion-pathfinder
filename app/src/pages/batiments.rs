//! Bâtiments : étages, salles, construction, achat, amélioration, recettes par salle.

use crate::app::App;
use crate::{theme, widgets};
use campagne::construction::{evenements_depuis_modele, ressources_admises};
use campagne::etat::Etat as EtatCampagne;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::{CoutCapitaux, Ressource};
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct Etat {
    selection: Option<IdBatiment>,
    creation: bool,
    nom: String,
    classement: String,
    localite: Option<IdLocalite>,
    modele: Option<String>,
    // ajout de salle (par étage)
    recherche: String,
    salle_choisie: Option<String>,
    etage_cible: Option<IdEtage>,
    deja_construite: bool,
    // nouvel étage
    nom_etage: String,
    // accélération et amélioration choisies
    facteur: u32,
    amelioration: Option<String>,
}

type Action = Option<(Evenement, String)>;

fn libelle_statut(s: &StatutSalle) -> (String, egui::Color32) {
    match s {
        StatutSalle::AConstruire => ("à construire".into(), theme::TEXTE_DOUX),
        StatutSalle::EnTravaux(c) => {
            let quoi = match &c.nature {
                NatureChantier::Construction => "en construction",
                NatureChantier::Amelioration { .. } => "en amélioration",
            };
            (
                format!("{quoi} — jour {}/{}", c.jours_ecoules, c.duree),
                theme::ORANGE,
            )
        }
        StatutSalle::Construite => ("construite".into(), theme::VERT),
    }
}

fn ligne_bonus(ui: &mut egui::Ui, bonus: &std::collections::BTreeMap<Ressource, u32>) {
    ui.horizontal_wrapped(|ui| {
        if bonus.is_empty() {
            widgets::texte_doux(ui, "aucune recette active");
        }
        for (r, b) in bonus {
            widgets::pastille(ui, *r, format!("+{b}"));
        }
    });
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    let etat = c.etat(&app.r).clone();
    let ids = (
        c.nouvel_id_batiment(),
        c.nouvel_id_etage(),
        c.nouvel_id_salle(),
    );
    let pendant_amelioration = c.options.production_pendant_amelioration;
    widgets::titre_page(
        ui,
        "Bâtiments",
        "Étages et salles, construction (Guide de campagne p. 85 et 92-95), recettes de chaque salle",
    );
    if app.ui_batiments.facteur == 0 {
        app.ui_batiments.facteur = 1;
    }
    let mut action: Action = None;

    ui.columns(2, |cols| {
        // ---------- liste ----------
        let ui = &mut cols[0];
        ui.set_max_width(320.0);
        if widgets::bouton_principal(ui, "+ Nouveau bâtiment", !etat.localites.is_empty()) {
            let s = &mut app.ui_batiments;
            s.creation = true;
            s.selection = None;
            s.localite = s.localite.or(etat.localites.keys().next().copied());
        }
        if etat.localites.is_empty() {
            widgets::texte_doux(ui, "Créez d'abord une localité.");
        }
        for l in etat.localites.values() {
            let bats: Vec<_> = etat.batiments_localite(l.id).collect();
            if bats.is_empty() {
                continue;
            }
            ui.add_space(6.0);
            ui.label(RichText::new(&l.nom).color(theme::OR));
            for b in bats {
                let choisi = app.ui_batiments.selection == Some(b.id);
                let en_travaux = etat
                    .salles_batiment(b.id)
                    .filter(|s| matches!(s.statut, StatutSalle::EnTravaux(_)))
                    .count();
                let texte = if en_travaux > 0 {
                    format!("{}  ⚒{en_travaux}", b.nom)
                } else {
                    b.nom.clone()
                };
                if ui
                    .selectable_label(
                        choisi,
                        RichText::new(texte).size(15.5).color(theme::OR_PALE),
                    )
                    .clicked()
                {
                    app.ui_batiments.selection = Some(b.id);
                    app.ui_batiments.creation = false;
                }
            }
        }

        // ---------- détail ----------
        let ui = &mut cols[1];
        if app.ui_batiments.creation {
            action = creation(app, ui, &etat, ids);
            return;
        }
        let Some(b) = app
            .ui_batiments
            .selection
            .and_then(|id| etat.batiments.get(&id))
            .cloned()
        else {
            widgets::texte_doux(ui, "Sélectionnez un bâtiment ou créez-en un.");
            return;
        };
        action = detail(app, ui, &etat, &b, ids, pendant_amelioration);
    });

    if let Some((ev, msg)) = action {
        if let Evenement::Groupe { evenements, .. } = &ev {
            if let Some(Evenement::BatimentCree { id, .. }) = evenements.first() {
                let id = *id;
                if app.executer(ev, &msg) {
                    app.ui_batiments.creation = false;
                    app.ui_batiments.selection = Some(id);
                }
                return;
            }
        }
        app.executer(ev, &msg);
    }
}

fn creation(
    app: &mut App,
    ui: &mut egui::Ui,
    etat: &EtatCampagne,
    ids: (IdBatiment, IdEtage, IdSalle),
) -> Action {
    let mut action = None;
    let r = app.r.clone();
    let s = &mut app.ui_batiments;
    widgets::carte(ui, |ui| {
        widgets::titre_section(ui, "Nouveau bâtiment");
        egui::Grid::new("creation_batiment")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Localité");
                let texte = s
                    .localite
                    .and_then(|id| etat.localites.get(&id))
                    .map(|l| l.nom.clone());
                egui::ComboBox::from_id_salt("bat_localite")
                    .selected_text(texte.unwrap_or_else(|| "Choisir…".into()))
                    .show_ui(ui, |ui| {
                        for l in etat.localites.values() {
                            ui.selectable_value(&mut s.localite, Some(l.id), &l.nom);
                        }
                    });
                ui.end_row();
                ui.label("Modèle");
                let texte = s
                    .modele
                    .as_ref()
                    .and_then(|m| r.modele_batiment(m))
                    .map(|m| format!("{} — {} ({} po)", m.nom, m.cout, m.prix_po))
                    .unwrap_or_else(|| "aucun (bâtiment vide)".into());
                egui::ComboBox::from_id_salt("bat_modele")
                    .selected_text(texte)
                    .width(360.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut s.modele, None, "aucun (bâtiment vide)");
                        for m in &r.modeles_batiments {
                            ui.selectable_value(
                                &mut s.modele,
                                Some(m.id.clone()),
                                format!("{} — p. {}", m.nom, m.source.page.unwrap_or(0)),
                            );
                        }
                    });
                ui.end_row();
                ui.label("Nom");
                ui.add(
                    egui::TextEdit::singleline(&mut s.nom)
                        .hint_text("ex. Le Dragon vert")
                        .desired_width(260.0),
                );
                ui.end_row();
                ui.label("Classement");
                ui.add(
                    egui::TextEdit::singleline(&mut s.classement)
                        .hint_text("auberge, temple, entrepôt…")
                        .desired_width(260.0),
                );
                ui.end_row();
            });
        if let Some(m) = s.modele.as_ref().and_then(|m| r.modele_batiment(m)) {
            widgets::texte_doux(ui, format!("Salles : {}", m.composition_imprimee));
            widgets::texte_doux(ui, "Toutes les salles sont créées « à construire » au rez-de-chaussée ; réorganisez-les ensuite.");
        }
        ui.add_space(4.0);
        let ok = !s.nom.trim().is_empty() && s.localite.is_some();
        if widgets::bouton_principal(ui, "Créer le bâtiment", ok) {
            let (b, e, sa) = ids;
            let l = s.localite.expect("choisie");
            let nom = s.nom.trim().to_string();
            let (evs, ignores) = match s.modele.as_ref().and_then(|m| r.modele_batiment(m)) {
                Some(m) => {
                    let (mut evs, ig) = evenements_depuis_modele(m, b, l, &nom, e, sa);
                    if !s.classement.trim().is_empty() {
                        if let Some(Evenement::BatimentCree { classement, .. }) = evs.first_mut() {
                            *classement = s.classement.trim().into();
                        }
                    }
                    (evs, ig)
                }
                None => (
                    vec![
                        Evenement::BatimentCree {
                            id: b,
                            localite: l,
                            nom: nom.clone(),
                            classement: s.classement.trim().into(),
                            notes: String::new(),
                            etages_successifs: false,
                            modele: None,
                        },
                        Evenement::EtageAjoute {
                            batiment: b,
                            id: e,
                            nom: "Rez-de-chaussée".into(),
                            mode: ModeChantier::Parallele,
                        },
                    ],
                    Vec::new(),
                ),
            };
            let mut msg = format!("Bâtiment « {nom} » créé");
            if !ignores.is_empty() {
                msg.push_str(&format!(
                    " — extensions du modèle non gérées pour l'instant : {}",
                    ignores.join(", ")
                ));
            }
            action = Some((
                Evenement::Groupe {
                    libelle: format!("Création du bâtiment {nom}"),
                    evenements: evs,
                },
                msg,
            ));
            s.nom.clear();
            s.classement.clear();
        }
    });
    action
}

#[allow(clippy::too_many_lines)]
fn detail(
    app: &mut App,
    ui: &mut egui::Ui,
    etat: &EtatCampagne,
    b: &Batiment,
    ids: (IdBatiment, IdEtage, IdSalle),
    pendant_amelioration: bool,
) -> Action {
    let mut action: Action = None;
    let r = app.r.clone();
    let limite = etat.limite_depense(b.localite, &r).unwrap_or(0);
    let localite = etat
        .localites
        .get(&b.localite)
        .map(|l| l.nom.clone())
        .unwrap_or_default();

    // ---------- en-tête et totaux ----------
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&b.nom)
                    .size(22.0)
                    .color(theme::OR_PALE)
                    .strong(),
            );
            if !b.classement.is_empty() {
                widgets::texte_doux(ui, &b.classement);
            }
            if let Some(m) = b.modele.as_ref().and_then(|m| r.modele_batiment(m)) {
                widgets::badge(ui, &format!("modèle : {}", m.nom), theme::BLEU);
            }
        });
        widgets::texte_doux(
            ui,
            format!("📍 {localite} — limite des dépenses : {limite} unités/jour (PC-14)"),
        );
        let reste = etat.cout_restant(etat.salles_batiment(b.id), &r);
        ui.horizontal(|ui| {
            ui.label("Reste à dépenser :");
            widgets::capitaux(ui, &reste, true);
        });
        ui.label(format!(
            "Durée restante estimée : {} jours",
            etat.duree_restante_batiment(b, &r)
        ));
        ui.horizontal(|ui| {
            ui.label("Recettes (bonus maximum par type) :");
            ligne_bonus(
                ui,
                &etat.bonus_par_ressource(etat.salles_batiment(b.id), &r, pendant_amelioration),
            );
        });
        let mut succ = b.etages_successifs;
        if ui
            .checkbox(&mut succ, "Étages construits l'un après l'autre")
            .on_hover_text("Option personnalisée : sinon, les étages avancent en même temps")
            .changed()
        {
            action = Some((
                Evenement::BatimentModifie {
                    id: b.id,
                    nom: b.nom.clone(),
                    classement: b.classement.clone(),
                    notes: b.notes.clone(),
                    etages_successifs: succ,
                },
                "Bâtiment modifié".into(),
            ));
        }
    });
    ui.add_space(6.0);

    let s = &mut app.ui_batiments;
    ui.horizontal(|ui| {
        ui.label("Accélération des prochains chantiers");
        egui::ComboBox::from_id_salt("facteur")
            .selected_text(if s.facteur <= 1 {
                "aucune".to_string()
            } else {
                format!("×{} Travail, durée ÷{}", s.facteur, s.facteur)
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut s.facteur, 1, "aucune");
                for k in 2..=4 {
                    ui.selectable_value(&mut s.facteur, k, format!("×{k} Travail, durée ÷{k}"));
                }
            });
    })
    .response
    .on_hover_text(
        "Guide de campagne p. 95 (Influence pour les équipes). Arrondi au jour supérieur (PC-10).",
    );

    // ---------- étages ----------
    for etage in &b.etages {
        ui.add_space(6.0);
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&etage.nom)
                        .size(17.0)
                        .color(theme::OR)
                        .strong(),
                );
                let mut mode = etage.mode;
                egui::ComboBox::from_id_salt(("mode", etage.id.0))
                    .selected_text(match mode {
                        ModeChantier::Parallele => "salles en parallèle",
                        ModeChantier::Sequentiel => "salles l'une après l'autre",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut mode,
                            ModeChantier::Parallele,
                            "salles en parallèle",
                        );
                        ui.selectable_value(
                            &mut mode,
                            ModeChantier::Sequentiel,
                            "salles l'une après l'autre",
                        );
                    });
                widgets::badge_statut(ui, domaine::Statut::Personnalise);
                if mode != etage.mode {
                    action = Some((
                        Evenement::EtageModifie {
                            batiment: b.id,
                            id: etage.id,
                            nom: etage.nom.clone(),
                            mode,
                        },
                        format!("Étage « {} » modifié", etage.nom),
                    ));
                }
            });
            let salles: Vec<SalleBatie> = etat.salles_etage(etage.id).cloned().collect();
            ui.horizontal_wrapped(|ui| {
                widgets::texte_doux(
                    ui,
                    format!(
                        "Durée restante : {} j",
                        etat.duree_restante_etage(etage, &r)
                    ),
                );
                ui.label("·");
                widgets::capitaux(ui, &etat.cout_restant(salles.iter(), &r), true);
            });
            ui.separator();

            for sb in &salles {
                let Some(u) = r.salle(&sb.reference) else {
                    continue;
                };
                let nom = etat.nom_salle(sb, &r);
                let (statut, couleur) = libelle_statut(&sb.statut);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(&nom)
                            .size(15.5)
                            .color(theme::OR_PALE)
                            .strong(),
                    );
                    if sb.nom.as_ref().is_some_and(|n| !n.trim().is_empty()) {
                        widgets::texte_doux(ui, &u.nom);
                    }
                    widgets::badge(ui, &statut, couleur);
                    if sb.brisee {
                        widgets::badge(ui, "brisée", theme::ROUGE);
                    }
                    widgets::texte_doux(ui, format!("p. {}", u.source.page.unwrap_or(0)));
                });
                ui.horizontal_wrapped(|ui| {
                    widgets::texte_doux(
                        ui,
                        format!(
                            "Coût : {} ({} po) · {} j · recettes : {}",
                            u.cout
                                .map(|c| c.to_string())
                                .unwrap_or("voir description".into()),
                            u.prix_po.unwrap_or(0),
                            u.duree_jours.unwrap_or(0),
                            u.recettes
                                .as_ref()
                                .map(|x| x.texte.as_str())
                                .unwrap_or("aucune"),
                        ),
                    );
                });

                // ressources cochées, revenus, état brisé
                let admises = ressources_admises(u, &etat.types_du_batiment(b.id, sb.id, &r));
                if !admises.is_empty() {
                    ui.horizontal_wrapped(|ui| {
                        let mut ress = sb.ressources.clone();
                        let mut revenus = sb.revenus_actifs;
                        let mut brisee = sb.brisee;
                        let mut change = false;
                        ui.label("Génère :");
                        for x in &admises {
                            let mut coche = ress.contains(x);
                            if ui
                                .checkbox(
                                    &mut coche,
                                    RichText::new(x.to_string())
                                        .color(theme::couleur_ressource(*x)),
                                )
                                .changed()
                            {
                                change = true;
                                if coche {
                                    ress.push(*x);
                                } else {
                                    ress.retain(|y| y != x);
                                }
                            }
                        }
                        ui.separator();
                        change |= ui.checkbox(&mut revenus, "revenus actifs").changed();
                        change |= ui
                            .checkbox(&mut brisee, "brisée")
                            .on_hover_text("Moitié des recettes (GC p. 92)")
                            .changed();
                        if change {
                            ress.sort();
                            action = Some((
                                Evenement::SalleModifiee {
                                    id: sb.id,
                                    etage: sb.etage,
                                    nom: sb.nom.clone(),
                                    ressources: ress,
                                    revenus_actifs: revenus,
                                    brisee,
                                },
                                format!("{nom} modifiée"),
                            ));
                        }
                    });
                }

                // actions selon le statut
                ui.horizontal_wrapped(|ui| match &sb.statut {
                    StatutSalle::AConstruire => {
                        let lib = if s.facteur > 1 {
                            format!("Lancer la construction (×{})", s.facteur)
                        } else {
                            "Lancer la construction".into()
                        };
                        if ui
                            .add_enabled(u.cout.is_some(), egui::Button::new(lib))
                            .clicked()
                        {
                            action = Some((
                                Evenement::ConstructionLancee {
                                    salle: sb.id,
                                    facteur: s.facteur.max(1),
                                },
                                format!("Construction de {nom} lancée"),
                            ));
                        }
                        if let Some(p) = u.prix_po {
                            if ui
                                .button(format!("Acheter terminée ({p} po)"))
                                .on_hover_text("Payé par la trésorerie de la localité")
                                .clicked()
                            {
                                action = Some((
                                    Evenement::SalleAchetee { salle: sb.id },
                                    format!("{nom} achetée"),
                                ));
                            }
                        }
                        if ui.small_button("Retirer").clicked() {
                            action = Some((
                                Evenement::SalleRetiree { id: sb.id },
                                format!("{nom} retirée"),
                            ));
                        }
                    }
                    StatutSalle::EnTravaux(ch) => {
                        let total = ch.cout.unites().max(1) as f32;
                        let avance = (ch.depense.unites() as f32 / total).min(1.0);
                        ui.add(
                            egui::ProgressBar::new(avance)
                                .desired_width(160.0)
                                .text(format!(
                                    "capitaux {}/{}",
                                    ch.depense.unites(),
                                    ch.cout.unites()
                                )),
                        );
                        let temps = if ch.duree == 0 {
                            1.0
                        } else {
                            (ch.jours_ecoules as f32 / ch.duree as f32).min(1.0)
                        };
                        ui.add(
                            egui::ProgressBar::new(temps)
                                .desired_width(160.0)
                                .text(format!("jours {}/{}", ch.jours_ecoules, ch.duree)),
                        );
                        let j = ch.jours_restants_estimes(limite);
                        widgets::texte_doux(
                            ui,
                            if j == u32::MAX {
                                "bloqué : limite de dépense nulle".to_string()
                            } else {
                                format!("≈ {j} j restants")
                            },
                        );
                        let reste: CoutCapitaux = ch.reste();
                        if reste.unites() > 0 {
                            widgets::texte_doux(ui, format!("manque : {reste}"));
                        }
                    }
                    StatutSalle::Construite => {
                        if !u.amelioration_en.is_empty() {
                            let choix = s
                                .amelioration
                                .clone()
                                .filter(|v| u.amelioration_en.contains(v));
                            let texte = choix
                                .as_ref()
                                .and_then(|v| r.salle(v))
                                .map(|x| x.nom.clone())
                                .unwrap_or("Améliorer en…".into());
                            egui::ComboBox::from_id_salt(("amelioration", sb.id.0))
                                .selected_text(texte)
                                .show_ui(ui, |ui| {
                                    for v in &u.amelioration_en {
                                        let n =
                                            r.salle(v).map(|x| x.nom.clone()).unwrap_or(v.clone());
                                        ui.selectable_value(
                                            &mut s.amelioration,
                                            Some(v.clone()),
                                            n,
                                        );
                                    }
                                });
                            if let Some(v) = choix {
                                if let Some(cible) = r.salle(&v) {
                                    if let Ok(d) = regles::calculs::cout_amelioration(u, cible) {
                                        widgets::texte_doux(
                                            ui,
                                            format!("{} · {} j", d.capitaux, d.duree_jours),
                                        );
                                    }
                                }
                                if ui.button("Lancer l'amélioration").clicked() {
                                    action = Some((
                                        Evenement::AmeliorationLancee {
                                            salle: sb.id,
                                            vers: v.clone(),
                                            facteur: s.facteur.max(1),
                                        },
                                        format!("Amélioration de {nom} lancée"),
                                    ));
                                }
                            }
                        }
                        if ui.small_button("Retirer").clicked() {
                            action = Some((
                                Evenement::SalleRetiree { id: sb.id },
                                format!("{nom} retirée"),
                            ));
                        }
                    }
                });
                ui.add_space(6.0);
            }

            // ajout d'une salle à cet étage
            if s.etage_cible == Some(etage.id) {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("🔎");
                    ui.add(
                        egui::TextEdit::singleline(&mut s.recherche)
                            .hint_text("chercher une salle")
                            .desired_width(180.0),
                    );
                    let texte = s
                        .salle_choisie
                        .as_ref()
                        .and_then(|v| r.salle(v))
                        .map(|x| x.nom.clone())
                        .unwrap_or("Choisir…".into());
                    egui::ComboBox::from_id_salt(("ajout", etage.id.0))
                        .selected_text(texte)
                        .width(260.0)
                        .show_ui(ui, |ui| {
                            let rech = s.recherche.trim().to_lowercase();
                            for x in r
                                .salles
                                .iter()
                                .filter(|x| rech.is_empty() || x.nom.to_lowercase().contains(&rech))
                            {
                                let info = format!(
                                    "{} — {} · {} j · {}",
                                    x.nom,
                                    x.cout
                                        .map(|c| c.to_string())
                                        .unwrap_or("voir description".into()),
                                    x.duree_jours.unwrap_or(0),
                                    x.recettes.as_ref().map(|y| y.texte.as_str()).unwrap_or("—")
                                );
                                ui.selectable_value(&mut s.salle_choisie, Some(x.id.clone()), info);
                            }
                        });
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut s.deja_construite, "déjà construite");
                    if widgets::bouton_principal(ui, "Ajouter la salle", s.salle_choisie.is_some())
                    {
                        let reference = s.salle_choisie.clone().expect("choisie");
                        let n = r
                            .salle(&reference)
                            .map(|x| x.nom.clone())
                            .unwrap_or_default();
                        action = Some((
                            Evenement::SalleAjoutee {
                                id: ids.2,
                                batiment: b.id,
                                etage: etage.id,
                                reference,
                                nom: None,
                                deja_construite: s.deja_construite,
                            },
                            format!("{n} ajoutée à {}", etage.nom),
                        ));
                    }
                    if ui.button("Fermer").clicked() {
                        s.etage_cible = None;
                    }
                });
            } else if ui.button("+ Ajouter une salle").clicked() {
                s.etage_cible = Some(etage.id);
                s.salle_choisie = None;
            }
        });
    }

    // ---------- nouvel étage ----------
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut s.nom_etage)
                .hint_text("nom du nouvel étage")
                .desired_width(200.0),
        );
        if ui
            .add_enabled(
                !s.nom_etage.trim().is_empty(),
                egui::Button::new("+ Ajouter un étage"),
            )
            .clicked()
        {
            action = Some((
                Evenement::EtageAjoute {
                    batiment: b.id,
                    id: ids.1,
                    nom: s.nom_etage.trim().into(),
                    mode: ModeChantier::Parallele,
                },
                "Étage ajouté".into(),
            ));
            s.nom_etage.clear();
        }
    });
    let _ = ids.0;
    action
}
