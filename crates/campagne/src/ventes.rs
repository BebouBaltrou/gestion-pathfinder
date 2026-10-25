//! Vente et abandon des biens (GC p. 84, phase d'activité, 4e et 5e étapes) et réparation
//! des salles brisées (GC p. 92).
//!
//! - Abandon : immédiat, sans compensation.
//! - Vente : « un montant égal à la moitié de son prix d'achat et de son coût de construction
//!   (le coût […] s'exprime en termes de po ou de Marchandises, Influence, Travail ou Magie) ».
//!   Décision de l'utilisateur (23/10/2026) : la moitié du prix d'achat, en po (modifiable).
//!   Délai pour trouver un acheteur saisi par le joueur (3d6 jours dans 75 % des cas, −1d6 par
//!   point d'Influence dépensé, minimum 0) ; dans les 25 % restants, aucun acheteur : le joueur
//!   garde ou abandonne le bien. Le bien reste au personnage jusqu'à la vente, puis le produit
//!   arrive dans la localité du bien.
//! - Réparation d'une salle brisée : « en règle générale […] un montant de po ou de capitaux
//!   égal à la moitié du prix de construction de la salle » ; immédiate.

use crate::etat::{Consequence, Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::*;
use domaine::{CoutCapitaux, Montant, TypeCapital};
use regles::Referentiel;

fn moitie(c: CoutCapitaux) -> CoutCapitaux {
    let mut m = c;
    for t in TypeCapital::TOUS {
        *m.get_mut(t) = c.get(t) / 2;
    }
    m
}

impl Etat {
    /// Localité où se trouve un bien.
    pub fn localite_actif(&self, a: Actif) -> Result<IdLocalite, Erreur> {
        Ok(match a {
            Actif::Batiment(b) => self.batiment(b)?.localite,
            Actif::Salle(s) => self.batiment(self.salle(s)?.batiment)?.localite,
            Actif::Organisation(o) => self.organisation(o)?.localite,
            Actif::Equipe(e) => self.equipe(e)?.localite,
        })
    }

    pub fn libelle_actif(&self, a: Actif, r: &Referentiel) -> String {
        match a {
            Actif::Batiment(b) => self
                .batiments
                .get(&b)
                .map(|x| format!("bâtiment {}", x.nom))
                .unwrap_or(b.to_string()),
            Actif::Salle(s) => self
                .salles
                .get(&s)
                .map(|x| {
                    let b = self
                        .batiments
                        .get(&x.batiment)
                        .map(|b| b.nom.clone())
                        .unwrap_or_default();
                    format!("{} ({b})", self.nom_salle(x, r))
                })
                .unwrap_or(s.to_string()),
            Actif::Organisation(o) => self
                .organisations
                .get(&o)
                .map(|x| format!("organisation {}", x.nom))
                .unwrap_or(o.to_string()),
            Actif::Equipe(e) => self
                .equipes
                .get(&e)
                .map(|x| {
                    let o = self
                        .organisations
                        .get(&x.organisation)
                        .map(|o| o.nom.clone())
                        .unwrap_or_default();
                    format!("{} ({o})", self.nom_equipe(x, r))
                })
                .unwrap_or(e.to_string()),
        }
    }

    pub fn en_vente(&self, a: Actif) -> Option<&Vente> {
        self.ventes.iter().find(|v| v.cible == a)
    }

    /// Prix et coût des parties construites ou recrutées d'un bien (pour la vente).
    pub fn valeur_actif(&self, a: Actif, r: &Referentiel) -> (Montant, CoutCapitaux) {
        let salle = |s: &SalleBatie| {
            let (mut p, mut c) = match s.statut {
                StatutSalle::Construite => r
                    .salle(&s.reference)
                    .map(|u| (u.prix_po.unwrap_or(0), u.cout.unwrap_or_default()))
                    .unwrap_or_default(),
                _ => (0, CoutCapitaux::default()),
            };
            // extensions terminées : leur prix s'ajoute à celui de la salle
            for x in &s.extensions {
                if x.statut == StatutExtension::Construite {
                    p += x.prix_po;
                    c = c + x.cout;
                }
            }
            (p, c)
        };
        let equipe = |e: &Equipe| match e.statut {
            StatutEquipe::Recrutee | StatutEquipe::EnDelocalisation(_) => r
                .equipe(&e.reference)
                .map(|u| (u.prix_po.unwrap_or(0), u.cout.unwrap_or_default()))
                .unwrap_or_default(),
            _ => (0, CoutCapitaux::default()),
        };
        let somme = |v: Vec<(u32, CoutCapitaux)>| {
            v.into_iter()
                .fold((0u32, CoutCapitaux::default()), |(p, c), (q, d)| {
                    (p + q, c + d)
                })
        };
        let (po, cout) = match a {
            Actif::Batiment(b) => somme(self.salles_batiment(b).map(salle).collect()),
            Actif::Salle(s) => self.salles.get(&s).map(salle).unwrap_or_default(),
            Actif::Organisation(o) => somme(self.equipes_organisation(o).map(equipe).collect()),
            Actif::Equipe(e) => self.equipes.get(&e).map(equipe).unwrap_or_default(),
        };
        (Montant::po(po as i64), cout)
    }

    /// Produit de vente proposé : moitié du prix en po, ou moitié du coût en capitaux.
    pub fn produit_vente(&self, a: Actif, r: &Referentiel) -> (Montant, CoutCapitaux) {
        let (po, cout) = self.valeur_actif(a, r);
        (Montant::pa(po.pc / 20), moitie(cout))
    }

    /// Coût de réparation proposé d'une salle : moitié du prix en po ou du coût en capitaux.
    pub fn cout_reparation(&self, s: IdSalle, r: &Referentiel) -> (Montant, CoutCapitaux) {
        let Some(u) = self.salles.get(&s).and_then(|x| r.salle(&x.reference)) else {
            return (Montant::ZERO, CoutCapitaux::default());
        };
        (
            Montant::pa(u.prix_po.unwrap_or(0) as i64 * 5),
            moitie(u.cout.unwrap_or_default()),
        )
    }

    /// Retire un bien et tout ce qui en dépend.
    fn retirer_actif(&mut self, a: Actif) {
        match a {
            Actif::Batiment(b) => {
                self.salles.retain(|_, s| s.batiment != b);
                for e in self.equipes.values_mut() {
                    if e.lieu == Some(b) {
                        e.lieu = None;
                    }
                }
                for g in self.gestionnaires.values_mut() {
                    g.affectations.retain(|x| *x != Affectation::Batiment(b));
                }
                self.batiments.remove(&b);
                self.ventes.retain(
                    |v| !matches!(v.cible, Actif::Salle(s) if !self.salles.contains_key(&s)),
                );
            }
            Actif::Salle(s) => {
                self.salles.remove(&s);
            }
            Actif::Organisation(o) => {
                self.equipes.retain(|_, e| e.organisation != o);
                for g in self.gestionnaires.values_mut() {
                    g.affectations
                        .retain(|x| *x != Affectation::Organisation(o));
                }
                self.organisations.remove(&o);
                self.ventes.retain(
                    |v| !matches!(v.cible, Actif::Equipe(e) if !self.equipes.contains_key(&e)),
                );
            }
            Actif::Equipe(e) => {
                self.equipes.remove(&e);
            }
        }
        self.ventes.retain(|v| v.cible != a);
    }

    pub(crate) fn appliquer_vente(
        &mut self,
        ev: &Evenement,
        r: &Referentiel,
    ) -> Result<(), Erreur> {
        match ev {
            Evenement::AbandonActif { cible } => {
                self.localite_actif(*cible)?;
                self.retirer_actif(*cible);
            }
            Evenement::VenteLancee {
                cible,
                jours,
                influence,
                po,
                capitaux,
            } => {
                let l = self.localite_actif(*cible)?;
                if self.en_vente(*cible).is_some() {
                    return Err(Erreur::Actif(format!(
                        "{} est déjà en vente",
                        self.libelle_actif(*cible, r)
                    )));
                }
                if po.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                if *influence > 0 {
                    let loc = self.localite_mut(l)?;
                    if loc.capitaux.influence < *influence {
                        return Err(Erreur::CapitalInsuffisant {
                            localite: loc.nom.clone(),
                            capital: TypeCapital::Influence,
                            disponible: loc.capitaux.influence,
                            demande: *influence,
                        });
                    }
                    loc.capitaux.influence -= influence;
                }
                let libelle = self.libelle_actif(*cible, r);
                self.ventes.push(Vente {
                    cible: *cible,
                    libelle,
                    localite: l,
                    jour_fin: self.jour + jours,
                    po: *po,
                    capitaux: *capitaux,
                });
                // délai nul : vente conclue sur-le-champ
                if *jours == 0 {
                    self.conclure_ventes();
                }
            }
            Evenement::VenteAnnulee { cible } => {
                if self.en_vente(*cible).is_none() {
                    return Err(Erreur::Actif("ce bien n'est pas en vente".into()));
                }
                self.ventes.retain(|v| v.cible != *cible);
            }
            Evenement::SalleReparee {
                salle,
                payeur,
                po,
                capitaux,
            } => {
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                if !s.brisee {
                    return Err(Erreur::Actif(format!("{nom} n'est pas brisée")));
                }
                if po.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                let l = self.batiment(s.batiment)?.localite;
                let loc = self.localite(l)?;
                for t in TypeCapital::TOUS {
                    if loc.capitaux.get(t) < capitaux.get(t) {
                        return Err(Erreur::CapitalInsuffisant {
                            localite: loc.nom.clone(),
                            capital: t,
                            disponible: loc.capitaux.get(t),
                            demande: capitaux.get(t),
                        });
                    }
                }
                if po.pc > 0 {
                    self.debiter(payeur.unwrap_or(Payeur::Localite(l)), *po)?;
                }
                let loc = self.localite_mut(l)?;
                for t in TypeCapital::TOUS {
                    *loc.capitaux.get_mut(t) -= capitaux.get(t);
                }
                self.salles.get_mut(salle).expect("salle").brisee = false;
            }
            _ => unreachable!("événement de vente attendu"),
        }
        Ok(())
    }

    /// Ventes dont l'acheteur paie aujourd'hui.
    pub(crate) fn conclure_ventes(&mut self) -> Vec<Consequence> {
        let jour = self.jour;
        let (finies, restantes): (Vec<Vente>, Vec<Vente>) = std::mem::take(&mut self.ventes)
            .into_iter()
            .partition(|v| v.jour_fin <= jour);
        self.ventes = restantes;
        let mut out = Vec::new();
        for v in finies {
            if self.localite_actif(v.cible).is_err() {
                continue; // bien disparu entre-temps
            }
            self.retirer_actif(v.cible);
            if let Some(l) = self.localites.get_mut(&v.localite) {
                l.tresorerie = l.tresorerie + v.po;
                l.capitaux = l.capitaux + v.capitaux;
            }
            out.push(Consequence::VenteConclue {
                libelle: v.libelle,
                po: v.po,
                capitaux: v.capitaux,
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::construction::evenements_depuis_modele;
    use crate::etat::{Consequence, Erreur};
    use crate::evenements::Evenement;
    use crate::modele::*;
    use crate::Campagne;
    use domaine::{CoutCapitaux, Montant};

    fn monde() -> (Campagne, regles::Referentiel, IdLocalite, IdBatiment) {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let l = c.nouvel_id_localite();
        let b = c.nouvel_id_batiment();
        c.executer(
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
            &r,
        )
        .unwrap();
        let modele = r.modele_batiment("taverne").unwrap();
        let (mut evs, _) = evenements_depuis_modele(
            &r,
            modele,
            b,
            l,
            "Taverne",
            c.nouvel_id_etage(),
            c.nouvel_id_salle(),
        );
        for e in evs.iter_mut() {
            if let Evenement::SalleAjoutee {
                deja_construite, ..
            } = e
            {
                *deja_construite = true;
            }
        }
        c.executer(
            Evenement::Groupe {
                libelle: "Taverne".into(),
                evenements: evs,
            },
            &r,
        )
        .unwrap();
        (c, r, l, b)
    }

    #[test]
    fn vente_d_un_batiment() {
        let (mut c, r, l, b) = monde();
        let modele = r.modele_batiment("taverne").unwrap();
        let (po, cap) = c.etat(&r).produit_vente(Actif::Batiment(b), &r);
        assert_eq!(po, Montant::pa(modele.prix_po as i64 * 5));
        assert_eq!(cap.travail, modele.cout.travail / 2);
        c.executer(
            Evenement::VenteLancee {
                cible: Actif::Batiment(b),
                jours: 10,
                influence: 0,
                po,
                capitaux: CoutCapitaux::default(),
            },
            &r,
        )
        .unwrap();
        // toujours possédé pendant la recherche d'un acheteur
        c.executer(Evenement::JoursAvances { nombre: 9 }, &r)
            .unwrap();
        assert!(c.etat(&r).batiments.contains_key(&b));
        let cons = c
            .executer(Evenement::JoursAvances { nombre: 1 }, &r)
            .unwrap();
        assert!(matches!(cons[..], [Consequence::VenteConclue { .. }]));
        let e = c.etat(&r);
        assert!(!e.batiments.contains_key(&b));
        assert!(e.salles.is_empty());
        assert_eq!(e.localite(l).unwrap().tresorerie, po);
    }

    #[test]
    fn abandon_et_reparation() {
        let (mut c, r, l, b) = monde();
        let s = *c.etat(&r).salles.keys().next().unwrap();
        let ref_salle = c.etat(&r).salles[&s].reference.clone();
        let u = r.salle(&ref_salle).unwrap().clone();
        // réparer une salle intacte : refusé
        assert!(matches!(
            c.executer(
                Evenement::SalleReparee {
                    salle: s,
                    payeur: None,
                    po: Montant::ZERO,
                    capitaux: CoutCapitaux::default()
                },
                &r
            ),
            Err(Erreur::Actif(_))
        ));
        let sb = c.etat(&r).salles[&s].clone();
        c.executer(
            Evenement::SalleModifiee {
                id: s,
                etage: sb.etage,
                nom: None,
                ressources: sb.ressources.clone(),
                revenus_actifs: true,
                brisee: true,
            },
            &r,
        )
        .unwrap();
        let (po, _) = c.etat(&r).cout_reparation(s, &r);
        assert_eq!(po, Montant::pa(u.prix_po.unwrap() as i64 * 5));
        c.executer(
            Evenement::AjustementLocalite {
                localite: l,
                delta_po: Montant::po(1000),
                ajout: CoutCapitaux::default(),
                retrait: CoutCapitaux::default(),
                motif: "test".into(),
            },
            &r,
        )
        .unwrap();
        c.executer(
            Evenement::SalleReparee {
                salle: s,
                payeur: None,
                po,
                capitaux: CoutCapitaux::default(),
            },
            &r,
        )
        .unwrap();
        assert!(!c.etat(&r).salles[&s].brisee);
        assert_eq!(
            c.etat(&r).localite(l).unwrap().tresorerie,
            Montant::po(1000) - po
        );
        c.executer(
            Evenement::AbandonActif {
                cible: Actif::Batiment(b),
            },
            &r,
        )
        .unwrap();
        assert!(c.etat(&r).salles.is_empty());
    }
}
