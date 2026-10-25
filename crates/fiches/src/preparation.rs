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
    for sn in 0..=9u32 {
        let normaux = sorts
            .iter()
            .filter(|s| s.niveau == sn && !s.domaine)
            .count() as u32;
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
        if s.domaine {
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
