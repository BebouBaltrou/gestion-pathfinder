//! Préparation des sorts des lanceurs qui préparent (prêtre, druide) : quels sorts, à quels
//! niveaux, dans quels emplacements. Les règles viennent des classes (tables de sorts par jour,
//! liste de sorts, domaines) ; rien n'est codé ici pour une classe en particulier.

use crate::divin::axes;
use crate::donnees::Referentiel;
use crate::fiche::Fiche;
use serde::{Deserialize, Serialize};

/// Un emplacement de sort préparé pour la journée.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SortPrepare {
    /// Niveau du sort (0 pour une oraison).
    pub niveau: u32,
    /// Nom du sort dans le catalogue (anglais).
    pub sort: String,
    /// Préparé dans l'emplacement de domaine.
    #[serde(default)]
    pub domaine: bool,
    /// Métamagies appliquées à la préparation (noms anglais) : le sort occupe un emplacement plus haut.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub metamagies: Vec<String>,
    /// Niveau visé, quand « Heighten Spell » figure dans `metamagies`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub niveau_heighten: Option<u32>,
}

/// Augmentation du niveau d'emplacement par métamagie (livre, données de `data/combat/metamagie.json`,
/// vérifiées par un test dans `combat`). « Heighten Spell » n'a pas d'augmentation fixe : c'est le
/// niveau visé.
pub const AUGMENTATION_METAMAGIE: &[(&str, u32)] = &[
    ("Empower Spell", 2),
    ("Enlarge Spell", 1),
    ("Extend Spell", 1),
    ("Maximize Spell", 3),
    ("Quicken Spell", 4),
    ("Silent Spell", 1),
    ("Still Spell", 1),
    ("Widen Spell", 3),
];

/// Niveau de l'emplacement occupé par un sort préparé avec ses métamagies. Erreur si une
/// métamagie n'a pas de règle ici, si Heighten est mal renseigné ou si le niveau dépasse 9.
pub fn niveau_emplacement(
    niveau: u32,
    metamagies: &[String],
    niveau_heighten: Option<u32>,
) -> Result<u32, String> {
    let mut base = niveau;
    let mut augmentation = 0;
    let heighten = metamagies.iter().any(|m| m == "Heighten Spell");
    for m in metamagies {
        if m == "Heighten Spell" {
            continue;
        }
        let (_, d) = AUGMENTATION_METAMAGIE
            .iter()
            .find(|(nom, _)| nom.eq_ignore_ascii_case(m))
            .ok_or_else(|| format!("métamagie inconnue : {m}"))?;
        augmentation += d;
    }
    if heighten {
        let n = niveau_heighten.ok_or("Heighten Spell : préciser le niveau visé")?;
        if n <= niveau || n > 9 {
            return Err("Heighten Spell : un niveau plus haut que le sort, 9 au plus".into());
        }
        base = n;
    } else if niveau_heighten.is_some() {
        return Err("niveau visé sans Heighten Spell".into());
    }
    let total = base + augmentation;
    if total > 9 {
        return Err(format!("emplacement de niveau {total} : au plus 9"));
    }
    Ok(total)
}

/// Descripteurs d'alignement qu'un lanceur ne peut pas utiliser, d'après son alignement et celui
/// de son dieu : « Un prêtre ne peut pas lancer de sorts dont l'alignement est opposé au sien ou à
/// celui de son dieu. »
pub fn descripteurs_interdits(fiche: &Fiche, r: &Referentiel) -> Vec<&'static str> {
    let mut interdits: Vec<&'static str> = Vec::new();
    let mut ajoute = |code: &str| {
        if let Some((loi, bien)) = axes(code) {
            match loi {
                0 => interdits.push("chaotic"),
                2 => interdits.push("lawful"),
                _ => {}
            }
            match bien {
                0 => interdits.push("evil"),
                2 => interdits.push("good"),
                _ => {}
            }
        }
    };
    if let Some(a) = &fiche.alignement {
        ajoute(a);
    }
    if let Some(d) = fiche.divinite.as_deref().and_then(|d| r.divinite(d)) {
        ajoute(&d.alignement);
    }
    interdits.sort_unstable();
    interdits.dedup();
    interdits
}

