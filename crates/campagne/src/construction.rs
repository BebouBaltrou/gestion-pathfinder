//! Bâtiments, étages, salles et chantiers (Guide de campagne p. 85 et 92-95).
//!
//! - Un chantier se termine quand tous ses capitaux sont dépensés **et** que sa durée est
//!   écoulée (GC p. 85, « Construction de bâtiments »).
//! - Chaque jour, les capitaux sont puisés dans les réserves de la localité, dans la limite
//!   des dépenses quotidiennes (GC p. 81), Magie comprise (PC-01). La limite est appliquée à la
//!   localité entière par défaut, ou par PJ présent en option (PC-14). Les recrutements
//!   d'équipes de la localité partagent cette limite et ces réserves.
//! - Étage en mode séquentiel : une seule salle en travaux avance à la fois (option
//!   personnalisée). Étages successifs : un étage n'avance que si les étages inférieurs n'ont
//!   plus de travaux en cours (option personnalisée).

use crate::etat::{Consequence, Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::*;
use domaine::fiches::{Admissibles, Unite};
use domaine::{CoutCapitaux, Montant, Ressource, TypeCapital};
use regles::Referentiel;
use std::collections::BTreeSet;

/// Ressources qu'une salle du référentiel admet, dans le contexte de son bâtiment.
pub fn ressources_admises(
    u: &Unite,
    autres_types_du_batiment: &BTreeSet<Ressource>,
) -> BTreeSet<Ressource> {
    match u.recettes.as_ref().map(|r| &r.admissibles) {
        None => BTreeSet::new(),
        Some(Admissibles::Liste { ressources }) => ressources.iter().copied().collect(),
        Some(Admissibles::Toutes) => Ressource::TOUTES.into_iter().collect(),
        Some(Admissibles::TypesDuBatiment) | Some(Admissibles::SalleHote) => {
            autres_types_du_batiment.clone()
        }
    }
}

/// Coche par défaut : toutes les ressources d'une liste explicite, rien sinon.
/// Type de revenu coché par défaut : le premier de la liste du livre (un seul type à la fois).
pub(crate) fn ressources_par_defaut(u: &Unite) -> Vec<Ressource> {
    match u.recettes.as_ref().map(|r| &r.admissibles) {
        Some(Admissibles::Liste { ressources }) => ressources.iter().take(1).copied().collect(),
        _ => Vec::new(),
    }
}

/// Un seul type de revenu coché à la fois : on garde le premier (anciens fichiers compris).
pub(crate) fn un_seul_type(ressources: &[Ressource]) -> Vec<Ressource> {
    ressources.iter().take(1).copied().collect()
}

/// Durée et coût d'un chantier avec accélération : durée ÷ k, capital accéléré × k (Travail
/// pour une salle, Influence pour une équipe ; GC p. 95 ; arrondi PC-10).
pub(crate) fn chantier_accelere(
    nature: NatureChantier,
    cout: CoutCapitaux,
    duree: u32,
    facteur: u32,
    jour: u32,
    accelere: TypeCapital,
) -> Chantier {
    let mut c = cout;
    *c.get_mut(accelere) *= facteur;
    Chantier {
        nature,
        cout: c,
        depense: CoutCapitaux::default(),
        duree: duree.div_ceil(facteur),
        jours_ecoules: 0,
        facteur,
        jour_debut: jour,
    }
}

/// Dépense d'un jour sur un chantier (dans la limite du budget et des réserves), puis +1 jour.
pub(crate) fn depenser_un_jour(ch: &mut Chantier, stock: &mut CoutCapitaux, budget: &mut u32) {
    let reste = ch.reste();
    for c in TypeCapital::TOUS {
        let n = reste.get(c).min(stock.get(c)).min(*budget);
        *ch.depense.get_mut(c) += n;
        *stock.get_mut(c) -= n;
        *budget -= n;
    }
    ch.jours_ecoules += 1;
}

impl Etat {
    pub fn batiment(&self, id: IdBatiment) -> Result<&Batiment, Erreur> {
        self.batiments.get(&id).ok_or(Erreur::BatimentInconnu(id))
    }
    pub fn salle(&self, id: IdSalle) -> Result<&SalleBatie, Erreur> {
        self.salles.get(&id).ok_or(Erreur::SalleInconnue(id))
    }
    fn salle_mut(&mut self, id: IdSalle) -> Result<&mut SalleBatie, Erreur> {
        self.salles.get_mut(&id).ok_or(Erreur::SalleInconnue(id))
    }

    /// Salles d'un étage, dans l'ordre.
    pub fn salles_etage(&self, etage: IdEtage) -> impl Iterator<Item = &SalleBatie> {
        self.salles.values().filter(move |s| s.etage == etage)
    }
    pub fn salles_batiment(&self, b: IdBatiment) -> impl Iterator<Item = &SalleBatie> {
        self.salles.values().filter(move |s| s.batiment == b)
    }
    pub fn batiments_localite(&self, l: IdLocalite) -> impl Iterator<Item = &Batiment> {
        self.batiments.values().filter(move |b| b.localite == l)
    }

    /// Nom affiché d'une salle : nom donné, sinon nom du référentiel.
    pub fn nom_salle(&self, s: &SalleBatie, r: &Referentiel) -> String {
        s.nom
            .clone()
            .filter(|n| !n.trim().is_empty())
            .or_else(|| r.salle(&s.reference).map(|u| u.nom.clone()))
            .unwrap_or_else(|| s.reference.clone())
    }

    /// Ressources admises par les autres salles construites du bâtiment (Clocher, Cour, Étalage).
    pub fn types_du_batiment(
        &self,
        b: IdBatiment,
        sauf: IdSalle,
        r: &Referentiel,
    ) -> BTreeSet<Ressource> {
        self.salles_batiment(b)
            .filter(|s| s.id != sauf)
            .filter_map(|s| r.salle(&s.reference))
            .filter(|u| {
                matches!(
                    u.recettes.as_ref().map(|x| &x.admissibles),
                    Some(Admissibles::Liste { .. } | Admissibles::Toutes)
                )
            })
            .flat_map(|u| ressources_admises(u, &BTreeSet::new()))
            .collect()
    }

    /// La salle a une extension Ameublement terminée.
    pub fn ameublement(s: &SalleBatie) -> bool {
        s.extensions
            .iter()
            .any(|e| e.reference == "ameublement" && e.statut == StatutExtension::Construite)
    }

    /// Bonus apporté par les extensions terminées de la salle (ameublement : +5, GC p. 108).
    pub fn bonus_extensions(s: &SalleBatie, r: &Referentiel) -> u32 {
        s.extensions
            .iter()
            .filter(|e| e.statut == StatutExtension::Construite)
            .filter_map(|e| r.extension(&e.reference))
            .filter_map(|u| u.recettes.as_ref())
            .map(|x| x.bonus)
            .sum()
    }

    /// Types de revenu qu'une salle peut générer : ceux de son profil ; pour une salle sans
    /// recettes propres mais meublée, ceux que le bâtiment génère déjà (lecture à valider).
    pub fn admises_salle(&self, s: &SalleBatie, r: &Referentiel) -> BTreeSet<Ressource> {
        let Some(u) = r.salle(&s.reference) else {
            return BTreeSet::new();
        };
        let types = self.types_du_batiment(s.batiment, s.id, r);
        if u.recettes.is_some() {
            ressources_admises(u, &types)
        } else if Self::bonus_extensions(s, r) > 0 {
            types
        } else {
            BTreeSet::new()
        }
    }

    pub(crate) fn pj_present(&self, l: IdLocalite) -> bool {
        self.personnages.values().any(|p| p.localite == Some(l))
    }

    pub(crate) fn appliquer_batiment(
        &mut self,
        ev: &Evenement,
        r: &Referentiel,
    ) -> Result<(), Erreur> {
        use Evenement::*;
        match ev {
            BatimentCree {
                id,
                localite,
                nom,
                classement,
                notes,
                etages_successifs,
                modele,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                if self.batiments.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                self.localite(*localite)?;
                self.batiments.insert(
                    *id,
                    Batiment {
                        id: *id,
                        localite: *localite,
                        nom: nom.clone(),
                        classement: classement.clone(),
                        notes: notes.clone(),
                        etages: Vec::new(),
                        etages_successifs: *etages_successifs,
                        modele: modele.clone(),
                    },
                );
            }
            BatimentModifie {
                id,
                nom,
                classement,
                notes,
                etages_successifs,
            } => {
                if nom.trim().is_empty() {
                    return Err(Erreur::NomVide);
                }
                let b = self
                    .batiments
                    .get_mut(id)
                    .ok_or(Erreur::BatimentInconnu(*id))?;
                b.nom = nom.clone();
                b.classement = classement.clone();
                b.notes = notes.clone();
                b.etages_successifs = *etages_successifs;
            }
            EtageAjoute {
                batiment,
                id,
                nom,
                mode,
            } => {
                if self
                    .batiments
                    .values()
                    .any(|b| b.etages.iter().any(|e| e.id == *id))
                {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                let b = self
                    .batiments
                    .get_mut(batiment)
                    .ok_or(Erreur::BatimentInconnu(*batiment))?;
                b.etages.push(Etage {
                    id: *id,
                    nom: nom.clone(),
                    mode: *mode,
                });
            }
            EtageModifie {
                batiment,
                id,
                nom,
                mode,
            } => {
                let b = self
                    .batiments
                    .get_mut(batiment)
                    .ok_or(Erreur::BatimentInconnu(*batiment))?;
                let e = b
                    .etages
                    .iter_mut()
                    .find(|e| e.id == *id)
                    .ok_or(Erreur::EtageInconnu(*id))?;
                e.nom = nom.clone();
                e.mode = *mode;
            }
            SalleAjoutee {
                id,
                batiment,
                etage,
                reference,
                nom,
                deja_construite,
            } => {
                if self.salles.contains_key(id) {
                    return Err(Erreur::IdDejaUtilise(id.to_string()));
                }
                let b = self.batiment(*batiment)?;
                if !b.etages.iter().any(|e| e.id == *etage) {
                    return Err(Erreur::EtageInconnu(*etage));
                }
                let u = r
                    .salle(reference)
                    .ok_or_else(|| Erreur::SalleReferentielInconnue(reference.clone()))?;
                self.salles.insert(
                    *id,
                    SalleBatie {
                        id: *id,
                        batiment: *batiment,
                        etage: *etage,
                        reference: reference.clone(),
                        nom: nom.clone(),
                        statut: if *deja_construite {
                            StatutSalle::Construite
                        } else {
                            StatutSalle::AConstruire
                        },
                        ressources: ressources_par_defaut(u),
                        revenus_actifs: true,
                        brisee: false,
                        extensions: Vec::new(),
                    },
                );
            }
            SalleModifiee {
                id,
                etage,
                nom,
                ressources,
                revenus_actifs,
                brisee,
            } => {
                let s = self.salle(*id)?.clone();
                let b = self.batiment(s.batiment)?;
                if !b.etages.iter().any(|e| e.id == *etage) {
                    return Err(Erreur::EtageInconnu(*etage));
                }
                let u = r
                    .salle(&s.reference)
                    .ok_or_else(|| Erreur::SalleReferentielInconnue(s.reference.clone()))?;
                let _ = u;
                let ok = self.admises_salle(&s, r);
                if let Some(x) = ressources.iter().find(|x| !ok.contains(x)) {
                    return Err(Erreur::RessourceNonAdmise {
                        salle: self.nom_salle(&s, r),
                        ressource: *x,
                    });
                }
                let m = self.salle_mut(*id)?;
                m.etage = *etage;
                m.nom = nom.clone();
                m.ressources = un_seul_type(ressources);
                m.revenus_actifs = *revenus_actifs;
                m.brisee = *brisee;
            }
            SalleRetiree { id } => {
                self.salle(*id)?;
                self.salles.remove(id);
            }
            ConstructionLancee { salle, facteur } => {
                if !(1..=4).contains(facteur) {
                    return Err(Erreur::FacteurInvalide(*facteur));
                }
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                if s.statut != StatutSalle::AConstruire {
                    return Err(Erreur::StatutIncompatible {
                        salle: nom,
                        attendu: "à construire",
                    });
                }
                let loc = self.batiment(s.batiment)?.localite;
                if !self.pj_present(loc) {
                    return Err(Erreur::AucunPjPresent(self.localite(loc)?.nom.clone()));
                }
                let u = r
                    .salle(&s.reference)
                    .ok_or_else(|| Erreur::SalleReferentielInconnue(s.reference.clone()))?;
                let (Some(cout), Some(duree)) = (u.cout, u.duree_jours) else {
                    return Err(Erreur::SansCoutFixe(nom));
                };
                let jour = self.jour;
                self.salle_mut(*salle)?.statut = StatutSalle::EnTravaux(chantier_accelere(
                    NatureChantier::Construction,
                    cout,
                    duree,
                    *facteur,
                    jour,
                    TypeCapital::Travail,
                ));
            }
            SalleAchetee {
                salle,
                payeur,
                prix,
            } => {
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                if s.statut != StatutSalle::AConstruire {
                    return Err(Erreur::StatutIncompatible {
                        salle: nom,
                        attendu: "à construire",
                    });
                }
                let u = r
                    .salle(&s.reference)
                    .ok_or_else(|| Erreur::SalleReferentielInconnue(s.reference.clone()))?;
                let prix = match prix {
                    Some(p) => *p,
                    None => Montant::po(u.prix_po.ok_or(Erreur::SansCoutFixe(nom))? as i64),
                };
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                let loc = self.batiment(s.batiment)?.localite;
                let payeur = payeur.unwrap_or(Payeur::Localite(loc));
                if prix.pc > 0 {
                    self.debiter(payeur, prix)?;
                }
                self.salle_mut(*salle)?.statut = StatutSalle::Construite;
            }
            AmeliorationLancee {
                salle,
                vers,
                facteur,
            } => {
                if !(1..=4).contains(facteur) {
                    return Err(Erreur::FacteurInvalide(*facteur));
                }
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                if s.statut != StatutSalle::Construite {
                    return Err(Erreur::StatutIncompatible {
                        salle: nom,
                        attendu: "construite",
                    });
                }
                let de = r
                    .salle(&s.reference)
                    .ok_or_else(|| Erreur::SalleReferentielInconnue(s.reference.clone()))?;
                let cible = r
                    .salle(vers)
                    .ok_or_else(|| Erreur::SalleReferentielInconnue(vers.clone()))?;
                if !de.amelioration_en.contains(vers) {
                    return Err(Erreur::AmeliorationImpossible {
                        de: de.nom.clone(),
                        vers: cible.nom.clone(),
                    });
                }
                let loc = self.batiment(s.batiment)?.localite;
                if !self.pj_present(loc) {
                    return Err(Erreur::AucunPjPresent(self.localite(loc)?.nom.clone()));
                }
                let diff = regles::calculs::cout_amelioration(de, cible)
                    .map_err(|_| Erreur::SansCoutFixe(nom))?;
                let jour = self.jour;
                self.salle_mut(*salle)?.statut = StatutSalle::EnTravaux(chantier_accelere(
                    NatureChantier::Amelioration { vers: vers.clone() },
                    diff.capitaux,
                    diff.duree_jours,
                    *facteur,
                    jour,
                    TypeCapital::Travail,
                ));
            }
            ExtensionAjoutee {
                salle,
                reference,
                deja_construite,
                cout,
                prix_po,
                duree,
            } => {
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                let x = r
                    .extension(reference)
                    .ok_or_else(|| Erreur::ExtensionInconnue(reference.clone()))?;
                if s.extensions.iter().any(|e| e.reference == *reference) {
                    return Err(Erreur::Extension(format!(
                        "{nom} a déjà l'extension {} (une de chaque type, GC p. 108)",
                        x.nom
                    )));
                }
                let (Some(c), Some(p), Some(d)) = (
                    cout.or(x.cout),
                    prix_po.or(x.prix_po),
                    duree.or(x.duree_jours),
                ) else {
                    return Err(Erreur::Extension(format!(
                        "{} : coût, prix et durée à saisir (voir la salle piégée, GC p. 106)",
                        x.nom
                    )));
                };
                self.salle_mut(*salle)?.extensions.push(ExtensionSalle {
                    reference: reference.clone(),
                    statut: if *deja_construite {
                        StatutExtension::Construite
                    } else {
                        StatutExtension::AConstruire
                    },
                    cout: c,
                    prix_po: p,
                    duree: d,
                });
            }
            ExtensionLancee {
                salle,
                reference,
                facteur,
            } => {
                if !(1..=4).contains(facteur) {
                    return Err(Erreur::FacteurInvalide(*facteur));
                }
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                if s.statut == StatutSalle::AConstruire {
                    return Err(Erreur::Extension(format!(
                        "{nom} doit être construite ou en construction avant son extension"
                    )));
                }
                let x = s
                    .extensions
                    .iter()
                    .find(|e| e.reference == *reference)
                    .cloned()
                    .ok_or_else(|| Erreur::Extension(format!("{nom} n'a pas cette extension")))?;
                if x.statut != StatutExtension::AConstruire {
                    return Err(Erreur::Extension(format!(
                        "l'extension de {nom} n'est plus à construire"
                    )));
                }
                let loc = self.batiment(s.batiment)?.localite;
                if !self.pj_present(loc) {
                    return Err(Erreur::AucunPjPresent(self.localite(loc)?.nom.clone()));
                }
                let jour = self.jour;
                let ch = chantier_accelere(
                    NatureChantier::Construction,
                    x.cout,
                    x.duree,
                    *facteur,
                    jour,
                    TypeCapital::Travail,
                );
                let m = self.salle_mut(*salle)?;
                if let Some(e) = m.extensions.iter_mut().find(|e| e.reference == *reference) {
                    e.statut = StatutExtension::EnTravaux(ch);
                }
            }
            ExtensionAchetee {
                salle,
                reference,
                payeur,
                prix,
            } => {
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                let x = s
                    .extensions
                    .iter()
                    .find(|e| e.reference == *reference)
                    .cloned()
                    .ok_or_else(|| Erreur::Extension(format!("{nom} n'a pas cette extension")))?;
                if x.statut != StatutExtension::AConstruire {
                    return Err(Erreur::Extension(format!(
                        "l'extension de {nom} n'est plus à construire"
                    )));
                }
                let prix = prix.unwrap_or(Montant::po(x.prix_po as i64));
                if prix.est_negatif() {
                    return Err(Erreur::MontantNonPositif);
                }
                let loc = self.batiment(s.batiment)?.localite;
                if prix.pc > 0 {
                    self.debiter(payeur.unwrap_or(Payeur::Localite(loc)), prix)?;
                }
                let m = self.salle_mut(*salle)?;
                if let Some(e) = m.extensions.iter_mut().find(|e| e.reference == *reference) {
                    e.statut = StatutExtension::Construite;
                }
            }
            ExtensionRetiree { salle, reference } => {
                let s = self.salle(*salle)?.clone();
                let nom = self.nom_salle(&s, r);
                match s.extensions.iter().find(|e| e.reference == *reference) {
                    Some(e) if e.statut == StatutExtension::AConstruire => {}
                    Some(_) => {
                        return Err(Erreur::Extension(format!(
                            "une extension commencée ou faite est permanente ({nom}, GC p. 108)"
                        )))
                    }
                    None => {
                        return Err(Erreur::Extension(format!("{nom} n'a pas cette extension")))
                    }
                }
                self.salle_mut(*salle)?
                    .extensions
                    .retain(|e| e.reference != *reference);
            }
            _ => unreachable!("événement non lié aux bâtiments"),
        }
        Ok(())
    }

    /// Salles dont le chantier avance aujourd'hui, selon les modes des étages et du bâtiment.
    pub fn chantiers_actifs(&self, b: &Batiment) -> Vec<IdSalle> {
        let mut actifs = Vec::new();
        for etage in &b.etages {
            let en_travaux: Vec<IdSalle> = self
                .salles_etage(etage.id)
                .filter(|s| matches!(s.statut, StatutSalle::EnTravaux(_)))
                .map(|s| s.id)
                .collect();
            match etage.mode {
                ModeChantier::Parallele => actifs.extend(&en_travaux),
                ModeChantier::Sequentiel => actifs.extend(en_travaux.first()),
            }
            if b.etages_successifs && !en_travaux.is_empty() {
                break; // les étages supérieurs attendent
            }
        }
        actifs
    }

    /// Un jour de travaux : dépense des capitaux (limite quotidienne par localité) puis temps.
    pub(crate) fn avancer_chantiers(&mut self, r: &Referentiel) -> Vec<Consequence> {
        let mut out = Vec::new();
        let localites: Vec<IdLocalite> = self.localites.keys().copied().collect();
        for l in localites {
            let mut budget = self.limite_depense(l, r).unwrap_or(0);
            let actifs: Vec<IdSalle> = self
                .batiments_localite(l)
                .cloned()
                .collect::<Vec<_>>()
                .iter()
                .flat_map(|b| self.chantiers_actifs(b))
                .collect();
            for id in actifs {
                let mut stock = self.localites[&l].capitaux;
                let Some(salle) = self.salles.get_mut(&id) else {
                    continue;
                };
                let StatutSalle::EnTravaux(ch) = &mut salle.statut else {
                    continue;
                };
                depenser_un_jour(ch, &mut stock, &mut budget);
                let termine = ch.termine();
                let nature = ch.nature.clone();
                self.localites.get_mut(&l).expect("localité").capitaux = stock;
                if termine {
                    let s = self.salles.get_mut(&id).expect("salle");
                    if let NatureChantier::Amelioration { vers } = nature {
                        s.reference = vers;
                        if let Some(u) = r.salle(&s.reference) {
                            s.ressources = ressources_par_defaut(u);
                        }
                    }
                    s.statut = StatutSalle::Construite;
                    let s = s.clone();
                    let nom = self.nom_salle(&s, r);
                    let batiment = self
                        .batiments
                        .get(&s.batiment)
                        .map(|b| b.nom.clone())
                        .unwrap_or_default();
                    out.push(Consequence::SalleTerminee {
                        salle: id,
                        nom,
                        batiment,
                    });
                }
            }
            // Extensions en travaux : même réserve, même limite quotidienne.
            let salles_ext: Vec<IdSalle> = self
                .salles
                .values()
                .filter(|s| {
                    self.batiments
                        .get(&s.batiment)
                        .is_some_and(|b| b.localite == l)
                        && s.extensions
                            .iter()
                            .any(|e| matches!(e.statut, StatutExtension::EnTravaux(_)))
                })
                .map(|s| s.id)
                .collect();
            for id in salles_ext {
                let mut stock = self.localites[&l].capitaux;
                let mut terminees = Vec::new();
                let s = self.salles.get_mut(&id).expect("salle");
                for e in s.extensions.iter_mut() {
                    if let StatutExtension::EnTravaux(ch) = &mut e.statut {
                        depenser_un_jour(ch, &mut stock, &mut budget);
                        if ch.termine() {
                            e.statut = StatutExtension::Construite;
                            terminees.push(e.reference.clone());
                        }
                    }
                }
                self.localites.get_mut(&l).expect("localité").capitaux = stock;
                let s = self.salles[&id].clone();
                for x in terminees {
                    let ext = r.extension(&x).map(|u| u.nom.clone()).unwrap_or(x);
                    out.push(Consequence::SalleTerminee {
                        salle: id,
                        nom: format!("{ext} de {}", self.nom_salle(&s, r)),
                        batiment: self
                            .batiments
                            .get(&s.batiment)
                            .map(|b| b.nom.clone())
                            .unwrap_or_default(),
                    });
                }
            }
            // Recrutements des équipes de la localité : même réserve, même limite quotidienne.
            out.extend(self.avancer_recrutements(l, &mut budget, r));
        }
        out
    }

    // ------------------------------------------------------------ durées et totaux

    /// Durée totale (jours) d'une salle à construire, en travaux ou construite (0).
    pub fn duree_salle(&self, s: &SalleBatie, r: &Referentiel) -> u32 {
        match &s.statut {
            StatutSalle::AConstruire => r
                .salle(&s.reference)
                .and_then(|u| u.duree_jours)
                .unwrap_or(0),
            StatutSalle::EnTravaux(c) => c.duree,
            StatutSalle::Construite => 0,
        }
    }

    /// Durée restante d'un étage : somme (séquentiel) ou maximum (parallèle) des salles.
    pub fn duree_restante_etage(&self, e: &Etage, r: &Referentiel) -> u32 {
        let durees = self.salles_etage(e.id).map(|s| {
            let salle = match &s.statut {
                StatutSalle::EnTravaux(c) => c.duree.saturating_sub(c.jours_ecoules),
                _ => self.duree_salle(s, r),
            };
            let ext = s
                .extensions
                .iter()
                .map(|x| match &x.statut {
                    StatutExtension::AConstruire => x.duree,
                    StatutExtension::EnTravaux(c) => c.duree.saturating_sub(c.jours_ecoules),
                    StatutExtension::Construite => 0,
                })
                .max()
                .unwrap_or(0);
            salle.max(ext)
        });
        match e.mode {
            ModeChantier::Parallele => durees.max().unwrap_or(0),
            ModeChantier::Sequentiel => durees.sum(),
        }
    }

    /// Durée restante d'un bâtiment : somme (étages successifs) ou maximum des étages.
    pub fn duree_restante_batiment(&self, b: &Batiment, r: &Referentiel) -> u32 {
        let d = b.etages.iter().map(|e| self.duree_restante_etage(e, r));
        if b.etages_successifs {
            d.sum()
        } else {
            d.max().unwrap_or(0)
        }
    }

    /// Capitaux restant à dépenser : salles à construire + chantiers en cours.
    pub fn cout_restant<'a>(
        &self,
        salles: impl Iterator<Item = &'a SalleBatie>,
        r: &Referentiel,
    ) -> CoutCapitaux {
        salles.fold(CoutCapitaux::default(), |acc, s| {
            let ext = s.extensions.iter().fold(CoutCapitaux::default(), |c, x| {
                c + match &x.statut {
                    StatutExtension::AConstruire => x.cout,
                    StatutExtension::EnTravaux(ch) => ch.reste(),
                    StatutExtension::Construite => CoutCapitaux::default(),
                }
            });
            acc + ext
                + match &s.statut {
                    StatutSalle::AConstruire => r
                        .salle(&s.reference)
                        .and_then(|u| u.cout)
                        .unwrap_or_default(),
                    StatutSalle::EnTravaux(c) => c.reste(),
                    StatutSalle::Construite => CoutCapitaux::default(),
                }
        })
    }

    /// Bonus de recettes par ressource cochée, pour les salles construites dont les revenus
    /// sont actifs (salle brisée : moitié, arrondi inférieur). Avec la répartition stricte, une
    /// salle ne contribue qu'à un type par jour : ces totaux sont des maximums par type.
    ///
    /// `production_pendant_amelioration` : option personnalisée, une salle en cours
    /// d'amélioration continue de produire avec son profil actuel.
    pub fn bonus_par_ressource<'a>(
        &self,
        salles: impl Iterator<Item = &'a SalleBatie>,
        r: &Referentiel,
        production_pendant_amelioration: bool,
    ) -> std::collections::BTreeMap<Ressource, u32> {
        let mut m = std::collections::BTreeMap::new();
        for s in salles {
            let produit = match &s.statut {
                StatutSalle::Construite => true,
                StatutSalle::EnTravaux(c) => {
                    production_pendant_amelioration
                        && matches!(c.nature, NatureChantier::Amelioration { .. })
                }
                StatutSalle::AConstruire => false,
            };
            if !produit || !s.revenus_actifs {
                continue;
            }
            let b = r
                .salle(&s.reference)
                .and_then(|u| u.recettes.as_ref())
                .map(|x| x.bonus)
                .unwrap_or(0)
                + Self::bonus_extensions(s, r);
            if b == 0 {
                continue;
            }
            let b = if s.brisee { b / 2 } else { b };
            for x in &s.ressources {
                *m.entry(*x).or_insert(0) += b;
            }
        }
        m
    }
}

