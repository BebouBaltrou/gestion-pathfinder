//! Utilisation d'objets en combat : potions, parchemins, baguettes, bâtons, objets à charges.
//!
//! Règles du Core relevées sur Archives of Nethys (voir `docs/POINTS_A_CONFIRMER_OBJETS.md`) :
//! potion p. 477, parchemin p. 490, bâton p. 491, baguette p. 496, activation p. 458.

use crate::economie::TypeAction;
use crate::etat::{ActionCombat, ErreurCombat, EtatCombat, Phase};
use crate::sorts::{composantes, referentiel, type_d_action};
use aventure::{ContextePj, ErreurAventure, EvenementAventure, GenreObjet, IdObjet, IdPj};
use temps::Duree;

/// Utilisation d'un objet : tout ce que le joueur choisit ou saisit.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParamsObjet {
    pub pj: IdPj,
    pub objet: IdObjet,
    /// Personnages joués qui reçoivent l'effet (une potion : le buveur seul).
    #[cfg_attr(feature = "serde", serde(default))]
    pub cibles: Vec<IdPj>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub duree: Option<Duree>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub choix_energie: Option<String>,
    /// Bâton : nombre de charges dépensées (un sort peut en coûter plusieurs).
    #[cfg_attr(feature = "serde", serde(default))]
    pub charges: u32,
    /// Parchemin d'un niveau de lanceur plus élevé que le sien : d20 du test de niveau de lanceur.
    #[cfg_attr(feature = "serde", serde(default))]
    pub niveau_de_lanceur_d20: Option<i32>,
    /// Test de niveau de lanceur raté : d20 du test de Sagesse DD 5 pour éviter l'accident.
    #[cfg_attr(feature = "serde", serde(default))]
    pub accident_d20: Option<i32>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub echec_arcanique_d100: Option<i32>,
    /// Objet sans sort : type d'action choisi (action simple par défaut).
    #[cfg_attr(feature = "serde", serde(default))]
    pub type_action: Option<TypeAction>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub resultat_de: Option<i32>,
}

fn invalide<T>(m: impl Into<String>) -> Result<T, ErreurCombat> {
    Err(ErreurCombat::Invalide(m.into()))
}

