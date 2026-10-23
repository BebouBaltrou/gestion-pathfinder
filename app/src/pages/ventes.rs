//! Vendre, abandonner (GC p. 84) et réparer (GC p. 92) : éléments réutilisés par les pages
//! Bâtiments et Organisations.

use crate::{theme, widgets};
use campagne::etat::Etat as EtatCampagne;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::{CoutCapitaux, Montant, TypeCapital};
use eframe::egui::{self, RichText};
use regles::Referentiel;

pub type Action = Option<(Evenement, String)>;

#[derive(Default)]
pub struct Formulaire {
    vente: Option<Actif>,
    abandon: Option<Actif>,
    en_capitaux: bool,
    po: String,
    capitaux: CoutCapitaux,
    jours: String,
    influence: u32,
}

/// Boutons « Vendre… » / « Abandonner… » d'un bien, avec leurs formulaires.
pub fn actions_actif(
    ui: &mut egui::Ui,
    f: &mut Formulaire,
    etat: &EtatCampagne,
    r: &Referentiel,
    cible: Actif,
) -> Action {
    let mut action = None;
    let libelle = etat.libelle_actif(cible, r);
    if let Some(v) = etat.en_vente(cible) {
        ui.horizontal_wrapped(|ui| {
            widgets::badge(
                ui,
                &format!(
                    "en vente : acheteur dans {} j",
                    v.jour_fin.saturating_sub(etat.jour)
                ),
                theme::BLEU,
            );
            let cap = if v.capitaux.unites() > 0 {
                format!(" + {}", v.capitaux)
            } else {
                String::new()
            };
            widgets::texte_doux(ui, format!("produit : {}{cap}", v.po));
            if ui.small_button("Annuler la vente").clicked() {
                action = Some((
                    Evenement::VenteAnnulee { cible },
                    format!("Vente annulée : {libelle}"),
                ));
            }
        });
        return action;
    }
    ui.horizontal_wrapped(|ui| {
        if ui
            .small_button("Vendre…")
            .on_hover_text("Moitié du prix ou du coût, acheteur trouvé en 3d6 jours (GC p. 84)")
            .clicked()
        {
            let (po, cap) = etat.produit_vente(cible, r);
            f.vente = Some(cible);
            f.abandon = None;
            f.en_capitaux = false;
            f.po = po.to_string();
            f.capitaux = cap;
            f.jours.clear();
            f.influence = 0;
        }
        if ui
            .small_button("Abandonner…")
            .on_hover_text("Sans compensation, immédiat et définitif (GC p. 84)")
            .clicked()
        {
            f.abandon = Some(cible);
            f.vente = None;
        }
    });

    if f.abandon == Some(cible) {
        widgets::carte(ui, |ui| {
            ui.label(
                RichText::new(format!("Abandonner {libelle} ?"))
                    .color(theme::ORANGE)
                    .strong(),
            );
            widgets::texte_doux(
                ui,
                "Sans aucune compensation ; le bien disparaît (annulable avec Ctrl+Z).",
            );
            ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("Abandonner").color(theme::ROUGE))
                    .clicked()
                {
                    action = Some((
                        Evenement::AbandonActif { cible },
                        format!("Abandon : {libelle}"),
                    ));
                    f.abandon = None;
                }
                if ui.button("Annuler").clicked() {
                    f.abandon = None;
                }
            });
        });
    }

    if f.vente == Some(cible) {
        let (prix, cout) = etat.valeur_actif(cible, r);
        let influence_dispo = etat
            .localite_actif(cible)
            .ok()
            .and_then(|l| etat.localites.get(&l))
            .map(|l| l.capitaux.influence)
            .unwrap_or(0);
        widgets::carte(ui, |ui| {
            ui.label(
                RichText::new(format!("Vendre {libelle}"))
                    .color(theme::OR)
                    .strong(),
            );
            widgets::texte_doux(
                ui,
                format!(
                    "Valeur des parties construites ou recrutées : {prix} ; {cout}. Le livre donne « la moitié \
                     de son prix d'achat et de son coût de construction » (en po ou en capitaux)."
                ),
            );
            ui.horizontal(|ui| {
                ui.radio_value(&mut f.en_capitaux, false, "Produit en po");
                ui.radio_value(&mut f.en_capitaux, true, "Produit en capitaux");
            });
            let mut po = Some(Montant::ZERO);
            if f.en_capitaux {
                ui.horizontal_wrapped(|ui| {
                    for t in TypeCapital::TOUS {
                        ui.label(
                            RichText::new(t.to_string()).color(theme::couleur_ressource(t.into())),
                        );
                        ui.add(egui::DragValue::new(f.capitaux.get_mut(t)).range(0..=9999));
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    ui.label("Montant");
                    po = widgets::champ_montant(ui, &mut f.po, 140.0).filter(|m| !m.est_negatif());
                });
            }
            ui.horizontal_wrapped(|ui| {
                ui.label("Jours pour trouver un acheteur");
                widgets::champ_entier(ui, &mut f.jours, "3d6", 60.0);
                ui.label("Influence dépensée");
                ui.add(egui::DragValue::new(&mut f.influence).range(0..=influence_dispo));
            });
            widgets::texte_doux(
                ui,
                "Lancez 3d6 (75 % des cas, le reste est à voir avec le MJ), puis retirez 1d6 par point \
                 d'Influence dépensé (minimum 0). Le bien reste à vous jusqu'à la vente.",
            );
            let jours = f.jours.trim().parse::<u32>().ok();
            ui.horizontal(|ui| {
                let ok = jours.is_some() && po.is_some();
                if widgets::bouton_principal(ui, "Mettre en vente", ok) {
                    let (po, capitaux) = if f.en_capitaux {
                        (Montant::ZERO, f.capitaux)
                    } else {
                        (po.expect("valide"), CoutCapitaux::default())
                    };
                    action = Some((
                        Evenement::VenteLancee {
                            cible,
                            jours: jours.expect("saisi"),
                            influence: f.influence,
                            po,
                            capitaux,
                        },
                        format!("Mise en vente : {libelle}"),
                    ));
                    f.vente = None;
                }
                if ui.button("Annuler").clicked() {
                    f.vente = None;
                }
            });
        });
    }
    action
}

/// Boutons de réparation d'une salle brisée (moitié du prix ou du coût, GC p. 92).
pub fn reparation(
    ui: &mut egui::Ui,
    etat: &EtatCampagne,
    r: &Referentiel,
    salle: &SalleBatie,
    payeur: Payeur,
) -> Action {
    let mut action = None;
    if !salle.brisee {
        return None;
    }
    let (po, cap) = etat.cout_reparation(salle.id, r);
    let nom = etat.nom_salle(salle, r);
    ui.horizontal_wrapped(|ui| {
        if ui
            .button(format!("Réparer ({po})"))
            .on_hover_text(
                "Moitié du prix de construction, payée par le choix « Achats payés par »",
            )
            .clicked()
        {
            action = Some((
                Evenement::SalleReparee {
                    salle: salle.id,
                    payeur: Some(payeur),
                    po,
                    capitaux: CoutCapitaux::default(),
                },
                format!("{nom} réparée"),
            ));
        }
        if cap.unites() > 0
            && ui
                .button(format!("Réparer en capitaux ({cap})"))
                .on_hover_text(
                    "Moitié du coût (arrondi inférieur), pris dans les réserves de la localité",
                )
                .clicked()
        {
            action = Some((
                Evenement::SalleReparee {
                    salle: salle.id,
                    payeur: None,
                    po: Montant::ZERO,
                    capitaux: cap,
                },
                format!("{nom} réparée"),
            ));
        }
    });
    action
}
