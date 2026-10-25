//! Phase des recettes et entretien (Guide de campagne p. 82-84 et 94).
//!
//! - **Tests de capitaux** : chaque jour, un test par ressource ; les bonus des salles et des
//!   équipes choisies y sont affectés (répartition officielle ou stricte). Résultat ÷ 10 : po au
//!   pa près, capitaux arrondis à l'inférieur (GC p. 79).
//! - **Absence** : par tranche de 7 jours sans PJ présent pendant la période, −7 po sur le total
//!   et −1 à chaque type de capital produit (minimum 0, GC p. 84).
//! - **Capitaux produits** : le joueur garde au plus la quantité obtenue et paie le Coût produit
//!   (Table 2-1, GC p. 78 et 94) ; il peut en garder moins.
//! - **Usure des capitaux** (GC p. 82) : −1 à chaque type en réserve par tranche de 7 jours
//!   consécutifs sans PJ ; 14 jours si un gestionnaire payé s'occupe de la localité (GC p. 89) ;
//!   aucune avec le réglage personnalisé correspondant.
//! - **Usure des entreprises** (GC p. 82) : au retour après 30 jours ou plus, test de prestige
//!   DD = jours d'absence − 10, sauf si un gestionnaire payé s'en occupe (GC p. 89).

use crate::etat::{Consequence, Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::*;
use domaine::{CoutCapitaux, Montant, Ressource, TypeCapital};
use regles::calculs::{cout_production, Gain, Jet};
use regles::options::OptionsRegles;
use regles::repartition::{tests_du_jour, Contribution, DetailTest};
use regles::Referentiel;
use std::collections::{BTreeMap, BTreeSet};

/// Paramètres d'une phase des recettes (mêmes champs que l'événement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemandeRecettes {
    pub localite: IdLocalite,
    pub jours: u32,
    pub affectations: Vec<(CleUnite, Vec<(Ressource, u32)>)>,
    pub jets: Vec<(Ressource, Vec<JetJour>)>,
    pub conserves: CoutCapitaux,
    pub payeur: Option<Payeur>,
    pub regles: ReglesTest,
}

/// Résultat détaillé d'une phase des recettes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilanRecettes {
    /// Premier et dernier jour couverts.
    pub du: u32,
    pub au: u32,
    /// Tests de chaque jour (un par ressource ayant des points).
    pub par_jour: Vec<Vec<DetailTest>>,
    pub po_brut: Montant,
    pub capitaux_bruts: CoutCapitaux,
    pub jours_absents: u32,
    pub tranches_absence: u32,
    pub po_net: Montant,
    pub capitaux_nets: CoutCapitaux,
    /// Coût produit des capitaux conservés.
    pub cout_conserves: Montant,
}

/// Unité productrice disponible dans une localité, pour l'interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniteProductrice {
    pub cle: CleUnite,
    /// Bâtiment (salle, ou lieu de travail d'une équipe) et organisation (équipe).
    pub batiment: Option<IdBatiment>,
    pub organisation: Option<IdOrganisation>,
    pub libelle: String,
    pub reference: String,
    pub bonus: u32,
    pub admises: BTreeSet<Ressource>,
}

impl Etat {
    /// Un gestionnaire actif, dont la localité payeuse n'est pas en négatif (salaire versé),
    /// est payé par cette localité ou s'occupe d'un bâtiment ou d'une organisation qui s'y trouve.
    pub fn gestionnaire_paye(&self, l: IdLocalite) -> bool {
        self.gestionnaires.values().any(|g| {
            let paye = g.actif
                && self
                    .localites
                    .get(&g.localite)
                    .is_some_and(|x| !x.tresorerie.est_negatif());
            let concerne = g.localite == l
                || g.affectations.iter().any(|a| match a {
                    Affectation::Batiment(b) => {
                        self.batiments.get(b).is_some_and(|b| b.localite == l)
                    }
                    Affectation::Organisation(o) => {
                        self.organisations.get(o).is_some_and(|o| o.localite == l)
                    }
                });
            paye && concerne
        })
    }

    /// Période d'usure des capitaux en jours, ou `None` si aucune usure.
    pub fn periode_usure(&self, l: IdLocalite) -> Option<u32> {
        if self.gestionnaire_paye(l) {
            if self.sans_usure_avec_gestionnaire {
                None
            } else {
                Some(14)
            }
        } else {
            Some(7)
        }
    }