/// Événements pour créer un bâtiment depuis un modèle du référentiel : un étage, toutes ses
/// salles à construire. Les extensions du modèle (ameublement, fortification) ne sont pas
/// encore gérées et sont signalées dans le retour.
pub fn evenements_depuis_modele(
    r: &Referentiel,
    modele: &domaine::fiches::Modele,
    batiment: IdBatiment,
    localite: IdLocalite,
    nom: &str,
    premier_etage: IdEtage,
    premiere_salle: IdSalle,
) -> (Vec<Evenement>, Vec<String>) {
    let mut evs = vec![
        Evenement::BatimentCree {
            id: batiment,
            localite,
            nom: nom.to_string(),
            classement: modele.nom.clone(),
            notes: String::new(),
            etages_successifs: false,
            modele: Some(modele.id.clone()),
        },
        Evenement::EtageAjoute {
            batiment,
            id: premier_etage,
            nom: "Rez-de-chaussée".into(),
            mode: ModeChantier::Parallele,
        },
    ];
    let mut ignores = Vec::new();
    let mut n = premiere_salle.0;
    let mut salles: Vec<(IdSalle, String)> = Vec::new();
    for e in &modele.composition {
        if let domaine::fiches::ElementModele::Unite { id, quantite } = e {
            for _ in 0..*quantite {
                evs.push(Evenement::SalleAjoutee {
                    id: IdSalle(n),
                    batiment,
                    etage: premier_etage,
                    reference: id.clone(),
                    nom: None,
                    deja_construite: false,
                });
                salles.push((IdSalle(n), id.clone()));
                n += 1;
            }
        }
    }
    for e in &modele.composition {
        if let domaine::fiches::ElementModele::Extension { id, quantite, sur } = e {
            let cibles = salles_designees(sur, &salles, r);
            let mut prises: Vec<IdSalle> = Vec::new();
            for (reference, nombre) in cibles {
                prises.extend(
                    salles
                        .iter()
                        .filter(|(_, x)| *x == reference)
                        .map(|(s, _)| *s)
                        .take(nombre as usize),
                );
            }
            if prises.len() as u32 != *quantite {
                ignores.push(format!("{quantite} × extension {id} ({sur})"));
                continue;
            }
            for s in prises {
                evs.push(Evenement::ExtensionAjoutee {
                    salle: s,
                    reference: id.clone(),
                    deja_construite: false,
                    cout: None,
                    prix_po: None,
                    duree: None,
                });
            }
        }
    }
    (evs, ignores)
}

