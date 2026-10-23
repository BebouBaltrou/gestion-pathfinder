//! Organisations et équipes : appartenance, lieu de travail, recrutement, achat, amélioration,
//! délocalisation (Guide de campagne p. 92-95).

use super::batiments::{choix_facteur_capital, choix_payeur, Acquisition};
use crate::app::App;
use crate::{theme, widgets};
use campagne::construction::ressources_admises;
use campagne::etat::Etat as EtatCampagne;
use campagne::evenements::Evenement;
use campagne::modele::*;
use campagne::organisation::evenements_depuis_modele_organisation;
use domaine::{CoutCapitaux, Montant, Ressource};
use eframe::egui::{self, RichText};
use std::collections::BTreeSet;

#[derive(Default)]
pub struct Etat {
    selection: Option<IdOrganisation>,
    creation: bool,
    nom: String,
    classement: String,
    localite: Option<IdLocalite>,
    modele: Option<String>,
    acquisition: Acquisition,
    prix: String,
    seulement_manquants: bool,
    payeur_pj: Option<IdPersonnage>,
    facteur: u32,
    // ajout d'équipe
    ajout_ouvert: bool,
    recherche: String,
    equipe_choisie: Option<String>,
    deja_recrutee: bool,
    amelioration: Option<String>,
    // délocalisation
    deloc: Option<IdEquipe>,
    deloc_vers: Option<IdLocalite>,
    deloc_prep: String,
    deloc_voyage: String,
    deloc_prix: String,
    deloc_org: bool,
    vente: super::ventes::Formulaire,
}

type Action = Option<(Evenement, String)>;

fn libelle_acquisition(a: Acquisition) -> &'static str {
    match a {
        Acquisition::Plan => "Seulement la liste (équipes à recruter, rien n'est lancé)",
        Acquisition::Possede => "Déjà possédée (gratuit, équipes recrutées)",
        Acquisition::Construire => "Recruter avec les capitaux de la localité",
        Acquisition::AcheterCapitaux => "Acheter les capitaux en po, puis recruter",
        Acquisition::AcheterBatiment => {
            "Acheter l'organisation constituée (prix en po, sans durée)"
        }
    }
}

