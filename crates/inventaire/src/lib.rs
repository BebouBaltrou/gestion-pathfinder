//! Moteur de l'inventaire, sans interface.
//!
//! - Un [`Modele`] est une entrée du catalogue (poids, valeur, type) : il ne change pas.
//! - Un [`Exemplaire`] est un objet réel (quantité, conteneur, note) qui renvoie à son modèle.
//! - Un exemplaire qui porte un [`Conteneur`] est aussi un inventaire.
//! - Chaque inventaire racine appartient à un [`Proprietaire`] générique.
//!
//! Aucun calcul de règle ici : bonus et effets viennent du moteur de règles.
//! Ce crate ne fait que le poids (somme), la capacité et les restrictions de contenu.

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Add;

pub mod catalogue;

use domaine::Montant;

/// Identifiant d'un exemplaire, attribué par le registre.
pub type Id = u64;

/// Poids en grammes, pour rester en entiers (le livre compte en g et en kg).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Poids(pub i64);

impl Poids {
    pub const ZERO: Poids = Poids(0);

    /// Poids de `n` unités de ce poids.
    pub fn fois(self, n: u32) -> Poids {
        Poids(self.0 * i64::from(n))
    }
}

impl Add for Poids {
    type Output = Poids;

    fn add(self, autre: Poids) -> Poids {
        Poids(self.0 + autre.0)
    }
}

/// Entrée du catalogue : ce qui est commun à tous les exemplaires d'un objet.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Modele {
    pub id: String,
    pub nom: String,
    /// Type défini en données (« arme », « munition », « contenant »…).
    pub type_objet: String,
    /// Prix d'une unité ; `None` si le livre ne le précise pas (« — » ou « spécial »).
    pub valeur: Option<Montant>,
    /// Poids d'une unité ; `None` si le livre dit « spécial ». « — » (négligeable) vaut zéro.
    pub poids: Option<Poids>,
    /// Provenance : catalogue, saisie libre ou import, avec sa référence.
    pub source: String,
    /// Propriétés propres au type (dégâts, critique, bonus de CA…), définies en données.
    #[cfg_attr(feature = "serde", serde(default))]
    pub proprietes: BTreeMap<String, String>,
}

/// Capacités d'un inventaire porté par un exemplaire.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Conteneur {
    /// Poids maximal du contenu ; `None` = illimité.
    pub capacite: Option<Poids>,
    /// Types d'objets acceptés ; `None` = tous.
    pub types_autorises: Option<Vec<String>>,
    /// `false` pour un conteneur extradimensionnel : son contenu ne pèse pas sur le parent.
    pub contenu_compte: bool,
}

/// Un objet réel, dans un conteneur.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Exemplaire {
    pub id: Id,
    /// `None` pour un inventaire racine, qui n'est pas un objet.
    pub modele: Option<String>,
    pub quantite: u32,
    /// `None` seulement pour un inventaire racine.
    pub parent: Option<Id>,
    pub conteneur: Option<Conteneur>,
    pub note: String,
}

/// Propriétaire générique : le type et l'identifiant pointent vers une entité existante.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Proprietaire {
    /// Type défini en données (« personnage », « batiment », « salle », « employe », « autre »…).
    pub type_id: String,
    /// Identifiant de l'entité dans la campagne.
    pub ref_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Erreur {
    ModeleInconnu(String),
    ExemplaireInconnu(Id),
    PasUnConteneur(Id),
    QuantiteInvalide,
    /// Ce n'est pas une pile : un conteneur ou une racine ne se scinde pas ni ne se fusionne.
    PasEmpilable(Id),
    FusionImpossible(Id, Id),
    CapaciteDepassee(Id),
    TypeRefuse {
        conteneur: Id,
        type_objet: String,
    },
    /// Un conteneur ne peut pas entrer dans lui-même ou dans son contenu.
    CycleInterdit,
    ConteneurNonVide(Id),
    RacineNonSupprimable(Id),
    RacineImmobile(Id),
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Erreur::ModeleInconnu(m) => write!(f, "modèle d'objet inconnu : {m}"),
            Erreur::ExemplaireInconnu(id) => write!(f, "objet inconnu : {id}"),
            Erreur::PasUnConteneur(id) => write!(f, "l'objet {id} ne peut pas contenir d'objets"),
            Erreur::QuantiteInvalide => write!(f, "quantité invalide"),
            Erreur::PasEmpilable(id) => write!(f, "l'objet {id} ne se scinde ni ne se fusionne"),
            Erreur::FusionImpossible(a, b) => {
                write!(f, "les objets {a} et {b} ne se fusionnent pas")
            }
            Erreur::CapaciteDepassee(id) => write!(f, "capacité du conteneur {id} dépassée"),
            Erreur::TypeRefuse {
                conteneur,
                type_objet,
            } => {
                write!(f, "le conteneur {conteneur} n'accepte pas « {type_objet} »")
            }
            Erreur::CycleInterdit => write!(f, "un conteneur ne peut pas entrer dans son contenu"),
            Erreur::ConteneurNonVide(id) => write!(f, "le conteneur {id} n'est pas vide"),
            Erreur::RacineNonSupprimable(id) => write!(f, "l'inventaire {id} ne se supprime pas"),
            Erreur::RacineImmobile(id) => write!(f, "l'inventaire {id} ne se déplace pas"),
        }
    }
}

