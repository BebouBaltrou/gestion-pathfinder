//! État courant, obtenu en appliquant les événements du journal un par un.

use crate::evenements::Evenement;
use crate::modele::*;
use domaine::{CoutCapitaux, Montant, TypeCapital};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Erreur {
    PersonnageInconnu(IdPersonnage),
    LocaliteInconnue(IdLocalite),
    IdDejaUtilise(String),
    NomVide,
    MontantNonPositif,
    CompteInsuffisant {
        pj: String,
        disponible: Montant,
        demande: Montant,
    },
    TresorerieInsuffisante {
        localite: String,
        disponible: Montant,
        demande: Montant,
    },
    CapitalInsuffisant {
        localite: String,
        capital: TypeCapital,
        disponible: u32,
        demande: u32,
    },
    InfluenceNonTransferable,
    TransfertVersElleMeme,
    CategorieInconnue(String),
    BatimentInconnu(IdBatiment),
    EtageInconnu(IdEtage),
    SalleInconnue(IdSalle),
    SalleReferentielInconnue(String),
    AucunPjPresent(String),
    StatutIncompatible {
        salle: String,
        attendu: &'static str,
    },
    SansCoutFixe(String),
    AmeliorationImpossible {
        de: String,
        vers: String,
    },
    RessourceNonAdmise {
        salle: String,
        ressource: domaine::Ressource,
    },
    FacteurInvalide(u32),
    EtageNonVide(String),
}

impl std::fmt::Display for Erreur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Erreur::PersonnageInconnu(id) => write!(f, "personnage inconnu ({id})"),
            Erreur::LocaliteInconnue(id) => write!(f, "localité inconnue ({id})"),
            Erreur::IdDejaUtilise(id) => write!(f, "identifiant déjà utilisé ({id})"),
            Erreur::NomVide => f.write_str("le nom ne peut pas être vide"),
            Erreur::MontantNonPositif => f.write_str("le montant doit être positif"),
            Erreur::CompteInsuffisant {
                pj,
                disponible,
                demande,
            } => {
                write!(
                    f,
                    "compte de {pj} insuffisant : {disponible} disponibles, {demande} demandés"
                )
            }
            Erreur::TresorerieInsuffisante {
                localite,
                disponible,
                demande,
            } => {
                write!(f, "trésorerie de {localite} insuffisante : {disponible} disponibles, {demande} demandés")
            }
            Erreur::CapitalInsuffisant {
                localite,
                capital,
                disponible,
                demande,
            } => {
                write!(
                    f,
                    "{localite} ne dispose que de {disponible} {capital} ({demande} demandés)"
                )
            }
            Erreur::InfluenceNonTransferable => f.write_str(
                "l'Influence est attachée à sa communauté et ne peut pas être transférée",
            ),
            Erreur::TransfertVersElleMeme => {
                f.write_str("la localité d'origine et la destination sont identiques")
            }
            Erreur::CategorieInconnue(id) => write!(f, "catégorie de communauté inconnue ({id})"),
            Erreur::BatimentInconnu(id) => write!(f, "bâtiment inconnu ({id})"),
            Erreur::EtageInconnu(id) => write!(f, "étage inconnu ({id})"),
            Erreur::SalleInconnue(id) => write!(f, "salle inconnue ({id})"),
            Erreur::SalleReferentielInconnue(id) => {
                write!(f, "salle absente du référentiel ({id})")
            }
            Erreur::AucunPjPresent(l) => write!(
                f,
                "aucun personnage n'est présent à {l} : un PJ doit se trouver dans la communauté \
                 au début des travaux (Guide de campagne p. 95)"
            ),
            Erreur::StatutIncompatible { salle, attendu } => {
                write!(f, "{salle} doit être {attendu}")
            }
            Erreur::SansCoutFixe(s) => write!(
                f,
                "{s} n'a pas de coût fixe : voir sa description dans le livre"
            ),
            Erreur::AmeliorationImpossible { de, vers } => {
                write!(f, "{de} ne s'améliore pas en {vers} selon le référentiel")
            }
            Erreur::RessourceNonAdmise { salle, ressource } => {
                write!(f, "{salle} ne peut pas produire de {ressource}")
            }
            Erreur::FacteurInvalide(k) => write!(f, "accélération invalide (×{k}) : 1, 2, 3 ou 4"),
            Erreur::EtageNonVide(e) => write!(f, "l'étage {e} contient des salles"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Etat {
    /// Jour de campagne courant (commence à 1).
    pub jour: u32,
    pub personnages: BTreeMap<IdPersonnage, Personnage>,
    pub localites: BTreeMap<IdLocalite, Localite>,
    pub transferts: Vec<Transfert>,
    pub batiments: BTreeMap<IdBatiment, Batiment>,
    /// Salles de tous les bâtiments ; l'ordre des id donne l'ordre dans l'étage.
    pub salles: BTreeMap<IdSalle, SalleBatie>,
    /// Limite des dépenses par PJ présent plutôt que par localité (PC-14, option).
    pub limite_par_pj: bool,
}

/// Effet notable d'un événement, pour l'historique (ex. arrivée d'un transfert).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Consequence {
    TransfertArrive(Transfert),
    SalleTerminee {
        salle: IdSalle,
        nom: String,
        batiment: String,
    },
}

