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

/// Types de propriétaire des inventaires d'un véhicule (cargaison) et d'une monture (sacoches).
pub const PROPRIETAIRE_VEHICULE: &str = "vehicule";
pub const PROPRIETAIRE_MONTURE: &str = "monture";

/// Identifiant de l'inventaire d'un véhicule dans le registre (« V-3 »).
pub fn ref_vehicule(id: IdVehicule) -> String {
    id.to_string()
}

/// Identifiant de l'inventaire d'une monture dans le registre (« M-2 »).
pub fn ref_monture(id: IdMonture) -> String {
    id.to_string()
}

/// Le véhicule dont l'inventaire porte cette référence (« V-3 »).
pub fn vehicule_de_ref(r: &str) -> Option<IdVehicule> {
    r.strip_prefix("V-")?.parse().ok().map(IdVehicule)
}

/// La monture dont l'inventaire porte cette référence (« M-2 »).
pub fn monture_de_ref(r: &str) -> Option<IdMonture> {
    r.strip_prefix("M-")?.parse().ok().map(IdMonture)
}

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
    /// Améliorations installées (bateaux).
    pub ameliorations: Vec<AmeliorationInstallee>,
    /// Passagers à bord (suivi).
    pub passagers: u32,
}

/// Une amélioration de bateau installée, avec ce qu'elle a coûté.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AmeliorationInstallee {
    /// Identifiant dans `ameliorations_navires.json`.
    pub id: String,
    /// Nombre d'unités (cubes, sabords…) pour les améliorations cumulables ; 1 sinon.
    pub quantite: u32,
    pub prix: Montant,
}

/// Barde portée par une monture : l'armure dont elle est tirée (nom anglais du catalogue) et ce
/// qu'elle a coûté.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BardeMonture {
    pub armure: String,
    pub prix: Montant,
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
    pub barde: Option<BardeMonture>,
}

impl Etat {
    /// Inventaire racine (cargaison) d'un véhicule, s'il en a un.
    pub fn inventaire_du_vehicule(&self, id: IdVehicule) -> Option<inventaire::Id> {
        self.inventaire
            .racine_du_proprietaire(PROPRIETAIRE_VEHICULE, &ref_vehicule(id))
    }

    /// Inventaire racine (sacoches, charge) d'une monture, s'il y en a un.
    pub fn inventaire_de_la_monture(&self, id: IdMonture) -> Option<inventaire::Id> {
        self.inventaire
            .racine_du_proprietaire(PROPRIETAIRE_MONTURE, &ref_monture(id))
    }

    /// Supprime les inventaires dont le véhicule ou la monture n'existe plus (aucun identifiant
    /// n'est consommé : les anciens journaux se rejouent à l'identique).
    fn purger_inventaires_des_biens(&mut self) {
        let orphelins: Vec<inventaire::Id> = self
            .inventaire
            .inventaires()
            .filter(|(_, p)| match p.type_id.as_str() {
                PROPRIETAIRE_VEHICULE => {
                    !self.vehicules.keys().any(|v| ref_vehicule(*v) == p.ref_id)
                }
                PROPRIETAIRE_MONTURE => !self.montures.keys().any(|m| ref_monture(*m) == p.ref_id),
                _ => false,
            })
            .map(|(racine, _)| racine)
            .collect();
        for racine in orphelins {
            self.inventaire.supprimer_inventaire(racine);
        }
    }