impl std::error::Error for Erreur {}

/// Ensemble des modèles, exemplaires et inventaires d'une campagne.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Registre {
    modeles: BTreeMap<String, Modele>,
    exemplaires: BTreeMap<Id, Exemplaire>,
    /// Propriétaire de chaque inventaire racine.
    proprietaires: BTreeMap<Id, Proprietaire>,
    dernier_id: Id,
}

impl Registre {
    pub fn ajouter_modele(&mut self, modele: Modele) {
        self.modeles.insert(modele.id.clone(), modele);
    }

    pub fn modele(&self, id: &str) -> Option<&Modele> {
        self.modeles.get(id)
    }

    pub fn exemplaire(&self, id: Id) -> Result<&Exemplaire, Erreur> {
        self.exemplaires
            .get(&id)
            .ok_or(Erreur::ExemplaireInconnu(id))
    }

    pub fn proprietaire(&self, racine: Id) -> Option<&Proprietaire> {
        self.proprietaires.get(&racine)
    }

    /// Crée l'inventaire racine d'un propriétaire (non supprimable).
    pub fn creer_inventaire(&mut self, proprietaire: Proprietaire, conteneur: Conteneur) -> Id {
        let id = self.nouvel_id();
        self.exemplaires.insert(
            id,
            Exemplaire {
                id,
                modele: None,
                quantite: 1,
                parent: None,
                conteneur: Some(conteneur),
                note: String::new(),
            },
        );
        self.proprietaires.insert(id, proprietaire);
        id
    }

    /// Ajoute `quantite` unités d'un modèle dans le conteneur `dans`.
    pub fn ajouter(&mut self, modele_id: &str, quantite: u32, dans: Id) -> Result<Id, Erreur> {
        self.inserer(modele_id, quantite, dans, None)
    }

    /// Ajoute un objet qui est aussi un conteneur (sac, étui, coffre…).
    pub fn ajouter_conteneur(
        &mut self,
        modele_id: &str,
        conteneur: Conteneur,
        dans: Id,
    ) -> Result<Id, Erreur> {
        self.inserer(modele_id, 1, dans, Some(conteneur))
    }

    /// Déplace un objet (et son contenu) vers un autre conteneur, si la capacité et les
    /// restrictions le permettent. Un objet n'est jamais copié.
    pub fn deplacer(&mut self, id: Id, vers: Id) -> Result<(), Erreur> {
        let exemplaire = self.exemplaire(id)?.clone();
        if exemplaire.parent.is_none() {
            return Err(Erreur::RacineImmobile(id));
        }
        self.exemplaire(vers)?;
        if exemplaire.parent == Some(vers) {
            return Ok(());
        }
        let mut courant = Some(vers);
        while let Some(c) = courant {
            if c == id {
                return Err(Erreur::CycleInterdit);
            }
            courant = self.exemplaire(c)?.parent;
        }
        let type_objet = self.type_de(&exemplaire);
        let poids = self.poids_total(id);
        // Détaché le temps de la vérification : son poids ne compte pas deux fois
        // sur les conteneurs communs à l'ancien et au nouvel emplacement.
        self.exemplaire_mut(id)?.parent = None;
        if let Err(erreur) = self.verifier_accueil(vers, &type_objet, poids) {
            self.exemplaire_mut(id)?.parent = exemplaire.parent;
            return Err(erreur);
        }
        self.exemplaire_mut(id)?.parent = Some(vers);
        Ok(())
    }