impl Etat {
    pub fn nouveau() -> Self {
        Etat {
            jour: 1,
            ..Etat::default()
        }
    }

    pub fn pj(&self, id: IdPersonnage) -> Result<&Personnage, Erreur> {
        self.personnages
            .get(&id)
            .ok_or(Erreur::PersonnageInconnu(id))
    }
    pub fn localite(&self, id: IdLocalite) -> Result<&Localite, Erreur> {
        self.localites.get(&id).ok_or(Erreur::LocaliteInconnue(id))
    }
    pub(crate) fn pj_mut(&mut self, id: IdPersonnage) -> Result<&mut Personnage, Erreur> {
        self.personnages
            .get_mut(&id)
            .ok_or(Erreur::PersonnageInconnu(id))
    }
    /// Retire des po d'une trésorerie ou d'un compte de PJ ; refusé si le solde ne suffit pas.
    pub(crate) fn debiter(&mut self, payeur: Payeur, montant: Montant) -> Result<(), Erreur> {
        match payeur {
            Payeur::Localite(id) => {
                let l = self.localite_mut(id)?;
                if l.tresorerie < montant {
                    return Err(Erreur::TresorerieInsuffisante {
                        localite: l.nom.clone(),
                        disponible: l.tresorerie,
                        demande: montant,
                    });
                }
                l.tresorerie = l.tresorerie - montant;
            }
            Payeur::Personnage(id) => {
                let p = self.pj_mut(id)?;
                if p.compte < montant {
                    return Err(Erreur::CompteInsuffisant {
                        pj: p.nom.clone(),
                        disponible: p.compte,
                        demande: montant,
                    });
                }
                p.compte = p.compte - montant;
            }
        }
        Ok(())
    }

    pub(crate) fn localite_mut(&mut self, id: IdLocalite) -> Result<&mut Localite, Erreur> {
        self.localites
            .get_mut(&id)
            .ok_or(Erreur::LocaliteInconnue(id))
    }

    /// Capitaux en route vers une localité.
    pub fn en_route_vers(&self, id: IdLocalite) -> impl Iterator<Item = &Transfert> {
        self.transferts.iter().filter(move |t| t.vers == id)
    }

    pub(crate) fn retirer_capitaux(l: &mut Localite, retrait: &CoutCapitaux) -> Result<(), Erreur> {
        for c in TypeCapital::TOUS {
            let (dispo, dem) = (l.capitaux.get(c), retrait.get(c));
            if dem > dispo {
                return Err(Erreur::CapitalInsuffisant {
                    localite: l.nom.clone(),
                    capital: c,
                    disponible: dispo,
                    demande: dem,
                });
            }
        }
        for c in TypeCapital::TOUS {
            *l.capitaux.get_mut(c) -= retrait.get(c);
        }
        Ok(())
    }

    fn verifier_categorie(r: &regles::Referentiel, c: &CategorieLocalite) -> Result<(), Erreur> {
        match c {
            CategorieLocalite::Officielle { id } if r.communaute(id).is_none() => {
                Err(Erreur::CategorieInconnue(id.clone()))
            }
            _ => Ok(()),
        }
    }

    /// Applique un événement. En cas d'erreur, l'état n'est pas modifié.
    pub fn appliquer(
        &mut self,
        ev: &Evenement,
        r: &regles::Referentiel,
    ) -> Result<Vec<Consequence>, Erreur> {
        let mut copie = self.clone();
        let c = copie.appliquer_sur_place(ev, r)?;
        *self = copie;
        Ok(c)
    }