    /// Verse `montant` au compte ou à la trésorerie de `beneficiaire`.
    pub(crate) fn crediter(
        &mut self,
        beneficiaire: Payeur,
        montant: Montant,
    ) -> Result<(), Erreur> {
        match beneficiaire {
            Payeur::Localite(id) => {
                let l = self.localite_mut(id)?;
                l.tresorerie = l.tresorerie + montant;
            }
            Payeur::Personnage(id) => {
                let p = self.pj_mut(id)?;
                p.compte = p.compte + montant;
            }
        }
        Ok(())
    }

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
                        ameliorations: Vec::new(),
                        passagers: 0,
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
                        barde: None,
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
            VehiculeAmelioree {
                id,
                amelioration,
                quantite,
                payeur,
                prix,
            } => {
                if amelioration.trim().is_empty() || *quantite == 0 {
                    return Err(Erreur::Actif("amélioration ou quantité invalide".into()));
                }
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                self.vehicules.get(id).ok_or(Erreur::VehiculeInconnu(*id))?;
                if prix.pc > 0 {
                    self.debiter(*payeur, *prix)?;
                }
                self.vehicules
                    .get_mut(id)
                    .expect("vérifié")
                    .ameliorations
                    .push(AmeliorationInstallee {
                        id: amelioration.clone(),
                        quantite: *quantite,
                        prix: *prix,
                    });
            }
            VehiculeAmeliorationRetiree { id, index } => {
                let v = self
                    .vehicules
                    .get_mut(id)
                    .ok_or(Erreur::VehiculeInconnu(*id))?;
                if *index >= v.ameliorations.len() {
                    return Err(Erreur::Actif("amélioration inconnue".into()));
                }
                v.ameliorations.remove(*index);
            }
            VehiculePassagers { id, nombre } => {
                self.vehicules
                    .get_mut(id)
                    .ok_or(Erreur::VehiculeInconnu(*id))?
                    .passagers = *nombre;
            }
            MontureBardee {
                id,
                armure,
                payeur,
                prix,
                gratuite,
            } => {
                if armure.trim().is_empty() {
                    return Err(Erreur::Actif("choisissez une armure pour la barde".into()));
                }
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                self.montures.get(id).ok_or(Erreur::MontureInconnue(*id))?;
                if prix.pc > 0 && !*gratuite {
                    self.debiter(*payeur, *prix)?;
                }
                self.montures.get_mut(id).expect("vérifié").barde = Some(BardeMonture {
                    armure: armure.clone(),
                    prix: *prix,
                });
            }
            MontureBardeVendue {
                id,
                beneficiaire,
                prix,
            } => {
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                let m = self.montures.get(id).ok_or(Erreur::MontureInconnue(*id))?;
                if m.barde.is_none() {
                    return Err(Erreur::Actif("cette monture ne porte pas de barde".into()));
                }
                self.crediter(*beneficiaire, *prix)?;
                self.montures.get_mut(id).expect("vérifié").barde = None;
            }
            MontureDebardee { id } => {
                let m = self
                    .montures
                    .get_mut(id)
                    .ok_or(Erreur::MontureInconnue(*id))?;
                if m.barde.take().is_none() {
                    return Err(Erreur::Actif("cette monture ne porte pas de barde".into()));
                }
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
        self.purger_inventaires_des_biens();
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
    /// Cargaison maximale du Manuel, en tonnes.
    pub cargaison_t_max: Option<f64>,
    /// Passagers (ou soldats) du Manuel.
    pub passagers_max: Option<u32>,
    /// Taille du bateau (G, TG, Gig, C) et nombres de cases, de rames et de cases de voile : hors
    /// Manuel (wiki), pour le coût de certaines améliorations.
    pub taille: Option<String>,
    pub cases: Option<u32>,
    pub rames: Option<u32>,
    pub cases_voiles: Option<u32>,
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
                    cargaison_t_max: v["cargaison_t_max"].as_f64(),
                    passagers_max: v["passagers_max"].as_u64().map(|n| n as u32),
                    taille: v["taille"].as_str().map(str::to_string),
                    cases: v["cases"].as_u64().map(|n| n as u32),
                    rames: v["rames"].as_u64().map(|n| n as u32),
                    cases_voiles: v["cases_voiles"].as_u64().map(|n| n as u32),
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

// ------------------------------------------------------------------ améliorations de bateau

/// Coût d'une amélioration de bateau.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq)]
pub enum CoutAmelioration {
    /// Pourcentage du prix de base du bateau.
    Pourcent(u32),
    Fixe(Montant),
    /// Prix selon la taille du bateau (G, TG, Gig, C).
    Taille(Vec<(String, Montant)>),
    /// Prix par unité : cube, sabord, ou, avec `compteur`, par case / rame / case de voile du modèle.
    ParUnite {
        prix: Montant,
        unite: String,
        compteur: Option<String>,
    },
    /// Prix à saisir (entre `min` et `max`).
    Libre {
        min: Montant,
        max: Montant,
    },
}

/// Effets chiffrés d'une amélioration (les autres effets sont dans le texte).
#[cfg(feature = "serde")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EffetsAmelioration {
    pub cargaison_pct: i32,
    pub passagers_pct: i32,
    pub vitesse_pct: i32,
    pub bonus_navigation: i32,
    /// Variation de la cargaison, en kg, par unité installée.
    pub cargaison_kg: i32,
}

/// Amélioration de bateau (`data/core/ameliorations_navires.json`, hors Manuel : Ultimate Equipment).
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct AmeliorationNavire {
    pub id: String,
    pub nom: String,
    pub prerequis: String,
    pub cout: CoutAmelioration,
    pub effets: EffetsAmelioration,
    pub cumulable: bool,
    pub a_la_construction: bool,
    pub texte: String,
}

#[cfg(feature = "serde")]
pub fn ameliorations_navires() -> &'static [AmeliorationNavire] {
    use serde_json::Value;
    static AMELIORATIONS: std::sync::OnceLock<Vec<AmeliorationNavire>> = std::sync::OnceLock::new();
    AMELIORATIONS.get_or_init(|| {
        let j: Value = serde_json::from_str(include_str!(
            "../../../data/core/ameliorations_navires.json"
        ))
        .expect("ameliorations_navires.json valide");
        let po = |v: &Value| Montant::po(v.as_i64().unwrap_or(0));
        j["ameliorations"]
            .as_array()
            .expect("liste d'améliorations")
            .iter()
            .map(|a| {
                let c = &a["cout"];
                let cout = match c["type"].as_str() {
                    Some("pourcent") => {
                        CoutAmelioration::Pourcent(c["valeur"].as_u64().unwrap_or(0) as u32)
                    }
                    Some("fixe") => CoutAmelioration::Fixe(po(&c["valeur"])),
                    Some("taille") => CoutAmelioration::Taille(
                        c["valeurs"]
                            .as_object()
                            .map(|o| o.iter().map(|(k, v)| (k.clone(), po(v))).collect())
                            .unwrap_or_default(),
                    ),
                    Some("par_unite") => CoutAmelioration::ParUnite {
                        prix: po(&c["valeur"]),
                        unite: c["unite"].as_str().unwrap_or("").to_string(),
                        compteur: c["compteur"].as_str().map(str::to_string),
                    },
                    _ => CoutAmelioration::Libre {
                        min: po(&c["min"]),
                        max: po(&c["max"]),
                    },
                };
                let e = &a["effets"];
                let pct = |k: &str| e[k].as_i64().unwrap_or(0) as i32;
                AmeliorationNavire {
                    id: a["id"].as_str().expect("id").to_string(),
                    nom: a["nom"].as_str().expect("nom").to_string(),
                    prerequis: a["prerequis"].as_str().unwrap_or("").to_string(),
                    cout,
                    effets: EffetsAmelioration {
                        cargaison_pct: pct("cargaison_pct"),
                        passagers_pct: pct("passagers_pct"),
                        vitesse_pct: pct("vitesse_pct"),
                        bonus_navigation: pct("bonus_navigation"),
                        cargaison_kg: pct("cargaison_kg"),
                    },
                    cumulable: a["cumulable"].as_bool().unwrap_or(false),
                    a_la_construction: a["a_la_construction"].as_bool().unwrap_or(false),
                    texte: a["texte"].as_str().unwrap_or("").to_string(),
                }
            })
            .collect()
    })
}