    /// Supprime un objet. Un conteneur doit être vide, et un inventaire racine ne se supprime pas.
    pub fn retirer(&mut self, id: Id) -> Result<(), Erreur> {
        if self.exemplaire(id)?.parent.is_none() {
            return Err(Erreur::RacineNonSupprimable(id));
        }
        if !self.enfants(id).is_empty() {
            return Err(Erreur::ConteneurNonVide(id));
        }
        self.exemplaires.remove(&id);
        Ok(())
    }

    /// Sépare `quantite` unités d'une pile dans un nouvel exemplaire, au même endroit.
    pub fn scinder(&mut self, id: Id, quantite: u32) -> Result<Id, Erreur> {
        let exemplaire = self.exemplaire(id)?.clone();
        if exemplaire.conteneur.is_some() || exemplaire.parent.is_none() {
            return Err(Erreur::PasEmpilable(id));
        }
        if quantite == 0 || quantite >= exemplaire.quantite {
            return Err(Erreur::QuantiteInvalide);
        }
        self.exemplaire_mut(id)?.quantite -= quantite;
        let nouveau = self.nouvel_id();
        self.exemplaires.insert(
            nouveau,
            Exemplaire {
                id: nouveau,
                quantite,
                ..exemplaire
            },
        );
        Ok(nouveau)
    }

    /// Réunit deux piles identiques (même modèle, lieu et note) dans `garde`.
    pub fn fusionner(&mut self, garde: Id, absorbe: Id) -> Result<(), Erreur> {
        let a = self.exemplaire(garde)?.clone();
        let b = self.exemplaire(absorbe)?.clone();
        let identiques = a.modele == b.modele
            && a.parent == b.parent
            && a.note == b.note
            && a.parent.is_some()
            && a.conteneur.is_none()
            && b.conteneur.is_none();
        if !identiques || garde == absorbe {
            return Err(Erreur::FusionImpossible(garde, absorbe));
        }
        let total = a
            .quantite
            .checked_add(b.quantite)
            .ok_or(Erreur::QuantiteInvalide)?;
        self.exemplaire_mut(garde)?.quantite = total;
        self.exemplaires.remove(&absorbe);
        Ok(())
    }

    /// Poids d'un objet : son poids propre, plus son contenu s'il compte.
    pub fn poids_total(&self, id: Id) -> Poids {
        let Some(exemplaire) = self.exemplaires.get(&id) else {
            return Poids::ZERO;
        };
        let propre = self.poids_propre(exemplaire);
        match &exemplaire.conteneur {
            Some(c) if c.contenu_compte => propre + self.poids_brut_contenu(id),
            _ => propre,
        }
    }

    /// Ce qui est dans un conteneur, tout poids confondu (sert à la capacité).
    pub fn poids_brut_contenu(&self, conteneur: Id) -> Poids {
        self.enfants(conteneur)
            .into_iter()
            .fold(Poids::ZERO, |somme, enfant| {
                somme + self.poids_total(enfant)
            })
    }

    /// Objets placés directement dans un conteneur.
    pub fn enfants(&self, conteneur: Id) -> Vec<Id> {
        self.exemplaires
            .values()
            .filter(|e| e.parent == Some(conteneur))
            .map(|e| e.id)
            .collect()
    }

    /// Inventaire racine d'un objet.
    pub fn racine_de(&self, id: Id) -> Result<Id, Erreur> {
        let mut courant = self.exemplaire(id)?;
        while let Some(parent) = courant.parent {
            courant = self.exemplaire(parent)?;
        }
        Ok(courant.id)
    }

    fn inserer(
        &mut self,
        modele_id: &str,
        quantite: u32,
        dans: Id,
        conteneur: Option<Conteneur>,
    ) -> Result<Id, Erreur> {
        if quantite == 0 {
            return Err(Erreur::QuantiteInvalide);
        }
        let modele = self
            .modele(modele_id)
            .ok_or_else(|| Erreur::ModeleInconnu(modele_id.to_string()))?
            .clone();
        self.verifier_accueil(
            dans,
            &modele.type_objet,
            modele.poids.unwrap_or(Poids::ZERO).fois(quantite),
        )?;
        let id = self.nouvel_id();
        self.exemplaires.insert(
            id,
            Exemplaire {
                id,
                modele: Some(modele.id),
                quantite,
                parent: Some(dans),
                conteneur,
                note: String::new(),
            },
        );
        Ok(id)
    }