    pub(crate) fn appliquer_sur_place(
        &mut self,
        ev: &Evenement,
        r: &regles::Referentiel,
    ) -> Result<Vec<Consequence>, Erreur> {
        use Evenement::*;
        let mut out = Vec::new();
        match ev {
            PersonnageCree {
                id,
                nom,
                compte_initial,
                prestige,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if self.personnages.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                self.personnages.insert(
                    *id,
                    Personnage {
                        id: *id,
                        nom: nom.clone(),
                        compte: *compte_initial,
                        localite: None,
                        prestige: *prestige,
                    },
                );
            }
            PersonnageModifie { id, nom, prestige } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                let p = self.pj_mut(*id)?;
                p.nom = nom.clone();
                p.prestige = *prestige;
            }
            PersonnageDeplace { id, vers } => {
                if let Some(l) = vers {
                    self.localite(*l)?;
                }
                self.pj_mut(*id)?.localite = *vers;
            }
            LocaliteCreee {
                id,
                nom,
                genre,
                population,
                categorie,
                notes,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if self.localites.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                Self::verifier_categorie(r, categorie)?;
                self.localites.insert(
                    *id,
                    Localite {
                        id: *id,
                        nom: nom.clone(),
                        genre: genre.clone(),
                        population: *population,
                        categorie: categorie.clone(),
                        tresorerie: Montant::ZERO,
                        capitaux: CoutCapitaux::default(),
                        notes: notes.clone(),
                    },
                );
            }
            LocaliteModifiee {
                id,
                nom,
                genre,
                population,
                categorie,
                notes,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                Self::verifier_categorie(r, categorie)?;
                let l = self.localite_mut(*id)?;
                l.nom = nom.clone();
                l.genre = genre.clone();
                l.population = *population;
                l.categorie = categorie.clone();
                l.notes = notes.clone();
            }
            InjectionPo {
                pj,
                localite,
                montant,
            } => {
                if montant.pa <= 0 {
                    return Err(Erreur::MontantNonPositif);
                }
                self.localite(*localite)?;
                let p = self.pj_mut(*pj)?;
                if p.compte < *montant {
                    return Err(Erreur::CompteInsuffisant {
                        pj: p.nom.clone(),
                        disponible: p.compte,
                        demande: *montant,
                    });
                }
                p.compte = p.compte - *montant;
                let l = self.localite_mut(*localite)?;
                l.tresorerie = l.tresorerie + *montant;
            }
            RetraitPo {
                localite,
                pj,
                montant,
            } => {
                if montant.pa <= 0 {
                    return Err(Erreur::MontantNonPositif);
                }
                self.pj(*pj)?;
                let l = self.localite_mut(*localite)?;
                if l.tresorerie < *montant {
                    return Err(Erreur::TresorerieInsuffisante {
                        localite: l.nom.clone(),
                        disponible: l.tresorerie,
                        demande: *montant,
                    });
                }
                l.tresorerie = l.tresorerie - *montant;
                let p = self.pj_mut(*pj)?;
                p.compte = p.compte + *montant;
            }
            AjustementCompte { pj, delta, .. } => {
                let p = self.pj_mut(*pj)?;
                p.compte = p.compte + *delta;
            }
            AjustementLocalite {
                localite,
                delta_po,
                ajout,
                retrait,
                ..
            } => {
                let l = self.localite_mut(*localite)?;
                Self::retirer_capitaux(l, retrait)?;
                l.capitaux = l.capitaux + *ajout;
                l.tresorerie = l.tresorerie + *delta_po;
            }
            TransfertEnvoye {
                id,
                de,
                vers,
                lot,
                duree_jours,
            } => {
                if de == vers {
                    return Err(Erreur::TransfertVersElleMeme);
                }
                if self.transferts.iter().any(|t| t.id == *id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                self.localite(*vers)?;
                let l = self.localite_mut(*de)?;
                match lot {
                    Lot::Capital(TypeCapital::Influence, _) => {
                        return Err(Erreur::InfluenceNonTransferable)
                    }
                    Lot::Capital(c, n) => {
                        if *n == 0 {
                            return Err(Erreur::MontantNonPositif);
                        }
                        let mut retrait = CoutCapitaux::default();
                        *retrait.get_mut(*c) = *n;
                        Self::retirer_capitaux(l, &retrait)?;
                    }
                    Lot::Po(m) => {
                        if m.pa <= 0 {
                            return Err(Erreur::MontantNonPositif);
                        }
                        if l.tresorerie < *m {
                            return Err(Erreur::TresorerieInsuffisante {
                                localite: l.nom.clone(),
                                disponible: l.tresorerie,
                                demande: *m,
                            });
                        }
                        l.tresorerie = l.tresorerie - *m;
                    }
                }
                let t = Transfert {
                    id: *id,
                    de: *de,
                    vers: *vers,
                    lot: *lot,
                    jour_depart: self.jour,
                    jour_arrivee: self.jour + duree_jours,
                };
                if *duree_jours == 0 {
                    self.livrer(&t)?;
                    out.push(Consequence::TransfertArrive(t));
                } else {
                    self.transferts.push(t);
                }
            }
            JoursAvances { nombre } => {
                // Jour par jour : arrivées des transferts, puis avancement des chantiers.
                for _ in 0..*nombre {
                    self.jour += 1;
                    let (arrives, restants): (Vec<_>, Vec<_>) =
                        std::mem::take(&mut self.transferts)
                            .into_iter()
                            .partition(|t| t.jour_arrivee <= self.jour);
                    self.transferts = restants;
                    for t in arrives {
                        self.livrer(&t)?;
                        out.push(Consequence::TransfertArrive(t));
                    }
                    out.extend(self.avancer_chantiers(r));
                }
            }
            Groupe { evenements, .. } => {
                for e in evenements {
                    out.extend(self.appliquer_sur_place(e, r)?);
                }
            }
            BatimentCree { .. }
            | BatimentModifie { .. }
            | EtageAjoute { .. }
            | EtageModifie { .. }
            | SalleAjoutee { .. }
            | SalleModifiee { .. }
            | SalleRetiree { .. }
            | ConstructionLancee { .. }
            | SalleAchetee { .. }
            | AmeliorationLancee { .. } => self.appliquer_batiment(ev, r)?,
            Paiement {
                payeur, montant, ..
            } => {
                if montant.pa <= 0 {
                    return Err(Erreur::MontantNonPositif);
                }
                self.debiter(*payeur, *montant)?;
            }
            CapitauxAchetes {
                localite,
                payeur,
                capitaux,
            } => {
                if capitaux.unites() == 0 {
                    return Err(Erreur::MontantNonPositif);
                }
                let prix = Montant::po(regles::calculs::valeur_achat(r, capitaux) as i64);
                self.debiter(*payeur, prix)?;
                let l = self.localite_mut(*localite)?;
                l.capitaux = l.capitaux + *capitaux;
            }
            ReglageLimiteDepense { par_pj } => self.limite_par_pj = *par_pj,
            Note { .. } => {}
        }
        Ok(out)
    }

