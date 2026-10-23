//! Organisations, équipes d'employés et gestionnaires (Guide de campagne p. 89-95).
//!
//! - Une équipe appartient à une organisation et travaille dans un bâtiment de sa localité, ou
//!   nulle part (recettes divisées par 2, arrondi inférieur : GC p. 94, PC-12 ; option).
//! - Recrutement : même mécanique que la construction d'une salle (capitaux puisés chaque jour
//!   dans les réserves de la localité, dans la limite des dépenses ; durée), accélération en
//!   Influence (GC p. 95). Un PJ doit être présent au lancement.
//! - Délocalisation (GC p. 95) : prix payé en po, préparation (1d6 jours par groupe, moitié
//!   des avantages) puis voyage (aucun avantage) ; l'équipe arrive sans lieu de travail.
//! - Gestionnaires (GC p. 89) : salaire quotidien en po, prélevé sur la trésorerie de leur
//!   localité, qui peut devenir négative.

use crate::construction::{
    chantier_accelere, depenser_un_jour, ressources_admises, ressources_par_defaut, un_seul_type,
};
use crate::etat::{Consequence, Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::*;
use domaine::fiches::{ElementModele, Modele};
use domaine::{Montant, Ressource, TypeCapital};
use regles::Referentiel;
use std::collections::{BTreeMap, BTreeSet};

/// Facteur de recettes d'une équipe, en quarts : 4 = entier, 2 = moitié, 1 = quart, 0 = rien.
fn demis(e: &Equipe, moitie_sans_batiment: bool) -> u32 {
    let mut f = 4; // en quarts, pour combiner deux divisions par 2
    match &e.statut {
        StatutEquipe::EnDelocalisation(d) if d.en_voyage() => return 0,
        StatutEquipe::EnDelocalisation(_) => f /= 2,
        _ => {}
    }
    if e.lieu.is_none() && moitie_sans_batiment {
        f /= 2;
    }
    f
}

impl Etat {
    pub fn organisation(&self, id: IdOrganisation) -> Result<&Organisation, Erreur> {
        self.organisations
            .get(&id)
            .ok_or(Erreur::OrganisationInconnue(id))
    }
    pub fn equipe(&self, id: IdEquipe) -> Result<&Equipe, Erreur> {
        self.equipes.get(&id).ok_or(Erreur::EquipeInconnue(id))
    }
    fn equipe_mut(&mut self, id: IdEquipe) -> Result<&mut Equipe, Erreur> {
        self.equipes.get_mut(&id).ok_or(Erreur::EquipeInconnue(id))
    }
    pub fn gestionnaire(&self, id: IdGestionnaire) -> Result<&GestionnaireEmploye, Erreur> {
        self.gestionnaires
            .get(&id)
            .ok_or(Erreur::GestionnaireInconnu(id))
    }

    pub fn equipes_organisation(&self, o: IdOrganisation) -> impl Iterator<Item = &Equipe> {
        self.equipes.values().filter(move |e| e.organisation == o)
    }
    pub fn equipes_batiment(&self, b: IdBatiment) -> impl Iterator<Item = &Equipe> {
        self.equipes.values().filter(move |e| e.lieu == Some(b))
    }
    pub fn organisations_localite(&self, l: IdLocalite) -> impl Iterator<Item = &Organisation> {
        self.organisations.values().filter(move |o| o.localite == l)
    }
    /// Gestionnaires affectés à un bâtiment ou à une organisation.
    pub fn gestionnaires_de(&self, a: Affectation) -> impl Iterator<Item = &GestionnaireEmploye> {
        self.gestionnaires
            .values()
            .filter(move |g| g.affectations.contains(&a))
    }

    /// Nombre de personnes d'une équipe : la taille de son profil (GC p. 95 « Taille »).
    pub fn effectif_equipe(&self, e: &Equipe, r: &Referentiel) -> u32 {
        match r.equipe(&e.reference).and_then(|u| u.taille.as_ref()) {
            Some(domaine::fiches::Taille::Personnes { nombre }) => *nombre,
            _ => 0,
        }
    }

    /// L'équipe est en place (recrutée, en amélioration ou en délocalisation).
    pub fn equipe_en_place(e: &Equipe) -> bool {
        match &e.statut {
            StatutEquipe::Recrutee | StatutEquipe::EnDelocalisation(_) => true,
            StatutEquipe::EnRecrutement(c) => {
                matches!(c.nature, NatureChantier::Amelioration { .. })
            }
            StatutEquipe::ARecruter => false,
        }
    }

    /// Effectif (en place, à venir : à recruter ou en recrutement) d'un ensemble d'équipes.
    pub fn effectif<'a>(
        &self,
        equipes: impl Iterator<Item = &'a Equipe>,
        r: &Referentiel,
    ) -> (u32, u32) {
        equipes.fold((0, 0), |(p, v), e| {
            let n = self.effectif_equipe(e, r);
            if Self::equipe_en_place(e) {
                (p + n, v)
            } else {
                (p, v + n)
            }
        })
    }

    pub fn nom_equipe(&self, e: &Equipe, r: &Referentiel) -> String {
        e.nom
            .clone()
            .filter(|n| !n.trim().is_empty())
            .or_else(|| r.equipe(&e.reference).map(|u| u.nom.clone()))
            .unwrap_or_else(|| e.reference.clone())
    }

    /// Salaire versé chaque jour par un gestionnaire actif (une fois par jour par défaut,
    /// une fois par affectation avec le réglage correspondant).
    pub fn salaire_quotidien(&self, g: &GestionnaireEmploye) -> Montant {
        if !g.actif {
            return Montant::ZERO;
        }
        let n = if self.salaire_par_affectation {
            g.affectations.len().max(1) as i64
        } else {
            1
        };
        Montant::pa(g.salaire.pa * n)
    }

    /// Total des salaires prélevés chaque jour sur la trésorerie d'une localité.
    pub fn salaires_localite(&self, l: IdLocalite) -> Montant {
        self.gestionnaires
            .values()
            .filter(|g| g.localite == l)
            .map(|g| self.salaire_quotidien(g))
            .sum()
    }

    fn verifier_lieu(
        &self,
        e: &Equipe,
        lieu: Option<IdBatiment>,
        r: &Referentiel,
    ) -> Result<(), Erreur> {
        if let Some(b) = lieu {
            let b = self.batiment(b)?;
            if b.localite != e.localite {
                return Err(Erreur::LieuHorsLocalite {
                    equipe: self.nom_equipe(e, r),
                    batiment: b.nom.clone(),
                });
            }
        }
        Ok(())
    }

    fn verifier_affectations(&self, a: &[Affectation]) -> Result<(), Erreur> {
        for x in a {
            match x {
                Affectation::Batiment(b) => {
                    self.batiment(*b)?;
                }
                Affectation::Organisation(o) => {
                    self.organisation(*o)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn appliquer_organisation(
        &mut self,
        ev: &Evenement,
        r: &Referentiel,
    ) -> Result<(), Erreur> {
        use Evenement::*;
        match ev {
            OrganisationCreee {
                id,
                localite,
                nom,
                classement,
                notes,
                modele,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if self.organisations.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                self.localite(*localite)?;
                self.organisations.insert(
                    *id,
                    Organisation {
                        id: *id,
                        localite: *localite,
                        nom: nom.clone(),
                        classement: classement.clone(),
                        notes: notes.clone(),
                        modele: modele.clone(),
                    },
                );
            }
            OrganisationModifiee {
                id,
                nom,
                classement,
                notes,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                let o = self
                    .organisations
                    .get_mut(id)
                    .ok_or(Erreur::OrganisationInconnue(*id))?;
                o.nom = nom.clone();
                o.classement = classement.clone();
                o.notes = notes.clone();
            }
            EquipeAjoutee {
                id,
                organisation,
                reference,
                nom,
                lieu,
                deja_recrutee,
            } => {
                if self.equipes.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                let localite = self.organisation(*organisation)?.localite;
                let u = r
                    .equipe(reference)
                    .ok_or_else(|| Erreur::EquipeReferentielInconnue(reference.clone()))?;
                let e = Equipe {
                    id: *id,
                    organisation: *organisation,
                    reference: reference.clone(),
                    nom: nom.clone(),
                    localite,
                    lieu: *lieu,
                    statut: if *deja_recrutee {
                        StatutEquipe::Recrutee
                    } else {
                        StatutEquipe::ARecruter
                    },
                    ressources: ressources_par_defaut(u),
                    revenus_actifs: true,
                };
                self.verifier_lieu(&e, *lieu, r)?;
                self.equipes.insert(*id, e);
            }
            EquipeModifiee {
                id,
                nom,
                ressources,
                revenus_actifs,
            } => {
                let e = self.equipe(*id)?.clone();
                let u = r
                    .equipe(&e.reference)
                    .ok_or_else(|| Erreur::EquipeReferentielInconnue(e.reference.clone()))?;
                let ok = ressources_admises(u, &BTreeSet::new());
                if let Some(x) = ressources.iter().find(|x| !ok.contains(x)) {
                    return Err(Erreur::RessourceNonAdmise {
                        salle: self.nom_equipe(&e, r),
                        ressource: *x,
                    });
                }
                let m = self.equipe_mut(*id)?;
                m.nom = nom.clone();
                m.ressources = un_seul_type(ressources);
                m.revenus_actifs = *revenus_actifs;
            }
            EquipeAffectee {
                id,
                organisation,
                lieu,
            } => {
                self.organisation(*organisation)?;
                let e = self.equipe(*id)?.clone();
                self.verifier_lieu(&e, *lieu, r)?;
                let m = self.equipe_mut(*id)?;
                m.organisation = *organisation;
                m.lieu = *lieu;
            }
            EquipeRetiree { id } => {
                self.equipe(*id)?;
                self.equipes.remove(id);
            }
            RecrutementLance { equipe, facteur } => {
                if !(1..=4).contains(facteur) {
                    return Err(Erreur::FacteurInvalide(*facteur));
                }
                let e = self.equipe(*equipe)?.clone();
                let nom = self.nom_equipe(&e, r);
                if e.statut != StatutEquipe::ARecruter {
                    return Err(Erreur::StatutIncompatible {
                        salle: nom,
                        attendu: "à recruter",
                    });
                }
                if !self.pj_present(e.localite) {
                    return Err(Erreur::AucunPjPresent(
                        self.localite(e.localite)?.nom.clone(),
                    ));
                }
                let u = r
                    .equipe(&e.reference)
                    .ok_or_else(|| Erreur::EquipeReferentielInconnue(e.reference.clone()))?;
                let (Some(cout), Some(duree)) = (u.cout, u.duree_jours) else {
                    return Err(Erreur::SansCoutFixe(nom));
                };
                let jour = self.jour;
                self.equipe_mut(*equipe)?.statut = StatutEquipe::EnRecrutement(chantier_accelere(
                    NatureChantier::Construction,
                    cout,
                    duree,
                    *facteur,
                    jour,
                    TypeCapital::Influence,
                ));
            }
            EquipeAchetee {
                equipe,
                payeur,
                prix,
            } => {
                let e = self.equipe(*equipe)?.clone();
                let nom = self.nom_equipe(&e, r);
                if e.statut != StatutEquipe::ARecruter {
                    return Err(Erreur::StatutIncompatible {
                        salle: nom,
                        attendu: "à recruter",
                    });
                }
                let u = r
                    .equipe(&e.reference)
                    .ok_or_else(|| Erreur::EquipeReferentielInconnue(e.reference.clone()))?;
                let prix = match prix {
                    Some(p) => *p,
                    None => Montant::po(u.prix_po.ok_or(Erreur::SansCoutFixe(nom))? as i64),
                };
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                if prix.pa > 0 {
                    self.debiter(payeur.unwrap_or(Payeur::Localite(e.localite)), prix)?;
                }
                self.equipe_mut(*equipe)?.statut = StatutEquipe::Recrutee;
            }
            AmeliorationEquipeLancee {
                equipe,
                vers,
                facteur,
            } => {
                if !(1..=4).contains(facteur) {
                    return Err(Erreur::FacteurInvalide(*facteur));
                }
                let e = self.equipe(*equipe)?.clone();
                let nom = self.nom_equipe(&e, r);
                if e.statut != StatutEquipe::Recrutee {
                    return Err(Erreur::StatutIncompatible {
                        salle: nom,
                        attendu: "recrutée",
                    });
                }
                let de = r
                    .equipe(&e.reference)
                    .ok_or_else(|| Erreur::EquipeReferentielInconnue(e.reference.clone()))?;
                let cible = r
                    .equipe(vers)
                    .ok_or_else(|| Erreur::EquipeReferentielInconnue(vers.clone()))?;
                if !de.amelioration_en.contains(vers) {
                    return Err(Erreur::AmeliorationImpossible {
                        de: de.nom.clone(),
                        vers: cible.nom.clone(),
                    });
                }
                if !self.pj_present(e.localite) {
                    return Err(Erreur::AucunPjPresent(
                        self.localite(e.localite)?.nom.clone(),
                    ));
                }
                let diff = regles::calculs::cout_amelioration(de, cible)
                    .map_err(|_| Erreur::SansCoutFixe(nom))?;
                let jour = self.jour;
                self.equipe_mut(*equipe)?.statut = StatutEquipe::EnRecrutement(chantier_accelere(
                    NatureChantier::Amelioration { vers: vers.clone() },
                    diff.capitaux,
                    diff.duree_jours,
                    *facteur,
                    jour,
                    TypeCapital::Influence,
                ));
            }
            DelocalisationLancee {
                equipe,
                vers,
                preparation,
                voyage,
                payeur,
                prix,
            } => {
                let e = self.equipe(*equipe)?.clone();
                let nom = self.nom_equipe(&e, r);
                if e.statut != StatutEquipe::Recrutee {
                    return Err(Erreur::StatutIncompatible {
                        salle: nom,
                        attendu: "recrutée",
                    });
                }
                self.localite(*vers)?;
                if *vers == e.localite {
                    return Err(Erreur::DelocalisationVersElleMeme);
                }
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                if prix.pa > 0 {
                    self.debiter(*payeur, *prix)?;
                }
                let m = self.equipe_mut(*equipe)?;
                m.lieu = None;
                m.statut = StatutEquipe::EnDelocalisation(Delocalisation {
                    vers: *vers,
                    preparation: *preparation,
                    voyage: *voyage,
                    jours_ecoules: 0,
                });
            }
            GestionnaireEmbauche {
                id,
                reference,
                nom,
                salaire,
                localite,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if self.gestionnaires.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                if salaire.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                self.localite(*localite)?;
                self.gestionnaires.insert(
                    *id,
                    GestionnaireEmploye {
                        id: *id,
                        reference: reference.clone(),
                        nom: nom.clone(),
                        salaire: *salaire,
                        localite: *localite,
                        affectations: Vec::new(),
                        actif: true,
                    },
                );
            }
            GestionnaireModifie {
                id,
                nom,
                salaire,
                localite,
                affectations,
                actif,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if salaire.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                self.localite(*localite)?;
                self.verifier_affectations(affectations)?;
                let g = self
                    .gestionnaires
                    .get_mut(id)
                    .ok_or(Erreur::GestionnaireInconnu(*id))?;
                g.nom = nom.clone();
                g.salaire = *salaire;
                g.localite = *localite;
                let mut a = affectations.clone();
                a.sort();
                a.dedup();
                g.affectations = a;
                g.actif = *actif;
            }
            GestionnaireRenvoye { id } => {
                self.gestionnaire(*id)?;
                self.gestionnaires.remove(id);
            }
            _ => unreachable!("événement non lié aux organisations"),
        }
        Ok(())
    }

    /// Un jour de recrutement pour les équipes d'une localité (budget partagé avec les salles).
    pub(crate) fn avancer_recrutements(
        &mut self,
        l: IdLocalite,
        budget: &mut u32,
        r: &Referentiel,
    ) -> Vec<Consequence> {
        let mut out = Vec::new();
        let ids: Vec<IdEquipe> = self
            .equipes
            .values()
            .filter(|e| e.localite == l && matches!(e.statut, StatutEquipe::EnRecrutement(_)))
            .map(|e| e.id)
            .collect();
        for id in ids {
            let mut stock = self.localites[&l].capitaux;
            let e = self.equipes.get_mut(&id).expect("équipe");
            let StatutEquipe::EnRecrutement(ch) = &mut e.statut else {
                continue;
            };
            depenser_un_jour(ch, &mut stock, budget);
            let termine = ch.termine();
            let nature = ch.nature.clone();
            self.localites.get_mut(&l).expect("localité").capitaux = stock;
            if termine {
                let e = self.equipes.get_mut(&id).expect("équipe");
                if let NatureChantier::Amelioration { vers } = nature {
                    e.reference = vers;
                    if let Some(u) = r.equipe(&e.reference) {
                        e.ressources = ressources_par_defaut(u);
                    }
                }
                e.statut = StatutEquipe::Recrutee;
                let e = e.clone();
                out.push(Consequence::EquipeRecrutee {
                    equipe: id,
                    nom: self.nom_equipe(&e, r),
                    organisation: self
                        .organisations
                        .get(&e.organisation)
                        .map(|o| o.nom.clone())
                        .unwrap_or_default(),
                });
            }
        }
        out
    }

    /// Un jour de délocalisation : préparation, voyage, arrivée.
    pub(crate) fn avancer_delocalisations(&mut self, r: &Referentiel) -> Vec<Consequence> {
        let mut out = Vec::new();
        let ids: Vec<IdEquipe> = self
            .equipes
            .values()
            .filter(|e| matches!(e.statut, StatutEquipe::EnDelocalisation(_)))
            .map(|e| e.id)
            .collect();
        for id in ids {
            let e = self.equipes.get_mut(&id).expect("équipe");
            let StatutEquipe::EnDelocalisation(d) = &mut e.statut else {
                continue;
            };
            d.jours_ecoules += 1;
            if d.jours_restants() == 0 {
                let vers = d.vers;
                e.localite = vers;
                e.lieu = None;
                e.statut = StatutEquipe::Recrutee;
                let e = e.clone();
                out.push(Consequence::EquipeArrivee {
                    equipe: id,
                    nom: self.nom_equipe(&e, r),
                    localite: self
                        .localites
                        .get(&vers)
                        .map(|l| l.nom.clone())
                        .unwrap_or_default(),
                });
            }
        }
        out
    }

    /// Salaires du jour, prélevés sur les trésoreries (qui peuvent devenir négatives).
    pub(crate) fn payer_salaires(&mut self) -> Vec<Consequence> {
        let mut out = Vec::new();
        let dus: Vec<(IdLocalite, Montant)> = self
            .gestionnaires
            .values()
            .map(|g| (g.localite, self.salaire_quotidien(g)))
            .filter(|(_, m)| m.pa > 0)
            .collect();
        for (l, m) in dus {
            if let Some(loc) = self.localites.get_mut(&l) {
                let avant = loc.tresorerie;
                loc.tresorerie = loc.tresorerie - m;
                if !avant.est_negatif() && loc.tresorerie.est_negatif() {
                    out.push(Consequence::TresorerieNegative {
                        localite: loc.nom.clone(),
                    });
                }
            }
        }
        out
    }

    /// Bonus de recettes par ressource cochée des équipes données (maximums par type, comme
    /// pour les salles). Sans lieu de travail : ÷ 2 si l'option est active (GC p. 94, PC-12) ;
    /// préparation d'une délocalisation : ÷ 2 ; voyage : rien (GC p. 95). Arrondi inférieur.
    pub fn bonus_equipes<'a>(
        &self,
        equipes: impl Iterator<Item = &'a Equipe>,
        r: &Referentiel,
        production_pendant_amelioration: bool,
        moitie_sans_batiment: bool,
    ) -> BTreeMap<Ressource, u32> {
        let mut m = BTreeMap::new();
        for e in equipes {
            let produit = match &e.statut {
                StatutEquipe::Recrutee | StatutEquipe::EnDelocalisation(_) => true,
                StatutEquipe::EnRecrutement(c) => {
                    production_pendant_amelioration
                        && matches!(c.nature, NatureChantier::Amelioration { .. })
                }
                StatutEquipe::ARecruter => false,
            };
            if !produit || !e.revenus_actifs {
                continue;
            }
            let Some(b) = r
                .equipe(&e.reference)
                .and_then(|u| u.recettes.as_ref())
                .map(|x| x.bonus)
            else {
                continue;
            };
            let b = b * demis(e, moitie_sans_batiment) / 4;
            for x in &e.ressources {
                *m.entry(*x).or_insert(0) += b;
            }
        }
        m
    }

    /// Capitaux restant à dépenser pour les équipes données.
    pub fn cout_restant_equipes<'a>(
        &self,
        equipes: impl Iterator<Item = &'a Equipe>,
        r: &Referentiel,
    ) -> domaine::CoutCapitaux {
        equipes.fold(domaine::CoutCapitaux::default(), |acc, e| {
            acc + match &e.statut {
                StatutEquipe::ARecruter => r
                    .equipe(&e.reference)
                    .and_then(|u| u.cout)
                    .unwrap_or_default(),
                StatutEquipe::EnRecrutement(c) => c.reste(),
                _ => domaine::CoutCapitaux::default(),
            }
        })
    }
}

/// Événements pour créer une organisation depuis un modèle : toutes ses équipes « à recruter »,
/// sans lieu de travail. Les éléments qui ne sont pas des équipes sont signalés.
pub fn evenements_depuis_modele_organisation(
    modele: &Modele,
    organisation: IdOrganisation,
    localite: IdLocalite,
    nom: &str,
    premiere_equipe: IdEquipe,
    r: &Referentiel,
) -> (Vec<Evenement>, Vec<String>) {
    let mut evs = vec![Evenement::OrganisationCreee {
        id: organisation,
        localite,
        nom: nom.to_string(),
        classement: modele.nom.clone(),
        notes: String::new(),
        modele: Some(modele.id.clone()),
    }];
    let mut ignores = Vec::new();
    let mut n = premiere_equipe.0;
    for e in &modele.composition {
        match e {
            ElementModele::Unite { id, quantite } if r.equipe(id).is_some() => {
                for _ in 0..*quantite {
                    evs.push(Evenement::EquipeAjoutee {
                        id: IdEquipe(n),
                        organisation,
                        reference: id.clone(),
                        nom: None,
                        lieu: None,
                        deja_recrutee: false,
                    });
                    n += 1;
                }
            }
            ElementModele::Unite { id, quantite } => ignores.push(format!("{quantite} × {id}")),
            ElementModele::Extension { id, quantite, sur } => {
                ignores.push(format!("{quantite} × extension {id} ({sur})"))
            }
        }
    }
    (evs, ignores)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Campagne;
    use domaine::CoutCapitaux;

    struct Monde {
        c: Campagne,
        r: Referentiel,
        l: IdLocalite,
        l2: IdLocalite,
        o: IdOrganisation,
    }

    impl Monde {
        fn ev(&mut self, ev: Evenement) -> Result<Vec<Consequence>, Erreur> {
            self.c.executer(ev, &self.r)
        }
        fn etat(&mut self) -> Etat {
            self.c.etat(&self.r).clone()
        }
        fn ajouter(&mut self, reference: &str, deja: bool) -> IdEquipe {
            let id = self.c.nouvel_id_equipe();
            let o = self.o;
            self.ev(Evenement::EquipeAjoutee {
                id,
                organisation: o,
                reference: reference.into(),
                nom: None,
                lieu: None,
                deja_recrutee: deja,
            })
            .unwrap();
            id
        }
    }

    /// Petite bourgade (limite 15/jour) avec réserves, un PJ présent, une organisation ;
    /// un village (L2) pour les délocalisations.
    fn monde() -> Monde {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        let l = c.nouvel_id_localite();
        let l2 = IdLocalite(l.0 + 1);
        let o = c.nouvel_id_organisation();
        for ev in [
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni".into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            Evenement::LocaliteCreee {
                id: l,
                nom: "Sandpoint".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "petite-bourgade".into(),
                },
                notes: String::new(),
            },
            Evenement::LocaliteCreee {
                id: l2,
                nom: "Magnimar".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: "village".into(),
                },
                notes: String::new(),
            },
            Evenement::PersonnageDeplace {
                id: pj,
                vers: Some(l),
            },
            Evenement::AjustementLocalite {
                localite: l,
                delta_po: Montant::po(1000),
                ajout: CoutCapitaux {
                    marchandises: 30,
                    influence: 30,
                    travail: 30,
                    magie: 10,
                },
                retrait: CoutCapitaux::default(),
                motif: "départ".into(),
            },
            Evenement::OrganisationCreee {
                id: o,
                localite: l,
                nom: "Guilde".into(),
                classement: String::new(),
                notes: String::new(),
                modele: None,
            },
        ] {
            c.executer(ev, &r).unwrap();
        }
        Monde { c, r, l, l2, o }
    }

    #[test]
    fn recrutement_avec_acceleration_en_influence() {
        let mut m = monde();
        // Tire-laine : 3 I, 1 T ; 1 jour
        let q = m.ajouter("tire-laine", false);
        m.ev(Evenement::RecrutementLance {
            equipe: q,
            facteur: 1,
        })
        .unwrap();
        let c = m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        assert!(matches!(c[..], [Consequence::EquipeRecrutee { .. }]));
        assert_eq!(m.etat().equipes[&q].statut, StatutEquipe::Recrutee);
        assert_eq!(m.etat().localite(m.l).unwrap().capitaux.influence, 27);
        // Bureaucrates : 2 M, 4 I, 2 T ; 2 jours ; ×2 → 8 I, 1 jour
        let b = m.ajouter("bureaucrates", false);
        m.ev(Evenement::RecrutementLance {
            equipe: b,
            facteur: 2,
        })
        .unwrap();
        let StatutEquipe::EnRecrutement(ch) = m.etat().equipes[&b].statut.clone() else {
            panic!()
        };
        assert_eq!((ch.cout.influence, ch.cout.travail, ch.duree), (8, 2, 1));
    }

    #[test]
    fn recettes_sans_batiment_et_delocalisation() {
        let mut m = monde();
        // Artisans : +4 po, Marchandises ou Travail
        let q = m.ajouter("artisans", true);
        let etat = m.etat();
        let e = &etat.equipes[&q];
        let bonus = etat.bonus_equipes([e].into_iter(), &m.r, false, true);
        assert_eq!(bonus.get(&Ressource::Po), Some(&2)); // sans bâtiment : ÷ 2
        let bonus = etat.bonus_equipes([e].into_iter(), &m.r, false, false);
        assert_eq!(bonus.get(&Ressource::Po), Some(&4)); // option désactivée
                                                         // Délocalisation : 100 po (moitié de 200), 2 jours de préparation, 3 de voyage
        let (l, l2) = (m.l, m.l2);
        m.ev(Evenement::DelocalisationLancee {
            equipe: q,
            vers: l2,
            preparation: 2,
            voyage: 3,
            payeur: Payeur::Localite(l),
            prix: Montant::po(100),
        })
        .unwrap();
        assert_eq!(m.etat().localite(l).unwrap().tresorerie, Montant::po(900));
        let etat = m.etat();
        let bonus = etat.bonus_equipes(etat.equipes.values(), &m.r, false, false);
        assert_eq!(bonus.get(&Ressource::Po), Some(&2)); // préparation : ÷ 2
        m.ev(Evenement::JoursAvances { nombre: 2 }).unwrap();
        let etat = m.etat();
        let bonus = etat.bonus_equipes(etat.equipes.values(), &m.r, false, false);
        assert_eq!(bonus.get(&Ressource::Po), Some(&0)); // voyage : rien
        let c = m.ev(Evenement::JoursAvances { nombre: 3 }).unwrap();
        assert!(matches!(c[..], [Consequence::EquipeArrivee { .. }]));
        assert_eq!(m.etat().equipes[&q].localite, l2);
    }

    #[test]
    fn lieu_de_travail_dans_la_localite() {
        let mut m = monde();
        let q = m.ajouter("artisans", true);
        let (b, b2) = (
            m.c.nouvel_id_batiment(),
            IdBatiment(m.c.nouvel_id_batiment().0 + 1),
        );
        let (l, l2, o) = (m.l, m.l2, m.o);
        for (id, loc) in [(b, l), (b2, l2)] {
            m.ev(Evenement::BatimentCree {
                id,
                localite: loc,
                nom: format!("Atelier {id}"),
                classement: String::new(),
                notes: String::new(),
                etages_successifs: false,
                modele: None,
            })
            .unwrap();
        }
        m.ev(Evenement::EquipeAffectee {
            id: q,
            organisation: o,
            lieu: Some(b),
        })
        .unwrap();
        assert_eq!(m.etat().equipes_batiment(b).count(), 1);
        assert!(matches!(
            m.ev(Evenement::EquipeAffectee {
                id: q,
                organisation: o,
                lieu: Some(b2),
            }),
            Err(Erreur::LieuHorsLocalite { .. })
        ));
    }

    #[test]
    fn salaires_quotidiens() {
        let mut m = monde();
        let (l, o) = (m.l, m.o);
        let g = m.c.nouvel_id_gestionnaire();
        m.ev(Evenement::GestionnaireEmbauche {
            id: g,
            reference: Some("abbe".into()),
            nom: "Frère Tobyn".into(),
            salaire: Montant::po(4),
            localite: l,
        })
        .unwrap();
        let b = m.c.nouvel_id_batiment();
        m.ev(Evenement::BatimentCree {
            id: b,
            localite: l,
            nom: "Temple".into(),
            classement: String::new(),
            notes: String::new(),
            etages_successifs: false,
            modele: None,
        })
        .unwrap();
        m.ev(Evenement::GestionnaireModifie {
            id: g,
            nom: "Frère Tobyn".into(),
            salaire: Montant::po(4),
            localite: l,
            affectations: vec![Affectation::Batiment(b), Affectation::Organisation(o)],
            actif: true,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 10 }).unwrap();
        // payé une fois par jour malgré deux affectations (PC-06)
        assert_eq!(m.etat().localite(l).unwrap().tresorerie, Montant::po(960));
        m.ev(Evenement::ReglageSalaireGestionnaire {
            par_affectation: true,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        assert_eq!(m.etat().localite(l).unwrap().tresorerie, Montant::po(952));
        // la trésorerie peut devenir négative, signalée une fois
        m.ev(Evenement::AjustementLocalite {
            localite: l,
            delta_po: Montant::po(-950),
            ajout: CoutCapitaux::default(),
            retrait: CoutCapitaux::default(),
            motif: "dépenses".into(),
        })
        .unwrap();
        let c = m.ev(Evenement::JoursAvances { nombre: 2 }).unwrap();
        assert_eq!(
            c.iter()
                .filter(|x| matches!(x, Consequence::TresorerieNegative { .. }))
                .count(),
            1
        );
        assert_eq!(m.etat().localite(l).unwrap().tresorerie, Montant::po(-14));
    }

    #[test]
    fn effectifs() {
        let mut m = monde();
        m.ajouter("gardes", true); // 5 personnes
        m.ajouter("acolyte", true); // 1
        m.ajouter("artisans", false); // 3, à recruter
        let etat = m.etat();
        let o = m.o;
        assert_eq!(etat.effectif(etat.equipes_organisation(o), &m.r), (6, 3));
    }

    #[test]
    fn modele_guilde_d_artisans() {
        let mut m = monde();
        let r = m.r.clone();
        let modele = r.modele_organisation("guilde-d-artisans").unwrap();
        let o = m.c.nouvel_id_organisation();
        let q = m.c.nouvel_id_equipe();
        let l = m.l;
        let (evs, ignores) =
            evenements_depuis_modele_organisation(modele, o, l, "Guilde de Sandpoint", q, &r);
        assert!(ignores.is_empty());
        m.ev(Evenement::Groupe {
            libelle: "Guilde".into(),
            evenements: evs,
        })
        .unwrap();
        let etat = m.etat();
        assert_eq!(etat.equipes_organisation(o).count(), 5);
        assert_eq!(
            etat.cout_restant_equipes(etat.equipes_organisation(o), &r),
            modele.cout
        );
    }
}
