//! Une campagne : journal d'événements + état recalculé, avec annuler/rétablir.
//!
//! L'état n'est jamais modifié directement : toute action passe par [`Campagne::executer`],
//! qui valide l'événement, l'ajoute au journal et met l'état à jour.

pub mod aventure;
pub mod calendrier;
pub mod catalogue_inventaire;
pub mod construction;
pub mod equipement_fiche;
pub mod etat;
pub mod evenements;
#[cfg(feature = "serde")]
pub mod fichier;
pub mod modele;
#[cfg(feature = "serde")]
pub mod modeles_perso;
pub mod organisation;
pub mod personnages;
pub mod prestige;
pub mod rapport;
pub mod recettes;
pub mod suppression;
pub mod ventes;

use calendrier::Calendrier;
use etat::{Consequence, Erreur, Etat};
use evenements::{Entree, Evenement};
use modele::*;
use regles::options::OptionsRegles;
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Campagne {
    pub nom: String,
    pub calendrier: Calendrier,
    pub options: OptionsRegles,
    /// Journal complet, y compris les entrées annulées (au-delà de `curseur`).
    pub journal: Vec<Entree>,
    /// Nombre d'entrées actives : `journal[..curseur]`.
    pub curseur: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    etat: Option<Etat>,
}

impl Campagne {
    pub fn nouvelle(nom: &str) -> Self {
        Campagne {
            nom: nom.to_string(),
            calendrier: Calendrier::default(),
            options: OptionsRegles::default(),
            journal: Vec::new(),
            curseur: 0,
            etat: Some(Etat::nouveau()),
        }
    }

    /// État courant (recalculé à partir du journal si nécessaire).
    pub fn etat(&mut self, r: &Referentiel) -> &Etat {
        if self.etat.is_none() {
            self.etat = Some(Self::rejouer(&self.journal[..self.curseur], r));
        }
        self.etat.as_ref().expect("état calculé")
    }

    fn rejouer(entrees: &[Entree], r: &Referentiel) -> Etat {
        let mut e = Etat::nouveau();
        for x in entrees {
            // Un journal enregistré n'est composé que d'événements déjà validés.
            let _ = e.appliquer(&x.evenement, r);
        }
        e
    }

    /// Valide puis enregistre un événement. Les entrées annulées non rétablies sont abandonnées.
    pub fn executer(&mut self, ev: Evenement, r: &Referentiel) -> Result<Vec<Consequence>, Erreur> {
        let mut etat = self.etat(r).clone();
        let jour = etat.jour;
        let consequences = etat.appliquer(&ev, r)?;
        self.journal.truncate(self.curseur);
        let numero = self.journal.last().map(|e| e.numero + 1).unwrap_or(1);
        self.journal.push(Entree {
            numero,
            jour,
            evenement: ev,
        });
        self.curseur = self.journal.len();
        self.etat = Some(etat);
        Ok(consequences)
    }

    pub fn peut_annuler(&self) -> bool {
        self.curseur > 0
    }
    pub fn peut_retablir(&self) -> bool {
        self.curseur < self.journal.len()
    }

    pub fn annuler(&mut self) -> Option<&Entree> {
        if !self.peut_annuler() {
            return None;
        }
        self.curseur -= 1;
        self.etat = None;
        self.journal.get(self.curseur)
    }

    pub fn retablir(&mut self) -> Option<&Entree> {
        if !self.peut_retablir() {
            return None;
        }
        self.curseur += 1;
        self.etat = None;
        self.journal.get(self.curseur - 1)
    }

    /// Entrées actives, de la plus récente à la plus ancienne.
    /// Le journal contient-il des événements du mode aventure ou combat ? (Alors le fichier est écrit
    /// au format 2, que l'ancienne application ne lit pas.)
    pub fn utilise_aventure(&self) -> bool {
        self.journal.iter().any(|e| e.evenement.utilise_aventure())
    }

    pub fn historique(&self) -> impl Iterator<Item = &Entree> {
        self.journal[..self.curseur].iter().rev()
    }