/// Vérifie une préparation complète pour `classe` (identifiant) ; `Err` explique le premier refus.
pub fn valider(
    fiche: &Fiche,
    r: &Referentiel,
    classe: &str,
    sorts: &[SortPrepare],
) -> Result<(), String> {
    let def = r
        .classe(classe)
        .ok_or_else(|| format!("classe inconnue : {classe}"))?;
    if !def.incantation.as_ref().is_some_and(|i| i.prepare()) {
        return Err(format!("{} ne prépare pas ses sorts", def.nom));
    }
    if fiche.niveau_de_classe(classe) == 0 {
        return Err(format!("le personnage n'a aucun niveau de {}", def.nom));
    }
    let v = fiche.calculer(r).map_err(|p| p.to_string())?;
    let inc = v
        .incantateurs
        .iter()
        .find(|i| i.classe == classe)
        .ok_or_else(|| format!("{} n'a pas d'incantation", def.nom))?;
    let cle = def.nom_en.to_lowercase();
    let interdits = descripteurs_interdits(fiche, r);
    // Sorts de domaine permis, par niveau.
    let domaine_permis: Vec<(u32, &str)> = fiche
        .domaines
        .iter()
        .filter_map(|id| r.domaine(id))
        .flat_map(|d| {
            d.sorts_de_domaine
                .iter()
                .filter_map(|s| s.nom_en.as_deref().map(|n| (s.niveau, n)))
        })
        .collect();
    // Magicien : un sort d'une école opposée coûte deux emplacements du même niveau.
    let magicien = classe == "magicien";
    let interdites = if magicien {
        crate::choix_classe::ecoles_opposees_du_catalogue(fiche)
    } else {
        Vec::new()
    };
    let double = |s: &SortPrepare| -> u32 {
        let opposee = r
            .sort(&s.sort)
            .is_some_and(|x| interdites.contains(&x.ecole.as_str()));
        if opposee {
            2
        } else {
            1
        }
    };
    // Emplacement occupé : le niveau du sort, ou plus haut avec ses métamagies (les erreurs sont
    // signalées plus bas, sort par sort).
    let slot = |s: &SortPrepare| {
        niveau_emplacement(s.niveau, &s.metamagies, s.niveau_heighten).unwrap_or(s.niveau)
    };
    for sn in 0..=9u32 {
        let normaux = sorts
            .iter()
            .filter(|s| slot(s) == sn && !s.domaine)
            .map(double)
            .sum::<u32>();
        let de_domaine = sorts.iter().filter(|s| s.niveau == sn && s.domaine).count() as u32;
        let e = inc.emplacements.iter().find(|e| e.niveau_de_sort == sn);
        let (permis, domaine) = match e {
            Some(e) if sn == 0 => (e.base.unwrap_or(0), 0),
            Some(e) => (e.par_jour.unwrap_or(0), e.domaine),
            None => (0, 0),
        };
        if normaux > permis {
            return Err(format!(
                "niveau {sn} : {normaux} sort(s) préparé(s), {permis} permis"
            ));
        }
        if de_domaine > domaine {
            return Err(format!(
                "niveau {sn} : {de_domaine} sort(s) de domaine, {domaine} emplacement(s) de domaine"
            ));
        }
    }
    for s in sorts {
        let def_sort = r
            .sort(&s.sort)
            .ok_or_else(|| format!("sort inconnu du catalogue : {}", s.sort))?;
        if s.niveau > 9 {
            return Err(format!("{} : niveau {} impossible", s.sort, s.niveau));
        }
        if !s.metamagies.is_empty() {
            if s.domaine {
                return Err(format!(
                    "{} : la métamagie ne s'applique pas à un sort de domaine",
                    s.sort
                ));
            }
            for m in &s.metamagies {
                let connue = fiche.dons_choisis().iter().any(|d| {
                    crate::catalogue::nom_de_base(d).eq_ignore_ascii_case(m)
                        && r.don(m)
                            .is_some_and(|don| don.types_en.iter().any(|t| t == "Metamagic"))
                });
                if !connue {
                    return Err(format!(
                        "{} : le personnage ne connaît pas le don {m}",
                        s.sort
                    ));
                }
            }
            niveau_emplacement(s.niveau, &s.metamagies, s.niveau_heighten)
                .map_err(|e| format!("{} : {e}", s.sort))?;
        }
        if magicien {
            // Grimoire : tours de magie d'office (sauf école interdite), autres sorts s'ils y figurent.
            if s.niveau == 0 {
                if interdites.contains(&def_sort.ecole.as_str()) {
                    return Err(format!(
                        "{} appartient à une école interdite : il n'est pas dans le grimoire",
                        s.sort
                    ));
                }
            } else if !fiche
                .grimoire
                .iter()
                .any(|g| g.eq_ignore_ascii_case(&s.sort))
            {
                return Err(format!("{} ne figure pas dans le grimoire", s.sort));
            }
            if s.domaine {
                let ecole = fiche
                    .ecole
                    .as_deref()
                    .map(crate::choix_classe::ecole_du_catalogue);
                if s.niveau == 0 || Some(def_sort.ecole.as_str()) != ecole {
                    return Err(format!(
                        "{} : l'emplacement supplémentaire sert à un sort de l'école de spécialisation",
                        s.sort
                    ));
                }
            }
            if def_sort.niveau_pour(&cle) != Some(s.niveau) {
                return Err(format!(
                    "{} n'est pas un sort de {} de niveau {}",
                    s.sort, def.nom, s.niveau
                ));
            }
        } else if s.domaine {
            if !domaine_permis
                .iter()
                .any(|(n, nom)| *n == s.niveau && nom.eq_ignore_ascii_case(&s.sort))
            {
                return Err(format!(
                    "{} n'est pas un sort de domaine de niveau {} pour ce personnage",
                    s.sort, s.niveau
                ));
            }
        } else if def_sort.niveau_pour(&cle) != Some(s.niveau) {
            return Err(format!(
                "{} n'est pas un sort de {} de niveau {}",
                s.sort, def.nom, s.niveau
            ));
        }
        if let Some(d) = def_sort
            .descripteurs
            .iter()
            .find(|d| interdits.contains(&d.to_lowercase().as_str()))
        {
            return Err(format!(
                "{} : un sort « {d} » est opposé à l'alignement du personnage ou de son dieu",
                s.sort
            ));
        }
    }
    Ok(())
}