/// Prix d'une amélioration pour un bateau, `None` si le prix est à saisir. `prix_base` est le
/// prix de base du bateau ; `quantite` compte les cubes ou les sabords des améliorations cumulables.
#[cfg(feature = "serde")]
pub fn cout_amelioration(
    a: &AmeliorationNavire,
    modele: &ModeleVehicule,
    prix_base: Montant,
    quantite: u32,
) -> Option<Montant> {
    match &a.cout {
        CoutAmelioration::Pourcent(p) => Some(Montant::pc(prix_base.pc * i64::from(*p) / 100)),
        CoutAmelioration::Fixe(m) => Some(*m),
        CoutAmelioration::Taille(t) => {
            let taille = modele.taille.as_deref()?;
            t.iter().find(|(k, _)| k == taille).map(|(_, m)| *m)
        }
        CoutAmelioration::ParUnite { prix, compteur, .. } => {
            let unites = match compteur.as_deref() {
                Some("cases") => modele.cases?,
                Some("rames") => modele.rames?,
                Some("cases_voiles") => modele.cases_voiles?,
                _ => quantite,
            };
            Some(Montant::pc(prix.pc * i64::from(unites)))
        }
        CoutAmelioration::Libre { .. } => None,
    }
}

/// Ce qu'un véhicule peut porter et sa vitesse, une fois ses améliorations installées.
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct Capacites {
    /// Cargaison maximale, en kg (`None` : le Manuel ne la donne pas).
    pub cargaison_kg: Option<f64>,
    pub passagers_max: Option<u32>,
    /// Variation de la vitesse de déplacement, en pourcentage.
    pub vitesse_pct: i32,
    pub bonus_navigation: i32,
}

