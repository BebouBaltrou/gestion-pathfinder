//! Personnages : création, compte personnel, déplacements, Prestige.

use crate::app::App;
use crate::{theme, widgets};
use campagne::evenements::Evenement;
use campagne::modele::{IdLocalite, IdPersonnage};
use domaine::Montant;
use eframe::egui::{self, RichText};

#[derive(Default)]
pub struct Etat {
    selection: Option<IdPersonnage>,
    nom: String,
    compte: String,
    prestige: String,
    edition_pour: Option<IdPersonnage>,
    edit_nom: String,
    edit_prestige: String,
    ajustement: String,
    motif: String,
}

fn lire_prestige(s: &str) -> Result<Option<i32>, ()> {
    let t = s.trim();
    if t.is_empty() {
        Ok(None)
    } else {
        t.parse().map(Some).map_err(|_| ())
    }
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    let etat = c.etat(&app.r).clone();
    let nouvel_id = c.nouvel_id_personnage();
    widgets::titre_page(
        ui,
        "Personnages",
        "Chaque PJ a un compte en po séparé des trésoreries des localités",
    );

    widgets::carte(ui, |ui| {
        widgets::titre_section(ui, "Nouveau personnage");
        let s = &mut app.ui_personnages;
        let mut creer = None;
        ui.horizontal(|ui| {
            ui.label("Nom");
            ui.add(egui::TextEdit::singleline(&mut s.nom).desired_width(180.0));
            ui.label("Compte initial");
            let saisie = widgets::champ_montant(ui, &mut s.compte, 130.0);
            let m = if s.compte.trim().is_empty() { Some(Montant::ZERO) } else { saisie };
            ui.label("Prestige");
            ui.add(egui::TextEdit::singleline(&mut s.prestige).hint_text("facultatif").desired_width(70.0))
                .on_hover_text("Valeur de Prestige, ou niveau + modificateur de Charisme sans le don (Guide de campagne p. 93)");
            let p = lire_prestige(&s.prestige);
            if widgets::bouton_principal(ui, "Créer", !s.nom.trim().is_empty() && m.is_some() && p.is_ok()) {
                creer = Some(Evenement::PersonnageCree {
                    id: nouvel_id,
                    nom: s.nom.trim().into(),
                    compte_initial: m.expect("valide"),
                    prestige: p.expect("valide"),
                });
            }
        });
        if let Some(ev) = creer {
            let nom = app.ui_personnages.nom.trim().to_string();
            if app.executer(ev, &format!("Personnage « {nom} » créé")) {
                let s = &mut app.ui_personnages;
                s.nom.clear();
                s.compte.clear();
                s.prestige.clear();
                s.selection = Some(nouvel_id);
            }
        }
    });
    ui.add_space(8.0);

    for p in etat.personnages.values() {
        let choisi = app.ui_personnages.selection == Some(p.id);
        widgets::carte(ui, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        choisi,
                        RichText::new(&p.nom)
                            .size(18.0)
                            .color(theme::OR_PALE)
                            .strong(),
                    )
                    .clicked()
                {
                    app.ui_personnages.selection = if choisi { None } else { Some(p.id) };
                }
                widgets::montant(ui, p.compte, 18.0);
                let lieu = p
                    .localite
                    .and_then(|id| etat.localites.get(&id))
                    .map(|l| l.nom.clone());
                widgets::texte_doux(
                    ui,
                    format!("📍 {}", lieu.unwrap_or_else(|| "hors localité".into())),
                );
                if let Some(pr) = p.prestige {
                    widgets::texte_doux(ui, format!("· Prestige {pr}"));
                }
            });
            if !choisi {
                return;
            }
            ui.separator();
            let mut ev: Option<(Evenement, String)> = None;
            let s = &mut app.ui_personnages;
            if s.edition_pour != Some(p.id) {
                s.edition_pour = Some(p.id);
                s.edit_nom = p.nom.clone();
                s.edit_prestige = p.prestige.map(|x| x.to_string()).unwrap_or_default();
            }
            ui.horizontal(|ui| {
                ui.label("Se trouve à");
                let mut lieu: Option<IdLocalite> = p.localite;
                let texte = lieu
                    .and_then(|id| etat.localites.get(&id))
                    .map(|l| l.nom.clone())
                    .unwrap_or("hors localité".into());
                egui::ComboBox::from_id_salt(("lieu", p.id.0))
                    .selected_text(texte)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut lieu, None, "hors localité");
                        for l in etat.localites.values() {
                            ui.selectable_value(&mut lieu, Some(l.id), &l.nom);
                        }
                    });
                if lieu != p.localite {
                    ev = Some((
                        Evenement::PersonnageDeplace {
                            id: p.id,
                            vers: lieu,
                        },
                        format!("{} s'est déplacé", p.nom),
                    ));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Gain / dépense");
                let d = widgets::champ_montant(ui, &mut s.ajustement, 130.0);
                ui.add(
                    egui::TextEdit::singleline(&mut s.motif)
                        .hint_text("motif (butin, achat…)")
                        .desired_width(200.0),
                );
                if widgets::bouton_principal(
                    ui,
                    "Appliquer",
                    d.is_some_and(|d| d.pa != 0) && !s.motif.trim().is_empty(),
                ) {
                    ev = Some((
                        Evenement::AjustementCompte {
                            pj: p.id,
                            delta: d.expect("valide"),
                            motif: s.motif.trim().into(),
                        },
                        format!("Compte de {} mis à jour", p.nom),
                    ));
                }
            });
            widgets::texte_doux(ui, "Montant négatif pour une dépense (ex. « -25 »).");
            ui.horizontal(|ui| {
                ui.label("Nom");
                ui.add(egui::TextEdit::singleline(&mut s.edit_nom).desired_width(160.0));
                ui.label("Prestige");
                ui.add(egui::TextEdit::singleline(&mut s.edit_prestige).desired_width(60.0));
                let pr = lire_prestige(&s.edit_prestige);
                if ui
                    .add_enabled(
                        !s.edit_nom.trim().is_empty() && pr.is_ok(),
                        egui::Button::new("Enregistrer"),
                    )
                    .clicked()
                {
                    ev = Some((
                        Evenement::PersonnageModifie {
                            id: p.id,
                            nom: s.edit_nom.trim().into(),
                            prestige: pr.expect("valide"),
                        },
                        "Personnage modifié".into(),
                    ));
                }
            });
            if let Some((e, msg)) = ev {
                if app.executer(e, &msg) {
                    app.ui_personnages.ajustement.clear();
                    app.ui_personnages.motif.clear();
                }
            }
        });
        ui.add_space(4.0);
    }
}