    /// Vérifie qu'un conteneur accepte un objet de ce type et de ce poids, puis que la capacité
    /// tient sur toute la chaîne des conteneurs dont le poids remonte.
    fn verifier_accueil(
        &self,
        conteneur: Id,
        type_objet: &str,
        poids: Poids,
    ) -> Result<(), Erreur> {
        let Some(c) = self.exemplaire(conteneur)?.conteneur.as_ref() else {
            return Err(Erreur::PasUnConteneur(conteneur));
        };
        if let Some(types) = &c.types_autorises {
            if !types.iter().any(|t| t == type_objet) {
                return Err(Erreur::TypeRefuse {
                    conteneur,
                    type_objet: type_objet.to_string(),
                });
            }
        }
        let mut courant = conteneur;
        loop {
            let exemplaire = self.exemplaire(courant)?;
            let Some(c) = exemplaire.conteneur.as_ref() else {
                return Ok(());
            };
            if let Some(capacite) = c.capacite {
                if self.poids_brut_contenu(courant) + poids > capacite {
                    return Err(Erreur::CapaciteDepassee(courant));
                }
            }
            // Le poids ne remonte plus si ce conteneur ne compte pas son contenu.
            match exemplaire.parent {
                Some(parent) if c.contenu_compte => courant = parent,
                _ => return Ok(()),
            }
        }
    }

    fn poids_propre(&self, exemplaire: &Exemplaire) -> Poids {
        exemplaire
            .modele
            .as_ref()
            .and_then(|m| self.modeles.get(m))
            .and_then(|m| m.poids)
            .map_or(Poids::ZERO, |p| p.fois(exemplaire.quantite))
    }

    fn type_de(&self, exemplaire: &Exemplaire) -> String {
        exemplaire
            .modele
            .as_ref()
            .and_then(|m| self.modeles.get(m))
            .map_or_else(String::new, |m| m.type_objet.clone())
    }

    fn exemplaire_mut(&mut self, id: Id) -> Result<&mut Exemplaire, Erreur> {
        self.exemplaires
            .get_mut(&id)
            .ok_or(Erreur::ExemplaireInconnu(id))
    }

    fn nouvel_id(&mut self) -> Id {
        self.dernier_id += 1;
        self.dernier_id
    }

    /// Applique une commande. Les identifiants sont attribués dans l'ordre : rejouer le même
    /// journal donne le même registre.
    pub fn appliquer(&mut self, commande: &Commande) -> Result<Option<Id>, Erreur> {
        match commande {
            Commande::AjouterModele(modele) => {
                self.ajouter_modele(modele.clone());
                Ok(None)
            }
            Commande::CreerInventaire {
                proprietaire,
                conteneur,
            } => Ok(Some(
                self.creer_inventaire(proprietaire.clone(), conteneur.clone()),
            )),
            Commande::Ajouter {
                modele,
                quantite,
                dans,
            } => self.ajouter(modele, *quantite, *dans).map(Some),
            Commande::AjouterConteneur {
                modele,
                conteneur,
                dans,
            } => self
                .ajouter_conteneur(modele, conteneur.clone(), *dans)
                .map(Some),
            Commande::Deplacer { id, vers } => self.deplacer(*id, *vers).map(|()| None),
            Commande::Retirer { id } => self.retirer(*id).map(|()| None),
            Commande::Scinder { id, quantite } => self.scinder(*id, *quantite).map(Some),
            Commande::Fusionner { garde, absorbe } => {
                self.fusionner(*garde, *absorbe).map(|()| None)
            }
        }
    }
}

