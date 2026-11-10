//! Objets magiques utilisés hors combat (Core) : potions, parchemins, baguettes, bâtons, objets à
//! charges. Mêmes règles que `objets.rs` (combat), sans économie d'actions ni ordre de tour.
//!
//! Rien n'est écrit ici : la fonction renvoie les événements d'aventure à enregistrer d'un bloc
//! (`Evenement::Groupe`). Soins et dégâts : le joueur saisit le total des dés à la main.

use crate::etat::ErreurCombat;
use crate::objets::ParamsObjet;
use crate::sorts::ParamsLancer;
use crate::sorts::{composantes, duree_suggeree, effet_du_sort, nom_fr, referentiel};
use crate::stats_sorts::statistiques;
use aventure::{
    ContextePj, ErreurAventure, EtatAventure, EvenementAventure, GenreObjet, IdPj, Protection,
    TYPES_D_ENERGIE,
};
use fiches::Effet;
use temps::Duree;

/// Ce qu'une utilisation hors combat produit : événements à enregistrer, et notes pour le journal.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SortieObjet {
    pub evenements: Vec<EvenementAventure>,
    pub notes: Vec<String>,
}

fn invalide<T>(m: impl Into<String>) -> Result<T, ErreurCombat> {
    Err(ErreurCombat::Invalide(m.into()))
}

fn referentiel_sort(nom: &str) -> Result<&'static fiches::catalogue::Sort, ErreurCombat> {
    referentiel()
        .sort(nom)
        .ok_or_else(|| ErreurCombat::Invalide(format!("sort inconnu : {nom}")))
}

