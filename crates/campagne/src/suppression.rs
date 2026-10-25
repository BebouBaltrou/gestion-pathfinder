//! Suppression d'une localité ou d'un personnage.
//!
//! Comme toute action, une suppression est un événement du journal : on peut l'annuler.

use crate::etat::{Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::*;

/// Ce qui disparaît avec une localité (pour prévenir le joueur avant de confirmer).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContenuLocalite {
    pub batiments: usize,
    pub organisations: usize,
    /// Équipes qui se trouvent dans la localité ou qui appartiennent à ses organisations.
    pub equipes: usize,
    pub gestionnaires: usize,
    /// Transferts en cours depuis ou vers la localité.
    pub transferts: usize,
    pub ventes: usize,
    /// Personnages présents : ils sont conservés, mais n'ont plus de localité.
    pub personnages_presents: usize,
}

impl Etat {
    /// Équipes qui disparaissent avec la localité.
    fn equipes_de_localite(&self, id: IdLocalite) -> Vec<IdEquipe> {
        self.equipes
            .values()
            .filter(|e| {
                e.localite == id
                    || self
                        .organisations
                        .get(&e.organisation)
                        .is_some_and(|o| o.localite == id)
            })
            .map(|e| e.id)
            .collect()
    }

    pub fn contenu_localite(&self, id: IdLocalite) -> ContenuLocalite {
        ContenuLocalite {
            batiments: self.batiments_localite(id).count(),
            organisations: self.organisations_localite(id).count(),
            equipes: self.equipes_de_localite(id).len(),
            gestionnaires: self
                .gestionnaires
                .values()
                .filter(|g| g.localite == id)
                .count(),
            transferts: self
                .transferts
                .iter()
                .filter(|t| t.de == id || t.vers == id)
                .count(),
            ventes: self.ventes.iter().filter(|v| v.localite == id).count(),
            personnages_presents: self.pj_presents(id) as usize,
        }
    }

    pub(crate) fn appliquer_suppression(&mut self, ev: &Evenement) -> Result<(), Erreur> {
        match ev {
            Evenement::LocaliteSupprimee { id } => self.supprimer_localite(*id),
            Evenement::PersonnageSupprime { id } => self.supprimer_personnage(*id),
            _ => Ok(()),
        }
    }

    fn supprimer_localite(&mut self, id: IdLocalite) -> Result<(), Erreur> {
        self.localite(id)?;
        // Une équipe en route vers la localité n'a nulle part où arriver : le joueur choisit
        // d'abord ce qu'il en fait (la délocaliser ailleurs n'est pas possible en cours de route).
        let en_route = self.equipes.values().any(|e| {
            matches!(&e.statut, StatutEquipe::EnDelocalisation(d) if d.vers == id)
                && e.localite != id
        });
        if en_route {
            return Err(Erreur::Actif(
                "une équipe est en route vers cette localité : attendez son arrivée".into(),
            ));
        }
        for o in self
            .organisations_localite(id)
            .map(|o| o.id)
            .collect::<Vec<_>>()
        {
            self.retirer_actif(Actif::Organisation(o));
        }
        for e in self.equipes_de_localite(id) {
            self.retirer_actif(Actif::Equipe(e));
        }
        for b in self
            .batiments_localite(id)
            .map(|b| b.id)
            .collect::<Vec<_>>()
        {
            self.retirer_actif(Actif::Batiment(b));
        }
        self.gestionnaires.retain(|_, g| g.localite != id);
        self.transferts.retain(|t| t.de != id && t.vers != id);
        self.ventes.retain(|v| v.localite != id);
        for p in self.personnages.values_mut() {
            if p.localite == Some(id) {
                p.localite = None;
            }
        }
        self.frequentees.remove(&id);
        self.absence.remove(&id);
        self.jours_absents.remove(&id);
        self.derniere_recette.remove(&id);
        self.prestige_en_attente.remove(&id);
        self.repartitions.retain(|k, _| match k {
            CleUnite::Salle(s) => self.salles.contains_key(s),
            CleUnite::Equipe(e) => self.equipes.contains_key(e),
        });
        self.localites.remove(&id);
        Ok(())
    }

