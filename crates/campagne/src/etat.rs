//! État courant, obtenu en appliquant les événements du journal un par un.

use crate::evenements::Evenement;
use crate::modele::*;
use domaine::{CoutCapitaux, Montant, TypeCapital};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Erreur {
    Inventaire(inventaire::Erreur),
    Notes(notes::Erreur),
    /// Refus venant des fiches de personnage (message déjà lisible).
    Fiche(String),
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
    /// Règle du don Prestige.
    Prestige(String),
    /// Limite de recrutement par le Prestige (GC p. 93).
    PrestigeInsuffisant {
        equipe: String,
        niveau: u32,
        prestige: i32,
    },
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
    ExtensionInconnue(String),
    /// Extension déjà présente, absente, ou dans un état qui ne permet pas l'action.
    Extension(String),
    OrganisationInconnue(IdOrganisation),
    EquipeInconnue(IdEquipe),
    GestionnaireInconnu(IdGestionnaire),
    EquipeReferentielInconnue(String),
    /// Le lieu de travail doit se trouver dans la localité de l'équipe.
    LieuHorsLocalite {
        equipe: String,
        batiment: String,
    },
    OrganisationNonVide(String),
    DelocalisationVersElleMeme,
    /// Phase des recettes refusée (affectation, jets, quantités…).
    Recettes(String),
    /// Vente, abandon ou réparation refusés.
    Actif(String),
    /// Mode aventure ou combat refusé (message déjà lisible).
    Aventure(String),
}

impl std::fmt::Display for Erreur {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Erreur::Fiche(m) => write!(f, "fiche : {m}"),
            Erreur::Inventaire(e) => write!(f, "{e}"),
            Erreur::Notes(e) => write!(f, "{e}"),
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
            Erreur::PrestigeInsuffisant {
                equipe,
                niveau,
                prestige,
            } => write!(
                f,
                "{equipe} : ses employés sont de niveau {niveau}, mais le meilleur Prestige parmi les                  personnages présents est {prestige} ; un personnage ne peut recruter que des employés                  de niveau inférieur ou égal à son Prestige (Guide de campagne p. 93)"
            ),
            Erreur::Prestige(m) => write!(f, "{m}"),
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
            Erreur::ExtensionInconnue(e) => write!(f, "extension absente du référentiel ({e})"),
            Erreur::Extension(m) => f.write_str(m),
            Erreur::OrganisationInconnue(id) => write!(f, "organisation inconnue ({id})"),
            Erreur::EquipeInconnue(id) => write!(f, "équipe inconnue ({id})"),
            Erreur::GestionnaireInconnu(id) => write!(f, "gestionnaire inconnu ({id})"),
            Erreur::EquipeReferentielInconnue(id) => {
                write!(f, "équipe absente du référentiel ({id})")
            }
            Erreur::LieuHorsLocalite { equipe, batiment } => write!(
                f,
                "{batiment} ne se trouve pas dans la localité de {equipe} : délocalisez d'abord l'équipe"
            ),
            Erreur::OrganisationNonVide(o) => write!(f, "l'organisation {o} contient encore des équipes"),
            Erreur::Recettes(m) => write!(f, "recettes : {m}"),
            Erreur::Actif(m) => f.write_str(m),
            Erreur::Aventure(m) => write!(f, "aventure : {m}"),
            Erreur::DelocalisationVersElleMeme => {
                f.write_str("l'équipe se trouve déjà dans cette localité")
            }
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
    pub organisations: BTreeMap<IdOrganisation, Organisation>,
    pub equipes: BTreeMap<IdEquipe, Equipe>,
    pub gestionnaires: BTreeMap<IdGestionnaire, GestionnaireEmploye>,
    /// Salaire d'un gestionnaire versé une fois par affectation (réglage ; défaut : une fois par jour).
    pub salaire_par_affectation: bool,
    // ---- présence, usure, recettes ----
    /// Localités où un PJ s'est déjà trouvé (l'entretien ne s'applique qu'à elles, GC p. 82).
    pub frequentees: std::collections::BTreeSet<IdLocalite>,
    /// Bâtiments mis en favori (recalculés à partir du journal).
    pub batiments_favoris: std::collections::BTreeSet<IdBatiment>,
    /// Jours consécutifs sans PJ présent, par localité.
    pub absence: BTreeMap<IdLocalite, u32>,
    /// Jours (numéros) sans PJ présent, par localité, pour la réduction des recettes (GC p. 84).
    pub jours_absents: BTreeMap<IdLocalite, std::collections::BTreeSet<u32>>,
    /// Dernier jour dont les recettes ont été perçues, par localité.
    pub derniere_recette: BTreeMap<IdLocalite, u32>,
    /// Premier jour de production de chaque unité (construite, achetée, recrutée…) : pas de
    /// recette pour les jours qui la précèdent.
    pub debut_production: BTreeMap<CleUnite, u32>,
    /// Dernière affectation de chaque unité, pour la reproposer.
    pub repartitions: BTreeMap<CleUnite, Vec<(domaine::Ressource, u32)>>,
    /// Réglage personnalisé : pas d'usure dans une localité qui a un gestionnaire payé.
    pub sans_usure_avec_gestionnaire: bool,
    /// Tests de prestige à faire (retour après 30 jours ou plus) : DD par localité (GC p. 82).
    pub prestige_en_attente: BTreeMap<IdLocalite, u32>,
    /// Ventes en attente d'un acheteur.
    pub ventes: Vec<Vente>,
    /// Limite des dépenses par PJ présent plutôt que par localité (PC-14, option).
    pub limite_par_pj: bool,
    /// Inventaires, objets et conteneurs (recalculés à partir du journal).
    pub inventaire: inventaire::Registre,
    /// Notes de la partie (recalculées à partir du journal).
    pub notes: notes::Registre,
    /// Fiches de personnage des PJ, par journal de fiche.
    pub fiches: BTreeMap<IdPersonnage, fiches::journal::Journal>,
    /// Acolytes du don Prestige : acolyte → maître.
    pub acolytes: BTreeMap<IdPersonnage, IdPersonnage>,
    /// Don Prestige : modificateurs cochés et partisans, par maître.
    pub prestige_don: BTreeMap<IdPersonnage, crate::prestige::PrestigeDon>,
    pub regles_personnages: crate::personnages::ReglesPersonnages,
    /// Mode aventure : horloge, santé, effets, ressources (`None` tant que le joueur ne l'a pas démarré).
    pub aventure: Option<aventure::EtatAventure>,
    /// Combats validés, du plus ancien au plus récent (historique, statistiques, export).
    pub combats: Vec<crate::aventure::CombatPasse>,
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
    EquipeRecrutee {
        equipe: IdEquipe,
        nom: String,
        organisation: String,
    },
    EquipeArrivee {
        equipe: IdEquipe,
        nom: String,
        localite: String,
    },
    /// La trésorerie d'une localité vient de passer sous zéro (salaires).
    TresorerieNegative {
        localite: String,
    },
    /// Usure des capitaux (GC p. 82) : −1 à chaque type en réserve.
    UsureCapitaux {
        localite: String,
    },
    /// Vente conclue : le bien est cédé, le produit arrive dans la localité.
    VenteConclue {
        libelle: String,
        po: Montant,
        capitaux: CoutCapitaux,
    },
    /// Retour après une longue absence : test de prestige à faire (GC p. 82).
    TestPrestigeRequis {
        localite: String,
        dd: u32,
    },
}