    fn livrer(&mut self, t: &Transfert) -> Result<(), Erreur> {
        let l = self.localite_mut(t.vers)?;
        match t.lot {
            Lot::Po(m) => l.tresorerie = l.tresorerie + m,
            Lot::Capital(c, n) => *l.capitaux.get_mut(c) += n,
        }
        Ok(())
    }

    /// Limite de dépense quotidienne de la localité (GC p. 81), officielle ou personnalisée.
    /// Limite quotidienne des dépenses de la localité (GC p. 81). Par défaut une seule limite
    /// par localité ; avec l'option « par PJ », la limite est multipliée par le nombre de PJ
    /// présents (au moins 1, pour que les chantiers continuent en leur absence).
    pub fn limite_depense(&self, id: IdLocalite, r: &regles::Referentiel) -> Option<u32> {
        let base = self.limite_depense_base(id, r)?;
        Some(if self.limite_par_pj {
            base * self.pj_presents(id).max(1)
        } else {
            base
        })
    }

    /// Nombre de PJ présents dans la localité.
    pub fn pj_presents(&self, id: IdLocalite) -> u32 {
        self.personnages
            .values()
            .filter(|p| p.localite == Some(id))
            .count() as u32
    }

    /// Limite imprimée (ou personnalisée) de la catégorie de la localité, pour un personnage.
    pub fn limite_depense_base(&self, id: IdLocalite, r: &regles::Referentiel) -> Option<u32> {
        match &self.localites.get(&id)?.categorie {
            CategorieLocalite::Officielle { id } => r.communaute(id).map(|c| c.limite_depense_jour),
            CategorieLocalite::Personnalisee {
                limite_depense_jour,
                ..
            } => Some(*limite_depense_jour),
        }
    }
}
