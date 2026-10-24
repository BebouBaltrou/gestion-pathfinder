//! Mode aventure et combat dans la campagne.
//!
//! L'état d'aventure (`aventure::EtatAventure`) vit dans l'état de la campagne ; chaque action est un
//! événement du journal (`Evenement::Aventure`), donc annuler/rétablir et l'historique restent uniques.
//! Un combat travaille sur un état séparé (crate `combat`) et n'entre dans le journal que validé
//! (`Evenement::CombatValide`). Le moteur de personnage reste le seul à calculer : les valeurs dont
//! l'aventure a besoin sont lues ici dans les fiches (`contexte_aventure`).

use crate::etat::{Consequence, Erreur, Etat};
use crate::evenements::Evenement;
use crate::modele::IdPersonnage;
use crate::personnages::ROUNDS_PAR_JOUR;
use aventure::{
    ContextePj, EmplacementPj, EtatAventure, EvenementAventure, IdPj, IncantationPj, Sortie,
};
use fiches::journal::EvenementFiche;
use fiches::{Carac, Referentiel as ReferentielFiches};
use std::collections::BTreeMap;
use temps::Instant;

fn erreur(m: impl Into<String>) -> Erreur {
    Erreur::Aventure(m.into())
}

impl Etat {
    /// Ce que l'aventure lit dans la fiche d'un PJ (`None` si le PJ n'a pas de fiche valide).
    pub fn contexte_aventure(&self, id: IdPersonnage) -> Option<ContextePj> {
        let r = ReferentielFiches::partage();
        let etat = self.fiche_de(id)?;
        let v = etat.fiche.calculer(r).ok()?;
        let fiche = &etat.fiche;
        let mod_de = |c: Carac| v.modificateurs.get(&c).copied().unwrap_or(0);
        let score_de = |c: Carac| v.scores.get(&c).map_or(10, |d| d.total);
        let dommages = Carac::TOUTES
            .into_iter()
            .filter_map(|c| {
                let d = fiche.dommages.get(&c).copied().unwrap_or(0);
                (d > 0).then_some((c, d))
            })
            .collect();
        let incantation = v.incantateurs.first().and_then(|inc| {
            let classe = r.classe(&inc.classe)?;
            let def = classe.incantation.as_ref()?;
            let carac = Carac::TOUTES
                .into_iter()
                .find(|c| c.abrege().eq_ignore_ascii_case(&def.caracteristique))?;
            let restants = etat.emplacements_restants(r);
            let emplacements = inc
                .emplacements
                .iter()
                .filter(|e| e.par_jour.is_some())
                .map(|e| EmplacementPj {
                    niveau_de_sort: e.niveau_de_sort,
                    restants: restants
                        .iter()
                        .find(|(c, n, _, _)| c == &inc.classe && *n == e.niveau_de_sort)
                        .map_or(0, |(_, _, rest, _)| *rest),
                    total: e.par_jour.unwrap_or(0),
                    dd: e.dd,
                })
                .collect();
            let mut sorts_connus: Vec<String> = fiche.sorts_connus.clone();
            sorts_connus.extend(inc.sorts_de_lignage.iter().map(|(n, _)| n.clone()));
            let metamagies = fiche
                .dons_choisis()
                .into_iter()
                .filter(|d| {
                    r.don(fiches::catalogue::nom_de_base(d))
                        .is_some_and(|don| don.types_en.iter().any(|t| t == "Metamagic"))
                })
                .collect();
            Some(IncantationPj {
                classe_en: classe.nom_en.to_lowercase(),
                classe: inc.classe.clone(),
                arcanique: def.genre == "profane",
                spontane: def.mode.starts_with("spontan"),
                niveau_de_lanceur: inc.niveau_de_lanceur,
                mod_incantation: mod_de(carac),
                score_incantation: score_de(carac),
                mod_sagesse: mod_de(Carac::Sag),
                bonus_concentration: 0,
                emplacements,
                sorts_connus,
                metamagies,
                echec_arcanique_pct: v.equipement.echec_sorts_pct,
                // décision du 24/10/2026 : la métamagie sera intégrée plus tard
                metamagie_active: false,
            })
        });
        Some(ContextePj {
            nom: fiche.nom.clone(),
            niveau: v.niveau_de_personnage,
            pv_max: v.pv_max.total,
            con_score: score_de(Carac::Con),
            con_mod: mod_de(Carac::Con),
            vigueur: v.vigueur.total,
            initiative: v.initiative.total,
            dommages,
            incantation,
        })
    }