impl EtatCombat {
    pub(crate) fn utiliser_objet(
        &mut self,
        a: &ParamsObjet,
        ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
    ) -> Result<(), ErreurCombat> {
        if self.phase != Phase::EnCours {
            return invalide("le combat n'est pas commencé");
        }
        self.verifier_engage(a.pj)?;
        let c = ctx(a.pj).ok_or(ErreurAventure::PjInconnu(a.pj))?;
        let objet = self
            .aventure
            .pjs
            .get(&a.pj)
            .and_then(|s| s.inventaire.get(&a.objet))
            .cloned()
            .ok_or_else(|| ErreurCombat::Invalide(format!("objet n° {} inconnu", a.objet)))?;
        let sort = match &objet.sort {
            Some(nom) => Some(
                referentiel()
                    .sort(nom)
                    .ok_or_else(|| ErreurCombat::Invalide(format!("sort inconnu : {nom}")))?,
            ),
            None => None,
        };
        let inc = c.incantation.clone();
        // Qui peut utiliser quoi, et en combien de temps
        let mut cl = objet.niveau_de_lanceur;
        let mut type_action = a.type_action.unwrap_or(TypeAction::Simple);
        let mut cibles = a.cibles.clone();
        let mut gestuel = false;
        match objet.genre {
            GenreObjet::Potion => {
                type_action = TypeAction::Simple; // boire une potion : action simple (p. 477)
                cibles = vec![a.pj]; // le buveur est le lanceur et la cible
            }
            GenreObjet::Parchemin | GenreObjet::Baguette | GenreObjet::Baton => {
                let (Some(inc), Some(s)) = (inc.as_ref(), sort) else {
                    return invalide(format!(
                        "{} ne lance pas de sorts : impossible d'utiliser {}",
                        c.nom, objet.nom
                    ));
                };
                let niveau_pour_lui = s.niveau_pour(&inc.classe_en).ok_or_else(|| {
                    ErreurCombat::Invalide(format!(
                        "{} n'est pas sur la liste de sorts de {} : {} ne peut pas s'en servir",
                        s.nom_en, c.nom, objet.nom
                    ))
                })?;
                if objet.genre == GenreObjet::Parchemin {
                    if objet.arcanique != inc.arcanique {
                        return invalide("un parchemin profane ne se lit qu'avec un lanceur profane, un parchemin divin qu'avec un lanceur divin (p. 490)");
                    }
                    if inc.score_incantation < 10 + niveau_pour_lui as i32 {
                        return invalide(format!(
                            "il faut un score de {} dans la caractéristique d'incantation pour un sort de niveau {niveau_pour_lui} (p. 490)",
                            10 + niveau_pour_lui
                        ));
                    }
                    // action simple ou temps d'incantation du sort s'il est plus long (p. 490)
                    type_action = match type_d_action(
                        s.temps_d_incantation_en.as_deref(),
                        false,
                        false,
                        false,
                    )
                    .map_err(ErreurCombat::Invalide)?
                    {
                        TypeAction::Complexe => TypeAction::Complexe,
                        _ => TypeAction::Simple,
                    };
                    gestuel = composantes(s.composantes_en.as_deref()).1;
                } else {
                    type_action = TypeAction::Simple; // déclenchement du sort : action simple
                    if objet.genre == GenreObjet::Baton {
                        cl = cl.max(inc.niveau_de_lanceur); // le porteur utilise son niveau s'il est plus haut (p. 491)
                    }
                }
            }
            GenreObjet::Munition => {
                return invalide("munitions : décompter chaque tir avec « objet utilisé » ; l'attaque se joue comme une action simple");
            }
            GenreObjet::Charges | GenreObjet::Autre => {}
        }
        // dés demandés (avant de dépenser quoi que ce soit)
        let besoin_cl = objet.genre == GenreObjet::Parchemin
            && inc
                .as_ref()
                .is_some_and(|i| i.niveau_de_lanceur < objet.niveau_de_lanceur);
        if besoin_cl && a.niveau_de_lanceur_d20.is_none() {
            return invalide(format!(
                "parchemin de niveau de lanceur {} : saisir le d20 du test de niveau de lanceur (DD {})",
                objet.niveau_de_lanceur,
                objet.niveau_de_lanceur + 1
            ));
        }
        let echec_pct = match (&inc, gestuel) {
            (Some(i), true) if i.arcanique => i.echec_arcanique_pct,
            _ => 0,
        };
        if echec_pct > 0 && a.echec_arcanique_d100.is_none() {
            return invalide(format!(
                "échec des sorts profanes : saisir le d100 (échec à {echec_pct} % ou moins)"
            ));
        }
        for (d, max, nom) in [
            (a.niveau_de_lanceur_d20, 20, "d20"),
            (a.accident_d20, 20, "d20"),
            (a.echec_arcanique_d100, 100, "d100"),
        ] {
            if d.is_some_and(|v| !(1..=max).contains(&v)) {
                return invalide(format!("{nom} : de 1 à {max}"));
            }
        }
        // économie d'actions
        self.appliquer_interne(
            &ActionCombat::Agir {
                pj: a.pj,
                type_action,
                libelle: format!("Utiliser {}", objet.nom),
                deplacement: false,
            },
            ctx,
        )?;
        // Parchemin : test de niveau de lanceur et accident éventuel
        if besoin_cl {
            let (d20, bonus) = (
                a.niveau_de_lanceur_d20.unwrap_or(0),
                inc.as_ref().map_or(0, |i| i.niveau_de_lanceur as i32),
            );
            let dd = objet.niveau_de_lanceur as i32 + 1;
            let total = d20 + bonus;
            if total >= dd {
                self.noter(format!(
                    "{} : test de niveau de lanceur {d20} + {bonus} = {total} contre DD {dd} : réussi.",
                    c.nom
                ));
            } else {
                let Some(acc) = a.accident_d20 else {
                    return invalide(format!(
                        "test de niveau de lanceur raté ({total} contre DD {dd}) : saisir le d20 du test de Sagesse DD 5 pour éviter un accident"
                    ));
                };
                let sagesse = inc.as_ref().map_or(0, |i| i.mod_sagesse);
                let evite = acc != 1 && acc + sagesse >= 5;
                if evite {
                    self.noter(format!(
                        "{} : test de niveau de lanceur raté ({total} contre DD {dd}), Sagesse {acc} + {sagesse} contre DD 5 réussi : pas d'accident, le parchemin n'est pas consommé.",
                        c.nom
                    ));
                    return Ok(());
                }
                // accident : parchemin consommé, sort perdu, conséquence à résoudre à la main
                let s = self.aventure.appliquer(
                    &EvenementAventure::ObjetUtilise {
                        pj: a.pj,
                        id: a.objet,
                        charges: 1,
                    },
                    ctx,
                )?;
                self.fusionner(s);
                self.noter(format!(
                    "{} : accident de parchemin (test de niveau de lanceur raté, Sagesse {acc} + {sagesse} contre DD 5 raté) : le sort est perdu ; résoudre l'accident à la main (p. 490).",
                    c.nom
                ));
                return Ok(());
            }
        }
        // dépense de l'objet
        let charges = match objet.genre {
            GenreObjet::Baton => a.charges.max(1),
            _ => 1,
        };
        let s = self.aventure.appliquer(
            &EvenementAventure::ObjetUtilise {
                pj: a.pj,
                id: a.objet,
                charges,
            },
            ctx,
        )?;
        self.fusionner(s);
        self.stats.entry(a.pj).or_default().objets_utilises += 1;
        if let Some(r) = a.resultat_de {
            self.noter(format!("{} : résultat du dé : {r}.", c.nom));
        }
        // échec des sorts profanes (parchemin)
        if let Some(d100) = a.echec_arcanique_d100 {
            if echec_pct > 0 {
                if d100 as u32 <= echec_pct {
                    self.noter(format!(
                        "{} : échec des sorts profanes ({d100} sur {echec_pct} %) : le sort est perdu.",
                        c.nom
                    ));
                    return Ok(());
                }
                self.noter(format!(
                    "{} : échec des sorts profanes évité ({d100} sur {echec_pct} %).",
                    c.nom
                ));
            }
        }
        if let Some(nom_sort) = &objet.sort {
            self.effet_de_sort(
                nom_sort,
                cl,
                &c.nom,
                &cibles,
                a.duree.clone(),
                a.choix_energie.as_deref(),
                false,
                ctx,
            )?;
        }
        Ok(())
    }
}
