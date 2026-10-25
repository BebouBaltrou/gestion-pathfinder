//! Page « Points à confirmer » : règles ambiguës du livre, lecture appliquée, contrôles du
//! référentiel.

use domaine::fiches::StatutPoint;
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PagePoints {
    /// Les avertissements sont masqués dans les calculs (Paramètres) mais restent listés ici.
    pub avertissements_masques: bool,
    pub controles: Vec<String>,
    pub total: u32,
    pub ouverts: u32,
    pub points: Vec<PointVue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PointVue {
    pub code: String,
    pub ouvert: bool,
    pub source: String,
    pub question: String,
    pub lecture_appliquee: String,
    pub decision: Option<String>,
}

/// `afficher_avertissements` : réglage de la campagne ouverte (vrai sans campagne).
pub fn page_points(r: &Referentiel, afficher_avertissements: bool) -> PagePoints {
    let ouverts = r
        .points_a_confirmer
        .iter()
        .filter(|p| p.statut == StatutPoint::Ouvert)
        .count() as u32;
    PagePoints {
        avertissements_masques: !afficher_avertissements,
        controles: regles::controles::controler(r)
            .iter()
            .map(|a| a.to_string())
            .collect(),
        total: r.points_a_confirmer.len() as u32,
        ouverts,
        points: r
            .points_a_confirmer
            .iter()
            .map(|p| PointVue {
                code: p.code.clone(),
                ouvert: p.statut == StatutPoint::Ouvert,
                source: r.reference(&p.source),
                question: p.question.clone(),
                lecture_appliquee: p.lecture_appliquee.clone(),
                decision: p.decision.clone(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_du_referentiel_officiel() {
        let r = regles::referentiel_officiel();
        let p = page_points(&r, true);
        assert!(p.total > 0);
        assert_eq!(p.total as usize, p.points.len());
        assert_eq!(
            p.ouverts as usize,
            p.points.iter().filter(|x| x.ouvert).count()
        );
        assert!(!p.avertissements_masques);
        assert!(page_points(&r, false).avertissements_masques);
    }
}
