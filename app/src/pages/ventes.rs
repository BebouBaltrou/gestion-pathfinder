//! Vendre, abandonner (GC p. 84) et réparer (GC p. 92) : éléments réutilisés par les pages
//! Bâtiments et Organisations.

use crate::{theme, widgets};
use campagne::etat::Etat as EtatCampagne;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::{CoutCapitaux, Montant};
use eframe::egui::{self, RichText};
use regles::Referentiel;

pub type Action = Option<(Evenement, String)>;

#[derive(Default)]
pub struct Formulaire {
    vente: Option<Actif>,
    abandon: Option<Actif>,
    sans_acheteur: bool,
    po: String,
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
            .on_hover_text(
                "Moitié du prix d'achat, acheteur trouvé en 3d6 jours dans 75 % des cas (GC p. 84)",
            )
            .clicked()
        {
            let (po, _) = etat.produit_vente(cible, r);
            f.vente = Some(cible);
            f.abandon = None;
            f.sans_acheteur = false;
            f.po = po.to_string();
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
                    "Prix d'achat des parties construites ou recrutées : {prix} (coût {cout}). \
                     Produit de la vente : la moitié du prix d'achat, en po (GC p. 84)."
                ),
            );
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut f.sans_acheteur,
                    false,
                    "Acheteur trouvé (75 % des cas)",
                );
                ui.radio_value(&mut f.sans_acheteur, true, "Aucun acheteur (25 %)");
            });
            if f.sans_acheteur {
                widgets::texte_doux(
                    ui,
                    "Personne ne reprend le bien : vous le gardez, ou vous l'abandonnez sans compensation.",
                );
                ui.horizontal(|ui| {
                    if ui.button("Garder le bien").clicked() {
                        action = Some((
                            Evenement::Note {
                                texte: format!("Aucun acheteur pour {libelle} : bien conservé"),
                            },
                            format!("Aucun acheteur : {libelle} conservé"),
                        ));
                        f.vente = None;
                    }
                    if ui
                        .button(RichText::new("Abandonner le bien").color(theme::ROUGE))
                        .clicked()
                    {
                        action = Some((
                            Evenement::AbandonActif { cible },
                            format!("Aucun acheteur : {libelle} abandonné"),
                        ));
                        f.vente = None;
                    }
                    if ui.button("Annuler").clicked() {
                        f.vente = None;
                    }
                });
                return;
            }
            let mut po = None;
            ui.horizontal(|ui| {
                ui.label("Produit");
                po = widgets::champ_montant(ui, &mut f.po, 140.0).filter(|m| !m.est_negatif());
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("Jours pour trouver l'acheteur");
                widgets::champ_entier(ui, &mut f.jours, "3d6", 60.0);
                ui.label("Influence dépensée");
                ui.add(egui::DragValue::new(&mut f.influence).range(0..=influence_dispo));
            });
            widgets::texte_doux(
                ui,
                "Lancez 3d6, puis retirez 1d6 par point d'Influence dépensé (minimum 0). \
                 Le bien reste à vous jusqu'à la vente.",
            );
            let jours = f.jours.trim().parse::<u32>().ok();
            ui.horizontal(|ui| {
                let ok = jours.is_some() && po.is_some();
                if widgets::bouton_principal(ui, "Mettre en vente", ok) {
                    action = Some((
                        Evenement::VenteLancee {
                            cible,
                            jours: jours.expect("saisi"),
                            influence: f.influence,
                            po: po.expect("valide"),
                            capitaux: CoutCapitaux::default(),
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
