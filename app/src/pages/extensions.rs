//! Extensions de salle (GC p. 108) : fortification, ameublement, piège. Une de chaque type par
//! salle, permanentes une fois commencées, conservées quand la salle est améliorée.

use crate::{theme, widgets};
use campagne::etat::Etat as EtatCampagne;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::CoutCapitaux;
use eframe::egui::{self, RichText};
use regles::Referentiel;

pub type Action = Option<(Evenement, String)>;

/// Saisie en cours d'une extension (le piège demande coût, prix et durée).
#[derive(Default)]
pub struct Formulaire {
    salle: Option<IdSalle>,
    reference: String,
    deja: bool,
    cout: CoutCapitaux,
    prix: String,
    duree: String,
}

fn libelle_statut(x: &ExtensionSalle) -> (String, egui::Color32) {
    match &x.statut {
        StatutExtension::AConstruire => ("à construire".into(), theme::TEXTE_DOUX),
        StatutExtension::EnTravaux(c) => (
            format!("en construction — jour {}/{}", c.jours_ecoules, c.duree),
            theme::ORANGE,
        ),
        StatutExtension::Construite => ("faite".into(), theme::VERT),
    }
}

/// Ligne « Extensions » d'une salle : badges, actions et boutons d'ajout.
#[allow(clippy::too_many_arguments)]
pub fn extensions_salle(
    ui: &mut egui::Ui,
    f: &mut Formulaire,
    etat: &EtatCampagne,
    r: &Referentiel,
    sb: &SalleBatie,
    payeur: Payeur,
    facteur: u32,
) -> Action {
    let mut action = None;
    let nom_salle = etat.nom_salle(sb, r);
    let nom_ext = |id: &str| r.extension(id).map(|u| u.nom.clone()).unwrap_or(id.into());

    for x in &sb.extensions {
        let n = nom_ext(&x.reference);
        let (statut, couleur) = libelle_statut(x);
        ui.horizontal_wrapped(|ui| {
            widgets::badge(ui, &n, theme::BLEU);
            widgets::badge(ui, &statut, couleur);
            if let Some(u) = r.extension(&x.reference) {
                if let Some(a) = &u.avantages {
                    widgets::texte_doux(ui, a);
                }
                if let Some(rec) = &u.recettes {
                    widgets::texte_doux(ui, &rec.texte);
                }
            }
            widgets::texte_doux(ui, format!("{} ({} po) · {} j", x.cout, x.prix_po, x.duree));
            if x.statut == StatutExtension::AConstruire {
                let lancer = if facteur > 1 {
                    format!("Lancer (×{facteur})")
                } else {
                    "Lancer".into()
                };
                if ui
                    .add_enabled(
                        sb.statut != StatutSalle::AConstruire,
                        egui::Button::new(lancer),
                    )
                    .on_disabled_hover_text("La salle doit d'abord être construite ou lancée")
                    .clicked()
                {
                    action = Some((
                        Evenement::ExtensionLancee {
                            salle: sb.id,
                            reference: x.reference.clone(),
                            facteur: facteur.max(1),
                        },
                        format!("{n} de {nom_salle} lancé(e)"),
                    ));
                }
                if ui
                    .button(format!("Acheter faite ({} po)", x.prix_po))
                    .on_hover_text("Payé par le choix « Achats payés par » ci-dessus")
                    .clicked()
                {
                    action = Some((
                        Evenement::ExtensionAchetee {
                            salle: sb.id,
                            reference: x.reference.clone(),
                            payeur: Some(payeur),
                            prix: None,
                        },
                        format!("{n} de {nom_salle} acheté(e)"),
                    ));
                }
                if ui.small_button("Déjà possédée").clicked() {
                    action = Some((
                        Evenement::ExtensionAchetee {
                            salle: sb.id,
                            reference: x.reference.clone(),
                            payeur: None,
                            prix: Some(domaine::Montant::ZERO),
                        },
                        format!("{n} de {nom_salle} marqué(e) comme possédé(e)"),
                    ));
                }
                if ui.small_button("Retirer").clicked() {
                    action = Some((
                        Evenement::ExtensionRetiree {
                            salle: sb.id,
                            reference: x.reference.clone(),
                        },
                        format!("{n} retiré(e) de {nom_salle}"),
                    ));
                }
            }
            if let StatutExtension::EnTravaux(ch) = &x.statut {
                let reste = ch.reste();
                if reste.unites() > 0 {
                    widgets::texte_doux(ui, format!("manque : {reste}"));
                }
            }
        });
    }

    // boutons d'ajout : une extension de chaque type
    let manquantes: Vec<&domaine::fiches::Unite> = r
        .extensions
        .iter()
        .filter(|u| !sb.extensions.iter().any(|x| x.reference == u.id))
        .collect();
    if !manquantes.is_empty() {
        ui.horizontal_wrapped(|ui| {
            widgets::texte_doux(ui, "Extensions :");
            for u in &manquantes {
                let aide = format!(
                    "{} — GC p. {}",
                    u.avantages
                        .clone()
                        .or(u.recettes.as_ref().map(|x| x.texte.clone()))
                        .unwrap_or("coût, prix et durée à saisir (voir la salle piégée)".into()),
                    u.source.page.unwrap_or(0)
                );
                if ui
                    .small_button(format!("+ {}", u.nom))
                    .on_hover_text(aide)
                    .clicked()
                {
                    *f = Formulaire {
                        salle: Some(sb.id),
                        reference: u.id.clone(),
                        ..Default::default()
                    };
                }
            }
        });
    }

    if f.salle == Some(sb.id) {
        if let Some(u) = r.extension(&f.reference) {
            widgets::carte(ui, |ui| {
                ui.label(
                    RichText::new(format!("{} sur {nom_salle}", u.nom))
                        .color(theme::OR)
                        .strong(),
                );
                let saisie = u.cout.is_none();
                let (mut cout, mut prix, mut duree) = (u.cout, u.prix_po, u.duree_jours);
                if saisie {
                    widgets::texte_doux(
                        ui,
                        "Le livre renvoie à la description du piège (salle piégée, GC p. 106) : \
                         saisissez son coût, son prix et sa durée.",
                    );
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Marchandises");
                        ui.add(egui::DragValue::new(&mut f.cout.marchandises));
                        ui.label("Influence");
                        ui.add(egui::DragValue::new(&mut f.cout.influence));
                        ui.label("Travail");
                        ui.add(egui::DragValue::new(&mut f.cout.travail));
                        ui.label("Magie");
                        ui.add(egui::DragValue::new(&mut f.cout.magie));
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Prix (po)");
                        prix = widgets::champ_entier(ui, &mut f.prix, "po", 80.0);
                        ui.label("Durée (jours)");
                        duree = widgets::champ_entier(ui, &mut f.duree, "j", 60.0);
                    });
                    cout = Some(f.cout);
                    widgets::badge_statut(ui, domaine::Statut::Personnalise);
                } else {
                    widgets::texte_doux(
                        ui,
                        format!(
                            "{} ({} po) · {} j — GC p. {}",
                            u.cout.unwrap_or_default(),
                            u.prix_po.unwrap_or(0),
                            u.duree_jours.unwrap_or(0),
                            u.source.page.unwrap_or(0)
                        ),
                    );
                }
                ui.checkbox(&mut f.deja, "déjà faite (gratuit)");
                let ok = cout.is_some() && prix.is_some() && duree.is_some();
                ui.horizontal(|ui| {
                    if widgets::bouton_principal(ui, "Ajouter l'extension", ok) {
                        action = Some((
                            Evenement::ExtensionAjoutee {
                                salle: sb.id,
                                reference: u.id.clone(),
                                deja_construite: f.deja,
                                cout: if saisie { cout } else { None },
                                prix_po: if saisie { prix } else { None },
                                duree: if saisie { duree } else { None },
                            },
                            format!("{} ajouté(e) à {nom_salle}", u.nom),
                        ));
                        f.salle = None;
                    }
                    if ui.button("Annuler").clicked() {
                        f.salle = None;
                    }
                });
            });
        }
    }
    action
}