/// Normalise un nom pour comparer « Chambre-forte » et « chambre forte ».
fn normaliser(t: &str) -> String {
    t.to_lowercase()
        .replace(['-', '’', '\'', ',', ';', '.', '(', ')'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Salles désignées par le libellé d'une extension de modèle (« dans la salle de bal, deux
/// chambres à coucher, un bureau… ») : (référence, nombre).
fn salles_designees(
    sur: &str,
    salles: &[(IdSalle, String)],
    r: &Referentiel,
) -> Vec<(String, u32)> {
    let texte = normaliser(sur);
    let mots: Vec<&str> = texte.split(' ').collect();
    let mut refs: Vec<String> = salles.iter().map(|(_, x)| x.clone()).collect();
    refs.sort();
    refs.dedup();
    // noms les plus longs d'abord (« salle du trône » avant « salle »)
    let mut noms: Vec<(String, Vec<String>)> = refs
        .iter()
        .filter_map(|x| r.salle(x).map(|u| (x.clone(), normaliser(&u.nom))))
        .map(|(x, n)| (x, n.split(' ').map(String::from).collect()))
        .collect();
    noms.sort_by_key(|(_, n)| std::cmp::Reverse(n.len()));
    let mut pris = vec![false; mots.len()];
    let mut out = Vec::new();
    for (reference, nom) in noms {
        let k = nom.len();
        if k == 0 || k > mots.len() {
            continue;
        }
        for i in 0..=mots.len() - k {
            if pris[i..i + k].iter().any(|x| *x) {
                continue;
            }
            // pluriel accepté : « chambres » pour « chambre »
            let ok = (0..k).all(|j| {
                mots[i + j] == nom[j] || mots[i + j].strip_suffix('s') == Some(nom[j].as_str())
            });
            if !ok {
                continue;
            }
            let nombre = match i.checked_sub(1).map(|p| mots[p]) {
                Some("deux") => 2,
                Some("trois") => 3,
                Some("quatre") => 4,
                _ => 1,
            };
            pris[i..i + k].iter_mut().for_each(|x| *x = true);
            out.push((reference.clone(), nombre));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Campagne;
    use domaine::Montant;

    struct Monde {
        c: Campagne,
        r: Referentiel,
        pj: IdPersonnage,
        l: IdLocalite,
        b: IdBatiment,
        e: IdEtage,
    }

    /// Petite bourgade (limite 15/jour), réserves 30 M, 10 I, 30 T, 5 Mg, 2 000 po, PJ présent.
    fn monde(categorie: &str) -> Monde {
        let r = regles::referentiel_officiel();
        let mut c = Campagne::nouvelle("Test");
        let pj = c.nouvel_id_personnage();
        let l = c.nouvel_id_localite();
        let b = c.nouvel_id_batiment();
        let e = c.nouvel_id_etage();
        for ev in [
            Evenement::PersonnageCree {
                id: pj,
                nom: "Seoni".into(),
                compte_initial: Montant::ZERO,
                prestige: None,
            },
            Evenement::LocaliteCreee {
                id: l,
                nom: "Pointesable".into(),
                genre: "ville".into(),
                population: None,
                categorie: CategorieLocalite::Officielle {
                    id: categorie.into(),
                },
                notes: String::new(),
            },
            Evenement::PersonnageDeplace {
                id: pj,
                vers: Some(l),
            },
            Evenement::AjustementLocalite {
                localite: l,
                delta_po: Montant::po(2000),
                ajout: CoutCapitaux {
                    marchandises: 30,
                    influence: 10,
                    travail: 30,
                    magie: 5,
                },
                retrait: CoutCapitaux::default(),
                motif: "départ".into(),
            },
            Evenement::BatimentCree {
                id: b,
                localite: l,
                nom: "La Rouille".into(),
                classement: "taverne".into(),
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
        ] {
            c.executer(ev, &r).unwrap();
        }
        Monde { c, r, pj, l, b, e }
    }

    impl Monde {
        fn ajouter(&mut self, reference: &str) -> IdSalle {
            let id = self.c.nouvel_id_salle();
            let ev = Evenement::SalleAjoutee {
                id,
                batiment: self.b,
                etage: self.e,
                reference: reference.into(),
                nom: None,
                deja_construite: false,
            };
            self.c.executer(ev, &self.r).unwrap();
            id
        }
        fn ev(&mut self, ev: Evenement) -> Result<Vec<Consequence>, Erreur> {
            self.c.executer(ev, &self.r)
        }
        fn statut(&mut self, s: IdSalle) -> StatutSalle {
            self.c.etat(&self.r).salle(s).unwrap().statut.clone()
        }
        fn chantier(&mut self, s: IdSalle) -> Chantier {
            match self.statut(s) {
                StatutSalle::EnTravaux(c) => c,
                x => panic!("pas en travaux : {x:?}"),
            }
        }
    }

    #[test]
    fn construction_d_un_bar() {
        // Bar : 6 M, 1 I, 5 T ; 16 jours (GC p. 98)
        let mut m = monde("petite-bourgade");
        let bar = m.ajouter("bar");
        assert_eq!(m.statut(bar), StatutSalle::AConstruire);
        m.ev(Evenement::ConstructionLancee {
            salle: bar,
            facteur: 1,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        let ch = m.chantier(bar);
        assert_eq!(
            ch.depense,
            CoutCapitaux {
                marchandises: 6,
                influence: 1,
                travail: 5,
                magie: 0
            }
        );
        assert_eq!(ch.jours_ecoules, 1);
        let l = m.l;
        assert_eq!(
            m.c.etat(&m.r).localite(l).unwrap().capitaux.marchandises,
            24
        );
        let c = m.ev(Evenement::JoursAvances { nombre: 14 }).unwrap();
        assert!(c.is_empty());
        let c = m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        assert!(matches!(&c[..], [Consequence::SalleTerminee { .. }]));
        assert_eq!(m.statut(bar), StatutSalle::Construite);
    }

    #[test]
    fn limite_quotidienne_et_reserves() {
        // Lieu-dit : 2 unités par jour ; le bar coûte 12 unités → 6 jours de dépenses
        let mut m = monde("lieu-dit");
        let bar = m.ajouter("bar");
        m.ev(Evenement::ConstructionLancee {
            salle: bar,
            facteur: 1,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        assert_eq!(m.chantier(bar).depense.unites(), 2);
        assert_eq!(m.chantier(bar).jours_restants_estimes(2), 15);
        // Réserves vides : le temps passe mais rien n'est dépensé
        let l = m.l;
        let stock = m.c.etat(&m.r).localite(l).unwrap().capitaux;
        m.ev(Evenement::AjustementLocalite {
            localite: l,
            delta_po: Montant::ZERO,
            ajout: CoutCapitaux::default(),
            retrait: stock,
            motif: "vol".into(),
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 20 }).unwrap();
        let ch = m.chantier(bar);
        assert_eq!((ch.depense.unites(), ch.jours_ecoules), (2, 21));
        assert!(!ch.termine());
    }

    #[test]
    fn pj_present_obligatoire() {
        let mut m = monde("petite-bourgade");
        let bar = m.ajouter("bar");
        let pj = m.pj;
        m.ev(Evenement::PersonnageDeplace { id: pj, vers: None })
            .unwrap();
        assert!(matches!(
            m.ev(Evenement::ConstructionLancee {
                salle: bar,
                facteur: 1
            }),
            Err(Erreur::AucunPjPresent(_))
        ));
    }

    #[test]
    fn acceleration_et_sequentiel() {
        let mut m = monde("metropole");
        let (b, e) = (m.b, m.e);
        m.ev(Evenement::EtageModifie {
            batiment: b,
            id: e,
            nom: "RDC".into(),
            mode: ModeChantier::Sequentiel,
        })
        .unwrap();
        let bar = m.ajouter("bar");
        let cuisine = m.ajouter("cuisine"); // 4 M, 4 T ; 12 jours
        m.ev(Evenement::ConstructionLancee {
            salle: bar,
            facteur: 2,
        })
        .unwrap();
        m.ev(Evenement::ConstructionLancee {
            salle: cuisine,
            facteur: 1,
        })
        .unwrap();
        let ch = m.chantier(bar);
        assert_eq!((ch.cout.travail, ch.duree), (10, 8));
        m.ev(Evenement::JoursAvances { nombre: 8 }).unwrap();
        assert_eq!(m.statut(bar), StatutSalle::Construite);
        assert_eq!(m.chantier(cuisine).jours_ecoules, 0); // attendait son tour
        m.ev(Evenement::JoursAvances { nombre: 12 }).unwrap();
        assert_eq!(m.statut(cuisine), StatutSalle::Construite);
    }

    #[test]
    fn limite_par_pj_option() {
        // Lieu-dit : 2 unités par jour et par personnage
        let mut m = monde("lieu-dit");
        let l = m.l;
        let id2 = m.c.nouvel_id_personnage();
        m.ev(Evenement::PersonnageCree {
            id: id2,
            nom: "Seconde".into(),
            compte_initial: Montant::ZERO,
            prestige: None,
        })
        .unwrap();
        m.ev(Evenement::PersonnageDeplace {
            id: id2,
            vers: Some(l),
        })
        .unwrap();
        assert_eq!(m.c.etat(&m.r).limite_depense(l, &m.r), Some(2));
        m.ev(Evenement::ReglageLimiteDepense { par_pj: true })
            .unwrap();
        assert_eq!(m.c.etat(&m.r).limite_depense(l, &m.r), Some(4));
        let bar = m.ajouter("bar");
        m.ev(Evenement::ConstructionLancee {
            salle: bar,
            facteur: 1,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 1 }).unwrap();
        assert_eq!(m.chantier(bar).depense.unites(), 4);
        // annuler le réglage revient à la limite par localité
        for _ in 0..4 {
            m.c.annuler(); // jours, lancement, salle, réglage
        }
        assert_eq!(m.c.etat(&m.r).limite_depense(l, &m.r), Some(2));
    }

    #[test]
    fn achats_payes_par_le_pj() {
        let mut m = monde("petite-bourgade");
        let (pj, l) = (m.pj, m.l);
        m.ev(Evenement::AjustementCompte {
            pj,
            delta: Montant::po(500),
            motif: "trésor".into(),
        })
        .unwrap();
        // Bâtiment déjà payé ailleurs : prix saisi par le joueur, payé par le PJ
        let bar = m.ajouter("bar");
        m.ev(Evenement::SalleAchetee {
            salle: bar,
            payeur: Some(Payeur::Personnage(pj)),
            prix: Some(Montant::po(150)),
        })
        .unwrap();
        assert_eq!(m.statut(bar), StatutSalle::Construite);
        // 2 Marchandises + 1 Magie = 2×20 + 100 = 140 po (Table 2-1)
        let avant = m.c.etat(&m.r).localite(l).unwrap().capitaux;
        m.ev(Evenement::CapitauxAchetes {
            localite: l,
            payeur: Payeur::Personnage(pj),
            capitaux: CoutCapitaux {
                marchandises: 2,
                magie: 1,
                ..Default::default()
            },
        })
        .unwrap();
        let etat = m.c.etat(&m.r).clone();
        assert_eq!(etat.personnages[&pj].compte, Montant::po(210));
        let apres = etat.localite(l).unwrap().capitaux;
        assert_eq!(
            (
                apres.marchandises - avant.marchandises,
                apres.magie - avant.magie
            ),
            (2, 1)
        );
        // Solde insuffisant : refusé
        assert!(matches!(
            m.ev(Evenement::Paiement {
                payeur: Payeur::Personnage(pj),
                montant: Montant::po(1000),
                motif: "trop cher".into(),
            }),
            Err(Erreur::CompteInsuffisant { .. })
        ));
    }

    #[test]
    fn achat_direct() {
        let mut m = monde("petite-bourgade");
        let bar = m.ajouter("bar");
        m.ev(Evenement::SalleAchetee {
            salle: bar,
            payeur: None,
            prix: None,
        })
        .unwrap();
        assert_eq!(m.statut(bar), StatutSalle::Construite);
        let l = m.l;
        assert_eq!(
            m.c.etat(&m.r).localite(l).unwrap().tresorerie,
            Montant::po(1750)
        );
    }

    #[test]
    fn amelioration_bibliotheque() {
        let mut m = monde("metropole");
        let id = m.c.nouvel_id_salle();
        let (b, e) = (m.b, m.e);
        m.ev(Evenement::SalleAjoutee {
            id,
            batiment: b,
            etage: e,
            reference: "bibliotheque".into(),
            nom: Some("Archives".into()),
            deja_construite: true,
        })
        .unwrap();
        let mauvais = m.ev(Evenement::AmeliorationLancee {
            salle: id,
            vers: "forge".into(),
            facteur: 1,
        });
        assert!(matches!(
            mauvais,
            Err(Erreur::AmeliorationImpossible { .. })
        ));
        m.ev(Evenement::AmeliorationLancee {
            salle: id,
            vers: "bibliotheque-magique".into(),
            facteur: 1,
        })
        .unwrap();
        let ch = m.chantier(id);
        assert_eq!(
            ch.cout,
            CoutCapitaux {
                marchandises: 1,
                influence: 1,
                travail: 1,
                magie: 2
            }
        );
        assert_eq!(ch.duree, 4);
        m.ev(Evenement::JoursAvances { nombre: 4 }).unwrap();
        let s = m.c.etat(&m.r).salle(id).unwrap().clone();
        assert_eq!(
            (s.reference.as_str(), s.statut),
            ("bibliotheque-magique", StatutSalle::Construite)
        );
        assert_eq!(s.nom.as_deref(), Some("Archives"));
    }

    #[test]
    fn ressources_cochees_et_bonus() {
        let mut m = monde("petite-bourgade");
        let id = m.c.nouvel_id_salle();
        let (b, e) = (m.b, m.e);
        m.ev(Evenement::SalleAjoutee {
            id,
            batiment: b,
            etage: e,
            reference: "bar".into(),
            nom: None,
            deja_construite: true,
        })
        .unwrap();
        let modif = |ress: Vec<Ressource>, brisee| Evenement::SalleModifiee {
            id,
            etage: e,
            nom: None,
            ressources: ress,
            revenus_actifs: true,
            brisee,
        };
        assert!(matches!(
            m.ev(modif(vec![Ressource::Magie], false)),
            Err(Erreur::RessourceNonAdmise { .. })
        ));
        m.ev(modif(vec![Ressource::Influence], true)).unwrap();
        let etat = m.c.etat(&m.r).clone();
        let bonus = etat.bonus_par_ressource(etat.salles_batiment(b), &m.r, false);
        assert_eq!(bonus.get(&Ressource::Influence), Some(&5)); // +10 brisé → +5
        assert_eq!(bonus.get(&Ressource::Po), None);
    }

    #[test]
    fn batiment_depuis_un_modele() {
        let mut m = monde("petite-bourgade");
        let modele = m.r.modele_batiment("taverne").unwrap().clone();
        let (b, e, s) = (
            m.c.nouvel_id_batiment(),
            m.c.nouvel_id_etage(),
            m.c.nouvel_id_salle(),
        );
        let l = m.l;
        let (evs, ignores) = evenements_depuis_modele(&m.r, &modele, b, l, "Le Dragon vert", e, s);
        assert!(ignores.is_empty());
        m.ev(Evenement::Groupe {
            libelle: "Taverne".into(),
            evenements: evs,
        })
        .unwrap();
        let etat = m.c.etat(&m.r).clone();
        assert_eq!(etat.salles_batiment(b).count(), 5);
        assert_eq!(
            etat.cout_restant(etat.salles_batiment(b), &m.r),
            modele.cout
        );
        // le bar (16 j) est la plus longue salle : durée du bâtiment en parallèle
        assert_eq!(
            etat.duree_restante_batiment(etat.batiment(b).unwrap(), &m.r),
            16
        );
        assert_eq!(m.c.nouvel_id_salle(), IdSalle(s.0 + 5));
        // annuler le groupe retire tout d'un coup
        m.c.annuler();
        assert!(!m.c.etat(&m.r).batiments.contains_key(&b));
    }

    #[test]
    fn extensions_des_modeles() {
        let mut m = monde("petite-bourgade");
        let ids: Vec<String> = m.r.modeles_batiments.iter().map(|x| x.id.clone()).collect();
        for id in ids {
            let modele = m.r.modele_batiment(&id).unwrap().clone();
            let (evs, ignores) = evenements_depuis_modele(
                &m.r,
                &modele,
                m.c.nouvel_id_batiment(),
                m.l,
                "x",
                m.c.nouvel_id_etage(),
                m.c.nouvel_id_salle(),
            );
            let n = evs
                .iter()
                .filter(|e| matches!(e, Evenement::ExtensionAjoutee { .. }))
                .count();
            println!("{id}: {n} extensions, ignorées {ignores:?}");
            assert!(ignores.is_empty(), "{id} : {ignores:?}");
            m.ev(Evenement::Groupe {
                libelle: id.clone(),
                evenements: evs,
            })
            .unwrap();
            let etat = m.c.etat(&m.r).clone();
            let b = *etat.batiments.keys().max().unwrap();
            // le coût du modèle (GC) inclut ses extensions
            assert_eq!(
                etat.cout_restant(etat.salles_batiment(b), &m.r),
                modele.cout,
                "{id}"
            );
            m.c.annuler();
        }
    }

    #[test]
    fn extensions_d_une_salle() {
        let mut m = monde("petite-bourgade");
        let bar = m.ajouter("bar");
        m.ev(Evenement::SalleAchetee {
            salle: bar,
            payeur: None,
            prix: None,
        })
        .ok();
        let ajout = |reference: &str| Evenement::ExtensionAjoutee {
            salle: bar,
            reference: reference.into(),
            deja_construite: false,
            cout: None,
            prix_po: None,
            duree: None,
        };
        m.ev(ajout("ameublement")).unwrap();
        // une seule extension de chaque type par salle
        assert!(m.ev(ajout("ameublement")).is_err());
        m.ev(ajout("fortification")).unwrap();
        m.ev(Evenement::ExtensionRetiree {
            salle: bar,
            reference: "fortification".into(),
        })
        .unwrap();
        m.ev(Evenement::ExtensionLancee {
            salle: bar,
            reference: "ameublement".into(),
            facteur: 1,
        })
        .unwrap();
        m.ev(Evenement::JoursAvances { nombre: 30 }).unwrap();
        let etat = m.c.etat(&m.r).clone();
        let s = etat.salle(bar).unwrap();
        assert_eq!(s.extensions.len(), 1);
        assert_eq!(s.extensions[0].statut, StatutExtension::Construite);
        // ameublement : +5 au test de la salle (GC p. 108)
        assert_eq!(Etat::bonus_extensions(s, &m.r), 5);
        // extension terminée : permanente
        assert!(m
            .ev(Evenement::ExtensionRetiree {
                salle: bar,
                reference: "ameublement".into(),
            })
            .is_err());
    }
}
