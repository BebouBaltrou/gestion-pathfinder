//! Utilisation d'objets en combat : potions, parchemins, baguettes, bâtons, objets à charges.
//!
//! Règles du Core relevées sur Archives of Nethys (voir `docs/POINTS_A_CONFIRMER_OBJETS.md`) :
//! potion p. 477, parchemin p. 490, bâton p. 491, baguette p. 496, activation p. 458.
//! Les mêmes contrôles servent hors combat (`hors_combat.rs`).

use crate::economie::TypeAction;
use crate::etat::{ActionCombat, ErreurCombat, EtatCombat, Phase};
use crate::sorts::{composantes, referentiel, type_d_action};
use aventure::{
    Consommable, ContextePj, ErreurAventure, EvenementAventure, GenreObjet, IdObjet, IdPj,
    IncantationPj,
};
use fiches::catalogue::Sort;
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

pub(crate) fn invalide<T>(m: impl Into<String>) -> Result<T, ErreurCombat> {
    Err(ErreurCombat::Invalide(m.into()))
}

/// Ce qu'un objet permet de lancer : temps d'incantation, niveau de lanceur, composantes.
pub(crate) struct Lancement {
    pub type_action: TypeAction,
    pub cl: u32,
    pub cibles: Vec<IdPj>,
    pub echec_pct: u32,
}

/// Sort contenu dans l'objet, s'il y en a un (nom inconnu du catalogue : erreur).
pub(crate) fn sort_de_l_objet(objet: &Consommable) -> Result<Option<&'static Sort>, ErreurCombat> {
    match &objet.sort {
        Some(nom) => referentiel()
            .sort(nom)
            .map(Some)
            .ok_or_else(|| ErreurCombat::Invalide(format!("sort inconnu : {nom}"))),
        None => Ok(None),
    }
}

