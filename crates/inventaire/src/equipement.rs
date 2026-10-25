//! Équipement porté : une vue sur l'inventaire.
//!
//! Équiper un objet l'affecte à un ou plusieurs emplacements sans le sortir de son conteneur :
//! il reste dans l'inventaire et compte dans le poids. Les emplacements, leur capacité et les
//! emplacements possibles par type d'objet sont des données ([`ReglesEmplacements`]), pas du code.
//! Un modèle peut aussi donner ses propres emplacements dans ses propriétés :
//! `emplacement` (identifiants séparés par des virgules) et `mains` (« 2 » : occupe toute la zone
//! de l'emplacement choisi, par exemple les deux mains).
//!
//! Aucun bonus n'est calculé ici : les effets d'un objet équipé viennent du moteur d'effets.

use std::collections::BTreeMap;

use crate::{Erreur, Id, Registre};

/// Un emplacement du corps.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Emplacement {
    pub id: String,
    /// Zone du corps, pour l'affichage et pour les objets à deux mains (« Mains tenues »).
    pub zone: String,
    pub nom: String,
    /// Nombre d'objets qu'il accepte ; 0 = illimité (objets sans emplacement).
    pub capacite: u32,
}

/// Les emplacements d'une campagne, définis en données.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReglesEmplacements {
    pub emplacements: Vec<Emplacement>,
    /// Emplacements possibles pour un type d'objet quand le modèle n'en donne pas.
    pub par_type: BTreeMap<String, Vec<String>>,
}

impl ReglesEmplacements {
    fn emplacement(&self, id: &str) -> Result<&Emplacement, Erreur> {
        self.emplacements
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| Erreur::EmplacementInconnu(id.to_string()))
    }
}

/// Un objet équipé et les emplacements qu'il occupe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equipe {
    pub objet: Id,
    pub emplacements: Vec<String>,
}

impl Registre {
    /// Remplace les emplacements de la campagne. Refusé si un objet est équipé dans un
    /// emplacement qui disparaît.
    pub fn definir_emplacements(&mut self, regles: ReglesEmplacements) -> Result<(), Erreur> {
        for emplacements in self.affectations.values() {
            for id in emplacements {
                if !regles.emplacements.iter().any(|e| &e.id == id) {
                    return Err(Erreur::EmplacementInconnu(id.clone()));
                }
            }
        }
        self.regles_emplacements = regles;
        Ok(())
    }

    pub fn regles_emplacements(&self) -> &ReglesEmplacements {
        &self.regles_emplacements
    }

    /// Emplacements où l'objet peut être équipé : ceux de son modèle, sinon ceux de son type.
    pub fn emplacements_possibles(&self, id: Id) -> Result<Vec<String>, Erreur> {
        let modele = self.modele_de(id)?;
        if let Some(liste) = modele.proprietes.get("emplacement") {
            return Ok(liste.split(',').map(|s| s.trim().to_string()).collect());
        }
        Ok(self
            .regles_emplacements
            .par_type
            .get(&modele.type_objet)
            .cloned()
            .unwrap_or_default())
    }

    /// Équipe un objet dans `emplacement`. Un objet à deux mains occupe toute la zone.
    pub fn equiper(&mut self, id: Id, emplacement: &str) -> Result<(), Erreur> {
        let e = self.exemplaire(id)?;
        if e.conteneur.is_some() || e.quantite != 1 || e.parent.is_none() {
            return Err(Erreur::PasEquipable(id));
        }
        if self.affectations.contains_key(&id) {
            return Err(Erreur::DejaEquipe(id));
        }
        if !self
            .emplacements_possibles(id)?
            .iter()
            .any(|p| p == emplacement)
        {
            return Err(Erreur::EmplacementIncompatible {
                objet: id,
                emplacement: emplacement.to_string(),
            });
        }
        let regles = &self.regles_emplacements;
        let choisi = regles.emplacement(emplacement)?;
        let occupes: Vec<String> = if self.mains_requises(id) >= 2 {
            regles
                .emplacements
                .iter()
                .filter(|e| e.zone == choisi.zone)
                .map(|e| e.id.clone())
                .collect()
        } else {
            vec![choisi.id.clone()]
        };
        let racine = self.racine_de(id)?;
        for slot in &occupes {
            let capacite = regles.emplacement(slot)?.capacite;
            if capacite > 0 && self.occupation(racine, slot) >= capacite {
                return Err(Erreur::EmplacementOccupe(slot.clone()));
            }
        }
        self.affectations.insert(id, occupes);
        Ok(())
    }