/// Un sort de soins (énergie positive) ou de blessure (énergie négative), pour la conversion
/// spontanée d'un sort préparé.
pub fn est_sort_de_conversion(nom_en: &str, positive: bool) -> bool {
    let debut = if positive { "Cure " } else { "Inflict " };
    nom_en.starts_with(debut)
}

#[cfg(test)]
mod tests_metamagie {
    use super::niveau_emplacement;

    fn m(noms: &[&str]) -> Vec<String> {
        noms.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn augmentation_et_heighten() {
        // Empower (+2) sur un sort de niveau 1 : emplacement 3.
        assert_eq!(niveau_emplacement(1, &m(&["Empower Spell"]), None), Ok(3));
        // Heighten au niveau 4 puis Extend (+1) : emplacement 5.
        assert_eq!(
            niveau_emplacement(1, &m(&["Heighten Spell", "Extend Spell"]), Some(4)),
            Ok(5)
        );
        // Sans métamagie : le niveau du sort.
        assert_eq!(niveau_emplacement(2, &[], None), Ok(2));
    }

    #[test]
    fn refus() {
        assert!(niveau_emplacement(1, &m(&["Heighten Spell"]), None).is_err());
        assert!(niveau_emplacement(3, &m(&["Heighten Spell"]), Some(3)).is_err());
        assert!(niveau_emplacement(1, &m(&["Empower Spell"]), Some(3)).is_err());
        assert!(niveau_emplacement(7, &m(&["Maximize Spell"]), None).is_err()); // 10 > 9
        assert!(niveau_emplacement(1, &m(&["Inconnue"]), None).is_err());
    }
}