    /// La localité a-t-elle une entreprise (salle productrice construite ou équipe recrutée) ?
    pub fn a_entreprise(&self, l: IdLocalite, r: &Referentiel) -> bool {
        self.unites_productrices(l, r, ReglesTest::officielles(), true)
            .next()
            .is_some()
    }

    /// Fin de journée : présence des PJ, usure des capitaux.
    pub(crate) fn suivre_presence(&mut self) -> Vec<Consequence> {
        let mut out = Vec::new();
        let localites: Vec<IdLocalite> = self.localites.keys().copied().collect();
        for l in localites {
            if self.pj_present(l) {
                self.frequentees.insert(l);
                self.absence.insert(l, 0);
                continue;
            }
            if !self.frequentees.contains(&l) {
                continue;
            }
            let jour = self.jour;
            let n = {
                let a = self.absence.entry(l).or_insert(0);
                *a += 1;
                *a
            };
            self.jours_absents.entry(l).or_default().insert(jour);
            if let Some(p) = self.periode_usure(l) {
                if n % p == 0 {
                    let loc = self.localites.get_mut(&l).expect("localité");
                    if loc.capitaux.unites() > 0 {
                        for t in TypeCapital::TOUS {
                            let v = loc.capitaux.get_mut(t);
                            *v = v.saturating_sub(1);
                        }
                        out.push(Consequence::UsureCapitaux {
                            localite: loc.nom.clone(),
                        });
                    }
                }
            }
        }
        out
    }

    /// Un PJ arrive dans une localité : test de prestige éventuel (usure des entreprises).
    pub(crate) fn arrivee_pj(&mut self, l: IdLocalite) -> Vec<Consequence> {
        let mut out = Vec::new();
        let absence = self.absence.get(&l).copied().unwrap_or(0);
        let protege = self.gestionnaire_paye(l);
        if self.frequentees.contains(&l) && absence >= 30 && !protege && self.a_entreprise_brut(l) {
            let dd = absence - 10;
            self.prestige_en_attente.insert(l, dd);
            out.push(Consequence::TestPrestigeRequis {
                localite: self
                    .localites
                    .get(&l)
                    .map(|x| x.nom.clone())
                    .unwrap_or_default(),
                dd,
            });
        }
        self.absence.insert(l, 0);
        self.frequentees.insert(l);
        out
    }

    /// Variante sans référentiel : une salle construite ou une équipe recrutée dans la localité.
    fn a_entreprise_brut(&self, l: IdLocalite) -> bool {
        self.salles.values().any(|s| {
            s.statut == StatutSalle::Construite
                && self
                    .batiments
                    .get(&s.batiment)
                    .is_some_and(|b| b.localite == l)
        }) || self
            .equipes
            .values()
            .any(|e| e.localite == l && e.statut == StatutEquipe::Recrutee)
    }

    /// Jours en attente de recettes pour une localité.
    pub fn jours_sans_recettes(&self, l: IdLocalite) -> u32 {
        let depuis = self.derniere_recette.get(&l).copied().unwrap_or(0);
        self.jour.saturating_sub(depuis)
    }