/// Événements d'une utilisation hors combat. `a.niveau_de_lanceur` remplace le niveau de l'objet
/// s'il est donné. `a.cibles` : bénéficiaires (une potion : le buveur seul).
pub fn evenements_objet_hors_combat(
    etat: &EtatAventure,
    a: &ParamsObjet,
    ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
) -> Result<SortieObjet, ErreurCombat> {
    let c = ctx(a.pj).ok_or(ErreurAventure::PjInconnu(a.pj))?;
    let objet = etat
        .pjs
        .get(&a.pj)
        .and_then(|s| s.inventaire.get(&a.objet))
        .cloned()
        .ok_or_else(|| ErreurCombat::Invalide(format!("objet n° {} inconnu", a.objet)))?;
    let mut sortie = SortieObjet::default();
    let depense = |charges: u32| EvenementAventure::ObjetUtilise {
        pj: a.pj,
        id: a.objet,
        charges,
    };
    if objet.genre == GenreObjet::Munition {
        return invalide(
            "munitions : décompter chaque tir avec « objet utilisé » (hors combat, tir par tir)",
        );
    }
    let Some(nom_sort) = objet.sort.clone() else {
        // objet sans sort (ou objet à charges sans effet automatique) : une utilisation
        let charges = if objet.genre == GenreObjet::Baton {
            a.charges.max(1)
        } else {
            1
        };
        sortie.evenements.push(depense(charges));
        sortie.notes.push(format!(
            "{} : utilisé ; appliquer l'effet à la main.",
            objet.nom
        ));
        return Ok(sortie);
    };
    let sort = referentiel_sort(&nom_sort)?;
    let inc = c.incantation.clone();
    let mut cl = a.niveau_de_lanceur.unwrap_or(objet.niveau_de_lanceur);
    let mut cibles = match objet.genre {
        GenreObjet::Potion => vec![a.pj],
        _ => a.cibles.clone(),
    };
    cibles.dedup();
    let mut gestuel = false;
    let mut besoin_cl = false;
    match objet.genre {
        GenreObjet::Potion | GenreObjet::Charges | GenreObjet::Autre => {}
        GenreObjet::Parchemin | GenreObjet::Baguette | GenreObjet::Baton => {
            let Some(i) = inc.as_ref() else {
                return invalide(format!(
                    "{} ne lance pas de sorts : impossible d'utiliser {}",
                    c.nom, objet.nom
                ));
            };
            let niveau_pour_lui = sort.niveau_pour(&i.classe_en).ok_or_else(|| {
                ErreurCombat::Invalide(format!(
                    "{} n'est pas sur la liste de sorts de {} : {} ne peut pas s'en servir",
                    sort.nom_en, c.nom, objet.nom
                ))
            })?;
            if objet.genre == GenreObjet::Parchemin {
                if objet.arcanique != i.arcanique {
                    return invalide("un parchemin profane ne se lit qu'avec un lanceur profane, un parchemin divin qu'avec un lanceur divin (p. 490)");
                }
                if i.score_incantation < 10 + niveau_pour_lui as i32 {
                    return invalide(format!(
                        "il faut un score de {} dans la caractéristique d'incantation pour un sort de niveau {niveau_pour_lui} (p. 490)",
                        10 + niveau_pour_lui
                    ));
                }
                gestuel = composantes(sort.composantes_en.as_deref()).1;
                besoin_cl = i.niveau_de_lanceur < cl;
            } else if objet.genre == GenreObjet::Baton {
                cl = cl.max(i.niveau_de_lanceur); // le porteur utilise son niveau s'il est plus haut (p. 491)
            }
        }
        GenreObjet::Munition => {}
    }
    // Saisies du joueur : vérifications avant toute dépense
    for (d, max, nom) in [
        (a.niveau_de_lanceur_d20, 20, "d20"),
        (a.accident_d20, 20, "d20"),
        (a.echec_arcanique_d100, 100, "d100"),
    ] {
        if d.is_some_and(|v| !(1..=max).contains(&v)) {
            return invalide(format!("{nom} : de 1 à {max}"));
        }
    }
    let echec_pct = match (&inc, gestuel) {
        (Some(i), true) if i.arcanique => i.echec_arcanique_pct,
        _ => 0,
    };
    if besoin_cl && a.niveau_de_lanceur_d20.is_none() {
        return invalide(format!(
            "parchemin de niveau de lanceur {cl} : saisir le d20 du test de niveau de lanceur (DD {})",
            cl + 1
        ));
    }
    if echec_pct > 0 && a.echec_arcanique_d100.is_none() {
        return invalide(format!(
            "échec des sorts profanes : saisir le d100 (échec à {echec_pct} % ou moins)"
        ));
    }
    // Test de niveau de lanceur d'un parchemin (p. 490)
    if besoin_cl {
        let bonus = inc.as_ref().map_or(0, |i| i.niveau_de_lanceur as i32);
        let d20 = a.niveau_de_lanceur_d20.unwrap_or(0);
        let dd = cl as i32 + 1;
        let total = d20 + bonus;
        if total >= dd {
            sortie.notes.push(format!(
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
            if acc != 1 && acc + sagesse >= 5 {
                sortie.notes.push(format!(
                    "{} : test de niveau de lanceur raté ({total} contre DD {dd}), Sagesse {acc} + {sagesse} contre DD 5 réussi : pas d'accident, le parchemin n'est pas consommé.",
                    c.nom
                ));
                return Ok(sortie);
            }
            sortie.evenements.push(depense(1));
            sortie.notes.push(format!(
                "{} : accident de parchemin : le sort est perdu ; résoudre l'accident à la main (p. 490).",
                c.nom
            ));
            return Ok(sortie);
        }
    }
    // Dépense de l'objet
    let charges = if objet.genre == GenreObjet::Baton {
        a.charges.max(1)
    } else {
        1
    };
    sortie.evenements.push(depense(charges));
    // Échec des sorts profanes (parchemin, geste) : le sort est perdu
    if echec_pct > 0 {
        let d100 = a.echec_arcanique_d100.unwrap_or(0) as u32;
        if d100 <= echec_pct {
            sortie.notes.push(format!(
                "{} : échec des sorts profanes ({d100} sur {echec_pct} %) : le sort est perdu.",
                c.nom
            ));
            return Ok(sortie);
        }
        sortie.notes.push(format!(
            "{} : échec des sorts profanes évité ({d100} sur {echec_pct} %).",
            c.nom
        ));
    }
    // Effets : soins ou dégâts (dés saisis), puis effets automatiques (durée, énergie)
    if !cibles.is_empty() {
        let soins = soins_ou_degats(sort, cl, &cibles, a.resultat_de, &mut sortie.notes)?;
        sortie.evenements.extend(soins);
        sortie.evenements.extend(effets_du_sort(
            sort,
            cl,
            &c.nom,
            &cibles,
            a.duree.clone(),
            a.choix_energie.as_deref(),
            &mut sortie.notes,
        )?);
    }
    Ok(sortie)
}

/// Soins ou dégâts d'un sort, d'après le total des dés saisi à la main. Sans formule simple,
/// sans total de dés, ou pour un sort à projectiles : rien n'est appliqué, une note le dit.
fn soins_ou_degats(
    sort: &fiches::catalogue::Sort,
    cl: u32,
    cibles: &[IdPj],
    de: Option<i32>,
    notes: &mut Vec<String>,
) -> Result<Vec<EvenementAventure>, ErreurCombat> {
    let st = statistiques(sort, cl);
    let Some(ligne) = st.degats.first() else {
        return Ok(vec![]);
    };
    let nom = nom_fr(&sort.nom_en);
    if st.nombre.is_some() {
        notes.push(format!(
            "{nom} : plusieurs projectiles ou rayons : dégâts à appliquer à la main."
        ));
        return Ok(vec![]);
    }
    let Some((n, faces, bonus)) = formule_simple(&ligne.formule) else {
        notes.push(format!(
            "{nom} : formule « {} » non reconnue : à appliquer à la main.",
            ligne.formule
        ));
        return Ok(vec![]);
    };
    let Some(total) = de else {
        notes.push(format!(
            "{nom} : {} ({}) : saisir le total des dés pour l'appliquer.",
            ligne.libelle, ligne.formule
        ));
        return Ok(vec![]);
    };
    if total < n as i32 || total > (n * faces) as i32 {
        return invalide(format!(
            "{nom} : total des dés de {n} à {} (formule {})",
            n * faces,
            ligne.formule
        ));
    }
    let montant = total + bonus;
    let soin = ligne.libelle == "Soins";
    let mut evenements = Vec::new();
    for cible in cibles {
        evenements.push(if soin {
            EvenementAventure::Soin {
                pj: *cible,
                nombre: montant,
            }
        } else {
            EvenementAventure::Degats {
                pj: *cible,
                nombre: montant,
                letaux: true,
            }
        });
    }
    let effet = if soin {
        format!("{montant} PV soignés")
    } else {
        format!("{montant} dégât(s), type « {} »", ligne.libelle)
    };
    let a_valider = if ligne.a_confirmer {
        " (lecture à valider)"
    } else {
        ""
    };
    notes.push(format!(
        "{nom} : dés {total} + {bonus} = {effet}{a_valider}."
    ));
    Ok(evenements)
}

/// Effets automatiques du sort (bonus, états, protections) pour chaque cible.
fn effets_du_sort(
    sort: &fiches::catalogue::Sort,
    cl: u32,
    lanceur: &str,
    cibles: &[IdPj],
    duree: Option<Duree>,
    choix_energie: Option<&str>,
    notes: &mut Vec<String>,
) -> Result<Vec<EvenementAventure>, ErreurCombat> {
    let nom_sort = &sort.nom_en;
    let Some(e) = effet_du_sort(nom_sort) else {
        notes.push(format!(
            "{} n'a pas d'effet automatique : l'appliquer à la main.",
            nom_fr(nom_sort)
        ));
        return Ok(vec![]);
    };
    let texte_duree = sort.duree_en.clone().unwrap_or_default();
    let d = duree.or_else(|| duree_suggeree(&texte_duree, cl));
    let duree = match d {
        Some(Duree::Instantanee) => None,
        autre => autre,
    };
    let duree = duree.ok_or_else(|| {
        ErreurCombat::Invalide(format!(
            "durée à saisir pour {} (« {texte_duree} » n'est pas une formule simple)",
            nom_fr(nom_sort)
        ))
    })?;
    let energie = match (e.choix_energie, choix_energie) {
        (true, Some(x)) if TYPES_D_ENERGIE.contains(&x.to_lowercase().as_str()) => {
            Some(x.to_lowercase())
        }
        (true, _) => {
            return invalide("choisir le type d'énergie : acide, froid, électricité, feu ou son")
        }
        _ => None,
    };
    let effets: Vec<Effet> = e
        .effets
        .iter()
        .map(|d| Effet::new(&d.cible, d.type_bonus, d.valeur, nom_sort))
        .collect();
    let protections: Vec<Protection> = e
        .protections
        .iter()
        .map(|pr| match pr {
            crate::sorts::ProtectionData::Reduction { valeur } => {
                Protection::Reduction { valeur: *valeur }
            }
            crate::sorts::ProtectionData::Immunite { type_degats } => Protection::Immunite {
                type_degats: type_degats.clone(),
            },
            crate::sorts::ProtectionData::Resistance { paliers } => Protection::Resistance {
                type_degats: energie.clone().unwrap_or_default(),
                valeur: paliers
                    .iter()
                    .filter(|(n, _)| *n <= cl)
                    .map(|(_, v)| *v)
                    .max()
                    .unwrap_or(0),
            },
        })
        .collect();
    let mut evenements = Vec::new();
    for cible in cibles {
        evenements.push(EvenementAventure::EffetAjoute {
            pj: *cible,
            nom: nom_fr(nom_sort),
            source: format!("sort {}, lancé par {lanceur}", nom_fr(nom_sort)),
            duree: duree.clone(),
            effets: effets.clone(),
            etats: e.etats.clone(),
            soin_par_round: 0,
            protections: protections.clone(),
        });
    }
    if let Some(n) = &e.notes {
        notes.push(format!("{} : {n}", nom_fr(nom_sort)));
    }
    Ok(evenements)
}

/// « 3d4+1 » -> (3, 4, 1) ; « 5d6 » -> (5, 6, 0).
fn formule_simple(f: &str) -> Option<(u32, u32, i32)> {
    let (n, reste) = f.trim().split_once('d')?;
    let n: u32 = n.trim().parse().ok()?;
    let (faces, bonus) = match reste.find(['+', '-']) {
        Some(i) => (&reste[..i], reste[i..].trim().parse::<i32>().ok()?),
        None => (reste, 0),
    };
    Some((n, faces.trim().parse().ok()?, bonus))
}

/// Lancer de sort hors combat : événements d'aventure, plus l'événement du journal de fiche
/// (emplacement dépensé, sort préparé marqué lancé). Tout s'enregistre d'un bloc.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SortieLancer {
    pub evenements: Vec<EvenementAventure>,
    pub fiche: Option<fiches::journal::EvenementFiche>,
    pub notes: Vec<String>,
}

/// Sort hors combat. Les emplacements et les sorts préparés restants viennent de la fiche
/// (contexte), donc rien n'est retranché deux fois : pas de dépense propre au combat.
pub fn lancer_hors_combat(
    a: &ParamsLancer,
    ctx: &dyn Fn(IdPj) -> Option<ContextePj>,
) -> Result<SortieLancer, ErreurCombat> {
    let c = ctx(a.pj).ok_or(ErreurAventure::PjInconnu(a.pj))?;
    let p = crate::etat::EtatCombat::preparer_sort(
        a.pj,
        &a.sort,
        &a.metamagies,
        a.niveau_heighten,
        ctx,
        &|_| 0,
        &|_, _| 0,
    )?;
    if p.restants == 0 {
        return invalide(format!(
            "plus d'emplacement de sort de niveau {} pour {}",
            p.niveau_emplacement, c.nom
        ));
    }
    if p.echec_pct > 0 && a.echec_arcanique_d100.is_none() {
        return invalide(format!(
            "échec des sorts profanes : saisir le d100 (échec à {} % ou moins)",
            p.echec_pct
        ));
    }
    if let Some(d) = a.echec_arcanique_d100 {
        if !(1..=100).contains(&d) {
            return invalide("d100 : de 1 à 100");
        }
    }
    let mut sortie = SortieLancer::default();
    if p.niveau_emplacement > 0 {
        sortie.fiche = Some(fiches::journal::EvenementFiche::SortLance {
            classe: p.inc.classe.clone(),
            niveau_de_sort: p.niveau_emplacement,
            sort: p.sort.nom_en.clone(),
        });
    }
    let cl = p.inc.niveau_de_lanceur;
    sortie.notes.push(format!(
        "{} lance {} : emplacement de niveau {}, DD {}, niveau de lanceur {cl}.",
        c.nom,
        nom_fr(&p.sort.nom_en),
        p.niveau_emplacement,
        p.dd
    ));
    // le sort est dépensé même s'il échoue
    if p.echec_pct > 0 {
        let d100 = a.echec_arcanique_d100.unwrap_or(0) as u32;
        if d100 <= p.echec_pct {
            sortie.notes.push(format!(
                "{} : échec des sorts profanes ({d100} sur {} %) : le sort est perdu.",
                c.nom, p.echec_pct
            ));
            return Ok(sortie);
        }
        sortie.notes.push(format!(
            "{} : échec des sorts profanes évité ({d100} sur {} %).",
            c.nom, p.echec_pct
        ));
    }
    let cibles: Vec<IdPj> = a.cibles.clone();
    if !cibles.is_empty() {
        sortie.evenements.extend(soins_ou_degats(
            p.sort,
            cl,
            &cibles,
            a.resultat_de,
            &mut sortie.notes,
        )?);
        sortie.evenements.extend(effets_du_sort(
            p.sort,
            cl,
            &c.nom,
            &cibles,
            a.duree.clone(),
            a.choix_energie.as_deref(),
            &mut sortie.notes,
        )?);
    }
    Ok(sortie)
}

#[cfg(test)]
mod tests {
    use super::formule_simple;

    #[test]
    fn formules_simples() {
        assert_eq!(formule_simple("3d4+1"), Some((3, 4, 1)));
        assert_eq!(formule_simple("5d6"), Some((5, 6, 0)));
        assert_eq!(formule_simple("1d8-2"), Some((1, 8, -2)));
        assert_eq!(formule_simple("pas une formule"), None);
    }
}
