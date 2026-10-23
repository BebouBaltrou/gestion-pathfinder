//! Effets temporisés : un seul modèle pour buffs, malus, états, poisons, maladies, sorts
//! persistants et entités. Le contenu (`T`) est opaque ici : les bonus typés restent dans
//! `fiches`, ce module ne gère que le temps, les expirations et les alertes.
//!
//! Chaque effet garde un **instant d'expiration absolu** (jamais un compteur qui décroît).

use crate::duree::{Duree, Echeance, MomentDuTour};
use crate::instant::Instant;
use crate::ErreurTemps;

pub type IdEffet = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Temporise<T> {
    pub id: IdEffet,
    pub nom: String,
    /// D'où vient l'effet (« sort Hâte, lancé par Mialee », « potion »…), pour l'explication.
    pub source: String,
    /// Le personnage (ou l'entité) qui porte l'effet.
    pub proprietaire: String,
    pub contenu: T,
    pub duree: Duree,
    pub debut: Instant,
    pub echeance: Echeance,
}

impl<T> Temporise<T> {
    /// Rounds restants à `maintenant`, si l'effet a une échéance.
    pub fn restant(&self, maintenant: Instant) -> Option<i64> {
        match self.echeance {
            Echeance::A(e) => Some(e.ecart(maintenant)),
            Echeance::Indeterminee => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum RaisonFin {
    /// L'échéance est atteinte.
    Echeance,
    /// Retiré à la main, déchargé, tour passé, concentration rompue…
    Retire,
    /// Dissipé par la conversion de fin de combat (§1.2).
    Dissipe,
}

/// Un effet terminé, gardé pour l'alerte d'expiration, la contestation et le recul d'horloge.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Termine<T> {
    pub effet: Temporise<T>,
    pub raison: RaisonFin,
    pub a: Instant,
    /// Le joueur a confirmé l'alerte.
    pub vue: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dissipation {
    pub id: IdEffet,
    pub nom: String,
    pub proprietaire: String,
    pub source: String,
    pub duree: Duree,
    /// Rounds qui restaient et sont perdus (`None` si l'effet n'avait pas d'échéance chiffrée).
    pub perdu: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arrondi {
    pub id: IdEffet,
    pub nom: String,
    pub proprietaire: String,
    pub avant: i64,
    pub apres: i64,
}

/// Ce que la conversion de fin de combat a changé, à montrer au joueur avant validation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BilanFinDeCombat {
    pub dissipes: Vec<Dissipation>,
    pub arrondis: Vec<Arrondi>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ApercuRecul {
    /// Effets expirés dans l'intervalle, qu'on peut réactiver (décochés par défaut).
    pub reactivables: Vec<IdEffet>,
    /// Effets créés après le nouvel instant : retirés si le joueur confirme.
    pub crees: Vec<IdEffet>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ensemble<T> {
    pub actifs: Vec<Temporise<T>>,
    pub termines: Vec<Termine<T>>,
    prochain_id: IdEffet,
}

impl<T> Default for Ensemble<T> {
    fn default() -> Self {
        Ensemble {
            actifs: Vec::new(),
            termines: Vec::new(),
            prochain_id: 1,
        }
    }
}

/// Minutes entières les plus proches d'un reliquat en rounds, au moins 1 (point 1 validé :
/// « l'entier le plus proche, minimum 1 » ; 5 rounds = 30 s arrondis vers le haut).
pub fn minutes_arrondies(rounds_restants: i64) -> i64 {
    ((rounds_restants + 5) / 10).max(1)
}

impl<T: Clone> Ensemble<T> {
    pub fn nouveau() -> Self {
        Self::default()
    }

    pub fn ajouter(
        &mut self,
        nom: &str,
        source: &str,
        proprietaire: &str,
        contenu: T,
        duree: Duree,
        debut: Instant,
    ) -> Result<IdEffet, ErreurTemps> {
        if duree == Duree::Instantanee {
            return Err(ErreurTemps::DureeInstantanee);
        }
        let id = self.prochain_id;
        self.prochain_id += 1;
        self.actifs.push(Temporise {
            id,
            nom: nom.to_string(),
            source: source.to_string(),
            proprietaire: proprietaire.to_string(),
            echeance: duree.echeance(debut),
            contenu,
            duree,
            debut,
        });
        Ok(id)
    }

    pub fn actif(&self, id: IdEffet) -> Option<&Temporise<T>> {
        self.actifs.iter().find(|e| e.id == id)
    }

    fn terminer(&mut self, indice: usize, raison: RaisonFin, a: Instant) {
        let effet = self.actifs.remove(indice);
        self.termines.push(Termine {
            effet,
            raison,
            a,
            vue: false,
        });
    }

    /// Retire un effet à la main (concentration rompue, charge utilisée, soin…).
    pub fn retirer(&mut self, id: IdEffet, a: Instant) -> Result<(), ErreurTemps> {
        let i = self
            .actifs
            .iter()
            .position(|e| e.id == id)
            .ok_or(ErreurTemps::EffetInconnu(id))?;
        self.terminer(i, RaisonFin::Retire, a);
        Ok(())
    }

    /// Fait expirer les effets dont l'échéance est atteinte à `maintenant`.
    /// Renvoie leurs identifiants dans l'ordre d'expiration. Le résultat ne dépend pas de la
    /// façon dont on a découpé le temps (un seul grand pas ou plusieurs petits).
    pub fn avancer_a(&mut self, maintenant: Instant) -> Vec<IdEffet> {
        let mut a_faire: Vec<(Instant, IdEffet)> = self
            .actifs
            .iter()
            .filter_map(|e| match e.echeance {
                Echeance::A(fin) if fin <= maintenant => Some((fin, e.id)),
                _ => None,
            })
            .collect();
        a_faire.sort();
        for (fin, id) in &a_faire {
            if let Some(i) = self.actifs.iter().position(|e| e.id == *id) {
                self.terminer(i, RaisonFin::Echeance, *fin);
            }
        }
        a_faire.into_iter().map(|(_, id)| id).collect()
    }

    /// Alertes d'expiration (ou de dissipation) pas encore confirmées.
    pub fn alertes(&self) -> Vec<&Termine<T>> {
        self.termines.iter().filter(|t| !t.vue).collect()
    }

    pub fn confirmer(&mut self, id: IdEffet) -> Result<(), ErreurTemps> {
        let t = self
            .termines
            .iter_mut()
            .find(|t| t.effet.id == id)
            .ok_or(ErreurTemps::EffetInconnu(id))?;
        t.vue = true;
        Ok(())
    }

    /// Repousse une expiration (erreur de saisie) ou conteste une dissipation : l'effet revient
    /// avec la durée saisie, comptée à partir de `maintenant`.
    pub fn retablir(
        &mut self,
        id: IdEffet,
        duree: Duree,
        maintenant: Instant,
    ) -> Result<(), ErreurTemps> {
        if duree == Duree::Instantanee {
            return Err(ErreurTemps::DureeInstantanee);
        }
        let i = self
            .termines
            .iter()
            .position(|t| t.effet.id == id && t.raison != RaisonFin::Retire)
            .ok_or(ErreurTemps::EffetInconnu(id))?;
        let mut effet = self.termines.remove(i).effet;
        effet.echeance = duree.echeance(maintenant);
        effet.duree = duree;
        self.actifs.push(effet);
        Ok(())
    }

    /// Fin des effets « jusqu'au début/à la fin du prochain tour » du propriétaire indiqué.
    pub fn terminer_pour_tour(
        &mut self,
        proprietaire: &str,
        moment: MomentDuTour,
        a: Instant,
    ) -> Vec<IdEffet> {
        let ids: Vec<IdEffet> = self
            .actifs
            .iter()
            .filter(|e| {
                e.proprietaire == proprietaire && e.duree == Duree::JusquauProchainTour(moment)
            })
            .map(|e| e.id)
            .collect();
        for id in &ids {
            let _ = self.retirer(*id, a);
        }
        ids
    }

    /// Conversion de fin de combat (§1.2) à `maintenant` :
    /// - durée en rounds ou « jusqu'à… » : dissipée ;
    /// - durée en minutes : le reliquat est arrondi à la minute la plus proche (au moins 1) ;
    /// - heures, jours, permanent, concentration, décharge : inchangés.
    ///
    /// Les effets déjà expirés pendant le combat l'ont été par `avancer_a`.
    pub fn fin_de_combat(&mut self, maintenant: Instant) -> BilanFinDeCombat {
        self.avancer_a(maintenant);
        let mut bilan = BilanFinDeCombat::default();
        let mut i = 0;
        while i < self.actifs.len() {
            let e = &self.actifs[i];
            if e.duree.dissipee_en_fin_de_combat() {
                bilan.dissipes.push(Dissipation {
                    id: e.id,
                    nom: e.nom.clone(),
                    proprietaire: e.proprietaire.clone(),
                    source: e.source.clone(),
                    duree: e.duree.clone(),
                    perdu: e.restant(maintenant),
                });
                self.terminer(i, RaisonFin::Dissipe, maintenant);
                continue;
            }
            if let (Duree::Minutes(_), Echeance::A(fin)) = (&e.duree, e.echeance) {
                let avant = fin.ecart(maintenant);
                let apres = minutes_arrondies(avant) * 10;
                if apres != avant {
                    bilan.arrondis.push(Arrondi {
                        id: e.id,
                        nom: e.nom.clone(),
                        proprietaire: e.proprietaire.clone(),
                        avant,
                        apres,
                    });
                    self.actifs[i].echeance = Echeance::A(maintenant.plus(apres));
                }
            }
            i += 1;
        }
        bilan
    }

    /// Ce qu'un recul d'horloge à `nouvel` toucherait (rien n'est modifié).
    pub fn apercu_recul(&self, nouvel: Instant) -> ApercuRecul {
        let reactivables = self
            .termines
            .iter()
            .filter(|t| t.raison == RaisonFin::Echeance && t.a > nouvel && t.effet.debut <= nouvel)
            .map(|t| t.effet.id)
            .collect();
        let crees = self
            .actifs
            .iter()
            .map(|e| (e.debut, e.id))
            .chain(self.termines.iter().map(|t| (t.effet.debut, t.effet.id)))
            .filter(|(d, _)| *d > nouvel)
            .map(|(_, id)| id)
            .collect();
        ApercuRecul {
            reactivables,
            crees,
        }
    }

    /// Applique un recul : un effet expiré reste expiré sauf s'il est dans `reactiver` ;
    /// les effets créés dans l'intervalle ne sont retirés que si `retirer_crees` (confirmation).
    pub fn appliquer_recul(
        &mut self,
        nouvel: Instant,
        reactiver: &[IdEffet],
        retirer_crees: bool,
    ) -> Result<(), ErreurTemps> {
        let apercu = self.apercu_recul(nouvel);
        if let Some(id) = reactiver
            .iter()
            .find(|id| !apercu.reactivables.contains(id))
        {
            return Err(ErreurTemps::EffetInconnu(*id));
        }
        if !apercu.crees.is_empty() && !retirer_crees {
            return Err(ErreurTemps::ConfirmationRequise);
        }
        self.actifs.retain(|e| !apercu.crees.contains(&e.id));
        self.termines
            .retain(|t| !apercu.crees.contains(&t.effet.id));
        for id in reactiver {
            if let Some(i) = self.termines.iter().position(|t| t.effet.id == *id) {
                let effet = self.termines.remove(i).effet;
                self.actifs.push(effet);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ens() -> Ensemble<&'static str> {
        Ensemble::nouveau()
    }

    #[test]
    fn expiration_absolue_et_alerte() {
        let mut e = ens();
        let t0 = Instant(1000);
        let id = e
            .ajouter("Bénédiction", "sort", "Mialee", "x", Duree::Minutes(1), t0)
            .unwrap();
        assert!(e.avancer_a(t0.plus(9)).is_empty());
        assert_eq!(e.avancer_a(t0.plus(10)), vec![id]);
        assert_eq!(e.alertes().len(), 1);
        assert_eq!(e.alertes()[0].a, t0.plus(10));
        e.confirmer(id).unwrap();
        assert!(e.alertes().is_empty());
    }

    #[test]
    fn repousser_remet_l_effet() {
        let mut e = ens();
        let id = e
            .ajouter("Hâte", "s", "A", "x", Duree::Rounds(2), Instant(0))
            .unwrap();
        e.avancer_a(Instant(5));
        e.retablir(id, Duree::Rounds(3), Instant(5)).unwrap();
        assert_eq!(e.actif(id).unwrap().echeance, Echeance::A(Instant(8)));
        assert!(e.retablir(id, Duree::Rounds(1), Instant(5)).is_err());
    }

    #[test]
    fn instantanee_refusee() {
        assert_eq!(
            ens().ajouter("x", "s", "A", "x", Duree::Instantanee, Instant(0)),
            Err(ErreurTemps::DureeInstantanee)
        );
    }

    #[test]
    fn fin_de_combat_selon_la_duree() {
        let mut e = ens();
        let t0 = Instant(0);
        let r = e
            .ajouter("Rounds", "s", "A", "x", Duree::Rounds(10), t0)
            .unwrap();
        let tour = e
            .ajouter(
                "Tour",
                "s",
                "A",
                "x",
                Duree::JusquauProchainTour(MomentDuTour::Fin),
                t0,
            )
            .unwrap();
        let m = e
            .ajouter("Min", "s", "A", "x", Duree::Minutes(1), t0)
            .unwrap();
        let h = e
            .ajouter("Heure", "s", "A", "x", Duree::Heures(1), t0)
            .unwrap();
        let p = e
            .ajouter("Perm", "s", "A", "x", Duree::Permanente, t0)
            .unwrap();
        // Combat de 3 rounds : il reste 7 rounds à « Min » (42 s) → arrondi à 1 min (minimum 1).
        let bilan = e.fin_de_combat(t0.plus(3));
        let ids: Vec<_> = bilan.dissipes.iter().map(|d| d.id).collect();
        assert_eq!(ids, vec![r, tour]);
        assert_eq!(bilan.dissipes[0].perdu, Some(7));
        assert_eq!(bilan.dissipes[1].perdu, None);
        assert_eq!(bilan.arrondis.len(), 1);
        assert_eq!(bilan.arrondis[0].id, m);
        assert_eq!((bilan.arrondis[0].avant, bilan.arrondis[0].apres), (7, 10));
        assert_eq!(e.actif(m).unwrap().echeance, Echeance::A(t0.plus(13)));
        assert_eq!(e.actif(h).unwrap().echeance, Echeance::A(Instant(600)));
        assert!(e.actif(p).is_some());
        assert_eq!(e.alertes().len(), 2);
    }

    #[test]
    fn arrondi_a_la_minute_la_plus_proche() {
        assert_eq!(minutes_arrondies(74), 7); // 7 min 24 s
        assert_eq!(minutes_arrondies(75), 8); // 7 min 30 s
        assert_eq!(minutes_arrondies(1), 1);
        assert_eq!(minutes_arrondies(4), 1);
        assert_eq!(minutes_arrondies(10), 1);
        assert_eq!(minutes_arrondies(16), 2);
    }

    #[test]
    fn effet_expire_pendant_le_combat_n_est_pas_dissipe() {
        let mut e = ens();
        let id = e
            .ajouter("Court", "s", "A", "x", Duree::Rounds(2), Instant(0))
            .unwrap();
        e.avancer_a(Instant(2));
        let bilan = e.fin_de_combat(Instant(5));
        assert!(bilan.dissipes.is_empty());
        assert_eq!(e.termines[0].raison, RaisonFin::Echeance);
        assert_eq!(e.termines[0].effet.id, id);
    }

    #[test]
    fn contester_une_dissipation() {
        let mut e = ens();
        let id = e
            .ajouter("Soutenu", "s", "A", "x", Duree::Rounds(10), Instant(0))
            .unwrap();
        e.fin_de_combat(Instant(3));
        assert!(e.actif(id).is_none());
        e.retablir(id, Duree::Minutes(1), Instant(3)).unwrap();
        assert_eq!(e.actif(id).unwrap().echeance, Echeance::A(Instant(13)));
    }

    #[test]
    fn fin_de_tour() {
        let mut e = ens();
        let a = e
            .ajouter(
                "Bouclier",
                "s",
                "A",
                "x",
                Duree::JusquauProchainTour(MomentDuTour::Debut),
                Instant(0),
            )
            .unwrap();
        e.ajouter(
            "Autre",
            "s",
            "B",
            "x",
            Duree::JusquauProchainTour(MomentDuTour::Debut),
            Instant(0),
        )
        .unwrap();
        assert_eq!(
            e.terminer_pour_tour("A", MomentDuTour::Fin, Instant(1)),
            Vec::<IdEffet>::new()
        );
        assert_eq!(
            e.terminer_pour_tour("A", MomentDuTour::Debut, Instant(1)),
            vec![a]
        );
        assert_eq!(e.actifs.len(), 1);
    }

    #[test]
    fn recul_ne_ressuscite_pas_sans_confirmation() {
        let mut e = ens();
        let ancien = e
            .ajouter("Ancien", "s", "A", "x", Duree::Minutes(1), Instant(0))
            .unwrap();
        e.avancer_a(Instant(50));
        let nouveau = e
            .ajouter("Nouveau", "s", "A", "x", Duree::Heures(1), Instant(40))
            .unwrap();
        let ap = e.apercu_recul(Instant(5));
        assert_eq!(ap.reactivables, vec![ancien]);
        assert_eq!(ap.crees, vec![nouveau]);
        // sans confirmation pour les effets créés : refus, rien n'a changé
        assert_eq!(
            e.appliquer_recul(Instant(5), &[], false),
            Err(ErreurTemps::ConfirmationRequise)
        );
        assert!(e.actif(nouveau).is_some());
        // avec confirmation, l'ancien reste expiré s'il n'est pas coché
        e.appliquer_recul(Instant(5), &[], true).unwrap();
        assert!(e.actif(ancien).is_none() && e.actif(nouveau).is_none());
        assert_eq!(e.termines.len(), 1);
    }

    #[test]
    fn recul_avec_reactivation() {
        let mut e = ens();
        let id = e
            .ajouter("Ancien", "s", "A", "x", Duree::Minutes(1), Instant(0))
            .unwrap();
        e.avancer_a(Instant(50));
        e.appliquer_recul(Instant(5), &[id], true).unwrap();
        assert_eq!(e.actif(id).unwrap().echeance, Echeance::A(Instant(10)));
        assert!(e.alertes().is_empty());
        assert!(e.appliquer_recul(Instant(5), &[999], true).is_err());
    }
}