    /// Salles et équipes de la localité qui produisent des recettes aujourd'hui.
    /// `seulement_actives` : écarte les unités dont les revenus sont désactivés.
    pub fn unites_productrices<'a>(
        &'a self,
        l: IdLocalite,
        r: &'a Referentiel,
        regles: ReglesTest,
        seulement_actives: bool,
    ) -> impl Iterator<Item = UniteProductrice> + 'a {
        let salles = self
            .salles
            .values()
            .filter(move |s| {
                self.batiments
                    .get(&s.batiment)
                    .is_some_and(|b| b.localite == l)
            })
            .map(|s| CleUnite::Salle(s.id));
        let equipes = self
            .equipes
            .values()
            .filter(move |e| e.localite == l)
            .map(|e| CleUnite::Equipe(e.id));
        salles.chain(equipes).filter_map(move |cle| {
            let c = self.contribution(cle, l, r, &regles).ok()?;
            if seulement_actives && !self.revenus_actifs(cle) {
                return None;
            }
            let (batiment, organisation) = match cle {
                CleUnite::Salle(s) => (self.salles.get(&s).map(|x| x.batiment), None),
                CleUnite::Equipe(e) => {
                    let e = self.equipes.get(&e)?;
                    (e.lieu, Some(e.organisation))
                }
            };
            Some(UniteProductrice {
                cle,
                batiment,
                organisation,
                libelle: c.libelle.clone(),
                reference: c.unite.id.clone(),
                bonus: regles::repartition::bonus_effectif(&c, &regles.options()),
                admises: regles::repartition::admises(c.unite, &c.types_du_batiment)
                    .into_iter()
                    .filter(|x| self.ressources_cochees(cle).contains(x))
                    .collect(),
            })
        })
    }

    /// Bonus effectif d'une unité avec les règles données.
    pub fn bonus_unite(
        &self,
        cle: CleUnite,
        l: IdLocalite,
        r: &Referentiel,
        regles: &ReglesTest,
    ) -> Option<u32> {
        let c = self.contribution(cle, l, r, regles).ok()?;
        Some(regles::repartition::bonus_effectif(&c, &regles.options()))
    }

    /// Types de revenu cochés pour l'unité (pages Bâtiments et Organisations).
    pub fn ressources_cochees(&self, cle: CleUnite) -> Vec<Ressource> {
        match cle {
            CleUnite::Salle(s) => self
                .salles
                .get(&s)
                .map(|x| x.ressources.clone())
                .unwrap_or_default(),
            CleUnite::Equipe(e) => self
                .equipes
                .get(&e)
                .map(|x| x.ressources.clone())
                .unwrap_or_default(),
        }
    }

    fn revenus_actifs(&self, cle: CleUnite) -> bool {
        match cle {
            CleUnite::Salle(s) => self.salles.get(&s).is_some_and(|x| x.revenus_actifs),
            CleUnite::Equipe(e) => self.equipes.get(&e).is_some_and(|x| x.revenus_actifs),
        }
    }

    /// Contribution d'une unité à un test (sans affectation si `vide`).
    fn contribution<'a>(
        &self,
        cle: CleUnite,
        l: IdLocalite,
        r: &'a Referentiel,
        regles: &ReglesTest,
    ) -> Result<Contribution<'a>, Erreur> {
        let refus = |m: String| Err(Erreur::Recettes(m));
        match cle {
            CleUnite::Salle(id) => {
                let s = self.salle(id)?;
                let b = self.batiment(s.batiment)?;
                let nom = format!("{} — {}", b.nom, self.nom_salle(s, r));
                if b.localite != l {
                    return refus(format!("{nom} n'est pas dans cette localité"));
                }
                let produit = match &s.statut {
                    StatutSalle::Construite => true,
                    StatutSalle::EnTravaux(c) => {
                        regles.production_pendant_amelioration
                            && matches!(c.nature, NatureChantier::Amelioration { .. })
                    }
                    StatutSalle::AConstruire => false,
                };
                if !produit {
                    return refus(format!("{nom} ne produit pas (pas encore construite)"));
                }
                let u = r
                    .salle(&s.reference)
                    .ok_or_else(|| Erreur::SalleReferentielInconnue(s.reference.clone()))?;
                let extensions = Etat::bonus_extensions(s, r);
                // Salle sans recettes propres mais meublée : l'ameublement porte le bonus,
                // sur les types que le bâtiment génère déjà (lecture à valider).
                let (unite, bonus_extensions, nom) = match (&u.recettes, extensions) {
                    (Some(_), 0) => (u, 0, nom),
                    (Some(_), x) => (u, x, format!("{nom} (meublée)")),
                    (None, x) if x > 0 => (
                        r.extension("ameublement")
                            .ok_or_else(|| Erreur::ExtensionInconnue("ameublement".into()))?,
                        x - r
                            .extension("ameublement")
                            .and_then(|e| e.recettes.as_ref())
                            .map(|e| e.bonus)
                            .unwrap_or(0),
                        format!("{nom} (ameublement)"),
                    ),
                    (None, _) => return refus(format!("{nom} ne produit aucune recette")),
                };
                Ok(Contribution {
                    libelle: nom,
                    unite,
                    types_du_batiment: self.types_du_batiment(s.batiment, s.id, r),
                    sans_batiment: false,
                    brisee: s.brisee,
                    delocalisation: false,
                    bonus_extensions,
                    affectation: Vec::new(),
                })
            }
            CleUnite::Equipe(id) => {
                let e = self.equipe(id)?;
                let o = self
                    .organisations
                    .get(&e.organisation)
                    .map(|o| o.nom.clone())
                    .unwrap_or_default();
                let nom = format!("{o} — {}", self.nom_equipe(e, r));
                if e.localite != l {
                    return refus(format!("{nom} n'est pas dans cette localité"));
                }
                let (produit, delocalisation) = match &e.statut {
                    StatutEquipe::Recrutee => (true, false),
                    StatutEquipe::EnDelocalisation(d) => (!d.en_voyage(), true),
                    StatutEquipe::EnRecrutement(c) => (
                        regles.production_pendant_amelioration
                            && matches!(c.nature, NatureChantier::Amelioration { .. }),
                        false,
                    ),
                    StatutEquipe::ARecruter => (false, false),
                };
                if !produit {
                    return refus(format!("{nom} ne produit pas actuellement"));
                }
                let u = r
                    .equipe(&e.reference)
                    .ok_or_else(|| Erreur::EquipeReferentielInconnue(e.reference.clone()))?;
                if u.recettes.is_none() {
                    return refus(format!("{nom} ne produit aucune recette"));
                }
                Ok(Contribution {
                    libelle: nom,
                    unite: u,
                    types_du_batiment: BTreeSet::new(),
                    sans_batiment: e.lieu.is_none(),
                    brisee: false,
                    delocalisation,
                    bonus_extensions: 0,
                    affectation: Vec::new(),
                })
            }
        }
    }

    /// Calcule une phase des recettes sans rien modifier (aperçu et validation).
    pub fn calculer_recettes(
        &self,
        d: &DemandeRecettes,
        r: &Referentiel,
    ) -> Result<BilanRecettes, Erreur> {
        self.localite(d.localite)?;
        // les jours en attente sont traités dans l'ordre, à partir du plus ancien
        let attente = self.jours_sans_recettes(d.localite);
        if attente == 0 {
            return Err(Erreur::Recettes(
                "aucun jour en attente de recettes : avancez le temps d'abord".into(),
            ));
        }
        if d.jours == 0 || d.jours > attente {
            return Err(Erreur::Recettes(format!(
                "nombre de jours invalide : {} (1 à {attente} en attente)",
                d.jours
            )));
        }
        let du = self.jour - attente + 1;
        let au = du + d.jours - 1;
        let options = d.regles.options();

        let mut vus = BTreeSet::new();
        let mut contributions = Vec::new();
        let mut cles = Vec::new();
        for (cle, aff) in &d.affectations {
            if !vus.insert(*cle) {
                return Err(Erreur::Recettes("une unité figure deux fois".into()));
            }
            let mut c = self.contribution(*cle, d.localite, r, &d.regles)?;
            let cochees = self.ressources_cochees(*cle);
            if let Some((x, _)) = aff.iter().find(|(x, p)| *p > 0 && !cochees.contains(x)) {
                return Err(Erreur::Recettes(format!(
                    "{} : {x} n'est pas coché comme type de revenu",
                    c.libelle
                )));
            }
            c.affectation = aff.clone();
            contributions.push(c);
            cles.push(*cle);
        }
        // jets : un par jour pour chaque ressource testée
        let mut jets: BTreeMap<Ressource, &Vec<JetJour>> = BTreeMap::new();
        for (res, v) in &d.jets {
            if v.len() != d.jours as usize {
                return Err(Erreur::Recettes(format!(
                    "{res} : {} jet(s) saisi(s) pour {} jour(s)",
                    v.len(),
                    d.jours
                )));
            }
            jets.insert(*res, v);
        }
        let mut par_jour = Vec::new();
        let mut po_brut = Montant::ZERO;
        let mut capitaux_bruts = CoutCapitaux::default();
        for jour in 0..d.jours as usize {
            // une unité ne rapporte qu'à partir du jour où elle produit (construite, achetée…)
            let jour_numero = du + jour as u32;
            let actives: Vec<Contribution> = contributions
                .iter()
                .zip(&cles)
                .filter(|(_, cle)| {
                    self.debut_production.get(cle).copied().unwrap_or(0) <= jour_numero
                })
                .map(|(c, _)| c.clone())
                .collect();
            let du_jour: BTreeMap<Ressource, Jet> = jets
                .iter()
                .map(|(res, v)| {
                    (
                        *res,
                        match v[jour] {
                            Some(de) => Jet::De(de),
                            None => Jet::Prendre10,
                        },
                    )
                })
                .collect();
            let tests = tests_du_jour(&actives, &du_jour, &options)
                .map_err(|e| Erreur::Recettes(e.to_string()))?;
            for t in &tests {
                if !jets.contains_key(&t.ressource) {
                    return Err(Erreur::Recettes(format!(
                        "aucun jet saisi pour {}",
                        t.ressource
                    )));
                }
                match t.gain {
                    Gain::Po(m) => po_brut = po_brut + m,
                    Gain::Capital(c, n) => *capitaux_bruts.get_mut(c) += n,
                }
            }
            par_jour.push(tests);
        }
        let jours_absents = self
            .jours_absents
            .get(&d.localite)
            .map(|s| s.range(du..=au).count() as u32)
            .unwrap_or(0);
        let tranches = jours_absents / 7;
        let po_net = Montant::pa((po_brut.pa - tranches as i64 * 70).max(0));
        let mut capitaux_nets = capitaux_bruts;
        for t in TypeCapital::TOUS {
            let v = capitaux_nets.get_mut(t);
            *v = v.saturating_sub(tranches);
        }
        for t in TypeCapital::TOUS {
            if d.conserves.get(t) > capitaux_nets.get(t) {
                return Err(Erreur::Recettes(format!(
                    "{t} : {} conservés pour {} produits",
                    d.conserves.get(t),
                    capitaux_nets.get(t)
                )));
            }
        }
        let cout_conserves = if d.regles.capitaux_produits_gratuits {
            Montant::ZERO
        } else {
            TypeCapital::TOUS
                .iter()
                .map(|t| cout_production(r, *t, d.conserves.get(*t)))
                .sum()
        };
        Ok(BilanRecettes {
            du,
            au,
            par_jour,
            po_brut,
            capitaux_bruts,
            jours_absents,
            tranches_absence: tranches,
            po_net,
            capitaux_nets,
            cout_conserves,
        })
    }

    pub(crate) fn percevoir_recettes(
        &mut self,
        ev: &Evenement,
        r: &Referentiel,
    ) -> Result<(), Erreur> {
        let Evenement::RecettesPercues {
            localite,
            jours,
            affectations,
            jets,
            conserves,
            payeur,
            regles,
        } = ev
        else {
            unreachable!("événement de recettes attendu")
        };
        let d = DemandeRecettes {
            localite: *localite,
            jours: *jours,
            affectations: affectations.clone(),
            jets: jets.clone(),
            conserves: *conserves,
            payeur: *payeur,
            regles: *regles,
        };
        let bilan = self.calculer_recettes(&d, r)?;
        // les recettes en po arrivent avant le paiement des capitaux produits
        let l = self.localite_mut(*localite)?;
        l.tresorerie = l.tresorerie + bilan.po_net;
        if bilan.cout_conserves.pa > 0 {
            self.debiter(
                payeur.unwrap_or(Payeur::Localite(*localite)),
                bilan.cout_conserves,
            )?;
        }
        let l = self.localite_mut(*localite)?;
        l.capitaux = l.capitaux + *conserves;
        self.derniere_recette.insert(*localite, bilan.au);
        for (cle, aff) in affectations {
            self.repartitions.insert(*cle, aff.clone());
        }
        Ok(())
    }
}