    /// Retire un objet de ses emplacements ; il reste dans l'inventaire. Refusé pour un objet collé.
    pub fn deequiper(&mut self, id: Id) -> Result<(), Erreur> {
        if !self.affectations.contains_key(&id) {
            return Err(Erreur::NonEquipe(id));
        }
        if self.exemplaire(id)?.colle {
            return Err(Erreur::ObjetColle(id));
        }
        self.affectations.remove(&id);
        Ok(())
    }

    /// Marque un objet comme collé (objet maudit) : il ne peut plus être déséquipé.
    pub fn definir_colle(&mut self, id: Id, colle: bool) -> Result<(), Erreur> {
        self.exemplaire(id)?;
        self.exemplaire_mut(id)?.colle = colle;
        Ok(())
    }

    /// Objets équipés d'un inventaire racine, dans l'ordre de leur identifiant.
    pub fn equipement_de(&self, racine: Id) -> Vec<Equipe> {
        self.affectations
            .iter()
            .filter(|(objet, _)| self.racine_de(**objet) == Ok(racine))
            .map(|(objet, emplacements)| Equipe {
                objet: *objet,
                emplacements: emplacements.clone(),
            })
            .collect()
    }

    /// Enregistre l'équipement actuel d'un inventaire sous un nom (« mode furtif »).
    pub fn enregistrer_jeu(&mut self, racine: Id, nom: &str) -> Result<(), Erreur> {
        self.proprietaire(racine)
            .ok_or(Erreur::ExemplaireInconnu(racine))?;
        let jeu: crate::JeuEquipement = self
            .equipement_de(racine)
            .into_iter()
            .map(|e| (e.objet, e.emplacements))
            .collect();
        self.jeux
            .entry(racine)
            .or_default()
            .insert(nom.to_string(), jeu);
        Ok(())
    }

    /// Bascule sur un jeu enregistré : déséquipe tout (sauf les objets collés), puis rééquipe le
    /// jeu. Tout ou rien : en cas d'erreur, rien ne change.
    pub fn appliquer_jeu(&mut self, racine: Id, nom: &str) -> Result<(), Erreur> {
        let jeu = self
            .jeux
            .get(&racine)
            .and_then(|j| j.get(nom))
            .cloned()
            .ok_or_else(|| Erreur::JeuInconnu(nom.to_string()))?;
        let avant = self.affectations.clone();
        let resultat = self.rebasculer(racine, &jeu);
        if resultat.is_err() {
            self.affectations = avant;
        }
        resultat
    }

    fn rebasculer(&mut self, racine: Id, jeu: &[(Id, Vec<String>)]) -> Result<(), Erreur> {
        for e in self.equipement_de(racine) {
            if !self.exemplaire(e.objet)?.colle {
                self.affectations.remove(&e.objet);
            }
        }
        for (objet, emplacements) in jeu {
            if self.affectations.contains_key(objet) {
                continue;
            }
            self.equiper(*objet, &emplacements[0])?;
        }
        Ok(())
    }

    /// Nombre d'objets d'un inventaire qui occupent déjà cet emplacement.
    fn occupation(&self, racine: Id, emplacement: &str) -> u32 {
        self.affectations
            .iter()
            .filter(|(objet, slots)| {
                slots.iter().any(|s| s == emplacement) && self.racine_de(**objet) == Ok(racine)
            })
            .count() as u32
    }

    fn mains_requises(&self, id: Id) -> u32 {
        self.modele_de(id)
            .ok()
            .and_then(|m| m.proprietes.get("mains"))
            .and_then(|m| m.parse().ok())
            .unwrap_or(1)
    }

    fn modele_de(&self, id: Id) -> Result<&crate::Modele, Erreur> {
        let nom = self
            .exemplaire(id)?
            .modele
            .as_ref()
            .ok_or(Erreur::PasEquipable(id))?;
        self.modele(nom)
            .ok_or_else(|| Erreur::ModeleInconnu(nom.clone()))
    }

    /// Un objet qui quitte l'inventaire (supprimé ou donné à un autre propriétaire) n'est plus
    /// équipé, lui ni ce qu'il contient.
    pub(crate) fn retirer_des_emplacements(&mut self, id: Id) {
        self.affectations.remove(&id);
        for enfant in self.enfants(id) {
            self.retirer_des_emplacements(enfant);
        }
    }
}
