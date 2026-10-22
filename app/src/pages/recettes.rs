//! Phase des recettes : tests de capitaux des bâtiments et organisations d'une localité
//! (Guide de campagne p. 79, 84 et 94).

use super::batiments::choix_payeur;
use crate::app::App;
use crate::{theme, widgets};
use campagne::evenements::Evenement;
use campagne::modele::*;
use campagne::recettes::{DemandeRecettes, UniteProductrice};
use domaine::{CoutCapitaux, Ressource, TypeCapital};
use eframe::egui::{self, RichText};
use regles::calculs::Gain;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub struct Etat {
    localite: Option<IdLocalite>,
    initialisee: Option<IdLocalite>,
    jours: String,
    batiments: BTreeSet<IdBatiment>,
    organisations: BTreeSet<IdOrganisation>,
    affectations: BTreeMap<CleUnite, Vec<(Ressource, u32)>>,
    /// Par ressource : faire 10 (vrai) ou dés saisis.
    prendre10: BTreeMap<Ressource, bool>,
    des: BTreeMap<Ressource, String>,
    conserves: CoutCapitaux,
    conserver_tout: bool,
    payeur_pj: Option<IdPersonnage>,
    detail_jours: bool,
}

/// Affectation par défaut : tout le bonus sur le premier type admis (po s'il est admis).
fn defaut(u: &UniteProductrice) -> Vec<(Ressource, u32)> {
    let r = if u.admises.contains(&Ressource::Po) {
        Some(Ressource::Po)
    } else {
        u.admises.iter().next().copied()
    };
    r.map(|r| vec![(r, u.bonus)]).unwrap_or_default()
}

fn valide(aff: &[(Ressource, u32)], u: &UniteProductrice, stricte: bool) -> bool {
    let somme: u32 = aff.iter().map(|(_, p)| p).sum();
    let utiles = aff.iter().filter(|(_, p)| *p > 0).count();
    somme == u.bonus
        && aff.iter().all(|(r, p)| *p == 0 || u.admises.contains(r))
        && (!stricte || utiles <= 1)
}

/// Lit « 12 5 18 » ou « 12, 5, 18 » : des d20.
fn lire_des(t: &str) -> Option<Vec<u8>> {
    t.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .filter(|x| !x.is_empty())
        .map(|x| x.parse::<u8>().ok().filter(|d| (1..=20).contains(d)))
        .collect()
}