    fn supprimer_personnage(&mut self, id: IdPersonnage) -> Result<(), Erreur> {
        self.pj(id)?;
        if self
            .aventure
            .as_ref()
            .is_some_and(|a| a.pjs.contains_key(&id.0))
        {
            return Err(Erreur::Aventure(
                "ce personnage participe à l'aventure en cours : terminez-la d'abord".into(),
            ));
        }
        self.personnages.remove(&id);
        self.fiches.remove(&id);
        self.prestige_don.remove(&id);
        // lien d'acolyte, dans un sens comme dans l'autre
        self.acolytes.retain(|a, m| *a != id && *m != id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::construction::evenements_depuis_modele;
    use crate::etat::Erreur;
    use crate::evenements::Evenement;
    use crate::modele::*;
    use crate::Campagne;
    use domaine::Montant;

    fn localite(c: &mut Campagne, r: &regles::Referentiel, nom: &str) -> IdLocalite {
        let l = c.nouvel_id_localite();
        c.executer(
            Evenement::LocaliteCreee {
                id: l,
                nom: nom.into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            r,
        )
        .unwrap();
        l
    }

    fn personnage(c: &mut Campagne, r: &regles::Referentiel, nom: &str) -> IdPersonnage {
        let id = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id,
                nom: nom.into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            r,
        )
        .unwrap();
        id
    }

    /// Deux localités : A (taverne, gestionnaire, PJ présent, transfert vers B) et B.
    fn monde() -> (
        Campagne,
        regles::Referentiel,
        IdLocalite,
        IdLocalite,
        IdPersonnage,
    ) {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let a = localite(&mut c, &r, "A");
        let b = localite(&mut c, &r, "B");
        let pj = personnage(&mut c, &r, "Héros");
        c.executer(
            Evenement::PersonnageDeplace {
                id: pj,
                vers: Some(a),
            },
            &r,
        )
        .unwrap();
        let (evs, _) = evenements_depuis_modele(
            &r,
            r.modele_batiment("taverne").unwrap(),
            c.nouvel_id_batiment(),
            a,
            "Taverne",
            c.nouvel_id_etage(),
            c.nouvel_id_salle(),
        );
        c.executer(
            Evenement::Groupe {
                libelle: "Taverne".into(),
                evenements: evs,
            },
            &r,
        )
        .unwrap();
        c.executer(
            Evenement::GestionnaireEmbauche {
                id: c.nouvel_id_gestionnaire(),
                reference: None,
                nom: "Intendant".into(),
                salaire: Montant::po(1),
                localite: a,
            },
            &r,
        )
        .unwrap();
        (c, r, a, b, pj)
    }

    #[test]
    fn suppression_d_une_localite_et_de_son_contenu() {
        let (mut c, r, a, b, pj) = monde();
        let contenu = c.etat(&r).contenu_localite(a);
        assert_eq!(contenu.batiments, 1);
        assert_eq!(contenu.gestionnaires, 1);
        assert_eq!(contenu.personnages_presents, 1);

        c.executer(Evenement::LocaliteSupprimee { id: a }, &r)
            .unwrap();
        let e = c.etat(&r);
        assert!(!e.localites.contains_key(&a));
        assert!(e.localites.contains_key(&b));
        assert!(e.batiments.is_empty());
        assert!(e.salles.is_empty());
        assert!(e.gestionnaires.is_empty());
        // le personnage est conservé, sans localité
        assert_eq!(e.personnages[&pj].localite, None);
        assert_eq!(e.personnages[&pj].compte, Montant::po(100));
    }

    #[test]
    fn localite_inconnue_refusee_et_suppression_annulable() {
        let (mut c, r, a, _, _) = monde();
        assert!(matches!(
            c.executer(
                Evenement::LocaliteSupprimee {
                    id: IdLocalite(999)
                },
                &r
            ),
            Err(Erreur::LocaliteInconnue(_))
        ));
        c.executer(Evenement::LocaliteSupprimee { id: a }, &r)
            .unwrap();
        c.annuler();
        let e = c.etat(&r);
        assert!(e.localites.contains_key(&a));
        assert_eq!(e.batiments.len(), 1);
        assert_eq!(e.gestionnaires.len(), 1);
    }

    #[test]
    fn transferts_de_la_localite_supprimes_et_ids_non_reutilises() {
        let (mut c, r, a, b, pj) = monde();
        c.executer(
            Evenement::AjustementLocalite {
                localite: a,
                delta_po: Montant::po(50),
                ajout: Default::default(),
                retrait: Default::default(),
                motif: "essai".into(),
            },
            &r,
        )
        .unwrap();
        c.executer(
            Evenement::TransfertEnvoye {
                id: c.nouvel_id_transfert(),
                de: a,
                vers: b,
                lot: Lot::Po(Montant::po(10)),
                duree_jours: 3,
            },
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).contenu_localite(b).transferts, 1);
        c.executer(Evenement::LocaliteSupprimee { id: a }, &r)
            .unwrap();
        assert!(c.etat(&r).transferts.is_empty());
        // le temps passe sans erreur
        c.executer(Evenement::JoursAvances { nombre: 5 }, &r)
            .unwrap();
        // une nouvelle localité ne reprend pas l'identifiant supprimé
        assert_ne!(c.nouvel_id_localite(), a);
        assert!(c.etat(&r).personnages.contains_key(&pj));
    }

    #[test]
    fn suppression_d_un_personnage() {
        let (mut c, r, _, _, pj) = monde();
        let autre = personnage(&mut c, &r, "Compagnon");
        c.executer(Evenement::PersonnageSupprime { id: pj }, &r)
            .unwrap();
        let e = c.etat(&r);
        assert!(!e.personnages.contains_key(&pj));
        assert!(e.personnages.contains_key(&autre));
        assert!(!e.fiches.contains_key(&pj));
        // plus personne n'est présent dans la localité
        assert!(matches!(
            c.executer(Evenement::PersonnageSupprime { id: pj }, &r),
            Err(Erreur::PersonnageInconnu(_))
        ));
        c.annuler();
        assert!(c.etat(&r).personnages.contains_key(&pj));
    }

    #[test]
    fn personnage_en_aventure_refuse() {
        let (mut c, r, _, _, pj) = monde();
        c.executer(
            Evenement::AventureDemarree {
                heure: 8,
                minute: 0,
                survie: false,
                degats_massifs: false,
            },
            &r,
        )
        .unwrap();
        // le personnage est suivi par l'aventure : on simule son entrée
        let mut e = c.etat(&r).clone();
        e.aventure.as_mut().unwrap().pjs.insert(
            pj.0,
            aventure::SuiviPj {
                vital: Default::default(),
                survie: aventure::Survie::nouvelle(temps::Instant::DEBUT),
                fatigue_de_survie: false,
                massifs_en_attente: false,
                etats_fiche: Default::default(),
                inventaire: Default::default(),
                ressources: Default::default(),
            },
        );
        assert!(matches!(
            e.appliquer(&Evenement::PersonnageSupprime { id: pj }, &r),
            Err(Erreur::Aventure(_))
        ));
    }
}