/// Opération sur l'inventaire, enregistrée dans le journal de campagne.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Commande {
    AjouterModele(Modele),
    CreerInventaire {
        proprietaire: Proprietaire,
        conteneur: Conteneur,
    },
    Ajouter {
        modele: String,
        quantite: u32,
        dans: Id,
    },
    AjouterConteneur {
        modele: String,
        conteneur: Conteneur,
        dans: Id,
    },
    Deplacer {
        id: Id,
        vers: Id,
    },
    Retirer {
        id: Id,
    },
    Scinder {
        id: Id,
        quantite: u32,
    },
    Fusionner {
        garde: Id,
        absorbe: Id,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modele(id: &str, type_objet: &str, poids: i64) -> Modele {
        Modele {
            id: id.into(),
            nom: id.into(),
            type_objet: type_objet.into(),
            valeur: None,
            poids: Some(Poids(poids)),
            source: "saisie libre".into(),
            proprietes: BTreeMap::new(),
        }
    }

    fn conteneur(capacite: Option<i64>, types: Option<&[&str]>, compte: bool) -> Conteneur {
        Conteneur {
            capacite: capacite.map(Poids),
            types_autorises: types.map(|t| t.iter().map(|s| s.to_string()).collect()),
            contenu_compte: compte,
        }
    }

    fn proprietaire() -> Proprietaire {
        Proprietaire {
            type_id: "personnage".into(),
            ref_id: "perso-1".into(),
        }
    }

    /// Registre avec un inventaire racine et les modèles de base.
    fn registre() -> (Registre, Id) {
        let mut r = Registre::default();
        r.ajouter_modele(modele("sac", "contenant", 100));
        r.ajouter_modele(modele("etui", "contenant", 0));
        r.ajouter_modele(modele("epee", "arme", 150));
        r.ajouter_modele(modele("fleche", "munition", 10));
        r.ajouter_modele(modele("parchemin", "parchemin", 5));
        let racine = r.creer_inventaire(proprietaire(), conteneur(None, None, true));
        (r, racine)
    }

    #[test]
    fn poids_compte_le_contenu_et_remonte_a_la_racine() {
        let (mut r, racine) = registre();
        let sac = r
            .ajouter_conteneur("sac", conteneur(None, None, true), racine)
            .unwrap();
        r.ajouter("epee", 1, sac).unwrap();
        assert_eq!(r.poids_total(sac), Poids(250));
        assert_eq!(r.poids_total(racine), Poids(250));
    }

    #[test]
    fn conteneur_extradimensionnel_ne_pese_pas_sur_le_parent() {
        let (mut r, racine) = registre();
        let sac = r
            .ajouter_conteneur("sac", conteneur(None, None, false), racine)
            .unwrap();
        r.ajouter("epee", 1, sac).unwrap();
        assert_eq!(r.poids_total(sac), Poids(100));
        assert_eq!(r.poids_total(racine), Poids(100));
    }

    #[test]
    fn capacite_refuse_le_depassement() {
        let (mut r, racine) = registre();
        let etui = r
            .ajouter_conteneur("sac", conteneur(Some(200), None, true), racine)
            .unwrap();
        r.ajouter("fleche", 10, etui).unwrap();
        r.ajouter("fleche", 10, etui).unwrap();
        assert_eq!(
            r.ajouter("fleche", 1, etui),
            Err(Erreur::CapaciteDepassee(etui))
        );
    }

    #[test]
    fn restriction_de_type_refuse_un_objet_etranger() {
        let (mut r, racine) = registre();
        let etui = r
            .ajouter_conteneur("sac", conteneur(None, Some(&["parchemin"]), true), racine)
            .unwrap();
        r.ajouter("parchemin", 3, etui).unwrap();
        assert_eq!(
            r.ajouter("epee", 1, etui),
            Err(Erreur::TypeRefuse {
                conteneur: etui,
                type_objet: "arme".into()
            })
        );
    }

    #[test]
    fn deplacer_refuse_les_cycles_et_la_racine() {
        let (mut r, racine) = registre();
        let sac = r
            .ajouter_conteneur("sac", conteneur(None, None, true), racine)
            .unwrap();
        let etui = r
            .ajouter_conteneur("etui", conteneur(None, None, true), sac)
            .unwrap();
        assert_eq!(r.deplacer(sac, etui), Err(Erreur::CycleInterdit));
        assert_eq!(r.deplacer(racine, sac), Err(Erreur::RacineImmobile(racine)));
        let epee = r.ajouter("epee", 1, racine).unwrap();
        r.deplacer(epee, etui).unwrap();
        assert_eq!(r.racine_de(epee), Ok(racine));
        assert_eq!(r.poids_total(racine), Poids(250));
    }

    #[test]
    fn capacite_du_parent_compte_le_poids_des_sous_conteneurs() {
        let (mut r, racine) = registre();
        let sac = r
            .ajouter_conteneur("sac", conteneur(Some(150), None, true), racine)
            .unwrap();
        let etui = r
            .ajouter_conteneur("etui", conteneur(None, None, true), sac)
            .unwrap();
        r.ajouter("fleche", 10, etui).unwrap();
        assert_eq!(
            r.ajouter("epee", 1, etui),
            Err(Erreur::CapaciteDepassee(sac))
        );
    }

    #[test]
    fn deplacer_dans_le_meme_arbre_ne_compte_pas_deux_fois() {
        let (mut r, racine) = registre();
        let sac = r
            .ajouter_conteneur("sac", conteneur(Some(150), None, true), racine)
            .unwrap();
        let etui = r
            .ajouter_conteneur("etui", conteneur(None, None, true), sac)
            .unwrap();
        let epee = r.ajouter("epee", 1, sac).unwrap();
        r.deplacer(epee, etui).unwrap();
        assert_eq!(r.poids_total(sac), Poids(250));
    }

    #[test]
    fn scission_et_fusion_gardent_le_poids() {
        let (mut r, racine) = registre();
        let fleches = r.ajouter("fleche", 10, racine).unwrap();
        let reste = r.scinder(fleches, 4).unwrap();
        assert_eq!(r.exemplaire(fleches).unwrap().quantite, 6);
        assert_eq!(r.exemplaire(reste).unwrap().quantite, 4);
        assert_eq!(r.poids_total(racine), Poids(100));
        r.fusionner(fleches, reste).unwrap();
        assert_eq!(r.exemplaire(fleches).unwrap().quantite, 10);
        assert_eq!(r.poids_total(racine), Poids(100));
    }

    #[test]
    fn racine_et_conteneur_non_vide_ne_se_suppriment_pas() {
        let (mut r, racine) = registre();
        assert_eq!(r.retirer(racine), Err(Erreur::RacineNonSupprimable(racine)));
        let sac = r
            .ajouter_conteneur("sac", conteneur(None, None, true), racine)
            .unwrap();
        r.ajouter("epee", 1, sac).unwrap();
        assert_eq!(r.retirer(sac), Err(Erreur::ConteneurNonVide(sac)));
    }

    #[test]
    fn rejouer_les_commandes_donne_le_meme_registre() {
        let commandes = vec![
            Commande::AjouterModele(modele("sac", "contenant", 100)),
            Commande::AjouterModele(modele("epee", "arme", 150)),
            Commande::CreerInventaire {
                proprietaire: proprietaire(),
                conteneur: conteneur(None, None, true),
            },
            Commande::AjouterConteneur {
                modele: "sac".into(),
                conteneur: conteneur(None, None, true),
                dans: 1,
            },
            Commande::Ajouter {
                modele: "epee".into(),
                quantite: 1,
                dans: 2,
            },
        ];
        let rejoue = || {
            let mut r = Registre::default();
            for c in &commandes {
                r.appliquer(c).unwrap();
            }
            r
        };
        assert_eq!(rejoue(), rejoue());
        assert_eq!(rejoue().poids_total(1), Poids(250));
    }

    /// Générateur pseudo-aléatoire déterministe, pour des séquences reproductibles.
    fn suivant(graine: &mut u64) -> u64 {
        *graine = graine
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *graine >> 33
    }

    #[test]
    fn invariants_tiennent_sur_des_operations_aleatoires() {
        let (mut r, racine) = registre();
        let mut graine = 42;
        for _ in 0..2000 {
            let ids: Vec<Id> = r.exemplaires.keys().copied().collect();
            let a = ids[suivant(&mut graine) as usize % ids.len()];
            let b = ids[suivant(&mut graine) as usize % ids.len()];
            let quantite = (suivant(&mut graine) % 5) as u32 + 1;
            let extradimensionnel = suivant(&mut graine).is_multiple_of(2);
            let _ = match suivant(&mut graine) % 6 {
                0 => r.ajouter("fleche", quantite, a).map(|_| ()),
                1 => r
                    .ajouter_conteneur("sac", conteneur(Some(300), None, extradimensionnel), a)
                    .map(|_| ()),
                2 => r.deplacer(a, b),
                3 => r.retirer(a),
                4 => r.scinder(a, quantite).map(|_| ()),
                _ => r.fusionner(a, b),
            };
            // Chaque objet remonte à la racine, sans cycle.
            for &id in r.exemplaires.keys() {
                assert_eq!(r.racine_de(id), Ok(racine));
            }
            // Aucun conteneur ne dépasse sa capacité.
            for e in r.exemplaires.values() {
                if let Some(Conteneur {
                    capacite: Some(cap),
                    ..
                }) = e.conteneur
                {
                    assert!(
                        r.poids_brut_contenu(e.id) <= cap,
                        "capacité dépassée sur {}",
                        e.id
                    );
                }
            }
        }
    }
}