fn libelle_statut(s: &StatutEquipe, etat: &EtatCampagne) -> (String, egui::Color32) {
    match s {
        StatutEquipe::ARecruter => ("à recruter".into(), theme::TEXTE_DOUX),
        StatutEquipe::EnRecrutement(c) => {
            let quoi = match &c.nature {
                NatureChantier::Construction => "en recrutement",
                NatureChantier::Amelioration { .. } => "en amélioration",
            };
            (
                format!("{quoi} — jour {}/{}", c.jours_ecoules, c.duree),
                theme::ORANGE,
            )
        }
        StatutEquipe::Recrutee => ("recrutée".into(), theme::VERT),
        StatutEquipe::EnDelocalisation(d) => {
            let vers = etat
                .localites
                .get(&d.vers)
                .map(|l| l.nom.clone())
                .unwrap_or_default();
            let phase = if d.en_voyage() {
                "en voyage (aucune recette)"
            } else {
                "préparation (½ recettes)"
            };
            (
                format!(
                    "délocalisation vers {vers} — {phase}, {} j",
                    d.jours_restants()
                ),
                theme::BLEU,
            )
        }
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
    let ids = (c.nouvel_id_organisation(), c.nouvel_id_equipe());
    let opts = (
        c.options.production_pendant_amelioration,
        c.options.moitie_equipe_sans_batiment,
        c.options.delocalisation_jet_unique,
    );
    widgets::titre_page(
        ui,
        "Organisations",
        "Équipes d'employés : appartenance, lieu de travail, recrutement (Guide de campagne p. 92-95)",
    );
    if app.ui_organisations.facteur == 0 {
        app.ui_organisations.facteur = 1;
    }
    let mut action: Action = None;

    ui.columns(2, |cols| {
        let ui = &mut cols[0];
        ui.set_max_width(320.0);
        if widgets::bouton_principal(ui, "+ Nouvelle organisation", !etat.localites.is_empty()) {
            let s = &mut app.ui_organisations;
            s.creation = true;
            s.selection = None;
            s.localite = s.localite.or(etat.localites.keys().next().copied());
        }
        if etat.localites.is_empty() {
            widgets::texte_doux(ui, "Créez d'abord une localité.");
        }
        for l in etat.localites.values() {
            let orgs: Vec<_> = etat.organisations_localite(l.id).collect();
            if orgs.is_empty() {
                continue;
            }
            ui.add_space(6.0);
            ui.label(RichText::new(&l.nom).color(theme::OR));
            for o in orgs {
                let choisi = app.ui_organisations.selection == Some(o.id);
                let n = etat.equipes_organisation(o.id).count();
                if ui
                    .selectable_label(
                        choisi,
                        RichText::new(format!("{}  ·  {n} équipe(s)", o.nom))
                            .size(15.5)
                            .color(theme::OR_PALE),
                    )
                    .clicked()
                {
                    app.ui_organisations.selection = Some(o.id);
                    app.ui_organisations.creation = false;
                }
            }
        }

        let ui = &mut cols[1];
        if app.ui_organisations.creation {
            action = creation(app, ui, &etat, ids);
            return;
        }
        let Some(o) = app
            .ui_organisations
            .selection
            .and_then(|id| etat.organisations.get(&id))
            .cloned()
        else {
            widgets::texte_doux(ui, "Sélectionnez une organisation ou créez-en une.");
            return;
        };
        action = detail(app, ui, &etat, &o, ids.1, opts);
    });

    if let Some((ev, msg)) = action {
        if let Evenement::Groupe { evenements, .. } = &ev {
            if let Some(Evenement::OrganisationCreee { id, .. }) = evenements.first() {
                let id = *id;
                if app.executer(ev, &msg) {
                    app.ui_organisations.creation = false;
                    app.ui_organisations.selection = Some(id);
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
    ids: (IdOrganisation, IdEquipe),
) -> Action {
    let mut action = None;
    let r = app.r.clone();
    let s = &mut app.ui_organisations;
    widgets::carte(ui, |ui| {
        widgets::titre_section(ui, "Nouvelle organisation");
        egui::Grid::new("creation_org")
            .num_columns(2)
            .spacing([12.0, 8.0])
            .show(ui, |ui| {
                ui.label("Localité");
                let texte = s
                    .localite
                    .and_then(|id| etat.localites.get(&id))
                    .map(|l| l.nom.clone());
                egui::ComboBox::from_id_salt("org_localite")
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
                    .and_then(|m| r.modele_organisation(m))
                    .map(|m| format!("{} — {} ({} po)", m.nom, m.cout, m.prix_po))
                    .unwrap_or_else(|| "aucun (organisation vide)".into());
                egui::ComboBox::from_id_salt("org_modele")
                    .selected_text(texte)
                    .width(360.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut s.modele, None, "aucun (organisation vide)");
                        for m in &r.modeles_organisations {
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
                        .hint_text("ex. Les Ombres du port")
                        .desired_width(260.0),
                );
                ui.end_row();
                ui.label("Classement");
                ui.add(
                    egui::TextEdit::singleline(&mut s.classement)
                        .hint_text("guilde, culte, compagnie…")
                        .desired_width(260.0),
                );
                ui.end_row();
            });

        let modele = s
            .modele
            .as_ref()
            .and_then(|m| r.modele_organisation(m))
            .cloned();
        let mut paiement: Option<(Payeur, Montant)> = None;
        let mut achat_capitaux: Option<(Payeur, CoutCapitaux)> = None;
        let mut ok_paiement = true;
        if let (Some(m), Some(l)) = (&modele, s.localite) {
            widgets::texte_doux(ui, format!("Équipes : {}", m.composition_imprimee));
            ui.add_space(4.0);
            ui.label(RichText::new("Acquisition").color(theme::OR).strong());
            for a in Acquisition::TOUTES {
                ui.radio_value(&mut s.acquisition, a, libelle_acquisition(a));
            }
            let besoin = {
                let mut c = m.cout;
                if matches!(
                    s.acquisition,
                    Acquisition::Construire | Acquisition::AcheterCapitaux
                ) {
                    c.influence *= s.facteur.max(1);
                }
                c
            };
            ui.add_space(4.0);
            match s.acquisition {
                Acquisition::Plan => widgets::texte_doux(
                    ui,
                    "Les équipes sont créées « à recruter », sans lieu de travail ; lancez-les ensuite une par une.",
                ),
                Acquisition::Possede => widgets::texte_doux(
                    ui,
                    "Toutes les équipes sont créées recrutées, sans rien payer.",
                ),
                Acquisition::Construire | Acquisition::AcheterCapitaux => {
                    ui.horizontal(|ui| {
                        ui.label("Accélération");
                        choix_facteur_capital(ui, "facteur_org", &mut s.facteur, "Influence");
                    });
                    ui.horizontal(|ui| {
                        ui.label("Capitaux nécessaires :");
                        widgets::capitaux(ui, &besoin, true);
                    });
                    widgets::texte_doux(
                        ui,
                        "Un PJ doit être présent. Chaque jour, les recrutements puisent dans les réserves de la localité, \
                         dans la limite de dépense (partagée avec les chantiers).",
                    );
                    if s.acquisition == Acquisition::AcheterCapitaux {
                        ui.checkbox(
                            &mut s.seulement_manquants,
                            "N'acheter que ce qui manque dans les réserves",
                        );
                        let reserves = etat
                            .localites
                            .get(&l)
                            .map(|x| x.capitaux)
                            .unwrap_or_default();
                        let mut a_acheter = besoin;
                        if s.seulement_manquants {
                            for t in domaine::TypeCapital::TOUS {
                                *a_acheter.get_mut(t) = besoin.get(t).saturating_sub(reserves.get(t));
                            }
                        }
                        let prix = regles::calculs::valeur_achat(&r, &a_acheter);
                        ui.horizontal(|ui| {
                            ui.label("À acheter :");
                            widgets::capitaux(ui, &a_acheter, true);
                            ui.label(
                                RichText::new(format!("= {prix} po"))
                                    .color(theme::OR_PALE)
                                    .strong(),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Payé par");
                            let p = choix_payeur(ui, "payeur_org", &mut s.payeur_pj, etat, l);
                            if a_acheter.unites() > 0 {
                                achat_capitaux = Some((p, a_acheter));
                            }
                        });
                    }
                }
                Acquisition::AcheterBatiment => {
                    ui.horizontal(|ui| {
                        ui.label("Prix payé");
                        let saisi = widgets::champ_montant(ui, &mut s.prix, 140.0);
                        widgets::texte_doux(
                            ui,
                            format!("prix du livre : {} po (vide = ce prix)", m.prix_po),
                        );
                        let montant = if s.prix.trim().is_empty() {
                            Some(Montant::po(m.prix_po as i64))
                        } else {
                            saisi.filter(|x| !x.est_negatif())
                        };
                        ok_paiement = montant.is_some();
                        if let Some(x) = montant {
                            paiement = Some((Payeur::Localite(l), x));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Payé par");
                        let p = choix_payeur(ui, "payeur_org", &mut s.payeur_pj, etat, l);
                        if let Some((q, _)) = paiement.as_mut() {
                            *q = p;
                        }
                    });
                }
            }
        }
        ui.add_space(4.0);
        let ok = !s.nom.trim().is_empty() && s.localite.is_some() && ok_paiement;
        if widgets::bouton_principal(ui, "Créer l'organisation", ok) {
            let (o, q) = ids;
            let l = s.localite.expect("choisie");
            let nom = s.nom.trim().to_string();
            let (evs, ignores) = match &modele {
                Some(m) => {
                    let (mut evs, ig) = evenements_depuis_modele_organisation(m, o, l, &nom, q, &r);
                    if !s.classement.trim().is_empty() {
                        if let Some(Evenement::OrganisationCreee { classement, .. }) =
                            evs.first_mut()
                        {
                            *classement = s.classement.trim().into();
                        }
                    }
                    let equipes: Vec<IdEquipe> = evs
                        .iter()
                        .filter_map(|x| match x {
                            Evenement::EquipeAjoutee { id, .. } => Some(*id),
                            _ => None,
                        })
                        .collect();
                    match s.acquisition {
                        Acquisition::Plan => {}
                        Acquisition::Possede | Acquisition::AcheterBatiment => {
                            for x in evs.iter_mut() {
                                if let Evenement::EquipeAjoutee { deja_recrutee, .. } = x {
                                    *deja_recrutee = true;
                                }
                            }
                            if let Some((payeur, montant)) = paiement.filter(|(_, x)| x.pa > 0) {
                                evs.push(Evenement::Paiement {
                                    payeur,
                                    montant,
                                    motif: format!("achat de l'organisation {nom}"),
                                });
                            }
                        }
                        Acquisition::Construire | Acquisition::AcheterCapitaux => {
                            if let Some((payeur, capitaux)) = achat_capitaux {
                                evs.push(Evenement::CapitauxAchetes {
                                    localite: l,
                                    payeur,
                                    capitaux,
                                });
                            }
                            for id in equipes {
                                evs.push(Evenement::RecrutementLance {
                                    equipe: id,
                                    facteur: s.facteur.max(1),
                                });
                            }
                        }
                    }
                    (evs, ig)
                }
                None => (
                    vec![Evenement::OrganisationCreee {
                        id: o,
                        localite: l,
                        nom: nom.clone(),
                        classement: s.classement.trim().into(),
                        notes: String::new(),
                        modele: None,
                    }],
                    Vec::new(),
                ),
            };
            let mut msg = format!("Organisation « {nom} » créée");
            if !ignores.is_empty() {
                msg.push_str(&format!(
                    " — éléments du modèle non gérés : {}",
                    ignores.join(", ")
                ));
            }
            action = Some((
                Evenement::Groupe {
                    libelle: format!("Création de l'organisation {nom}"),
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
    o: &Organisation,
    nouvelle_equipe: IdEquipe,
    (pendant_amelioration, moitie_sans_batiment, jet_unique): (bool, bool, bool),
) -> Action {
    let mut action: Action = None;
    let r = app.r.clone();
    let siege = etat
        .localites
        .get(&o.localite)
        .map(|l| l.nom.clone())
        .unwrap_or_default();
    let equipes: Vec<Equipe> = etat.equipes_organisation(o.id).cloned().collect();

    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&o.nom)
                    .size(22.0)
                    .color(theme::OR_PALE)
                    .strong(),
            );
            if !o.classement.is_empty() {
                widgets::texte_doux(ui, &o.classement);
            }
            if let Some(m) = o.modele.as_ref().and_then(|m| r.modele_organisation(m)) {
                widgets::badge(ui, &format!("modèle : {}", m.nom), theme::BLEU);
            }
        });
        widgets::texte_doux(ui, format!("📍 siège : {siege}"));
        ui.horizontal(|ui| {
            ui.label("Reste à dépenser :");
            widgets::capitaux(ui, &etat.cout_restant_equipes(equipes.iter(), &r), true);
        });
        ui.horizontal(|ui| {
            ui.label("Recettes (bonus maximum par type) :");
            ligne_bonus(
                ui,
                &etat.bonus_equipes(
                    equipes.iter(),
                    &r,
                    pendant_amelioration,
                    moitie_sans_batiment,
                ),
            );
        });
        let g: Vec<String> = etat
            .gestionnaires_de(Affectation::Organisation(o.id))
            .map(|g| g.nom.clone())
            .collect();
        widgets::texte_doux(
            ui,
            if g.is_empty() {
                "Aucun gestionnaire (page Gestionnaires pour en affecter un)".to_string()
            } else {
                format!("Gestionnaire(s) : {}", g.join(", "))
            },
        );
    });
    if let Some(a) = super::ventes::actions_actif(
        ui,
        &mut app.ui_organisations.vente,
        etat,
        &r,
        Actif::Organisation(o.id),
    ) {
        action = Some(a);
    }
    ui.add_space(6.0);

    let s = &mut app.ui_organisations;
    let mut payeur = Payeur::Localite(o.localite);
    ui.horizontal_wrapped(|ui| {
        ui.label("Accélération des prochains recrutements");
        choix_facteur_capital(ui, "facteur_eq", &mut s.facteur, "Influence");
        ui.separator();
        ui.label("Achats payés par");
        payeur = choix_payeur(ui, "payeur_eq", &mut s.payeur_pj, etat, o.localite);
    });

    for e in &equipes {
        let Some(u) = r.equipe(&e.reference) else {
            continue;
        };
        let nom = etat.nom_equipe(e, &r);
        let (statut, couleur) = libelle_statut(&e.statut, etat);
        ui.add_space(6.0);
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(&nom)
                        .size(16.0)
                        .color(theme::OR_PALE)
                        .strong(),
                );
                if e.nom.as_ref().is_some_and(|n| !n.trim().is_empty()) {
                    widgets::texte_doux(ui, &u.nom);
                }
                widgets::badge(ui, &statut, couleur);
                if e.localite != o.localite {
                    let l = etat
                        .localites
                        .get(&e.localite)
                        .map(|l| l.nom.clone())
                        .unwrap_or_default();
                    widgets::badge(ui, &format!("à {l}"), theme::BLEU);
                }
                widgets::texte_doux(ui, format!("p. {}", u.source.page.unwrap_or(0)));
            });
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

            // appartenance et lieu de travail
            ui.horizontal_wrapped(|ui| {
                let mut org = e.organisation;
                let mut lieu = e.lieu;
                ui.label("Membre de");
                egui::ComboBox::from_id_salt(("org", e.id.0))
                    .selected_text(
                        etat.organisations
                            .get(&org)
                            .map(|x| x.nom.clone())
                            .unwrap_or_default(),
                    )
                    .show_ui(ui, |ui| {
                        for x in etat.organisations.values() {
                            ui.selectable_value(&mut org, x.id, &x.nom);
                        }
                    });
                ui.label("travaille");
                let actif = !matches!(e.statut, StatutEquipe::EnDelocalisation(_));
                ui.add_enabled_ui(actif, |ui| {
                    egui::ComboBox::from_id_salt(("lieu", e.id.0))
                        .selected_text(match lieu.and_then(|b| etat.batiments.get(&b)) {
                            Some(b) => format!("à {}", b.nom),
                            None => "sans bâtiment".to_string(),
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut lieu,
                                None,
                                if moitie_sans_batiment {
                                    "sans bâtiment (recettes ÷ 2)"
                                } else {
                                    "sans bâtiment"
                                },
                            );
                            for b in etat.batiments_localite(e.localite) {
                                ui.selectable_value(&mut lieu, Some(b.id), format!("à {}", b.nom));
                            }
                        });
                });
                if org != e.organisation || lieu != e.lieu {
                    action = Some((
                        Evenement::EquipeAffectee {
                            id: e.id,
                            organisation: org,
                            lieu,
                        },
                        format!("{nom} réaffectée"),
                    ));
                }
            });

            // ressources produites
            let admises = ressources_admises(u, &BTreeSet::new());
            if !admises.is_empty() {
                ui.horizontal_wrapped(|ui| {
                    let mut ress = e.ressources.clone();
                    let mut revenus = e.revenus_actifs;
                    let mut change = false;
                    ui.label("Génère :");
                    for x in &admises {
                        let mut coche = ress.contains(x);
                        if ui
                            .checkbox(
                                &mut coche,
                                RichText::new(x.to_string()).color(theme::couleur_ressource(*x)),
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
                    if change {
                        ress.sort();
                        action = Some((
                            Evenement::EquipeModifiee {
                                id: e.id,
                                nom: e.nom.clone(),
                                ressources: ress,
                                revenus_actifs: revenus,
                            },
                            format!("{nom} modifiée"),
                        ));
                    }
                });
            }

            // actions selon le statut
            ui.horizontal_wrapped(|ui| match &e.statut {
                StatutEquipe::ARecruter => {
                    let lib = if s.facteur > 1 {
                        format!("Lancer le recrutement (×{})", s.facteur)
                    } else {
                        "Lancer le recrutement".into()
                    };
                    if ui.add_enabled(u.cout.is_some(), egui::Button::new(lib)).clicked() {
                        action = Some((
                            Evenement::RecrutementLance {
                                equipe: e.id,
                                facteur: s.facteur.max(1),
                            },
                            format!("Recrutement de {nom} lancé"),
                        ));
                    }
                    if let Some(p) = u.prix_po {
                        if ui
                            .button(format!("Acquérir d'un coup ({p} po)"))
                            .on_hover_text("Payé par le choix « Achats payés par » ci-dessus (GC p. 93)")
                            .clicked()
                        {
                            action = Some((
                                Evenement::EquipeAchetee {
                                    equipe: e.id,
                                    payeur: Some(payeur),
                                    prix: None,
                                },
                                format!("{nom} acquise"),
                            ));
                        }
                    }
                    if ui
                        .button("Déjà recrutée")
                        .on_hover_text("Marque l'équipe recrutée sans rien payer")
                        .clicked()
                    {
                        action = Some((
                            Evenement::EquipeAchetee {
                                equipe: e.id,
                                payeur: None,
                                prix: Some(Montant::ZERO),
                            },
                            format!("{nom} marquée comme recrutée"),
                        ));
                    }
                    if ui.small_button("Retirer").clicked() {
                        action = Some((Evenement::EquipeRetiree { id: e.id }, format!("{nom} retirée")));
                    }
                }
                StatutEquipe::EnRecrutement(ch) => {
                    let total = ch.cout.unites().max(1) as f32;
                    ui.add(
                        egui::ProgressBar::new((ch.depense.unites() as f32 / total).min(1.0))
                            .desired_width(160.0)
                            .text(format!("capitaux {}/{}", ch.depense.unites(), ch.cout.unites())),
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
                    let reste = ch.reste();
                    if reste.unites() > 0 {
                        widgets::texte_doux(ui, format!("manque : {reste}"));
                    }
                }
                StatutEquipe::Recrutee => {
                    if !u.amelioration_en.is_empty() {
                        let choix = s
                            .amelioration
                            .clone()
                            .filter(|v| u.amelioration_en.contains(v));
                        let texte = choix
                            .as_ref()
                            .and_then(|v| r.equipe(v))
                            .map(|x| x.nom.clone())
                            .unwrap_or("Améliorer en…".into());
                        egui::ComboBox::from_id_salt(("amel", e.id.0))
                            .selected_text(texte)
                            .show_ui(ui, |ui| {
                                for v in &u.amelioration_en {
                                    let n = r.equipe(v).map(|x| x.nom.clone()).unwrap_or(v.clone());
                                    ui.selectable_value(&mut s.amelioration, Some(v.clone()), n);
                                }
                            });
                        if let Some(v) = choix {
                            if let Some(cible) = r.equipe(&v) {
                                if let Ok(d) = regles::calculs::cout_amelioration(u, cible) {
                                    widgets::texte_doux(ui, format!("{} · {} j", d.capitaux, d.duree_jours));
                                }
                            }
                            if ui.button("Lancer l'amélioration").clicked() {
                                action = Some((
                                    Evenement::AmeliorationEquipeLancee {
                                        equipe: e.id,
                                        vers: v.clone(),
                                        facteur: s.facteur.max(1),
                                    },
                                    format!("Amélioration de {nom} lancée"),
                                ));
                            }
                        }
                    }
                    if ui.button("Délocaliser…").clicked() {
                        s.deloc = Some(e.id);
                        s.deloc_vers = None;
                        s.deloc_prep.clear();
                        s.deloc_voyage.clear();
                        s.deloc_prix = u
                            .prix_po
                            .map(|p| Montant::pa(p as i64 * 5).to_string())
                            .unwrap_or_default();
                    }
                    if ui.small_button("Retirer").clicked() {
                        action = Some((Evenement::EquipeRetiree { id: e.id }, format!("{nom} retirée")));
                    }
                }
                StatutEquipe::EnDelocalisation(_) => {
                    widgets::texte_doux(ui, "L'équipe arrivera sans lieu de travail ; affectez-la ensuite à un bâtiment.");
                }
            });

            if matches!(
                e.statut,
                StatutEquipe::Recrutee | StatutEquipe::EnDelocalisation(_)
            ) {
                if let Some(a) =
                    super::ventes::actions_actif(ui, &mut s.vente, etat, &r, Actif::Equipe(e.id))
                {
                    action = Some(a);
                }
            }

            // formulaire de délocalisation
            if s.deloc == Some(e.id) && e.statut == StatutEquipe::Recrutee {
                ui.separator();
                widgets::texte_doux(
                    ui,
                    "Délocaliser (GC p. 95) : moitié du prix de recrutement, 1d6 jours de préparation \
                     par groupe (½ recettes), puis le temps du voyage (aucune recette).",
                );
                egui::Grid::new(("deloc", e.id.0))
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Vers");
                        egui::ComboBox::from_id_salt(("deloc_vers", e.id.0))
                            .selected_text(
                                s.deloc_vers
                                    .and_then(|l| etat.localites.get(&l))
                                    .map(|l| l.nom.clone())
                                    .unwrap_or("Choisir…".into()),
                            )
                            .show_ui(ui, |ui| {
                                for l in etat.localites.values().filter(|l| l.id != e.localite) {
                                    ui.selectable_value(&mut s.deloc_vers, Some(l.id), &l.nom);
                                }
                            });
                        ui.end_row();
                        ui.label("Préparation (jet de 1d6)");
                        widgets::champ_entier(ui, &mut s.deloc_prep, "1 à 6", 80.0);
                        ui.end_row();
                        ui.label("Voyage (jours)");
                        widgets::champ_entier(ui, &mut s.deloc_voyage, "ex. 4", 80.0);
                        ui.end_row();
                        ui.label("Prix");
                        widgets::champ_montant(ui, &mut s.deloc_prix, 140.0);
                        ui.end_row();
                    });
                let prep = s
                    .deloc_prep
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|x| (1..=6).contains(x));
                let voyage = s.deloc_voyage.trim().parse::<u32>().ok();
                let prix = if s.deloc_prix.trim().is_empty() {
                    Some(Montant::ZERO)
                } else {
                    Montant::depuis_texte(&s.deloc_prix).filter(|m| !m.est_negatif())
                };
                ui.horizontal(|ui| {
                    let ok = s.deloc_vers.is_some()
                        && prep.is_some()
                        && voyage.is_some()
                        && prix.is_some();
                    if widgets::bouton_principal(ui, "Lancer la délocalisation", ok) {
                        action = Some((
                            Evenement::DelocalisationLancee {
                                equipe: e.id,
                                vers: s.deloc_vers.expect("choisie"),
                                preparation: prep.expect("saisie"),
                                voyage: voyage.expect("saisi"),
                                payeur,
                                prix: prix.expect("saisi"),
                            },
                            format!("Délocalisation de {nom} lancée"),
                        ));
                        s.deloc = None;
                    }
                    if ui.button("Annuler").clicked() {
                        s.deloc = None;
                    }
                    widgets::texte_doux(ui, "Payé par le choix « Achats payés par » ci-dessus.");
                });
            }
        });
    }

    // délocalisation de toute l'organisation (règle personnalisée : un seul jet)
    let recrutees: Vec<&Equipe> = equipes
        .iter()
        .filter(|e| e.statut == StatutEquipe::Recrutee)
        .collect();
    if jet_unique && recrutees.len() > 1 {
        ui.add_space(6.0);
        if !s.deloc_org {
            if ui
                .button(format!(
                    "Délocaliser toute l'organisation ({} équipes recrutées)…",
                    recrutees.len()
                ))
                .clicked()
            {
                s.deloc_org = true;
                s.deloc = None;
                s.deloc_vers = None;
                s.deloc_prep.clear();
                s.deloc_voyage.clear();
                let total: i64 = recrutees
                    .iter()
                    .filter_map(|e| r.equipe(&e.reference).and_then(|u| u.prix_po))
                    .map(|p| p as i64 * 5)
                    .sum();
                s.deloc_prix = Montant::pa(total).to_string();
            }
        } else {
            widgets::carte(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    widgets::titre_section(ui, "Délocaliser toute l'organisation");
                    widgets::badge_statut(ui, domaine::Statut::Personnalise);
                });
                widgets::texte_doux(
                    ui,
                    "Règle personnalisée : un seul jet de 1d6 pour la préparation de toutes les équipes recrutées. \
                     Prix proposé : la moitié de leurs prix de recrutement (GC p. 95).",
                );
                egui::Grid::new("deloc_org")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Vers");
                        egui::ComboBox::from_id_salt("deloc_org_vers")
                            .selected_text(
                                s.deloc_vers
                                    .and_then(|l| etat.localites.get(&l))
                                    .map(|l| l.nom.clone())
                                    .unwrap_or("Choisir…".into()),
                            )
                            .show_ui(ui, |ui| {
                                for l in etat.localites.values() {
                                    ui.selectable_value(&mut s.deloc_vers, Some(l.id), &l.nom);
                                }
                            });
                        ui.end_row();
                        ui.label("Préparation (jet de 1d6)");
                        widgets::champ_entier(ui, &mut s.deloc_prep, "1 à 6", 80.0);
                        ui.end_row();
                        ui.label("Voyage (jours)");
                        widgets::champ_entier(ui, &mut s.deloc_voyage, "ex. 4", 80.0);
                        ui.end_row();
                        ui.label("Prix total");
                        widgets::champ_montant(ui, &mut s.deloc_prix, 140.0);
                        ui.end_row();
                    });
                let prep = s
                    .deloc_prep
                    .trim()
                    .parse::<u32>()
                    .ok()
                    .filter(|x| (1..=6).contains(x));
                let voyage = s.deloc_voyage.trim().parse::<u32>().ok();
                let prix = if s.deloc_prix.trim().is_empty() {
                    Some(Montant::ZERO)
                } else {
                    Montant::depuis_texte(&s.deloc_prix).filter(|m| !m.est_negatif())
                };
                // les équipes déjà dans la localité de destination ne bougent pas
                let a_deplacer: Vec<&&Equipe> = recrutees
                    .iter()
                    .filter(|e| Some(e.localite) != s.deloc_vers)
                    .collect();
                ui.horizontal(|ui| {
                    let ok = s.deloc_vers.is_some()
                        && prep.is_some()
                        && voyage.is_some()
                        && prix.is_some()
                        && !a_deplacer.is_empty();
                    if widgets::bouton_principal(ui, "Lancer la délocalisation", ok) {
                        let vers = s.deloc_vers.expect("choisie");
                        let evs = a_deplacer
                            .iter()
                            .enumerate()
                            .map(|(i, e)| Evenement::DelocalisationLancee {
                                equipe: e.id,
                                vers,
                                preparation: prep.expect("saisie"),
                                voyage: voyage.expect("saisi"),
                                payeur,
                                // le prix total est porté par la première équipe
                                prix: if i == 0 {
                                    prix.expect("saisi")
                                } else {
                                    Montant::ZERO
                                },
                            })
                            .collect();
                        action = Some((
                            Evenement::Groupe {
                                libelle: format!("Délocalisation de l'organisation {}", o.nom),
                                evenements: evs,
                            },
                            format!("Délocalisation de {} lancée", o.nom),
                        ));
                        s.deloc_org = false;
                    }
                    if ui.button("Annuler").clicked() {
                        s.deloc_org = false;
                    }
                    widgets::texte_doux(ui, "Payé par le choix « Achats payés par » ci-dessus.");
                });
            });
        }
    }

    // ajout d'une équipe
    ui.add_space(6.0);
    if s.ajout_ouvert {
        widgets::carte(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label("🔎");
                ui.add(
                    egui::TextEdit::singleline(&mut s.recherche)
                        .hint_text("chercher une équipe")
                        .desired_width(180.0),
                );
                let texte = s
                    .equipe_choisie
                    .as_ref()
                    .and_then(|v| r.equipe(v))
                    .map(|x| x.nom.clone())
                    .unwrap_or("Choisir…".into());
                egui::ComboBox::from_id_salt("ajout_equipe")
                    .selected_text(texte)
                    .width(280.0)
                    .show_ui(ui, |ui| {
                        let rech = s.recherche.trim().to_lowercase();
                        for x in r
                            .equipes
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
                            ui.selectable_value(&mut s.equipe_choisie, Some(x.id.clone()), info);
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.checkbox(&mut s.deja_recrutee, "déjà recrutée");
                if widgets::bouton_principal(ui, "Ajouter l'équipe", s.equipe_choisie.is_some()) {
                    let reference = s.equipe_choisie.clone().expect("choisie");
                    let n = r
                        .equipe(&reference)
                        .map(|x| x.nom.clone())
                        .unwrap_or_default();
                    action = Some((
                        Evenement::EquipeAjoutee {
                            id: nouvelle_equipe,
                            organisation: o.id,
                            reference,
                            nom: None,
                            lieu: None,
                            deja_recrutee: s.deja_recrutee,
                        },
                        format!("{n} ajoutée à {}", o.nom),
                    ));
                }
                if ui.button("Fermer").clicked() {
                    s.ajout_ouvert = false;
                }
            });
        });
    } else if ui.button("+ Ajouter une équipe").clicked() {
        s.ajout_ouvert = true;
        s.equipe_choisie = None;
    }
    action
}