    /// Contextes de tous les PJ qui ont une fiche, indexés par numéro de PJ.
    pub fn contextes_aventure(&self) -> BTreeMap<IdPj, ContextePj> {
        self.personnages
            .keys()
            .filter_map(|id| self.contexte_aventure(*id).map(|c| (id.0, c)))
            .collect()
    }

    /// Répercute ce que l'aventure demande : événements de fiche, puis jours d'intermède.
    pub(crate) fn repercuter_sortie(
        &mut self,
        s: Sortie,
        r: &regles::Referentiel,
    ) -> Result<Vec<Consequence>, Erreur> {
        for (pj, ev) in s.fiche {
            // Retirer un état ou un effet déjà absent de la fiche n'est pas une erreur.
            let tolerant = matches!(
                ev,
                EvenementFiche::EtatRetire(_) | EvenementFiche::EffetTemporaireRetire(_)
            );
            match self.appliquer_fiche(IdPersonnage(pj), &ev) {
                Err(_) if tolerant => {}
                autre => autre?,
            }
        }
        if s.minuits > 0 {
            let jours = u32::try_from(s.minuits).map_err(|_| erreur("trop de jours d'un coup"))?;
            return self.avancer_jours(jours, r);
        }
        Ok(Vec::new())
    }

    /// Un événement de jours de calendrier passe pendant une aventure : son horloge suit.
    pub(crate) fn suivre_les_jours_dans_l_aventure(
        &mut self,
        nombre: u32,
        r: &regles::Referentiel,
    ) -> Result<(), Erreur> {
        let Some(mut av) = self.aventure.take() else {
            return Ok(());
        };
        let contextes = self.contextes_aventure();
        let ctx = |id: IdPj| contextes.get(&id).cloned();
        let ev = EvenementAventure::AvancerTemps {
            rounds: nombre as i64 * ROUNDS_PAR_JOUR as i64,
        };
        let res = av.appliquer(&ev, &ctx);
        self.aventure = Some(av);
        let mut s = res.map_err(|e| erreur(e.to_string()))?;
        // les jours de calendrier sont déjà comptés par l'appelant
        s.minuits = 0;
        self.repercuter_sortie(s, r)?;
        Ok(())
    }

    pub(crate) fn appliquer_aventure(
        &mut self,
        ev: &Evenement,
        r: &regles::Referentiel,
    ) -> Result<Vec<Consequence>, Erreur> {
        match ev {
            Evenement::AventureDemarree {
                heure,
                minute,
                survie,
                degats_massifs,
            } => {
                if self.aventure.is_some() {
                    return Err(erreur("une aventure est déjà en cours"));
                }
                let debut = Instant::depuis(self.jour as i64, *heure, *minute)
                    .ok_or_else(|| erreur("heure invalide"))?;
                let mut a = EtatAventure::nouveau(debut, *survie);
                a.degats_massifs = *degats_massifs;
                self.aventure = Some(a);
                Ok(Vec::new())
            }
            Evenement::AventureTerminee => {
                if self.aventure.take().is_none() {
                    return Err(erreur("aucune aventure en cours"));
                }
                Ok(Vec::new())
            }
            Evenement::Aventure(e) => {
                let mut av = self
                    .aventure
                    .take()
                    .ok_or_else(|| erreur("aucune aventure en cours : la démarrer d'abord"))?;
                let contextes = self.contextes_aventure();
                let ctx = |id: IdPj| contextes.get(&id).cloned();
                let res = av.appliquer(e, &ctx);
                self.aventure = Some(av);
                let s = res.map_err(|e| erreur(e.to_string()))?;
                self.repercuter_sortie(s, r)
            }
            Evenement::CombatValide {
                aventure,
                fiche,
                minuits,
                ..
            } => {
                if self.aventure.is_none() {
                    return Err(erreur("aucune aventure en cours"));
                }
                self.aventure = Some((**aventure).clone());
                self.repercuter_sortie(
                    Sortie {
                        fiche: fiche.clone(),
                        minuits: *minuits,
                        expires: Vec::new(),
                    },
                    r,
                )
            }
            _ => Ok(Vec::new()),
        }
    }
}
