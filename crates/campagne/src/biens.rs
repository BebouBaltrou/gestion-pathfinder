//! Biens de campagne : véhicules (propriétaire : une localité, une organisation ou un PJ) et
//! montures (propriétaire comme un véhicule ; confiées à un PJ, un PNJ du don Prestige ou une équipe).
//!
//! Les caractéristiques des modèles viennent du Manuel des joueurs (`data/core/vehicules_manuel.json`
//! et `regles::montures`). Les attributions (équipes et gestionnaires d'un véhicule, détenteur
//! d'une monture) sont des règles **personnalisées** : le Manuel n'en dit rien.

use crate::etat::{Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::*;
use domaine::Montant;

/// À qui appartient un véhicule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProprietaireBien {
    Localite(IdLocalite),
    Organisation(IdOrganisation),
    Personnage(IdPersonnage),
}

/// À qui une monture est confiée, sans changer de propriétaire : un PJ, un acolyte du don Prestige
/// (un personnage lié), une équipe, ou un partisan du don (rang dans la liste du maître).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DetenteurMonture {
    Personnage(IdPersonnage),
    Equipe(IdEquipe),
    Partisan { maitre: IdPersonnage, index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Vehicule {
    pub id: IdVehicule,
    /// Identifiant du modèle (`vehicules_manuel.json` : « drakkar », « chariot »…).
    pub modele: String,
    pub nom: String,
    pub proprietaire: ProprietaireBien,
    /// Prix d'achat, compté dans le patrimoine du propriétaire.
    pub prix: Montant,
    /// Équipes affectées à ce véhicule (l'équipage).
    pub equipes: Vec<IdEquipe>,
    pub gestionnaires: Vec<IdGestionnaire>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Monture {
    pub id: IdMonture,
    /// Nom du modèle (« Cheval léger », « Poney »…).
    pub modele: String,
    pub nom: String,
    /// Compté dans son patrimoine (prix d'achat).
    pub proprietaire: ProprietaireBien,
    /// À qui la monture est confiée (`None` : à l'écurie du propriétaire).
    pub detenteur: Option<DetenteurMonture>,
    pub prix: Montant,
}

impl Etat {
    fn verifier_proprietaire(&self, p: &ProprietaireBien) -> Result<(), Erreur> {
        match p {
            ProprietaireBien::Localite(l) => self.localite(*l).map(|_| ()),
            ProprietaireBien::Organisation(o) => self.organisation(*o).map(|_| ()),
            ProprietaireBien::Personnage(pj) => self.pj(*pj).map(|_| ()),
        }
    }

    fn verifier_detenteur(&self, d: &DetenteurMonture) -> Result<(), Erreur> {
        match d {
            DetenteurMonture::Personnage(pj) => self.pj(*pj).map(|_| ()),
            DetenteurMonture::Equipe(e) => self.equipe(*e).map(|_| ()),
            DetenteurMonture::Partisan { maitre, index } => self
                .prestige_don
                .get(maitre)
                .filter(|d| *index < d.partisans.len())
                .map(|_| ())
                .ok_or_else(|| Erreur::Prestige("partisan inconnu".into())),
        }
    }

    /// Propriétaire d'une monture : celui qui est donné, sinon (anciennes sauvegardes) celui qu'on
    /// déduit de qui la détient.
    fn proprietaire_monture(
        &self,
        donne: Option<ProprietaireBien>,
        detenteur: Option<DetenteurMonture>,
    ) -> Result<ProprietaireBien, Erreur> {
        if let Some(p) = donne {
            return Ok(p);
        }
        match detenteur {
            Some(DetenteurMonture::Personnage(pj)) => Ok(ProprietaireBien::Personnage(pj)),
            Some(DetenteurMonture::Partisan { maitre, .. }) => {
                Ok(ProprietaireBien::Personnage(maitre))
            }
            Some(DetenteurMonture::Equipe(e)) => {
                Ok(ProprietaireBien::Organisation(self.equipe(e)?.organisation))
            }
            None => Err(Erreur::Actif(
                "une monture a besoin d'un propriétaire".into(),
            )),
        }
    }

    pub(crate) fn appliquer_biens(&mut self, ev: &Evenement) -> Result<(), Erreur> {
        use Evenement::*;
        match ev {
            VehiculeAchete {
                id,
                modele,
                nom,
                proprietaire,
                payeur,
                prix,
            } => {
                if nom.trim().is_empty() || modele.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if self.vehicules.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                self.verifier_proprietaire(proprietaire)?;
                if prix.pc > 0 {
                    self.debiter(*payeur, *prix)?;
                }
                self.vehicules.insert(
                    *id,
                    Vehicule {
                        id: *id,
                        modele: modele.clone(),
                        nom: nom.clone(),
                        proprietaire: *proprietaire,
                        prix: *prix,
                        equipes: Vec::new(),
                        gestionnaires: Vec::new(),
                    },
                );
            }
            VehiculeModifie {
                id,
                nom,
                proprietaire,
                equipes,
                gestionnaires,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                self.vehicules.get(id).ok_or(Erreur::VehiculeInconnu(*id))?;
                self.verifier_proprietaire(proprietaire)?;
                for e in equipes {
                    self.equipe(*e)?;
                }
                for g in gestionnaires {
                    self.gestionnaire(*g)?;
                }
                let mut equipes = equipes.clone();
                equipes.sort();
                equipes.dedup();
                let mut gestionnaires = gestionnaires.clone();
                gestionnaires.sort();
                gestionnaires.dedup();
                let v = self.vehicules.get_mut(id).expect("vérifié");
                v.nom = nom.clone();
                v.proprietaire = *proprietaire;
                v.equipes = equipes;
                v.gestionnaires = gestionnaires;
            }
            VehiculeRetire { id } => {
                self.vehicules
                    .remove(id)
                    .ok_or(Erreur::VehiculeInconnu(*id))?;
            }
            MontureAchetee {
                id,
                modele,
                nom,
                proprietaire,
                detenteur,
                payeur,
                prix,
            } => {
                if nom.trim().is_empty() || modele.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if self.montures.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                let proprietaire = self.proprietaire_monture(*proprietaire, *detenteur)?;
                self.verifier_proprietaire(&proprietaire)?;
                if let Some(d) = detenteur {
                    self.verifier_detenteur(d)?;
                }
                if prix.pc > 0 {
                    self.debiter(*payeur, *prix)?;
                }
                self.montures.insert(
                    *id,
                    Monture {
                        id: *id,
                        modele: modele.clone(),
                        nom: nom.clone(),
                        proprietaire,
                        detenteur: *detenteur,
                        prix: *prix,
                    },
                );
            }
            MontureModifiee {
                id,
                nom,
                proprietaire,
                detenteur,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                let actuel = self
                    .montures
                    .get(id)
                    .ok_or(Erreur::MontureInconnue(*id))?
                    .proprietaire;
                let proprietaire = proprietaire.unwrap_or(actuel);
                self.verifier_proprietaire(&proprietaire)?;
                if let Some(d) = detenteur {
                    self.verifier_detenteur(d)?;
                }
                let m = self.montures.get_mut(id).expect("vérifié");
                m.nom = nom.clone();
                m.proprietaire = proprietaire;
                m.detenteur = *detenteur;
            }
            MontureRetiree { id } => {
                self.montures
                    .remove(id)
                    .ok_or(Erreur::MontureInconnue(*id))?;
            }
            _ => unreachable!("événement non lié aux biens"),
        }
        Ok(())
    }

    /// Après une suppression (localité, PJ, équipe, gestionnaire…) : retire des biens ce qui
    /// pointe vers un élément disparu. Un bien dont le propriétaire a disparu disparaît aussi.
    pub(crate) fn purger_biens(&mut self) {
        let equipes: std::collections::BTreeSet<IdEquipe> = self.equipes.keys().copied().collect();
        let gestionnaires: std::collections::BTreeSet<IdGestionnaire> =
            self.gestionnaires.keys().copied().collect();
        let existe = |p: &ProprietaireBien| match p {
            ProprietaireBien::Localite(l) => self.localites.contains_key(l),
            ProprietaireBien::Organisation(o) => self.organisations.contains_key(o),
            ProprietaireBien::Personnage(pj) => self.personnages.contains_key(pj),
        };
        let vivants: Vec<IdVehicule> = self
            .vehicules
            .values()
            .filter(|v| existe(&v.proprietaire))
            .map(|v| v.id)
            .collect();
        self.vehicules.retain(|id, _| vivants.contains(id));
        for v in self.vehicules.values_mut() {
            v.equipes.retain(|e| equipes.contains(e));
            v.gestionnaires.retain(|g| gestionnaires.contains(g));
        }
        let vivantes: Vec<IdMonture> = self
            .montures
            .values()
            .filter(|m| existe(&m.proprietaire))
            .map(|m| m.id)
            .collect();
        self.montures.retain(|id, _| vivantes.contains(id));
        // Un détenteur disparu : la monture revient à l'écurie de son propriétaire.
        let valides: Vec<IdMonture> = self
            .montures
            .values()
            .filter(|m| {
                m.detenteur
                    .is_none_or(|d| self.verifier_detenteur(&d).is_ok())
            })
            .map(|m| m.id)
            .collect();
        for m in self.montures.values_mut() {
            if !valides.contains(&m.id) {
                m.detenteur = None;
            }
        }
    }
}

// ------------------------------------------------------------------ catalogue des modèles

/// Prix du catalogue d'équipement (livre anglais : mêmes valeurs que le Manuel pour ces objets).
#[cfg(feature = "serde")]
fn prix_catalogue(nom_en: &str) -> Option<Montant> {
    let materiel = &fiches::Referentiel::partage().equipement.materiel;
    materiel
        .iter()
        .find(|o| o.nom_en == nom_en)
        .and_then(|o| crate::catalogue_inventaire::montant(&o.cout))
}

/// Modèle de véhicule du Manuel (`data/core/vehicules_manuel.json`).
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct ModeleVehicule {
    pub id: String,
    pub nom: String,
    pub terrestre: bool,
    pub equipage: Option<String>,
    pub equipage_min: Option<u32>,
    pub equipage_max: Option<u32>,
    pub passagers: Option<String>,
    pub cargaison: Option<String>,
    pub tracte_par: Option<String>,
    pub vitesse_km_h: Option<f64>,
    pub vitesse_km_jour: Option<f64>,
    pub prix: Option<Montant>,
}

#[cfg(feature = "serde")]
pub fn modeles_vehicules() -> &'static [ModeleVehicule] {
    use serde_json::Value;
    static MODELES: std::sync::OnceLock<Vec<ModeleVehicule>> = std::sync::OnceLock::new();
    MODELES.get_or_init(|| {
        let j: Value =
            serde_json::from_str(include_str!("../../../data/core/vehicules_manuel.json"))
                .expect("vehicules_manuel.json valide");
        j["vehicules"]
            .as_array()
            .expect("liste de véhicules")
            .iter()
            .map(|v| {
                let texte = |k: &str| match &v[k] {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                };
                let cargaison = match &v["cargaison_t"] {
                    Value::Number(n) if n.as_f64() == Some(0.0) => {
                        Some("aucune cargaison".to_string())
                    }
                    Value::Number(n) => Some(format!("{n} t")),
                    Value::String(s) => Some(match s.find(" (") {
                        Some(i) => format!("{} t{}", &s[..i], &s[i..]),
                        None => format!("{s} t"),
                    }),
                    _ => None,
                };
                ModeleVehicule {
                    id: v["id"].as_str().expect("id").to_string(),
                    nom: v["nom"].as_str().expect("nom").to_string(),
                    terrestre: v["terrestre"].as_bool().unwrap_or(false),
                    equipage: texte("equipage"),
                    equipage_min: v["equipage_min"].as_u64().map(|n| n as u32),
                    equipage_max: v["equipage_max"].as_u64().map(|n| n as u32),
                    passagers: texte("passagers"),
                    cargaison,
                    tracte_par: texte("tracte_par"),
                    vitesse_km_h: v["vitesse_km_h"].as_f64(),
                    vitesse_km_jour: v["vitesse_km_jour"].as_f64(),
                    prix: v["nom_en"].as_str().and_then(prix_catalogue),
                }
            })
            .collect()
    })
}

/// Prix d'une monture du catalogue (le chien de selle n'y figure pas : prix à saisir).
#[cfg(feature = "serde")]
pub fn prix_monture(nom: &str) -> Option<Montant> {
    let nom_en = match nom {
        "Cheval léger" => "Horse (light)",
        "Cheval lourd" => "Horse (heavy)",
        "Poney" => "Horse (pony)",
        _ => return None,
    };
    prix_catalogue(nom_en)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etat::Erreur;
    use crate::Campagne;
    use domaine::Montant;

    fn monde() -> (Campagne, regles::Referentiel, IdLocalite, IdPersonnage) {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let l = c.nouvel_id_localite();
        c.executer(
            Evenement::LocaliteCreee {
                id: l,
                nom: "Port".into(),
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
        let pj = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: pj,
                nom: "Héros".into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        (c, r, l, pj)
    }

    fn acheter(
        c: &mut Campagne,
        r: &regles::Referentiel,
        l: IdLocalite,
        pj: IdPersonnage,
        prix: i64,
    ) -> Result<IdVehicule, Erreur> {
        let id = c.nouvel_id_vehicule();
        c.executer(
            Evenement::VehiculeAchete {
                id,
                modele: "barque".into(),
                nom: "La Mouette".into(),
                proprietaire: ProprietaireBien::Localite(l),
                payeur: Payeur::Personnage(pj),
                prix: Montant::po(prix),
            },
            r,
        )?;
        Ok(id)
    }

    #[test]
    fn achat_d_un_vehicule_debite_le_payeur_choisi() {
        let (mut c, r, l, pj) = monde();
        let id = acheter(&mut c, &r, l, pj, 50).unwrap();
        let e = c.etat(&r);
        assert_eq!(e.personnages[&pj].compte, Montant::po(50));
        assert_eq!(e.vehicules[&id].proprietaire, ProprietaireBien::Localite(l));
        assert_eq!(e.vehicules[&id].prix, Montant::po(50));
    }

    #[test]
    fn achat_refuse_sans_fonds_et_ne_change_rien() {
        let (mut c, r, l, pj) = monde();
        assert!(acheter(&mut c, &r, l, pj, 500).is_err());
        let e = c.etat(&r);
        assert!(e.vehicules.is_empty());
        assert_eq!(e.personnages[&pj].compte, Montant::po(100));
    }

    #[test]
    fn la_suppression_du_proprietaire_retire_le_vehicule() {
        let (mut c, r, l, pj) = monde();
        acheter(&mut c, &r, l, pj, 50).unwrap();
        c.executer(Evenement::LocaliteSupprimee { id: l }, &r)
            .unwrap();
        assert!(c.etat(&r).vehicules.is_empty());
    }

    #[test]
    fn monture_attribuee_a_un_pj_puis_retiree_avec_lui() {
        let (mut c, r, _, pj) = monde();
        let id = c.nouvel_id_monture();
        c.executer(
            Evenement::MontureAchetee {
                id,
                modele: "Cheval léger".into(),
                nom: "Éclair".into(),
                proprietaire: None,
                detenteur: Some(DetenteurMonture::Personnage(pj)),
                payeur: Payeur::Personnage(pj),
                prix: Montant::po(75),
            },
            &r,
        )
        .unwrap();
        // sans propriétaire donné (ancienne sauvegarde) : déduit de qui la détient
        assert_eq!(
            c.etat(&r).montures[&id].proprietaire,
            ProprietaireBien::Personnage(pj)
        );
        assert_eq!(
            c.etat(&r).montures[&id].detenteur,
            Some(DetenteurMonture::Personnage(pj))
        );
        assert_eq!(c.etat(&r).personnages[&pj].compte, Montant::po(25));
        c.executer(Evenement::PersonnageSupprime { id: pj }, &r)
            .unwrap();
        assert!(c.etat(&r).montures.is_empty());
    }

    #[test]
    fn nom_vide_et_detenteur_inconnu_refuses() {
        let (mut c, r, l, pj) = monde();
        let id = acheter(&mut c, &r, l, pj, 10).unwrap();
        assert!(matches!(
            c.executer(
                Evenement::VehiculeModifie {
                    id,
                    nom: "  ".into(),
                    proprietaire: ProprietaireBien::Localite(l),
                    equipes: vec![],
                    gestionnaires: vec![],
                },
                &r
            ),
            Err(Erreur::NomVide)
        ));
        assert!(matches!(
            c.executer(
                Evenement::MontureAchetee {
                    id: c.nouvel_id_monture(),
                    modele: "Poney".into(),
                    nom: "Pip".into(),
                    proprietaire: None,
                    detenteur: Some(DetenteurMonture::Equipe(IdEquipe(99))),
                    payeur: Payeur::Personnage(pj),
                    prix: Montant::po(1),
                },
                &r
            ),
            Err(Erreur::EquipeInconnue(_))
        ));
    }

    fn acheter_monture(
        c: &mut Campagne,
        r: &regles::Referentiel,
        proprietaire: ProprietaireBien,
        detenteur: Option<DetenteurMonture>,
        pj: IdPersonnage,
    ) -> IdMonture {
        let id = c.nouvel_id_monture();
        c.executer(
            Evenement::MontureAchetee {
                id,
                modele: "Poney".into(),
                nom: "Pip".into(),
                proprietaire: Some(proprietaire),
                detenteur,
                payeur: Payeur::Personnage(pj),
                prix: Montant::po(30),
            },
            r,
        )
        .unwrap();
        id
    }

    #[test]
    fn la_monture_confiee_garde_son_proprietaire() {
        let (mut c, r, l, pj) = monde();
        let id = acheter_monture(&mut c, &r, ProprietaireBien::Localite(l), None, pj);
        assert_eq!(c.etat(&r).montures[&id].detenteur, None);
        c.executer(
            Evenement::MontureModifiee {
                id,
                nom: "Pip".into(),
                proprietaire: None,
                detenteur: Some(DetenteurMonture::Personnage(pj)),
            },
            &r,
        )
        .unwrap();
        let m = &c.etat(&r).montures[&id];
        assert_eq!(m.proprietaire, ProprietaireBien::Localite(l));
        assert_eq!(m.detenteur, Some(DetenteurMonture::Personnage(pj)));
    }

    #[test]
    fn le_detenteur_disparu_rend_la_monture_a_l_ecurie_du_proprietaire() {
        let (mut c, r, l, pj) = monde();
        let autre = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id: autre,
                nom: "Écuyer".into(),
                compte_initial: Montant::po(0),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        let id = acheter_monture(
            &mut c,
            &r,
            ProprietaireBien::Localite(l),
            Some(DetenteurMonture::Personnage(autre)),
            pj,
        );
        c.executer(Evenement::PersonnageSupprime { id: autre }, &r)
            .unwrap();
        let m = &c.etat(&r).montures[&id];
        assert_eq!(m.detenteur, None);
        assert_eq!(m.proprietaire, ProprietaireBien::Localite(l));
    }

    #[test]
    fn monture_confiee_a_un_partisan_du_don_prestige_suit_son_rang() {
        use crate::etat::Etat;
        use crate::prestige::Partisan;
        let r = regles::referentiel_officiel();
        let mut e = Etat::default();
        let pj = IdPersonnage(1);
        e.appliquer(
            &Evenement::PersonnageCree {
                id: pj,
                nom: "Héros".into(),
                compte_initial: Montant::po(100),
                prestige: None,
            },
            &r,
        )
        .unwrap();
        // trois partisans du don Prestige (l'état du don est posé directement)
        let don = e.prestige_don.entry(pj).or_default();
        for nom in ["Ada", "Bob", "Cléo"] {
            don.partisans.push(Partisan {
                niveau: 1,
                nom: nom.into(),
            });
        }
        let id = IdMonture(1);
        e.appliquer(
            &Evenement::MontureAchetee {
                id,
                modele: "Poney".into(),
                nom: "Pip".into(),
                proprietaire: Some(ProprietaireBien::Personnage(pj)),
                detenteur: Some(DetenteurMonture::Partisan {
                    maitre: pj,
                    index: 2,
                }),
                payeur: Payeur::Personnage(pj),
                prix: Montant::po(30),
            },
            &r,
        )
        .unwrap();
        // Bob (rang 1) est retiré : Cléo passe au rang 1 et garde la monture
        e.appliquer(
            &Evenement::PartisanRetire {
                maitre: pj,
                index: 1,
            },
            &r,
        )
        .unwrap();
        assert_eq!(
            e.montures[&id].detenteur,
            Some(DetenteurMonture::Partisan {
                maitre: pj,
                index: 1
            })
        );
        // Cléo est retirée à son tour : la monture revient à l'écurie du propriétaire
        e.appliquer(
            &Evenement::PartisanRetire {
                maitre: pj,
                index: 1,
            },
            &r,
        )
        .unwrap();
        assert_eq!(e.montures[&id].detenteur, None);
        assert_eq!(
            e.montures[&id].proprietaire,
            ProprietaireBien::Personnage(pj)
        );
        // un partisan qui n'existe pas est refusé
        assert!(e
            .appliquer(
                &Evenement::MontureModifiee {
                    id,
                    nom: "Pip".into(),
                    proprietaire: None,
                    detenteur: Some(DetenteurMonture::Partisan {
                        maitre: pj,
                        index: 5
                    }),
                },
                &r
            )
            .is_err());
    }
}