impl Etat {
    pub fn nouveau() -> Self {
        Etat {
            jour: 1,
            ..Etat::default()
        }
    }

    /// Fait passer `nombre` jours de calendrier : arrivées des transferts, chantiers, salaires,
    /// présence, ventes (jour par jour), puis le temps des fiches.
    pub(crate) fn avancer_jours(
        &mut self,
        nombre: u32,
        r: &regles::Referentiel,
    ) -> Result<Vec<Consequence>, Erreur> {
        let mut out = Vec::new();
        for _ in 0..nombre {
            self.jour += 1;
            let (arrives, restants): (Vec<_>, Vec<_>) = std::mem::take(&mut self.transferts)
                .into_iter()
                .partition(|t| t.jour_arrivee <= self.jour);
            self.transferts = restants;
            for t in arrives {
                self.livrer(&t)?;
                out.push(Consequence::TransfertArrive(t));
            }
            out.extend(self.avancer_chantiers(r));
            out.extend(self.avancer_delocalisations(r));
            out.extend(self.payer_salaires());
            out.extend(self.suivre_presence());
            out.extend(self.conclure_ventes());
        }
        self.faire_passer_le_temps_des_fiches(nombre);
        Ok(out)
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
        copie.marquer_debuts(self);
        *self = copie;
        Ok(c)
    }

    /// Unités qui produisent (ou produiraient avec l'option) et leur localité. Une délocalisation
    /// en voyage ne compte pas : la production reprend à l'arrivée.
    fn unites_candidates(&self) -> BTreeMap<CleUnite, IdLocalite> {
        let salles = self.salles.values().filter_map(|s| {
            let produit = match &s.statut {
                StatutSalle::Construite => true,
                StatutSalle::EnTravaux(c) => {
                    matches!(c.nature, NatureChantier::Amelioration { .. })
                }
                StatutSalle::AConstruire => false,
            };
            let l = self.batiments.get(&s.batiment)?.localite;
            produit.then_some((CleUnite::Salle(s.id), l))
        });
        let equipes = self.equipes.values().filter_map(|e| {
            let produit = match &e.statut {
                StatutEquipe::Recrutee => true,
                StatutEquipe::EnDelocalisation(d) => !d.en_voyage(),
                StatutEquipe::EnRecrutement(c) => {
                    matches!(c.nature, NatureChantier::Amelioration { .. })
                }
                StatutEquipe::ARecruter => false,
            };
            produit.then_some((CleUnite::Equipe(e.id), e.localite))
        });
        salles.chain(equipes).collect()
    }