impl ReglesTest {
    /// Règles officielles (répartition libre, ÷ 2 sans bâtiment, pas de production en travaux).
    pub fn officielles() -> Self {
        ReglesTest {
            repartition_stricte: false,
            moitie_equipe_sans_batiment: true,
            production_pendant_amelioration: false,
            capitaux_produits_gratuits: false,
        }
    }

    pub fn depuis_options(o: &OptionsRegles) -> Self {
        ReglesTest {
            repartition_stricte: o.repartition_stricte,
            moitie_equipe_sans_batiment: o.moitie_equipe_sans_batiment,
            production_pendant_amelioration: o.production_pendant_amelioration,
            capitaux_produits_gratuits: !o.capitaux_produits_payants,
        }
    }

    pub fn options(&self) -> OptionsRegles {
        OptionsRegles {
            repartition_stricte: self.repartition_stricte,
            moitie_equipe_sans_batiment: self.moitie_equipe_sans_batiment,
            production_pendant_amelioration: self.production_pendant_amelioration,
            capitaux_produits_payants: !self.capitaux_produits_gratuits,
            ..OptionsRegles::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Campagne;

    struct Monde {
        c: Campagne,
        r: Referentiel,
        pj: IdPersonnage,
        l: IdLocalite,
        bar: IdSalle,
    }

    impl Monde {
        fn ev(&mut self, ev: Evenement) -> Result<Vec<Consequence>, Erreur> {
            self.c.executer(ev, &self.r)
        }
        fn etat(&mut self) -> Etat {
            self.c.etat(&self.r).clone()
        }
    }

    /// Petite bourgade, 100 po, un PJ présent, une taverne avec un bar construit
    /// (Bar : +10 po ou Influence… selon le référentiel).
    fn monde() -> Monde {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        let l = c.nouvel_id_localite();
        let b = c.nouvel_id_batiment();
        let e = c.nouvel_id_etage();
        let bar = c.nouvel_id_salle();
        for ev in [
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni".into(),
                compte_initial: Montant::ZERO,
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
            Evenement::PersonnageDeplace {
                id: pj,
                vers: Some(l),
            },
            Evenement::AjustementLocalite {
                localite: l,
                delta_po: Montant::po(100),
                ajout: CoutCapitaux {
                    marchandises: 5,
                    influence: 5,
                    travail: 5,
                    magie: 0,
                },
                retrait: CoutCapitaux::default(),
                motif: "départ".into(),
            },
            Evenement::BatimentCree {
                id: b,
                localite: l,
                nom: "Taverne".into(),
                classement: String::new(),
                notes: String::new(),
                etages_successifs: false,
                modele: None,
            },
            Evenement::EtageAjoute {
                batiment: b,
                id: e,
                nom: "RDC".into(),
                mode: ModeChantier::Parallele,
            },
            Evenement::SalleAjoutee {
                id: bar,
                batiment: b,
                etage: e,
                reference: "bar".into(),
                nom: None,
                deja_construite: true,
            },
        ] {
            c.executer(ev, &r).unwrap();
        }
        // le bar est construit au jour 1 : il ne produit qu'à partir du jour 2
        c.executer(Evenement::JoursAvances { nombre: 1 }, &r)
            .unwrap();
        Monde { c, r, pj, l, bar }
    }

    fn demande(m: &Monde, jours: u32, res: Ressource, jets: Vec<JetJour>) -> DemandeRecettes {
        let bonus = m.r.salle("bar").unwrap().recettes.as_ref().unwrap().bonus;
        DemandeRecettes {
            localite: m.l,
            jours,
            affectations: vec![(CleUnite::Salle(m.bar), vec![(res, bonus)])],
            jets: vec![(res, jets)],
            conserves: CoutCapitaux::default(),
            payeur: None,
            regles: ReglesTest::officielles(),
        }
    }

    #[test]
    fn recettes_en_po_prendre_10() {
        let mut m = monde();
        let bonus = m.r.salle("bar").unwrap().recettes.as_ref().unwrap().bonus;
        m.ev(Evenement::JoursAvances { nombre: 2 }).unwrap();
        // 3 jours en attente (jours 1 à 3), faire 10 chaque jour : 3 × (10 + bonus) pa
        let d = demande(&m, 3, Ressource::Po, vec![None, None, None]);
        let bilan = m.etat().calculer_recettes(&d, &m.r).unwrap();
        assert_eq!(bilan.po_net, Montant::pa(3 * (10 + bonus as i64)));
        assert_eq!(bilan.par_jour.len(), 3);
        m.ev(Evenement::RecettesPercues {
            localite: d.localite,
            jours: d.jours,
            affectations: d.affectations.clone(),
            jets: d.jets.clone(),
            conserves: d.conserves,
            payeur: None,
            regles: d.regles,
        })
        .unwrap();
        let etat = m.etat();
        assert_eq!(
            etat.localite(m.l).unwrap().tresorerie,
            Montant::po(100) + bilan.po_net
        );
        assert_eq!(etat.jours_sans_recettes(m.l), 0);
        assert_eq!(
            etat.repartitions[&CleUnite::Salle(m.bar)],
            d.affectations[0].1
        );
    }

    #[test]
    fn capitaux_produits_payes_au_cout_produit() {
        let mut m = monde();
        let admises = regles::repartition::admises(m.r.salle("bar").unwrap(), &BTreeSet::new());
        let res = *admises.iter().find(|x| x.capital().is_some()).unwrap();
        let t = res.capital().unwrap();
        let etage = m.etat().salle(m.bar).unwrap().etage;
        m.ev(Evenement::SalleModifiee {
            id: m.bar,
            etage,
            nom: None,
            ressources: vec![res],
            revenus_actifs: true,
            brisee: false,
        })
        .unwrap();
        // jet de 20 : (20 + bonus) ÷ 10 unités, on en garde 1
        let mut d = demande(&m, 1, res, vec![Some(20)]);
        let bilan = m.etat().calculer_recettes(&d, &m.r).unwrap();
        assert!(bilan.capitaux_nets.get(t) >= 2);
        *d.conserves.get_mut(t) = 1;
        let bilan = m.etat().calculer_recettes(&d, &m.r).unwrap();
        let cout = m.r.valeur(res).unwrap().cout_produit_po as i64;
        assert_eq!(bilan.cout_conserves, Montant::po(cout));
        let avant = m.etat().localite(m.l).unwrap().capitaux.get(t);
        m.ev(Evenement::RecettesPercues {
            localite: d.localite,
            jours: 1,
            affectations: d.affectations.clone(),
            jets: d.jets.clone(),
            conserves: d.conserves,
            payeur: None,
            regles: d.regles,
        })
        .unwrap();
        let etat = m.etat();
        assert_eq!(etat.localite(m.l).unwrap().capitaux.get(t), avant + 1);
        assert_eq!(
            etat.localite(m.l).unwrap().tresorerie,
            Montant::po(100 - cout)
        );
        // option personnalisée : capitaux produits gratuits (sur le jour suivant)
        m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        let mut g = d.clone();
        g.regles.capitaux_produits_gratuits = true;
        assert_eq!(
            m.etat().calculer_recettes(&g, &m.r).unwrap().cout_conserves,
            Montant::ZERO
        );
        // garder plus que produit : refusé
        *d.conserves.get_mut(t) = 99;
        assert!(m.etat().calculer_recettes(&d, &m.r).is_err());
    }

    #[test]
    fn absence_reduit_recettes_et_use_les_capitaux() {
        let mut m = monde();
        let pj = m.pj;
        m.ev(Evenement::PersonnageDeplace { id: pj, vers: None })
            .unwrap();
        let c = m.ev(Evenement::JoursAvances { nombre: 14 }).unwrap();
        // usure : −1 à chaque type au 7e et au 14e jour
        assert_eq!(
            c.iter()
                .filter(|x| matches!(x, Consequence::UsureCapitaux { .. }))
                .count(),
            2
        );
        assert_eq!(
            m.etat().localite(m.l).unwrap().capitaux,
            CoutCapitaux {
                marchandises: 3,
                influence: 3,
                travail: 3,
                magie: 0
            }
        );
        // recettes sur 15 jours dont 14 d'absence : 2 tranches → −14 po
        let d = demande(&m, 15, Ressource::Po, vec![None; 15]);
        let bilan = m.etat().calculer_recettes(&d, &m.r).unwrap();
        assert_eq!(bilan.tranches_absence, 2);
        assert_eq!(bilan.po_net, Montant::pa((bilan.po_brut.pa - 140).max(0)));
    }

    #[test]
    fn gestionnaire_ralentit_ou_supprime_l_usure_et_prestige() {
        let mut m = monde();
        let (pj, l) = (m.pj, m.l);
        let g = m.c.nouvel_id_gestionnaire();
        m.ev(Evenement::GestionnaireEmbauche {
            id: g,
            reference: None,
            nom: "Ameiko".into(),
            salaire: Montant::pa(1),
            localite: l,
        })
        .unwrap();
        m.ev(Evenement::PersonnageDeplace { id: pj, vers: None })
            .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 14 }).unwrap();
        // une seule usure (14 jours au lieu de 7)
        assert_eq!(m.etat().localite(l).unwrap().capitaux.marchandises, 4);
        m.ev(Evenement::ReglageUsure {
            sans_usure_avec_gestionnaire: true,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 30 }).unwrap();
        assert_eq!(m.etat().localite(l).unwrap().capitaux.marchandises, 4);
        // retour après 44 jours avec gestionnaire payé : pas de test de prestige
        let c = m
            .ev(Evenement::PersonnageDeplace {
                id: pj,
                vers: Some(l),
            })
            .unwrap();
        assert!(c.is_empty());
        // sans gestionnaire : test de prestige DD = 31 − 10
        m.ev(Evenement::GestionnaireRenvoye { id: g }).unwrap();
        m.ev(Evenement::PersonnageDeplace { id: pj, vers: None })
            .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 31 }).unwrap();
        let c = m
            .ev(Evenement::PersonnageDeplace {
                id: pj,
                vers: Some(l),
            })
            .unwrap();
        assert!(matches!(
            c[..],
            [Consequence::TestPrestigeRequis { dd: 21, .. }]
        ));
        assert_eq!(m.etat().prestige_en_attente.get(&l), Some(&21));
    }

    #[test]
    fn batiment_ne_rapporte_qu_a_partir_de_sa_construction() {
        let mut m = monde();
        m.ev(Evenement::JoursAvances { nombre: 4 }).unwrap();
        // une seconde salle achetée au jour 6 : elle ne rapporte qu'à partir du jour 7
        let bar2 = m.c.nouvel_id_salle();
        let batiment = m.etat().salle(m.bar).unwrap().batiment;
        let etage = m.etat().salle(m.bar).unwrap().etage;
        m.ev(Evenement::SalleAjoutee {
            id: bar2,
            batiment,
            etage,
            reference: "bar".into(),
            nom: None,
            deja_construite: true,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        let bonus = m.r.salle("bar").unwrap().recettes.as_ref().unwrap().bonus;
        let d = DemandeRecettes {
            localite: m.l,
            jours: 6,
            affectations: vec![(CleUnite::Salle(bar2), vec![(Ressource::Po, bonus)])],
            jets: vec![(Ressource::Po, vec![None; 6])],
            conserves: CoutCapitaux::default(),
            payeur: None,
            regles: ReglesTest::officielles(),
        };
        let bilan = m.etat().calculer_recettes(&d, &m.r).unwrap();
        // seul le jour 7 rapporte : 10 + bonus
        assert_eq!(bilan.po_brut, Montant::pa(10 + bonus as i64));
        assert_eq!(
            bilan.par_jour[..5].iter().filter(|j| !j.is_empty()).count(),
            0
        );
    }

    #[test]
    fn jours_en_attente_encaisses_une_seule_fois() {
        let mut m = monde();
        m.ev(Evenement::JoursAvances { nombre: 8 }).unwrap();
        assert_eq!(m.etat().jours_sans_recettes(m.l), 9);
        let percevoir = |m: &mut Monde, n: u32| {
            let d = demande(m, n, Ressource::Po, vec![None; n as usize]);
            m.ev(Evenement::RecettesPercues {
                localite: d.localite,
                jours: d.jours,
                affectations: d.affectations.clone(),
                jets: d.jets.clone(),
                conserves: d.conserves,
                payeur: None,
                regles: d.regles,
            })
        };
        // 4 jours sur 9 : il en reste 5
        percevoir(&mut m, 4).unwrap();
        assert_eq!(m.etat().jours_sans_recettes(m.l), 5);
        // pas plus que les jours en attente
        assert!(percevoir(&mut m, 6).is_err());
        percevoir(&mut m, 5).unwrap();
        assert_eq!(m.etat().jours_sans_recettes(m.l), 0);
        // plus rien à encaisser tant que le temps n'avance pas
        assert!(percevoir(&mut m, 1).is_err());
    }
}