/// Capacités d'un véhicule : celles du Manuel, modifiées par ses améliorations (les pourcentages
/// s'additionnent : lecture à valider).
#[cfg(feature = "serde")]
pub fn capacites_vehicule(v: &Vehicule) -> Capacites {
    let modele = modeles_vehicules().iter().find(|m| m.id == v.modele);
    let catalogue = ameliorations_navires();
    let (mut cargaison_pct, mut passagers_pct, mut vitesse_pct, mut navigation, mut kg) =
        (0, 0, 0, 0, 0i64);
    for i in &v.ameliorations {
        if let Some(a) = catalogue.iter().find(|a| a.id == i.id) {
            cargaison_pct += a.effets.cargaison_pct;
            passagers_pct += a.effets.passagers_pct;
            vitesse_pct += a.effets.vitesse_pct;
            navigation += a.effets.bonus_navigation;
            kg += i64::from(a.effets.cargaison_kg) * i64::from(i.quantite);
        }
    }
    Capacites {
        cargaison_kg: modele.and_then(|m| m.cargaison_t_max).map(|t| {
            (((t * 1000.0).round() as i64 * (100 + i64::from(cargaison_pct)) / 100 + kg).max(0))
                as f64
        }),
        passagers_max: modele
            .and_then(|m| m.passagers_max)
            .map(|n| (f64::from(n) * (1.0 + f64::from(passagers_pct) / 100.0)).round() as u32),
        vitesse_pct,
        bonus_navigation: navigation,
    }
}

// ------------------------------------------------------------------ barde

/// Armure du catalogue utilisable comme barde (légère, intermédiaire ou lourde).
#[cfg(feature = "serde")]
#[derive(Debug, Clone, PartialEq)]
pub struct ModeleBarde {
    pub nom_en: String,
    pub nom: String,
    pub genre: regles::montures::GenreBarde,
    pub bonus_ca: i32,
    pub dex_max: Option<i32>,
    pub malus_armure: i32,
    /// Prix et poids de l'armure pour une créature de taille M.
    pub prix_base: Option<Montant>,
    pub poids_base_kg: Option<f64>,
}

#[cfg(feature = "serde")]
impl ModeleBarde {
    /// Prix de la barde : deux fois celui de l'armure pour une créature de taille M, quatre fois
    /// pour une de taille G (Manuel p. 162).
    pub fn prix(&self, grande: bool) -> Option<Montant> {
        self.prix_base
            .map(|m| Montant::pc(m.pc * if grande { 4 } else { 2 }))
    }

    /// Poids de la barde, en kg : celui de l'armure pour une créature de taille M, le double pour
    /// une de taille G (Manuel p. 162).
    pub fn poids_kg(&self, grande: bool) -> Option<f64> {
        self.poids_base_kg
            .map(|p| p * if grande { 2.0 } else { 1.0 })
    }
}