/// Qui peut utiliser quoi, et en combien de temps (règles des objets, Core p. 477 à 496).
pub(crate) fn lancement(
    objet: &Consommable,
    sort: Option<&'static Sort>,
    inc: Option<&IncantationPj>,
    a: &ParamsObjet,
    nom: &str,
) -> Result<Lancement, ErreurCombat> {
    let mut l = Lancement {
        type_action: a.type_action.unwrap_or(TypeAction::Simple),
        cl: objet.niveau_de_lanceur,
        cibles: a.cibles.clone(),
        echec_pct: 0,
    };
    let mut gestuel = false;
    match objet.genre {
        GenreObjet::Potion => {
            l.type_action = TypeAction::Simple; // boire une potion : action simple (p. 477)
            l.cibles = vec![a.pj]; // le buveur est le lanceur et la cible
        }
        GenreObjet::Parchemin | GenreObjet::Baguette | GenreObjet::Baton => {
            let (Some(inc), Some(s)) = (inc, sort) else {
                return invalide(format!(
                    "{nom} ne lance pas de sorts : impossible d'utiliser {}",
                    objet.nom
                ));
            };
            let niveau_pour_lui = s.niveau_pour(&inc.classe_en).ok_or_else(|| {
                ErreurCombat::Invalide(format!(
                    "{} n'est pas sur la liste de sorts de {nom} : {} ne peut pas s'en servir",
                    s.nom_en, objet.nom
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
                l.type_action =
                    match type_d_action(s.temps_d_incantation_en.as_deref(), false, false, false)
                        .map_err(ErreurCombat::Invalide)?
                    {
                        TypeAction::Complexe => TypeAction::Complexe,
                        _ => TypeAction::Simple,
                    };
                gestuel = composantes(s.composantes_en.as_deref()).1;
            } else {
                l.type_action = TypeAction::Simple; // déclenchement du sort : action simple
                if objet.genre == GenreObjet::Baton {
                    // le porteur utilise son niveau s'il est plus haut (p. 491)
                    l.cl = l.cl.max(inc.niveau_de_lanceur);
                }
            }
        }
        GenreObjet::Munition => {
            return invalide("munitions : décompter chaque tir avec « objet utilisé » ; l'attaque se joue comme une action simple");
        }
        GenreObjet::Charges | GenreObjet::Autre => {}
    }
    l.echec_pct = match (inc, gestuel) {
        (Some(i), true) if i.arcanique => i.echec_arcanique_pct,
        _ => 0,
    };
    Ok(l)
}

/// Saisies à fournir avant de dépenser quoi que ce soit : d20, d100, bornes.
pub(crate) fn controler_saisies(
    objet: &Consommable,
    l: &Lancement,
    inc: Option<&IncantationPj>,
    a: &ParamsObjet,
) -> Result<(), ErreurCombat> {
    if besoin_de_niveau_de_lanceur(objet, inc) && a.niveau_de_lanceur_d20.is_none() {
        return invalide(format!(
            "parchemin de niveau de lanceur {} : saisir le d20 du test de niveau de lanceur (DD {})",
            objet.niveau_de_lanceur,
            objet.niveau_de_lanceur + 1
        ));
    }
    if l.echec_pct > 0 && a.echec_arcanique_d100.is_none() {
        return invalide(format!(
            "échec des sorts profanes : saisir le d100 (échec à {} % ou moins)",
            l.echec_pct
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
    Ok(())
}

fn besoin_de_niveau_de_lanceur(objet: &Consommable, inc: Option<&IncantationPj>) -> bool {
    objet.genre == GenreObjet::Parchemin
        && inc.is_some_and(|i| i.niveau_de_lanceur < objet.niveau_de_lanceur)
}

/// Résultat du test de niveau de lanceur d'un parchemin (Core p. 490), avec sa remarque.
pub(crate) enum TestLanceur {
    /// Pas de test (le lanceur est assez fort).
    Inutile,
    Reussi(String),
    /// Test raté, accident évité : le parchemin n'est pas consommé.
    Evite(String),
    /// Test raté, accident : parchemin consommé, sort perdu.
    Accident(String),
}

pub(crate) fn test_lanceur(
    objet: &Consommable,
    inc: Option<&IncantationPj>,
    a: &ParamsObjet,
    nom: &str,
) -> Result<TestLanceur, ErreurCombat> {
    if !besoin_de_niveau_de_lanceur(objet, inc) {
        return Ok(TestLanceur::Inutile);
    }
    let d20 = a.niveau_de_lanceur_d20.unwrap_or(0);
    let bonus = inc.map_or(0, |i| i.niveau_de_lanceur as i32);
    let dd = objet.niveau_de_lanceur as i32 + 1;
    let total = d20 + bonus;
    if total >= dd {
        return Ok(TestLanceur::Reussi(format!(
            "{nom} : test de niveau de lanceur {d20} + {bonus} = {total} contre DD {dd} : réussi."
        )));
    }
    let Some(acc) = a.accident_d20 else {
        return invalide(format!(
            "test de niveau de lanceur raté ({total} contre DD {dd}) : saisir le d20 du test de Sagesse DD 5 pour éviter un accident"
        ));
    };
    let sagesse = inc.map_or(0, |i| i.mod_sagesse);
    if acc != 1 && acc + sagesse >= 5 {
        return Ok(TestLanceur::Evite(format!(
            "{nom} : test de niveau de lanceur raté ({total} contre DD {dd}), Sagesse {acc} + {sagesse} contre DD 5 réussi : pas d'accident, le parchemin n'est pas consommé."
        )));
    }
    Ok(TestLanceur::Accident(format!(
        "{nom} : accident de parchemin (test de niveau de lanceur raté, Sagesse {acc} + {sagesse} contre DD 5 raté) : le sort est perdu ; résoudre l'accident à la main (p. 490)."
    )))
}

/// Échec des sorts profanes d'un parchemin ou d'un objet : remarque et « sort perdu ».
pub(crate) fn echec_arcanique(
    d100: Option<i32>,
    echec_pct: u32,
    nom: &str,
) -> Option<(String, bool)> {
    let d100 = d100?;
    if echec_pct == 0 {
        return None;
    }
    if d100 as u32 <= echec_pct {
        return Some((
            format!(
                "{nom} : échec des sorts profanes ({d100} sur {echec_pct} %) : le sort est perdu."
            ),
            true,
        ));
    }
    Some((
        format!("{nom} : échec des sorts profanes évité ({d100} sur {echec_pct} %)."),
        false,
    ))
}

/// Charges dépensées : un bâton peut en coûter plusieurs, les autres objets une seule.
pub(crate) fn charges_a_depenser(objet: &Consommable, a: &ParamsObjet) -> u32 {
    match objet.genre {
        GenreObjet::Baton => a.charges.max(1),
        _ => 1,
    }
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
        let sort = sort_de_l_objet(&objet)?;
        let inc = c.incantation.clone();
        let l = lancement(&objet, sort, inc.as_ref(), a, &c.nom)?;
        controler_saisies(&objet, &l, inc.as_ref(), a)?;
        // le test de niveau de lanceur est calculé avant toute dépense
        let test = test_lanceur(&objet, inc.as_ref(), a, &c.nom)?;
        // économie d'actions
        self.appliquer_interne(
            &ActionCombat::Agir {
                pj: a.pj,
                type_action: l.type_action,
                libelle: format!("Utiliser {}", objet.nom),
                deplacement: false,
            },
            ctx,
        )?;
        match test {
            TestLanceur::Inutile => {}
            TestLanceur::Reussi(n) => self.noter(n),
            TestLanceur::Evite(n) => {
                self.noter(n);
                return Ok(());
            }
            TestLanceur::Accident(n) => {
                let s = self.aventure.appliquer(
                    &EvenementAventure::ObjetUtilise {
                        pj: a.pj,
                        id: a.objet,
                        charges: 1,
                    },
                    ctx,
                )?;
                self.fusionner(s);
                self.noter(n);
                return Ok(());
            }
        }
        // dépense de l'objet
        let s = self.aventure.appliquer(
            &EvenementAventure::ObjetUtilise {
                pj: a.pj,
                id: a.objet,
                charges: charges_a_depenser(&objet, a),
            },
            ctx,
        )?;
        self.fusionner(s);
        self.stats.entry(a.pj).or_default().objets_utilises += 1;
        if let Some(r) = a.resultat_de {
            self.noter(format!("{} : résultat du dé : {r}.", c.nom));
        }
        if let Some((n, perdu)) = echec_arcanique(a.echec_arcanique_d100, l.echec_pct, &c.nom) {
            self.noter(n);
            if perdu {
                return Ok(());
            }
        }
        if let Some(nom_sort) = &objet.sort {
            self.effet_de_sort(
                nom_sort,
                l.cl,
                &c.nom,
                &l.cibles,
                a.duree.clone(),
                a.choix_energie.as_deref(),
                false,
                ctx,
            )?;
        }
        Ok(())
    }
}