    // ---- identifiants : jamais réutilisés tant que l'entrée annulée peut être rétablie ----
    fn max_id(&self, f: impl Fn(&Evenement) -> Option<u32>) -> u32 {
        fn tous<'a>(e: &'a Evenement, out: &mut Vec<&'a Evenement>) {
            out.push(e);
            if let Evenement::Groupe { evenements, .. } = e {
                for x in evenements {
                    tous(x, out);
                }
            }
        }
        let mut evs = Vec::new();
        for e in &self.journal {
            tous(&e.evenement, &mut evs);
        }
        evs.into_iter().filter_map(f).max().unwrap_or(0)
    }
    pub fn nouvel_id_batiment(&self) -> IdBatiment {
        IdBatiment(
            1 + self.max_id(|e| match e {
                Evenement::BatimentCree { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_etage(&self) -> IdEtage {
        IdEtage(
            1 + self.max_id(|e| match e {
                Evenement::EtageAjoute { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_salle(&self) -> IdSalle {
        IdSalle(
            1 + self.max_id(|e| match e {
                Evenement::SalleAjoutee { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_personnage(&self) -> IdPersonnage {
        IdPersonnage(
            1 + self.max_id(|e| match e {
                Evenement::PersonnageCree { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_localite(&self) -> IdLocalite {
        IdLocalite(
            1 + self.max_id(|e| match e {
                Evenement::LocaliteCreee { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_organisation(&self) -> IdOrganisation {
        IdOrganisation(
            1 + self.max_id(|e| match e {
                Evenement::OrganisationCreee { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_equipe(&self) -> IdEquipe {
        IdEquipe(
            1 + self.max_id(|e| match e {
                Evenement::EquipeAjoutee { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_gestionnaire(&self) -> IdGestionnaire {
        IdGestionnaire(
            1 + self.max_id(|e| match e {
                Evenement::GestionnaireEmbauche { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }
    pub fn nouvel_id_transfert(&self) -> IdTransfert {
        IdTransfert(
            1 + self.max_id(|e| match e {
                Evenement::TransfertEnvoye { id, .. } => Some(id.0),
                _ => None,
            }),
        )
    }

    /// Date du jour courant : « jour 12 — juredi 12 Abadius 4712 AR ».
    pub fn date_courante(&mut self, r: &Referentiel) -> String {
        let j = self.etat(r).jour;
        format!("Jour {j} — {}", self.calendrier.libelle(j))
    }

    /// Description lisible d'une entrée, avec les noms connus dans l'état courant.
    pub fn decrire(&mut self, e: &Evenement, r: &Referentiel) -> String {
        let etat = self.etat(r).clone();
        let pj = |id: &IdPersonnage| {
            etat.personnages
                .get(id)
                .map(|p| p.nom.clone())
                .unwrap_or(id.to_string())
        };
        let lo = |id: &IdLocalite| {
            etat.localites
                .get(id)
                .map(|l| l.nom.clone())
                .unwrap_or(id.to_string())
        };
        let bat = |id: &IdBatiment| {
            etat.batiments
                .get(id)
                .map(|b| b.nom.clone())
                .unwrap_or(id.to_string())
        };
        let sal = |id: &IdSalle| {
            etat.salles
                .get(id)
                .map(|s| etat.nom_salle(s, r))
                .unwrap_or(id.to_string())
        };
        let qui = |p: &Payeur| match p {
            Payeur::Localite(l) => format!("la trésorerie de {}", lo(l)),
            Payeur::Personnage(x) => format!("le compte de {}", pj(x)),
        };
        let org = |id: &IdOrganisation| {
            etat.organisations
                .get(id)
                .map(|o| o.nom.clone())
                .unwrap_or(id.to_string())
        };
        let eq = |id: &IdEquipe| {
            etat.equipes
                .get(id)
                .map(|e| etat.nom_equipe(e, r))
                .unwrap_or(id.to_string())
        };
        let ges = |id: &IdGestionnaire| {
            etat.gestionnaires
                .get(id)
                .map(|g| g.nom.clone())
                .unwrap_or(id.to_string())
        };
        let ref_equipe = |id: &String| r.equipe(id).map(|u| u.nom.clone()).unwrap_or(id.clone());
        let ref_ext = |id: &String| r.extension(id).map(|u| u.nom.clone()).unwrap_or(id.clone());
        let ref_salle = |id: &String| r.salle(id).map(|u| u.nom.clone()).unwrap_or(id.clone());
        use Evenement::*;
        match e {
            PersonnageCree {
                nom,
                compte_initial,
                ..
            } => format!("Création du personnage {nom} (compte : {compte_initial})"),
            PersonnageModifie { id, .. } => format!("Modification du personnage {}", pj(id)),
            PrestigeDonRegle { pj: p, .. } => {
                format!("Prestige (don) : modificateurs de {} modifiés", pj(p))
            }
            AcolyteLie { acolyte, maitre } => {
                format!("{} devient l'acolyte de {}", pj(acolyte), pj(maitre))
            }
            AcolyteDelie { acolyte } => format!("{} n'est plus acolyte", pj(acolyte)),
            PartisanAjoute {
                maitre,
                niveau,
                nom,
            } => format!("{} recrute le partisan {nom} (niveau {niveau})", pj(maitre)),
            PartisanRetire { maitre, .. } => format!("{} retire un partisan", pj(maitre)),
            Inventaire(_) => "Opération sur l'inventaire".to_string(),
            Notes(c) => match c {
                notes::Commande::Creer { titre, texte, .. } => {
                    format!("Note ajoutée : {}", if titre.is_empty() { texte } else { titre })
                }
                notes::Commande::Modifier { titre, .. } => format!("Note modifiée : {titre}"),
                notes::Commande::Epingler { epingle: true, .. } => "Note épinglée".to_string(),
                notes::Commande::Epingler { .. } => "Note désépinglée".to_string(),
                notes::Commande::Supprimer { .. } => "Note supprimée".to_string(),
            },
            PersonnageDeplace { id, vers } => match vers {
                Some(l) => format!("{} se rend à {}", pj(id), lo(l)),
                None => format!("{} quitte toute localité", pj(id)),
            },
            LocaliteCreee { nom, genre, .. } => format!("Création de la localité {nom} ({genre})"),
            LocaliteModifiee { id, .. } => format!("Modification de la localité {}", lo(id)),
            LocaliteSupprimee { id } => format!("Suppression de la localité {}", lo(id)),
            PersonnageSupprime { id } => format!("Suppression du personnage {}", pj(id)),
            InjectionPo {
                pj: p,
                localite,
                montant,
            } => {
                format!(
                    "Injection de {montant} du compte de {} vers {}",
                    pj(p),
                    lo(localite)
                )
            }
            RetraitPo {
                localite,
                pj: p,
                montant,
            } => {
                format!(
                    "Retrait de {montant} de {} vers le compte de {}",
                    lo(localite),
                    pj(p)
                )
            }
            AjustementCompte {
                pj: p,
                delta,
                motif,
            } => format!("Compte de {} : {delta} ({motif})", pj(p)),
            AjustementLocalite {
                localite,
                delta_po,
                ajout,
                retrait,
                motif,
            } => {
                let mut parts = Vec::new();
                if delta_po.pc != 0 {
                    parts.push(format!("{delta_po}"));
                }
                if ajout.unites() > 0 {
                    parts.push(format!("+ {ajout}"));
                }
                if retrait.unites() > 0 {
                    parts.push(format!("− {retrait}"));
                }
                format!(
                    "Ajustement de {} : {} ({motif})",
                    lo(localite),
                    parts.join(", ")
                )
            }
            TransfertEnvoye {
                de,
                vers,
                lot,
                duree_jours,
                ..
            } => {
                format!(
                    "Transfert de {lot} de {} vers {} ({duree_jours} j)",
                    lo(de),
                    lo(vers)
                )
            }
            JoursAvances { nombre } => format!("{nombre} jour(s) écoulé(s)"),
            BatimentCree {
                nom,
                localite,
                modele,
                ..
            } => {
                let m = modele
                    .as_ref()
                    .and_then(|m| r.modele_batiment(m))
                    .map(|m| format!(" (modèle {})", m.nom))
                    .unwrap_or_default();
                format!("Création du bâtiment {nom} à {}{m}", lo(localite))
            }
            BatimentModifie { nom, .. } => format!("Modification du bâtiment {nom}"),
            BatimentFavori { id, favori } => format!(
                "{} {}",
                if *favori { "Favori :" } else { "Retrait des favoris :" },
                bat(id)
            ),
            EtageAjoute { batiment, nom, .. } => {
                format!("Étage « {nom} » ajouté à {}", bat(batiment))
            }
            EtageModifie { batiment, nom, .. } => {
                format!("Étage « {nom} » de {} modifié", bat(batiment))
            }
            SalleAjoutee {
                batiment,
                reference,
                deja_construite,
                ..
            } => format!(
                "Salle {} ajoutée à {}{}",
                ref_salle(reference),
                bat(batiment),
                if *deja_construite {
                    " (déjà construite)"
                } else {
                    ""
                }
            ),
            SalleModifiee { id, .. } => format!("Salle {} modifiée", sal(id)),
            SalleRetiree { id } => format!("Salle {} retirée", sal(id)),
            ExtensionAjoutee {
                salle,
                reference,
                deja_construite,
                ..
            } => format!(
                "Extension {} ajoutée à {}{}",
                ref_ext(reference),
                sal(salle),
                if *deja_construite { " (déjà faite)" } else { "" }
            ),
            ExtensionLancee {
                salle,
                reference,
                facteur,
            } => {
                let acc = if *facteur > 1 {
                    format!(" (accélération ×{facteur})")
                } else {
                    String::new()
                };
                format!(
                    "Construction de l'extension {} de {} lancée{acc}",
                    ref_ext(reference),
                    sal(salle)
                )
            }
            ExtensionAchetee {
                salle, reference, ..
            } => format!(
                "Extension {} de {} achetée terminée",
                ref_ext(reference),
                sal(salle)
            ),
            ExtensionRetiree { salle, reference } => format!(
                "Extension {} retirée de {}",
                ref_ext(reference),
                sal(salle)
            ),
            ConstructionLancee { salle, facteur } => {
                let acc = if *facteur > 1 {
                    format!(" (accélération ×{facteur})")
                } else {
                    String::new()
                };
                format!("Construction de {} lancée{acc}", sal(salle))
            }
            SalleAchetee {
                salle,
                payeur,
                prix,
            } => {
                let p = prix.map(|m| format!(" pour {m}")).unwrap_or_default();
                let qui = payeur
                    .map(|x| format!(", payée par {}", qui(&x)))
                    .unwrap_or_default();
                format!("{} achetée terminée{p}{qui}", sal(salle))
            }
            Paiement {
                payeur,
                montant,
                motif,
            } => format!("Paiement de {montant} par {} : {motif}", qui(payeur)),
            CapitauxAchetes {
                localite,
                payeur,
                capitaux,
            } => format!(
                "Achat de {capitaux} pour {} ({} po, payés par {})",
                lo(localite),
                regles::calculs::valeur_achat(r, capitaux),
                qui(payeur)
            ),
            AmeliorationLancee {
                salle,
                vers,
                facteur,
            } => {
                let acc = if *facteur > 1 {
                    format!(" (accélération ×{facteur})")
                } else {
                    String::new()
                };
                format!(
                    "Amélioration de {} en {} lancée{acc}",
                    sal(salle),
                    ref_salle(vers)
                )
            }
            Groupe {
                libelle,
                evenements,
            } => format!("{libelle} ({} opérations)", evenements.len()),
            OrganisationCreee { nom, localite, .. } => {
                format!("Création de l'organisation {nom} ({})", lo(localite))
            }
            OrganisationModifiee { nom, .. } => format!("Modification de l'organisation {nom}"),
            EquipeAjoutee {
                organisation,
                reference,
                deja_recrutee,
                ..
            } => format!(
                "Équipe {} ajoutée à {}{}",
                ref_equipe(reference),
                org(organisation),
                if *deja_recrutee { " (déjà recrutée)" } else { "" }
            ),
            EquipeModifiee { id, .. } => format!("Équipe {} modifiée", eq(id)),
            EquipeAffectee {
                id,
                organisation,
                lieu,
            } => format!(
                "{} : membre de {}, travaille {}",
                eq(id),
                org(organisation),
                match lieu {
                    Some(b) => format!("à {}", bat(b)),
                    None => "sans bâtiment".to_string(),
                }
            ),
            EquipeRetiree { id } => format!("Équipe {} retirée", eq(id)),
            RecrutementLance { equipe, facteur } => {
                let acc = if *facteur > 1 {
                    format!(" (accélération ×{facteur})")
                } else {
                    String::new()
                };
                format!("Recrutement de {} lancé{acc}", eq(equipe))
            }
            EquipeAchetee {
                equipe,
                payeur,
                prix,
            } => {
                let p = prix.map(|m| format!(" pour {m}")).unwrap_or_default();
                let q = payeur
                    .map(|x| format!(", payée par {}", qui(&x)))
                    .unwrap_or_default();
                format!("{} acquise d'un coup{p}{q}", eq(equipe))
            }
            AmeliorationEquipeLancee { equipe, vers, .. } => {
                format!("Amélioration de {} en {} lancée", eq(equipe), ref_equipe(vers))
            }
            DelocalisationLancee {
                equipe,
                vers,
                preparation,
                voyage,
                payeur,
                prix,
            } => format!(
                "Délocalisation de {} vers {} ({preparation} j de préparation, {voyage} j de voyage, {prix} payés par {})",
                eq(equipe),
                lo(vers),
                qui(payeur)
            ),
            GestionnaireEmbauche {
                nom,
                salaire,
                localite,
                ..
            } => format!(
                "Embauche du gestionnaire {nom} ({salaire}/jour, payé par {})",
                lo(localite)
            ),
            GestionnaireModifie { nom, .. } => format!("Gestionnaire {nom} modifié"),
            GestionnaireRenvoye { id } => format!("Gestionnaire {} renvoyé", ges(id)),
            AbandonActif { cible } => format!("Abandon : {}", etat.libelle_actif(*cible, r)),
            VenteLancee {
                cible,
                jours,
                influence,
                po,
                capitaux,
            } => {
                let inf = if *influence > 0 {
                    format!(", {influence} Influence dépensée")
                } else {
                    String::new()
                };
                let cap = if capitaux.unites() > 0 {
                    format!(" + {capitaux}")
                } else {
                    String::new()
                };
                format!(
                    "Mise en vente : {} pour {po}{cap}, acheteur dans {jours} j{inf}",
                    etat.libelle_actif(*cible, r)
                )
            }
            VenteAnnulee { cible } => {
                format!("Vente annulée : {}", etat.libelle_actif(*cible, r))
            }
            SalleReparee {
                salle, po, capitaux, ..
            } => {
                let cap = if capitaux.unites() > 0 {
                    format!(" + {capitaux}")
                } else {
                    String::new()
                };
                format!("Réparation de {} ({po}{cap})", sal(salle))
            }
            RecettesPercues {
                localite,
                jours,
                conserves,
                ..
            } => {
                let garde = if conserves.unites() > 0 {
                    format!(", capitaux conservés : {conserves}")
                } else {
                    String::new()
                };
                format!("Recettes de {} sur {jours} jour(s){garde}", lo(localite))
            }
            ReglageUsure {
                sans_usure_avec_gestionnaire,
            } => {
                if *sans_usure_avec_gestionnaire {
                    "Usure : aucune dans les localités avec un gestionnaire payé".to_string()
                } else {
                    "Usure : règle officielle (ralentie par un gestionnaire)".to_string()
                }
            }
            TestPrestigeResolu { localite, reussi } => format!(
                "Test de prestige à {} : {}",
                lo(localite),
                if *reussi { "réussi" } else { "raté" }
            ),
            ReglageSalaireGestionnaire { par_affectation } => {
                if *par_affectation {
                    "Salaire des gestionnaires : une fois par affectation".to_string()
                } else {
                    "Salaire des gestionnaires : une fois par jour".to_string()
                }
            }
            ReglageLimiteDepense { par_pj } => {
                if *par_pj {
                    "Limite des dépenses : une par PJ présent".to_string()
                } else {
                    "Limite des dépenses : une par localité".to_string()
                }
            }
            Fiche { pj: id_pj, evenement } => format!(
                "Fiche de {} : {}",
                pj(id_pj),
                evenement.description()
            ),
            ReglagePersonnages { .. } => "Réglages des personnages modifiés".to_string(),
            AventureDemarree { heure, minute, .. } => {
                format!("Début de l'aventure à {heure:02}:{minute:02}")
            }
            Aventure(ev) => ev.description(),
            CombatValide { resume, .. } => format!("Combat terminé : {resume}"),
            AventureTerminee => "Fin de l'aventure".to_string(),
            Note { texte } => format!("Note : {texte}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domaine::{CoutCapitaux, Montant, TypeCapital};

    fn base() -> (Campagne, Referentiel, IdPersonnage, IdLocalite, IdLocalite) {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Valeros".into(),
                compte_initial: Montant::po(500),
                prestige: Some(5),
            },
            &r,
        )
        .unwrap();
        let a = c.nouvel_id_localite();
        c.executer(
            Evenement::LocaliteCreee {
                id: a,
                nom: "Pointesable".into(),
                genre: "ville".into(),
                population: Some(1240),
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            &r,
        )
        .unwrap();
        let b = c.nouvel_id_localite();
        c.executer(
            Evenement::LocaliteCreee {
                id: b,
                nom: "Fort Ilsur".into(),
                genre: "avant-poste".into(),
                population: None,
                categorie: CategorieLocalite::Personnalisee {
                    nom: "Avant-poste".into(),
                    limite_depense_jour: 3,
                },
                notes: String::new(),
            },
            &r,
        )
        .unwrap();
        (c, r, pj, a, b)
    }

    #[test]
    fn injection_retrait_et_soldes() {
        let (mut c, r, pj, a, _) = base();
        c.executer(
            Evenement::InjectionPo {
                pj,
                localite: a,
                montant: Montant::po(120),
            },
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).pj(pj).unwrap().compte, Montant::po(380));
        assert_eq!(c.etat(&r).localite(a).unwrap().tresorerie, Montant::po(120));
        let e = c.executer(
            Evenement::RetraitPo {
                localite: a,
                pj,
                montant: Montant::po(200),
            },
            &r,
        );
        assert!(matches!(e, Err(Erreur::TresorerieInsuffisante { .. })));
        c.executer(
            Evenement::RetraitPo {
                localite: a,
                pj,
                montant: Montant::pa(15),
            },
            &r,
        )
        .unwrap();
        assert_eq!(
            c.etat(&r).localite(a).unwrap().tresorerie,
            Montant::pa(1185)
        );
        assert_eq!(c.journal.len(), 5); // l'opération refusée n'est pas journalisée
    }

    #[test]
    fn limites_officielle_et_personnalisee() {
        let (mut c, r, _, a, b) = base();
        assert_eq!(c.etat(&r).limite_depense(a, &r), Some(15));
        assert_eq!(c.etat(&r).limite_depense(b, &r), Some(3));
    }

    #[test]
    fn transfert_avec_delai_et_influence_interdite() {
        let (mut c, r, _, a, b) = base();
        let ajout = CoutCapitaux {
            marchandises: 6,
            influence: 2,
            travail: 0,
            magie: 0,
        };
        c.executer(
            Evenement::AjustementLocalite {
                localite: a,
                delta_po: Montant::ZERO,
                ajout,
                retrait: CoutCapitaux::default(),
                motif: "récompense".into(),
            },
            &r,
        )
        .unwrap();
        let t = c.nouvel_id_transfert();
        let inf = Evenement::TransfertEnvoye {
            id: t,
            de: a,
            vers: b,
            lot: Lot::Capital(TypeCapital::Influence, 1),
            duree_jours: 2,
        };
        assert_eq!(c.executer(inf, &r), Err(Erreur::InfluenceNonTransferable));
        c.executer(
            Evenement::TransfertEnvoye {
                id: t,
                de: a,
                vers: b,
                lot: Lot::Capital(TypeCapital::Marchandises, 4),
                duree_jours: 3,
            },
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).localite(a).unwrap().capitaux.marchandises, 2);
        assert_eq!(c.etat(&r).localite(b).unwrap().capitaux.marchandises, 0);
        assert_eq!(c.etat(&r).transferts.len(), 1);
        c.executer(Evenement::JoursAvances { nombre: 2 }, &r)
            .unwrap();
        assert_eq!(c.etat(&r).localite(b).unwrap().capitaux.marchandises, 0);
        let arr = c
            .executer(Evenement::JoursAvances { nombre: 1 }, &r)
            .unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(c.etat(&r).localite(b).unwrap().capitaux.marchandises, 4);
        assert_eq!(c.etat(&r).jour, 4);
    }

    #[test]
    fn capitaux_jamais_negatifs_mais_po_oui() {
        let (mut c, r, _, a, _) = base();
        let retrait = CoutCapitaux {
            magie: 1,
            ..CoutCapitaux::default()
        };
        let e = c.executer(
            Evenement::AjustementLocalite {
                localite: a,
                delta_po: Montant::ZERO,
                ajout: CoutCapitaux::default(),
                retrait,
                motif: "x".into(),
            },
            &r,
        );
        assert!(matches!(e, Err(Erreur::CapitalInsuffisant { .. })));
        c.executer(
            Evenement::AjustementLocalite {
                localite: a,
                delta_po: Montant::po(-7),
                ajout: CoutCapitaux::default(),
                retrait: CoutCapitaux::default(),
                motif: "salaire".into(),
            },
            &r,
        )
        .unwrap();
        assert!(c.etat(&r).localite(a).unwrap().tresorerie.est_negatif());
    }

    #[test]
    fn inventaire_annulable_et_rejouable() {
        use inventaire::{Commande, Conteneur, Modele, Poids, Proprietaire};
        let (mut c, r, _pj, _a, _) = base();
        let modele = Modele {
            id: "epee".into(),
            nom: "Épée".into(),
            type_objet: "arme".into(),
            valeur: Some(Montant::po(15)),
            poids: Some(Poids(150)),
            source: "saisie libre".into(),
            proprietes: Default::default(),
        };
        let conteneur = Conteneur {
            capacite: None,
            types_autorises: None,
            contenu_compte: true,
        };
        let proprietaire = Proprietaire {
            type_id: "personnage".into(),
            ref_id: "1".into(),
        };
        for commande in [
            Commande::AjouterModele(modele),
            Commande::CreerInventaire {
                proprietaire,
                conteneur,
            },
            Commande::Ajouter {
                modele: "epee".into(),
                quantite: 1,
                dans: 1,
            },
        ] {
            c.executer(Evenement::Inventaire(commande), &r).unwrap();
        }
        assert_eq!(c.etat(&r).inventaire.poids_total(1), Poids(150));
        c.annuler();
        assert!(c.etat(&r).inventaire.exemplaire(2).is_err());
        c.retablir();
        assert_eq!(c.etat(&r).inventaire.poids_total(1), Poids(150));
    }

    #[test]
    fn notes_annulables_et_rejouables() {
        use notes::{Commande, Genre, Quand};
        let (mut c, r, ..) = base();
        c.executer(
            Evenement::Notes(Commande::Creer {
                genre: Genre::Mj,
                rubrique: None,
                titre: "Le MJ a dit".into(),
                texte: "Le dragon est rouge".into(),
                etiquettes: vec![],
                quand: Quand {
                    jour: 1,
                    heure: None,
                    reel: 0,
                },
            }),
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).notes.nombre(), 1);
        assert!(c.journal.last().unwrap().evenement.est_une_note());
        c.annuler();
        assert_eq!(c.etat(&r).notes.nombre(), 0);
        c.retablir();
        assert_eq!(c.etat(&r).notes.nombre(), 1);
        // Une note vide est refusée et ne change rien.
        let refus = c.executer(
            Evenement::Notes(Commande::Creer {
                genre: Genre::Session,
                rubrique: None,
                titre: " ".into(),
                texte: String::new(),
                etiquettes: vec![],
                quand: Quand {
                    jour: 1,
                    heure: None,
                    reel: 0,
                },
            }),
            &r,
        );
        assert!(refus.is_err());
        assert_eq!(c.etat(&r).notes.nombre(), 1);
    }

    #[test]
    fn annuler_retablir_et_ids_non_reutilises() {
        let (mut c, r, pj, a, _) = base();
        c.executer(
            Evenement::InjectionPo {
                pj,
                localite: a,
                montant: Montant::po(100),
            },
            &r,
        )
        .unwrap();
        c.annuler();
        assert_eq!(c.etat(&r).pj(pj).unwrap().compte, Montant::po(500));
        assert!(c.peut_retablir());
        c.retablir();
        assert_eq!(c.etat(&r).pj(pj).unwrap().compte, Montant::po(400));
        // annuler la création de la 2e localité puis en créer une nouvelle : nouvel id
        c.annuler();
        c.annuler();
        let id = c.nouvel_id_localite();
        assert_eq!(id, IdLocalite(3));
        c.executer(
            Evenement::Note {
                texte: "branche".into(),
            },
            &r,
        )
        .unwrap();
        // la nouvelle branche abandonne les entrées annulées : PJ, Pointesable, note
        assert!(!c.peut_retablir());
        assert_eq!(c.historique().count(), 3);
    }

    #[test]
    fn descriptions_lisibles() {
        let (mut c, r, pj, a, _) = base();
        let ev = Evenement::InjectionPo {
            pj,
            localite: a,
            montant: Montant::pa(47),
        };
        assert_eq!(
            c.decrire(&ev, &r),
            "Injection de 4 po 7 pa du compte de Valeros vers Pointesable"
        );
        assert_eq!(c.date_courante(&r), "Jour 1 — lunedi 1 Abadius 4712 AR");
    }
}

#[cfg(test)]
mod tests_aventure;