#[cfg(feature = "serde")]
pub fn modeles_barde() -> &'static [ModeleBarde] {
    use fiches::equipement::GenreProtection;
    use regles::montures::GenreBarde;
    static BARDES: std::sync::OnceLock<Vec<ModeleBarde>> = std::sync::OnceLock::new();
    BARDES.get_or_init(|| {
        fiches::Referentiel::partage()
            .equipement
            .armures
            .iter()
            .filter_map(|a| {
                let genre = match a.genre {
                    GenreProtection::Legere => GenreBarde::Legere,
                    GenreProtection::Intermediaire => GenreBarde::Intermediaire,
                    GenreProtection::Lourde => GenreBarde::Lourde,
                    _ => return None,
                };
                Some(ModeleBarde {
                    nom_en: a.nom_en.clone(),
                    nom: if a.nom.is_empty() {
                        a.nom_en.clone()
                    } else {
                        a.nom.clone()
                    },
                    genre,
                    bonus_ca: a.bonus_armure.unwrap_or(0),
                    dex_max: a.dex_max,
                    malus_armure: a.malus_armure.unwrap_or(0),
                    prix_base: crate::catalogue_inventaire::montant(&a.cout),
                    // 1 livre = 500 g dans la version française
                    poids_base_kg: a.poids_lb.map(|lb| lb * 0.5),
                })
            })
            .collect()
    })
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
                equipe: None,
                note: String::new(),
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

    fn riche(c: &mut Campagne, r: &regles::Referentiel) -> IdPersonnage {
        let id = c.nouvel_id_personnage();
        c.executer(
            Evenement::PersonnageCree {
                id,
                nom: "Riche".into(),
                compte_initial: Montant::po(50_000),
                prestige: None,
            },
            r,
        )
        .unwrap();
        id
    }

    fn acheter_modele(
        c: &mut Campagne,
        r: &regles::Referentiel,
        modele: &str,
        l: IdLocalite,
        payeur: IdPersonnage,
        prix: i64,
    ) -> IdVehicule {
        let id = c.nouvel_id_vehicule();
        c.executer(
            Evenement::VehiculeAchete {
                id,
                modele: modele.into(),
                nom: modele.into(),
                proprietaire: ProprietaireBien::Localite(l),
                payeur: Payeur::Personnage(payeur),
                prix: Montant::po(prix),
            },
            r,
        )
        .unwrap();
        id
    }

    #[cfg(feature = "serde")]
    #[test]
    fn les_ameliorations_se_paient_et_changent_les_capacites() {
        let (mut c, r, l, _) = monde();
        let riche = riche(&mut c, &r);
        let drakkar = acheter_modele(&mut c, &r, "drakkar", l, riche, 10_000);
        let sans = capacites_vehicule(&c.etat(&r).vehicules[&drakkar]);
        assert_eq!(sans.cargaison_kg, Some(50_000.0));
        assert_eq!(sans.passagers_max, Some(120));
        // 15 % du prix de base (10 000 po) : 1 500 po, cargaison +10 %
        let a = ameliorations_navires()
            .iter()
            .find(|a| a.id == "capacite_de_chargement")
            .unwrap();
        let m = modeles_vehicules()
            .iter()
            .find(|m| m.id == "drakkar")
            .unwrap();
        let prix = cout_amelioration(a, m, Montant::po(10_000), 1).unwrap();
        assert_eq!(prix, Montant::po(1_500));
        c.executer(
            Evenement::VehiculeAmelioree {
                id: drakkar,
                amelioration: a.id.clone(),
                quantite: 1,
                payeur: Payeur::Personnage(riche),
                prix,
            },
            &r,
        )
        .unwrap();
        assert_eq!(
            c.etat(&r).personnages[&riche].compte,
            Montant::po(50_000 - 10_000 - 1_500)
        );
        assert_eq!(
            capacites_vehicule(&c.etat(&r).vehicules[&drakkar]).cargaison_kg,
            Some(55_000.0)
        );
        // le blindage retire 15 % : 10 % - 15 % = -5 % au total
        c.executer(
            Evenement::VehiculeAmelioree {
                id: drakkar,
                amelioration: "blindage".into(),
                quantite: 1,
                payeur: Payeur::Personnage(riche),
                prix: Montant::po(3_000),
            },
            &r,
        )
        .unwrap();
        let cap = capacites_vehicule(&c.etat(&r).vehicules[&drakkar]);
        assert_eq!(cap.cargaison_kg, Some(47_500.0));
        assert_eq!(cap.vitesse_pct, -20);
        assert_eq!(cap.bonus_navigation, -1);
        // retrait de la première amélioration (sans remboursement)
        c.executer(
            Evenement::VehiculeAmeliorationRetiree {
                id: drakkar,
                index: 0,
            },
            &r,
        )
        .unwrap();
        assert_eq!(c.etat(&r).vehicules[&drakkar].ameliorations.len(), 1);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn couts_par_unite_et_par_taille() {
        let m = |id: &str| modeles_vehicules().iter().find(|m| m.id == id).unwrap();
        let a = |id: &str| ameliorations_navires().iter().find(|a| a.id == id).unwrap();
        // bélier : 1 000 po pour un bateau de taille C
        assert_eq!(
            cout_amelioration(a("belier"), m("drakkar"), Montant::po(10_000), 1),
            Some(Montant::po(1_000))
        );
        // coque magique : 4 500 po par case de bateau (3 cases pour un drakkar)
        assert_eq!(
            cout_amelioration(a("coque_magique"), m("drakkar"), Montant::po(10_000), 1),
            Some(Montant::po(13_500))
        );
        // rames magiques : 100 po par rame (40 rames)
        assert_eq!(
            cout_amelioration(a("rames_magiques"), m("drakkar"), Montant::po(10_000), 1),
            Some(Montant::po(4_000))
        );
        // compartiments secrets : 500 po par cube, ici 3 cubes
        assert_eq!(
            cout_amelioration(
                a("compartiments_secrets"),
                m("drakkar"),
                Montant::po(10_000),
                3
            ),
            Some(Montant::po(1_500))
        );
        // figure de proue : prix à saisir
        assert_eq!(
            cout_amelioration(a("figure_de_proue"), m("drakkar"), Montant::po(10_000), 1),
            None
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn la_barde_coute_deux_ou_quatre_fois_l_armure() {
        let chaine = modeles_barde()
            .iter()
            .find(|b| b.nom_en == "Chainmail")
            .unwrap();
        assert_eq!(chaine.genre, regles::montures::GenreBarde::Intermediaire);
        // cotte de mailles : 150 po, 40 livres (20 kg)
        assert_eq!(chaine.prix(false), Some(Montant::po(300)));
        assert_eq!(chaine.prix(true), Some(Montant::po(600)));
        assert_eq!(chaine.poids_kg(false), Some(20.0));
        assert_eq!(chaine.poids_kg(true), Some(40.0));
    }

    #[test]
    fn equiper_et_retirer_une_barde() {
        let (mut c, r, l, pj) = monde();
        let id = acheter_monture(&mut c, &r, ProprietaireBien::Localite(l), None, pj);
        c.executer(
            Evenement::MontureBardee {
                id,
                armure: "Padded".into(),
                payeur: Payeur::Personnage(pj),
                prix: Montant::po(10),
                gratuite: false,
            },
            &r,
        )
        .unwrap();
        assert_eq!(
            c.etat(&r).montures[&id].barde.as_ref().unwrap().armure,
            "Padded"
        );
        assert_eq!(
            c.etat(&r).personnages[&pj].compte,
            Montant::po(100 - 30 - 10)
        );
        c.executer(Evenement::MontureDebardee { id }, &r).unwrap();
        assert!(c.etat(&r).montures[&id].barde.is_none());
        // plus de barde à retirer
        assert!(c.executer(Evenement::MontureDebardee { id }, &r).is_err());
    }

    #[test]
    fn passagers_a_bord_et_inventaire_supprime_avec_le_vehicule() {
        use inventaire::{Commande, Conteneur, Proprietaire};
        let (mut c, r, l, pj) = monde();
        let id = acheter_vehicule_simple(&mut c, &r, l, pj);
        c.executer(Evenement::VehiculePassagers { id, nombre: 3 }, &r)
            .unwrap();
        assert_eq!(c.etat(&r).vehicules[&id].passagers, 3);
        c.executer(
            Evenement::Inventaire(Commande::CreerInventaire {
                proprietaire: Proprietaire {
                    type_id: PROPRIETAIRE_VEHICULE.into(),
                    ref_id: ref_vehicule(id),
                },
                conteneur: Conteneur {
                    capacite: None,
                    types_autorises: None,
                    contenu_compte: true,
                },
            }),
            &r,
        )
        .unwrap();
        assert!(c.etat(&r).inventaire_du_vehicule(id).is_some());
        c.executer(Evenement::VehiculeRetire { id }, &r).unwrap();
        assert!(c.etat(&r).inventaire_du_vehicule(id).is_none());
    }

    fn acheter_vehicule_simple(
        c: &mut Campagne,
        r: &regles::Referentiel,
        l: IdLocalite,
        pj: IdPersonnage,
    ) -> IdVehicule {
        acheter_modele(c, r, "barque", l, pj, 10)
    }
}