    /// Enregistre le premier jour de production d'une unité qui devient productrice : le jour
    /// suivant la construction ou l'achat. Si la localité n'avait encore aucune unité productrice,
    /// ses recettes en attente commencent à ce jour-là (pas avant la construction ou l'achat).
    fn marquer_debuts(&mut self, avant: &Etat) {
        let anciennes = avant.unites_candidates();
        for (cle, l) in self.unites_candidates() {
            if anciennes.contains_key(&cle) {
                continue;
            }
            if !anciennes.values().any(|x| *x == l) && !self.derniere_recette.contains_key(&l) {
                self.derniere_recette.insert(l, self.jour);
            }
            self.debut_production.insert(cle, self.jour + 1);
        }
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
            Notes(commande) => {
                self.notes.appliquer(commande).map_err(Erreur::Notes)?;
            }
            Inventaire(commande) => {
                self.inventaire
                    .appliquer(commande)
                    .map_err(Erreur::Inventaire)?;
            }
            PersonnageDeplace { id, vers } => {
                if let Some(l) = vers {
                    self.localite(*l)?;
                }
                self.pj_mut(*id)?.localite = *vers;
                if let Some(l) = vers {
                    out.extend(self.arrivee_pj(*l));
                }
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
            LocaliteSupprimee { .. } | PersonnageSupprime { .. } => {
                self.appliquer_suppression(ev)?
            }
            InjectionPo {
                pj,
                localite,
                montant,
            } => {
                if montant.pc <= 0 {
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
                if montant.pc <= 0 {
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
                        if m.pc <= 0 {
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
                // Si une aventure est en cours, son horloge suit : les effets en cours expirent.
                self.suivre_les_jours_dans_l_aventure(*nombre, r)?;
                out.extend(self.avancer_jours(*nombre, r)?);
            }
            AventureDemarree { .. } | Aventure(_) | CombatValide { .. } | AventureTerminee => {
                out.extend(self.appliquer_aventure(ev, r)?)
            }
            Groupe { evenements, .. } => {
                for e in evenements {
                    out.extend(self.appliquer_sur_place(e, r)?);
                }
            }
            BatimentFavori { id, favori } => {
                self.batiment(*id)?;
                if *favori {
                    self.batiments_favoris.insert(*id);
                } else {
                    self.batiments_favoris.remove(id);
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
            | ExtensionAjoutee { .. }
            | ExtensionLancee { .. }
            | ExtensionAchetee { .. }
            | ExtensionRetiree { .. }
            | AmeliorationLancee { .. } => self.appliquer_batiment(ev, r)?,
            Paiement {
                payeur, montant, ..
            } => {
                if montant.pc <= 0 {
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
            Fiche { pj, evenement } => self.appliquer_fiche(*pj, evenement)?,
            ReglagePersonnages { regles } => self.regles_personnages = *regles,
            RecettesPercues { .. } => self.percevoir_recettes(ev, r)?,
            AbandonActif { .. }
            | VenteLancee { .. }
            | VenteAnnulee { .. }
            | SalleReparee { .. } => self.appliquer_vente(ev, r)?,
            ReglageUsure {
                sans_usure_avec_gestionnaire,
            } => self.sans_usure_avec_gestionnaire = *sans_usure_avec_gestionnaire,
            TestPrestigeResolu { localite, .. } => {
                self.prestige_en_attente.remove(localite);
            }
            ReglageSalaireGestionnaire { par_affectation } => {
                self.salaire_par_affectation = *par_affectation
            }
            OrganisationCreee { .. }
            | OrganisationModifiee { .. }
            | EquipeAjoutee { .. }
            | EquipeModifiee { .. }
            | EquipeAffectee { .. }
            | EquipeRetiree { .. }
            | RecrutementLance { .. }
            | EquipeAchetee { .. }
            | AmeliorationEquipeLancee { .. }
            | DelocalisationLancee { .. }
            | GestionnaireEmbauche { .. }
            | GestionnaireModifie { .. }
            | GestionnaireRenvoye { .. } => self.appliquer_organisation(ev, r)?,
            PrestigeDonRegle { .. }
            | AcolyteLie { .. }
            | AcolyteDelie { .. }
            | PartisanAjoute { .. }
            | PartisanRetire { .. } => self.appliquer_prestige(ev)?,
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