pub fn afficher(app: &mut App, ui: &mut egui::Ui) {
    let Some(c) = app.campagne.as_mut() else {
        return;
    };
    let etat = c.etat(&app.r).clone();
    let regles_test = ReglesTest::depuis_options(&c.options);
    let r = app.r.clone();
    widgets::titre_page(
        ui,
        "Recettes",
        "Tests de capitaux des bâtiments et organisations (Guide de campagne p. 79, 84 et 94)",
    );
    let s = &mut app.ui_recettes;
    if etat.localites.is_empty() {
        widgets::texte_doux(ui, "Créez d'abord une localité.");
        return;
    }
    if s.localite.is_none_or(|l| !etat.localites.contains_key(&l)) {
        s.localite = etat.localites.keys().next().copied();
    }
    let l = s.localite.expect("localité");

    // ---------- localité et période ----------
    let attente = etat.jours_sans_recettes(l);
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_wrapped(|ui| {
            ui.label("Localité");
            egui::ComboBox::from_id_salt("rec_loc")
                .selected_text(etat.localites[&l].nom.clone())
                .show_ui(ui, |ui| {
                    for x in etat.localites.values() {
                        ui.selectable_value(&mut s.localite, Some(x.id), &x.nom);
                    }
                });
            ui.separator();
            ui.label("Jours à traiter");
            widgets::champ_entier(ui, &mut s.jours, "jours", 60.0);
            widgets::texte_doux(
                ui,
                format!("({attente} jour(s) sans recettes, jusqu'à aujourd'hui inclus)"),
            );
        });
    });
    let l = s.localite.expect("localité");
    if s.initialisee != Some(l) {
        s.initialisee = Some(l);
        s.batiments = etat.batiments_localite(l).map(|b| b.id).collect();
        s.organisations = etat.organisations.keys().copied().collect();
        s.affectations.clear();
        s.jours = etat.jours_sans_recettes(l).max(1).to_string();
        s.conserver_tout = true;
    }
    let jours = s.jours.trim().parse::<u32>().unwrap_or(0);

    // ---------- participants ----------
    let unites: Vec<UniteProductrice> =
        etat.unites_productrices(l, &r, regles_test, true).collect();
    let orgs: BTreeSet<IdOrganisation> = unites.iter().filter_map(|u| u.organisation).collect();
    ui.add_space(6.0);
    widgets::titre_section(ui, "Qui produit");
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        if unites.is_empty() {
            widgets::texte_doux(
                ui,
                "Aucune salle construite ni équipe recrutée ne produit de recettes ici \
                 (ou leurs revenus sont désactivés).",
            );
            return;
        }
        ui.horizontal_wrapped(|ui| {
            for b in etat.batiments_localite(l) {
                if !unites.iter().any(|u| u.batiment == Some(b.id)) {
                    continue;
                }
                let mut coche = s.batiments.contains(&b.id);
                if ui.checkbox(&mut coche, format!("🏛 {}", b.nom)).changed() {
                    if coche {
                        s.batiments.insert(b.id);
                    } else {
                        s.batiments.remove(&b.id);
                    }
                }
            }
            for o in &orgs {
                let nom = etat
                    .organisations
                    .get(o)
                    .map(|x| x.nom.clone())
                    .unwrap_or_default();
                let mut coche = s.organisations.contains(o);
                if ui.checkbox(&mut coche, format!("⚔ {nom}")).changed() {
                    if coche {
                        s.organisations.insert(*o);
                    } else {
                        s.organisations.remove(o);
                    }
                }
            }
        });
        widgets::texte_doux(
            ui,
            "Une équipe participe si son organisation ou le bâtiment où elle travaille est coché.",
        );
    });
    let sans_type: Vec<&UniteProductrice> =
        unites.iter().filter(|u| u.admises.is_empty()).collect();
    if !sans_type.is_empty() {
        let noms: Vec<&str> = sans_type.iter().map(|u| u.libelle.as_str()).collect();
        widgets::texte_doux(
            ui,
            format!(
                "Sans type de revenu coché, donc écartées : {} (cochez « Génère » dans les pages Bâtiments ou Organisations)",
                noms.join(", ")
            ),
        );
    }
    let actives: Vec<&UniteProductrice> = unites
        .iter()
        .filter(|u| !u.admises.is_empty())
        .filter(|u| match u.cle {
            CleUnite::Salle(_) => u.batiment.is_some_and(|b| s.batiments.contains(&b)),
            CleUnite::Equipe(_) => {
                u.organisation.is_some_and(|o| s.organisations.contains(&o))
                    || u.batiment.is_some_and(|b| s.batiments.contains(&b))
            }
        })
        .collect();
    if actives.is_empty() {
        return;
    }

    // ---------- répartition ----------
    let stricte = regles_test.repartition_stricte;
    for u in &actives {
        // (ré)initialise seulement si rien n'est saisi ou si un type n'est plus coché :
        // une somme incomplète en cours de saisie est conservée (signalée en rouge)
        let ok = s.affectations.get(&u.cle).is_some_and(|a| {
            a.iter().all(|(r, p)| *p == 0 || u.admises.contains(r))
                && (!stricte || a.iter().filter(|(_, p)| *p > 0).count() <= 1)
        });
        if !ok {
            let precedente = etat
                .repartitions
                .get(&u.cle)
                .filter(|a| valide(a, u, stricte))
                .cloned();
            s.affectations
                .insert(u.cle, precedente.unwrap_or_else(|| defaut(u)));
        }
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        widgets::titre_section(ui, "Répartition des bonus");
        widgets::badge_statut(
            ui,
            if stricte {
                domaine::Statut::Personnalise
            } else {
                domaine::Statut::Officiel
            },
        );
    });
    widgets::texte_doux(
        ui,
        if stricte {
            "Règle personnalisée : chaque unité donne tout son bonus à un seul type par jour."
        } else {
            "Règle officielle : le bonus peut aller à un type ou être réparti entre les types admis (GC p. 94)."
        },
    );
    ui.horizontal(|ui| {
        if ui.button("Tout en po quand c'est possible").clicked() {
            for u in &actives {
                s.affectations.insert(u.cle, defaut(u));
            }
        }
        if ui
            .button("Reprendre la répartition précédente")
            .on_hover_text("Celle des dernières recettes perçues pour chaque unité")
            .clicked()
        {
            for u in &actives {
                if let Some(a) = etat
                    .repartitions
                    .get(&u.cle)
                    .filter(|a| valide(a, u, stricte))
                {
                    s.affectations.insert(u.cle, a.clone());
                }
            }
        }
    });
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::Grid::new("repartition")
            .num_columns(3)
            .striped(true)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                for u in &actives {
                    ui.label(&u.libelle);
                    ui.label(
                        RichText::new(format!("+{}", u.bonus))
                            .color(theme::OR_PALE)
                            .strong(),
                    );
                    let aff = s.affectations.get_mut(&u.cle).expect("initialisée");
                    ui.horizontal_wrapped(|ui| {
                        if stricte {
                            let mut choix = aff.iter().find(|(_, p)| *p > 0).map(|(r, _)| *r);
                            if u.bonus == 0 {
                                widgets::texte_doux(ui, "bonus nul");
                            }
                            for x in &u.admises {
                                ui.radio_value(
                                    &mut choix,
                                    Some(*x),
                                    RichText::new(x.to_string())
                                        .color(theme::couleur_ressource(*x)),
                                );
                            }
                            if let Some(x) = choix {
                                *aff = vec![(x, u.bonus)];
                            }
                        } else {
                            let mut points: BTreeMap<Ressource, u32> =
                                aff.iter().copied().collect();
                            for x in &u.admises {
                                let v = points.entry(*x).or_insert(0);
                                ui.label(
                                    RichText::new(x.to_string())
                                        .color(theme::couleur_ressource(*x)),
                                );
                                ui.add(egui::DragValue::new(v).range(0..=u.bonus));
                            }
                            let somme: u32 = points.values().sum();
                            if somme != u.bonus {
                                ui.label(
                                    RichText::new(format!("{somme}/{} affectés", u.bonus))
                                        .color(theme::ROUGE),
                                );
                            }
                            *aff = points.into_iter().filter(|(_, p)| *p > 0).collect();
                        }
                    });
                    ui.end_row();
                }
            });
    });

    // ---------- jets ----------
    let mut ressources: BTreeSet<Ressource> = BTreeSet::new();
    let mut affectations = Vec::new();
    for u in &actives {
        let a = s.affectations.get(&u.cle).cloned().unwrap_or_default();
        ressources.extend(a.iter().filter(|(_, p)| *p > 0).map(|(r, _)| *r));
        affectations.push((u.cle, a));
    }
    ui.add_space(6.0);
    widgets::titre_section(ui, "Jets");
    let mut jets: Vec<(Ressource, Vec<JetJour>)> = Vec::new();
    let mut jets_ok = jours > 0;
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        if ressources.is_empty() {
            widgets::texte_doux(ui, "Aucun point affecté.");
        }
        for res in &ressources {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(format!("Test de {res}"))
                        .color(theme::couleur_ressource(*res))
                        .strong(),
                );
                let p10 = s.prendre10.entry(*res).or_insert(true);
                ui.checkbox(p10, "faire 10")
                    .on_hover_text("Autorisé pour les tests de capitaux (GC p. 79 et 84)");
                if *p10 {
                    jets.push((*res, vec![None; jours as usize]));
                } else {
                    let t = s.des.entry(*res).or_default();
                    ui.add(
                        egui::TextEdit::singleline(t)
                            .hint_text(format!("{jours} résultat(s) de d20, ex. 12 5 18"))
                            .desired_width(260.0),
                    );
                    match lire_des(t) {
                        Some(v) if v.len() == jours as usize => {
                            jets.push((*res, v.into_iter().map(Some).collect()));
                        }
                        Some(v) => {
                            jets_ok = false;
                            ui.label(
                                RichText::new(format!("{} / {jours} jet(s)", v.len()))
                                    .color(theme::ROUGE),
                            );
                        }
                        None => {
                            jets_ok = false;
                            ui.label(RichText::new("valeurs de 1 à 20").color(theme::ROUGE));
                        }
                    }
                }
            });
        }
    });

    // ---------- aperçu ----------
    if !jets_ok || ressources.is_empty() {
        return;
    }
    let mut demande = DemandeRecettes {
        localite: l,
        jours,
        affectations,
        jets,
        conserves: CoutCapitaux::default(),
        payeur: None,
        regles: regles_test,
    };
    let bilan_brut = match etat.calculer_recettes(&demande, &r) {
        Ok(b) => b,
        Err(e) => {
            ui.label(RichText::new(format!("⚠ {e}")).color(theme::ROUGE));
            return;
        }
    };
    if s.conserver_tout {
        s.conserves = bilan_brut.capitaux_nets;
    }
    for t in TypeCapital::TOUS {
        let max = bilan_brut.capitaux_nets.get(t);
        let v = s.conserves.get_mut(t);
        *v = (*v).min(max);
    }
    demande.conserves = s.conserves;
    let mut payeur = Payeur::Localite(l);
    let mut action = None;
    ui.add_space(6.0);
    widgets::titre_section(ui, "Résultat");
    widgets::carte(ui, |ui| {
        ui.set_width(ui.available_width());
        // détail du premier jour
        if let Some(tests) = bilan_brut.par_jour.first() {
            for t in tests {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new(t.ressource.to_string())
                            .color(theme::couleur_ressource(t.ressource))
                            .strong(),
                    );
                    let lignes: Vec<String> =
                        t.lignes.iter().map(|(n, p)| format!("{n} +{p}")).collect();
                    widgets::texte_doux(ui, lignes.join(" · "));
                });
                if jours == 1 {
                    ui.label(format!(
                        "   {} {}",
                        if t.prendre10 { "faire 10 :" } else { "d20 :" },
                        t.formule
                    ));
                }
            }
        }
        if jours > 1 {
            ui.checkbox(&mut s.detail_jours, format!("Détail des {jours} jours"));
            if s.detail_jours {
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        for (i, tests) in bilan_brut.par_jour.iter().enumerate() {
                            let jour = bilan_brut.du + i as u32;
                            let txt: Vec<String> = tests
                                .iter()
                                .map(|t| format!("{} : {}", t.ressource, t.formule))
                                .collect();
                            widgets::texte_doux(ui, format!("Jour {jour} — {}", txt.join(" ; ")));
                        }
                    });
            }
        }
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Total sur {jours} jour(s) :"));
            widgets::montant(ui, bilan_brut.po_brut, 16.0);
            widgets::capitaux(ui, &bilan_brut.capitaux_bruts, true);
        });
        if bilan_brut.tranches_absence > 0 {
            ui.label(
                RichText::new(format!(
                    "Absence : {} jour(s) sans PJ → {} tranche(s) de 7 jours : −{} po et −{} par capital (GC p. 84)",
                    bilan_brut.jours_absents,
                    bilan_brut.tranches_absence,
                    7 * bilan_brut.tranches_absence,
                    bilan_brut.tranches_absence
                ))
                .color(theme::ORANGE),
            );
        } else if bilan_brut.jours_absents > 0 {
            widgets::texte_doux(
                ui,
                format!(
                    "{} jour(s) sans PJ, moins d'une tranche de 7 : aucune réduction",
                    bilan_brut.jours_absents
                ),
            );
        }
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Gagné :").strong());
            widgets::montant(ui, bilan_brut.po_net, 18.0);
            widgets::capitaux(ui, &bilan_brut.capitaux_nets, true);
        });
        if bilan_brut.capitaux_nets.unites() > 0 {
            ui.add_space(4.0);
            ui.label(
                RichText::new(if regles_test.capitaux_produits_gratuits {
                    "Capitaux produits encaissés"
                } else {
                    "Capitaux produits à payer (Coût produit)"
                })
                .color(theme::OR)
                .strong(),
            );
            if regles_test.capitaux_produits_gratuits {
                ui.horizontal_wrapped(|ui| {
                    widgets::badge_statut(ui, domaine::Statut::Personnalise);
                    widgets::texte_doux(
                        ui,
                        "Option des Paramètres : encaissés sans payer. Vous pouvez quand même en garder moins.",
                    );
                });
            } else {
                widgets::texte_doux(
                    ui,
                    "Le test donne le droit de produire ces capitaux ; chaque unité gardée se paie au \
                     Coût produit (Table 2-1, GC p. 78-79 et 94). Vous pouvez en garder moins.",
                );
            }
            ui.checkbox(&mut s.conserver_tout, "tout garder");
            ui.horizontal_wrapped(|ui| {
                for t in TypeCapital::TOUS {
                    let max = bilan_brut.capitaux_nets.get(t);
                    if max == 0 {
                        continue;
                    }
                    let res: Ressource = t.into();
                    ui.label(RichText::new(t.to_string()).color(theme::couleur_ressource(res)));
                    let v = s.conserves.get_mut(t);
                    if ui.add(egui::DragValue::new(v).range(0..=max)).changed() {
                        s.conserver_tout = false;
                    }
                    widgets::texte_doux(
                        ui,
                        if regles_test.capitaux_produits_gratuits {
                            format!("/ {max}")
                        } else {
                            format!(
                                "/ {max} ({} po l'unité)",
                                r.valeur(res).map(|x| x.cout_produit_po).unwrap_or(0)
                            )
                        },
                    );
                }
            });
            if !regles_test.capitaux_produits_gratuits {
                ui.horizontal(|ui| {
                    ui.label("Payés par");
                    payeur = choix_payeur(ui, "payeur_rec", &mut s.payeur_pj, &etat, l);
                });
            }
        }
    });
    demande.payeur = Some(payeur);
    let bilan = etat.calculer_recettes(&demande, &r);
    ui.add_space(4.0);
    match &bilan {
        Ok(b) => {
            ui.horizontal(|ui| {
                if b.cout_conserves.pa > 0 {
                    ui.label("Coût des capitaux gardés :");
                    widgets::montant(ui, b.cout_conserves, 16.0);
                }
                if widgets::bouton_principal(ui, "Encaisser les recettes", true) {
                    action = Some(Evenement::RecettesPercues {
                        localite: l,
                        jours,
                        affectations: demande.affectations.clone(),
                        jets: demande.jets.clone(),
                        conserves: demande.conserves,
                        payeur: demande.payeur,
                        regles: demande.regles,
                    });
                }
            });
        }
        Err(e) => {
            ui.label(RichText::new(format!("⚠ {e}")).color(theme::ROUGE));
        }
    }
    if let Some(ev) = action {
        s.initialisee = None;
        app.executer(ev, "Recettes encaissées");
    }
}
