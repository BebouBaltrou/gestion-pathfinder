//! Page Référentiel : consultation des données officielles du livre, chacune avec sa source
//! et son statut.

use crate::batiments::id_ressource;
use domaine::fiches::{ElementModele, Modele, StatutPoint, Taille, Unite};
use regles::Referentiel;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct PageReferentiel {
    pub salles: Vec<UniteVue>,
    pub equipes: Vec<UniteVue>,
    pub extensions: Vec<UniteVue>,
    pub gestionnaires: Vec<GestionnaireVue>,
    pub modeles_batiments: Vec<ModeleVue>,
    pub modeles_organisations: Vec<ModeleVue>,
    pub communautes: Vec<CommunauteVue>,
    pub capitaux: Vec<CapitalVue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct UniteVue {
    pub id: String,
    pub nom: String,
    pub recettes: String,
    pub avantages: Option<String>,
    pub cout: String,
    pub prix: String,
    pub duree: String,
    pub taille: String,
    pub page: String,
    /// « Guide de campagne p. 102 — Labyrinthe »
    pub source: String,
    pub ameliore_de: Vec<String>,
    pub ameliore_en: Vec<String>,
    /// « officiel » ou « a_confirmer ».
    pub statut: String,
    pub point: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct GestionnaireVue {
    pub nom: String,
    pub salaire: String,
    pub competences: String,
    pub page: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ModeleVue {
    pub nom: String,
    pub page: String,
    pub cout: String,
    pub prix: String,
    pub composition: String,
    /// Texte imprimé, montré seulement quand le modèle est lié à un point à confirmer.
    pub imprime: Option<String>,
    pub statut: String,
    pub point: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CommunauteVue {
    pub nom: String,
    pub population: String,
    pub limite: String,
    pub statut: String,
    pub point: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CapitalVue {
    pub ressource: String,
    pub nom: String,
    pub cout_achat: String,
    pub cout_produit: String,
}

fn statut(r: &Referentiel, pc: &Option<String>) -> String {
    match pc.as_deref().and_then(|c| r.point(c)) {
        Some(p) if p.statut == StatutPoint::Ouvert => "a_confirmer",
        _ => "officiel",
    }
    .into()
}

fn taille(t: &Option<Taille>) -> String {
    match t {
        Some(Taille::Cases { min, max }) => format!("{min}-{max} cases"),
        Some(Taille::Personnes { nombre }) => format!("{nombre} pers."),
        Some(Taille::Texte { texte }) => texte.clone(),
        None => "—".into(),
    }
}

fn unite(r: &Referentiel, u: &Unite, liste: &[Unite]) -> UniteVue {
    let noms = |ids: &[String]| -> Vec<String> {
        ids.iter()
            .map(|i| {
                liste
                    .iter()
                    .find(|x| &x.id == i)
                    .map(|x| x.nom.clone())
                    .unwrap_or(i.clone())
            })
            .collect()
    };
    UniteVue {
        id: u.id.clone(),
        nom: u.nom.clone(),
        recettes: u
            .recettes
            .as_ref()
            .map(|x| x.texte.clone())
            .unwrap_or("—".into()),
        avantages: u.avantages.clone(),
        cout: u
            .cout
            .map(|c| c.to_string())
            .unwrap_or("voir description".into()),
        prix: u.prix_po.map(|p| format!("{p} po")).unwrap_or("—".into()),
        duree: u
            .duree_jours
            .map(|d| format!("{d} j"))
            .unwrap_or("—".into()),
        taille: taille(&u.taille),
        page: u.source.page.map(|p| format!("p. {p}")).unwrap_or_default(),
        source: r.reference(&u.source),
        ameliore_de: noms(&u.amelioration_de),
        ameliore_en: noms(&u.amelioration_en),
        statut: statut(r, &u.point_a_confirmer),
        point: u.point_a_confirmer.clone(),
    }
}

fn modele(r: &Referentiel, m: &Modele, unites: &[Unite]) -> ModeleVue {
    let elements: Vec<String> = m
        .composition
        .iter()
        .map(|e| match e {
            ElementModele::Unite { id, quantite } => {
                let nom = unites
                    .iter()
                    .find(|u| &u.id == id)
                    .map(|u| u.nom.as_str())
                    .unwrap_or(id);
                format!("{quantite} × {nom}")
            }
            ElementModele::Extension { id, quantite, sur } => {
                let nom = r
                    .extension(id)
                    .map(|u| u.nom.as_str())
                    .unwrap_or(id.as_str());
                format!("{quantite} × extension {nom} ({sur})")
            }
        })
        .collect();
    ModeleVue {
        nom: m.nom.clone(),
        page: format!("p. {}", m.source.page.unwrap_or(0)),
        cout: m.cout.to_string(),
        prix: format!("{} po", m.prix_po),
        composition: elements.join(" · "),
        imprime: m
            .point_a_confirmer
            .is_some()
            .then(|| m.composition_imprimee.clone()),
        statut: statut(r, &m.point_a_confirmer),
        point: m.point_a_confirmer.clone(),
    }
}

pub fn page_referentiel(r: &Referentiel) -> PageReferentiel {
    PageReferentiel {
        salles: r.salles.iter().map(|u| unite(r, u, &r.salles)).collect(),
        equipes: r.equipes.iter().map(|u| unite(r, u, &r.equipes)).collect(),
        extensions: r
            .extensions
            .iter()
            .map(|u| unite(r, u, &r.extensions))
            .collect(),
        gestionnaires: r
            .gestionnaires
            .iter()
            .map(|g| GestionnaireVue {
                nom: g.nom.clone(),
                salaire: format!("{} po/jour", g.salaire_po_jour),
                competences: g.competences.clone(),
                page: g.source.page.map(|p| format!("p. {p}")).unwrap_or_default(),
            })
            .collect(),
        modeles_batiments: r
            .modeles_batiments
            .iter()
            .map(|m| modele(r, m, &r.salles))
            .collect(),
        modeles_organisations: r
            .modeles_organisations
            .iter()
            .map(|m| modele(r, m, &r.equipes))
            .collect(),
        communautes: r
            .communautes
            .iter()
            .map(|c| CommunauteVue {
                nom: c.nom.clone(),
                population: match c.population_max {
                    Some(m) => format!("{} à {}", c.population_min, m),
                    None => format!("plus de {}", c.population_min.saturating_sub(1)),
                },
                limite: format!("{} unités", c.limite_depense_jour),
                statut: statut(r, &c.point_a_confirmer),
                point: c.point_a_confirmer.clone(),
            })
            .collect(),
        capitaux: r
            .capitaux
            .iter()
            .map(|v| CapitalVue {
                ressource: id_ressource(v.ressource).into(),
                nom: v.ressource.to_string(),
                cout_achat: format!("{} po", v.cout_achat_po),
                cout_produit: format!("{} po", v.cout_produit_po),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_referentiel_officiel_est_complet() {
        let r = regles::referentiel_officiel();
        let p = page_referentiel(&r);
        assert_eq!(p.salles.len(), r.salles.len());
        assert_eq!(p.equipes.len(), r.equipes.len());
        assert_eq!(p.extensions.len(), r.extensions.len());
        assert_eq!(p.gestionnaires.len(), r.gestionnaires.len());
        assert_eq!(p.modeles_batiments.len(), r.modeles_batiments.len());
        assert_eq!(p.communautes.len(), r.communautes.len());
        assert_eq!(p.capitaux.len(), r.capitaux.len());
        assert!(p.salles.iter().all(|s| !s.source.is_empty()));
        // chaque statut est « officiel » ou « a_confirmer » ; « a_confirmer » seulement avec un point
        assert!(p
            .salles
            .iter()
            .chain(&p.equipes)
            .chain(&p.extensions)
            .all(|u| {
                (u.statut == "officiel" || u.statut == "a_confirmer")
                    && (u.statut == "officiel" || u.point.is_some())
            }));
    }
}
