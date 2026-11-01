//! Actions de la page Bâtiments : l'interface envoie une action décrite en données, le cœur la
//! traduit en événement du journal (et refuse les saisies incorrectes).

use crate::batiments::ressource;
use crate::Capitaux;
use campagne::evenements::Evenement;
use campagne::modele::*;
use domaine::{CoutCapitaux, Montant};
use regles::Referentiel;

/// Action demandée par l'interface sur un bâtiment, une salle ou une extension.
/// `payeur` : compte d'un PJ ; sinon la trésorerie de la localité du bien.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", rename_all = "camelCase"))]
pub enum ActionBatiment {
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    EtagesSuccessifs { batiment: u32, actif: bool },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Favori { batiment: u32, actif: bool },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ModeEtage {
        batiment: u32,
        etage: u32,
        sequentiel: bool,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AjouterEtage { batiment: u32, nom: String },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AjouterSalle {
        batiment: u32,
        etage: u32,
        reference: String,
        deja: bool,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ReglerSalle {
        salle: u32,
        ressource: Option<String>,
        revenus_actifs: bool,
        brisee: bool,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    LancerConstruction { salle: u32, facteur: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AcheterSalle { salle: u32, payeur: Option<u32> },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    SallePossedee { salle: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerSalle { salle: u32 },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Ameliorer {
        salle: u32,
        vers: String,
        facteur: u32,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Reparer {
        salle: u32,
        en_capitaux: bool,
        payeur: Option<u32>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    LancerExtension {
        salle: u32,
        reference: String,
        facteur: u32,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AcheterExtension {
        salle: u32,
        reference: String,
        payeur: Option<u32>,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    ExtensionPossedee { salle: u32, reference: String },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    RetirerExtension { salle: u32, reference: String },
    /// Ajout d'une extension ; coût, prix et durée seulement si le livre n'en donne pas (piège).
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AjouterExtension {
        salle: u32,
        reference: String,
        deja: bool,
        cout: Option<Capitaux>,
        prix: String,
        duree: String,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    AnnulerVente { cible: crate::ventes::Cible },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    Abandonner {
        cible: crate::ventes::Cible,
        sans_acheteur: bool,
    },
    #[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
    GarderSansAcheteur { cible: crate::ventes::Cible },
}

fn payeur_de(payeur: Option<u32>, localite: IdLocalite) -> Payeur {
    match payeur {
        Some(p) => Payeur::Personnage(IdPersonnage(p)),
        None => Payeur::Localite(localite),
    }
}

/// Traduit une action en événement du journal, avec le message de succès.
pub fn action(
    c: &mut campagne::Campagne,
    r: &Referentiel,
    a: ActionBatiment,
) -> Result<(Evenement, String), String> {
    use ActionBatiment as A;
    let etat = c.etat(r).clone();
    let salle = |id: u32| {
        etat.salles
            .get(&IdSalle(id))
            .cloned()
            .ok_or_else(|| "Salle introuvable".to_string())
    };
    let loc_salle = |id: u32| {
        etat.localite_actif(Actif::Salle(IdSalle(id)))
            .map_err(|e| e.to_string())
    };
    let nom_salle = |sb: &SalleBatie| etat.nom_salle(sb, r);
    let facteur = |f: u32| f.clamp(1, 4);
    Ok(match a {
        A::Favori { batiment, actif } => {
            etat.batiment(IdBatiment(batiment))
                .map_err(|e| e.to_string())?;
            (
                Evenement::BatimentFavori {
                    id: IdBatiment(batiment),
                    favori: actif,
                },
                if actif { "Ajouté aux favoris" } else { "Retiré des favoris" }.into(),
            )
        }
        A::EtagesSuccessifs { batiment, actif } => {
            let b = etat
                .batiments
                .get(&IdBatiment(batiment))
                .ok_or("Bâtiment introuvable")?;
            (
                Evenement::BatimentModifie {
                    id: b.id,
                    nom: b.nom.clone(),
                    classement: b.classement.clone(),
                    notes: b.notes.clone(),
                    etages_successifs: actif,
                },
                "Bâtiment modifié".into(),
            )
        }
        A::ModeEtage {
            batiment,
            etage,
            sequentiel,
        } => {
            let b = etat
                .batiments
                .get(&IdBatiment(batiment))
                .ok_or("Bâtiment introuvable")?;
            let e = b
                .etages
                .iter()
                .find(|e| e.id.0 == etage)
                .ok_or("Étage introuvable")?;
            (
                Evenement::EtageModifie {
                    batiment: b.id,
                    id: e.id,
                    nom: e.nom.clone(),
                    mode: if sequentiel {
                        ModeChantier::Sequentiel
                    } else {
                        ModeChantier::Parallele
                    },
                },
                format!("Étage « {} » modifié", e.nom),
            )
        }
        A::AjouterEtage { batiment, nom } => {
            if nom.trim().is_empty() {
                return Err("Donnez un nom à l'étage".into());
            }
            (
                Evenement::EtageAjoute {
                    batiment: IdBatiment(batiment),
                    id: c.nouvel_id_etage(),
                    nom: nom.trim().into(),
                    mode: ModeChantier::Parallele,
                },
                "Étage ajouté".into(),
            )
        }
        A::AjouterSalle {
            batiment,
            etage,
            reference,
            deja,
        } => {
            let u = r.salle(&reference).ok_or("Salle inconnue")?;
            (
                Evenement::SalleAjoutee {
                    id: c.nouvel_id_salle(),
                    batiment: IdBatiment(batiment),
                    etage: IdEtage(etage),
                    reference: reference.clone(),
                    nom: None,
                    deja_construite: deja,
                },
                format!("{} ajoutée", u.nom),
            )
        }
        A::ReglerSalle {
            salle: id,
            ressource: res,
            revenus_actifs,
            brisee,
        } => {
            let sb = salle(id)?;
            let ressources = match res {
                Some(x) => vec![ressource(&x).ok_or("Ressource inconnue")?],
                None => Vec::new(),
            };
            (
                Evenement::SalleModifiee {
                    id: sb.id,
                    etage: sb.etage,
                    nom: sb.nom.clone(),
                    ressources,
                    revenus_actifs,
                    brisee,
                },
                format!("{} modifiée", nom_salle(&sb)),
            )
        }
        A::LancerConstruction {
            salle: id,
            facteur: f,
        } => {
            let sb = salle(id)?;
            (
                Evenement::ConstructionLancee {
                    salle: sb.id,
                    facteur: facteur(f),
                },
                format!("Construction de {} lancée", nom_salle(&sb)),
            )
        }
        A::AcheterSalle { salle: id, payeur } => {
            let sb = salle(id)?;
            (
                Evenement::SalleAchetee {
                    salle: sb.id,
                    payeur: Some(payeur_de(payeur, loc_salle(id)?)),
                    prix: None,
                },
                format!("{} achetée", nom_salle(&sb)),
            )
        }
        A::SallePossedee { salle: id } => {
            let sb = salle(id)?;
            (
                Evenement::SalleAchetee {
                    salle: sb.id,
                    payeur: None,
                    prix: Some(Montant::ZERO),
                },
                format!("{} marquée comme possédée", nom_salle(&sb)),
            )
        }
        A::RetirerSalle { salle: id } => {
            let sb = salle(id)?;
            (
                Evenement::SalleRetiree { id: sb.id },
                format!("{} retirée", nom_salle(&sb)),
            )
        }
        A::Ameliorer {
            salle: id,
            vers,
            facteur: f,
        } => {
            let sb = salle(id)?;
            (
                Evenement::AmeliorationLancee {
                    salle: sb.id,
                    vers,
                    facteur: facteur(f),
                },
                format!("Amélioration de {} lancée", nom_salle(&sb)),
            )
        }
        A::Reparer {
            salle: id,
            en_capitaux,
            payeur,
        } => {
            let sb = salle(id)?;
            let (po, cap) = etat.cout_reparation(sb.id, r);
            (
                if en_capitaux {
                    Evenement::SalleReparee {
                        salle: sb.id,
                        payeur: None,
                        po: Montant::ZERO,
                        capitaux: cap,
                    }
                } else {
                    Evenement::SalleReparee {
                        salle: sb.id,
                        payeur: Some(payeur_de(payeur, loc_salle(id)?)),
                        po,
                        capitaux: CoutCapitaux::default(),
                    }
                },
                format!("{} réparée", nom_salle(&sb)),
            )
        }
        A::LancerExtension {
            salle: id,
            reference,
            facteur: f,
        } => {
            let sb = salle(id)?;
            (
                Evenement::ExtensionLancee {
                    salle: sb.id,
                    reference,
                    facteur: facteur(f),
                },
                format!("Extension de {} lancée", nom_salle(&sb)),
            )
        }
        A::AcheterExtension {
            salle: id,
            reference,
            payeur,
        } => {
            let sb = salle(id)?;
            (
                Evenement::ExtensionAchetee {
                    salle: sb.id,
                    reference,
                    payeur: Some(payeur_de(payeur, loc_salle(id)?)),
                    prix: None,
                },
                format!("Extension de {} achetée", nom_salle(&sb)),
            )
        }
        A::ExtensionPossedee {
            salle: id,
            reference,
        } => {
            let sb = salle(id)?;
            (
                Evenement::ExtensionAchetee {
                    salle: sb.id,
                    reference,
                    payeur: None,
                    prix: Some(Montant::ZERO),
                },
                format!("Extension de {} marquée comme possédée", nom_salle(&sb)),
            )
        }
        A::RetirerExtension {
            salle: id,
            reference,
        } => {
            let sb = salle(id)?;
            (
                Evenement::ExtensionRetiree {
                    salle: sb.id,
                    reference,
                },
                format!("Extension retirée de {}", nom_salle(&sb)),
            )
        }
        A::AjouterExtension {
            salle: id,
            reference,
            deja,
            cout,
            prix,
            duree,
        } => {
            let sb = salle(id)?;
            let u = r.extension(&reference).ok_or("Extension inconnue")?;
            let (cout, prix_po, duree) = if u.cout.is_none() {
                (
                    Some(cout.ok_or("Saisissez le coût de l'extension")?.into()),
                    Some(
                        prix.trim()
                            .parse()
                            .map_err(|_| "Saisissez le prix en po (nombre entier)".to_string())?,
                    ),
                    Some(
                        duree
                            .trim()
                            .parse()
                            .map_err(|_| "Saisissez la durée en jours".to_string())?,
                    ),
                )
            } else {
                (None, None, None)
            };
            (
                Evenement::ExtensionAjoutee {
                    salle: sb.id,
                    reference: reference.clone(),
                    deja_construite: deja,
                    cout,
                    prix_po,
                    duree,
                },
                format!("{} ajouté(e) à {}", u.nom, nom_salle(&sb)),
            )
        }
        A::AnnulerVente { cible } => {
            let a: Actif = cible.into();
            (
                Evenement::VenteAnnulee { cible: a },
                format!("Vente annulée : {}", etat.libelle_actif(a, r)),
            )
        }
        A::Abandonner {
            cible,
            sans_acheteur,
        } => {
            let a: Actif = cible.into();
            let l = etat.libelle_actif(a, r);
            (
                Evenement::AbandonActif { cible: a },
                if sans_acheteur {
                    format!("Aucun acheteur : {l} abandonné")
                } else {
                    format!("Abandon : {l}")
                },
            )
        }
        A::GarderSansAcheteur { cible } => {
            let l = etat.libelle_actif(cible.into(), r);
            (
                Evenement::Note {
                    texte: format!("Aucun acheteur pour {l} : bien conservé"),
                },
                format!("Aucun acheteur : {l} conservé"),
            )
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batiments::*;
    use campagne::modeles_perso::ModelesPerso;
    use campagne::Campagne;

    fn campagne_de_base(r: &Referentiel) -> Campagne {
        crate::localites::campagne_sandpoint(r).0
    }

    #[test]
    fn creation_vide_ajout_de_salle_et_fiche() {
        let r = regles::referentiel_officiel();
        let perso = ModelesPerso::default();
        let mut c = campagne_de_base(&r);
        let s = SaisieBatiment {
            localite: Some(1),
            nom: "Le Dragon vert".into(),
            facteur: 1,
            ..Default::default()
        };
        let (ev, msg) = creation(&mut c, &r, &perso, &s).unwrap();
        assert!(msg.contains("Dragon vert"));
        c.executer(ev, &r).unwrap();
        let p = page_batiments(&mut c, &r, &perso);
        assert_eq!(p.groupes[0].batiments[0].nom, "Le Dragon vert");
        let reference = r.salles[0].id.clone();
        let (ev, _) = action(
            &mut c,
            &r,
            ActionBatiment::AjouterSalle {
                batiment: 1,
                etage: 1,
                reference,
                deja: true,
            },
        )
        .unwrap();
        c.executer(ev, &r).unwrap();
        let f = fiche_batiment(&mut c, &r, 1).unwrap();
        assert_eq!(f.etages.len(), 1);
        assert_eq!(f.etages[0].salles.len(), 1);
        assert_eq!(f.etages[0].salles[0].statut, "construite");
        assert!(f.vente.en_cours.is_none());
    }

    #[test]
    fn creation_depuis_un_modele_avec_construction() {
        let r = regles::referentiel_officiel();
        let perso = ModelesPerso::default();
        let mut c = campagne_de_base(&r);
        let modele = r.modeles_batiments[0].id.clone();
        // un PJ doit être présent dans la localité pour lancer une construction
        let pj = c.nouvel_id_personnage();
        c.executer(
            crate::personnages::creation(pj, "Seoni", "", "").unwrap(),
            &r,
        )
        .unwrap();
        c.executer(crate::personnages::deplacement(pj.0, Some(1)), &r)
            .unwrap();
        let s = SaisieBatiment {
            localite: Some(1),
            modele: Some(modele),
            nom: "Test".into(),
            acquisition: Acquisition::Construire,
            facteur: 2,
            ..Default::default()
        };
        let ap = apercu_creation(&mut c, &r, &perso, &s).unwrap();
        assert!(ap.besoin.travail > 0 || ap.besoin.marchandises > 0);
        let (ev, _) = creation(&mut c, &r, &perso, &s).unwrap();
        c.executer(ev, &r).unwrap();
        let f = fiche_batiment(&mut c, &r, 1).unwrap();
        assert!(f
            .etages
            .iter()
            .flat_map(|e| &e.salles)
            .any(|s| s.statut == "en_travaux"));
    }

    #[test]
    fn saisies_et_actions_refusees() {
        let r = regles::referentiel_officiel();
        let perso = ModelesPerso::default();
        let mut c = campagne_de_base(&r);
        let sans_nom = SaisieBatiment {
            localite: Some(1),
            ..Default::default()
        };
        assert!(creation(&mut c, &r, &perso, &sans_nom).is_err());
        assert!(action(
            &mut c,
            &r,
            ActionBatiment::AjouterEtage {
                batiment: 1,
                nom: " ".into()
            }
        )
        .is_err());
        assert!(action(&mut c, &r, ActionBatiment::SallePossedee { salle: 99 }).is_err());
        assert_eq!(ressource("magie"), Some(domaine::Ressource::Magie));
        assert_eq!(ressource("inconnue"), None);
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests_json {
    use super::*;

    /// L'interface envoie ces formes exactes (voir desktop/ui/src/api.ts).
    #[test]
    fn actions_lues_comme_l_interface_les_envoie() {
        let a: ActionBatiment = serde_json::from_str(
            r#"{"type":"ajouterSalle","batiment":1,"etage":2,"reference":"bar","deja":true}"#,
        )
        .unwrap();
        assert!(matches!(
            a,
            ActionBatiment::AjouterSalle {
                etage: 2,
                deja: true,
                ..
            }
        ));
        let a: ActionBatiment = serde_json::from_str(
            r#"{"type":"reglerSalle","salle":3,"ressource":"magie","revenusActifs":true,"brisee":false}"#,
        )
        .unwrap();
        assert!(matches!(
            a,
            ActionBatiment::ReglerSalle {
                revenus_actifs: true,
                ..
            }
        ));
        let a: ActionBatiment = serde_json::from_str(
            r#"{"type":"abandonner","cible":{"genre":"salle","id":4},"sansAcheteur":false}"#,
        )
        .unwrap();
        assert!(matches!(
            a,
            ActionBatiment::Abandonner {
                sans_acheteur: false,
                ..
            }
        ));
        let a: ActionBatiment = serde_json::from_str(
            r#"{"type":"ajouterExtension","salle":1,"reference":"x","deja":false,"cout":{"marchandises":1,"influence":0,"travail":2,"magie":0},"prix":"5","duree":"3"}"#,
        )
        .unwrap();
        assert!(matches!(
            a,
            ActionBatiment::AjouterExtension { cout: Some(_), .. }
        ));
        let s: crate::batiments::SaisieBatiment = serde_json::from_str(
            r#"{"localite":1,"modele":null,"nom":"A","classement":"","acquisition":"acheterCapitaux","facteur":2,"seulementManquants":true,"prix":"","payeur":null}"#,
        )
        .unwrap();
        assert_eq!(
            s.acquisition,
            crate::batiments::Acquisition::AcheterCapitaux
        );
    }
}
