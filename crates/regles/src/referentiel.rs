use domaine::fiches::*;
use domaine::{Ressource, Source};

/// Ensemble des données officielles du système d'intermède.
#[derive(Debug, Clone)]
pub struct Referentiel {
    pub sources: Vec<LivreSource>,
    pub capitaux: Vec<ValeurCapital>,
    pub communautes: Vec<CategorieCommunaute>,
    pub salles: Vec<Unite>,
    pub extensions: Vec<Unite>,
    pub equipes: Vec<Unite>,
    pub gestionnaires: Vec<Gestionnaire>,
    pub modeles_batiments: Vec<Modele>,
    pub modeles_organisations: Vec<Modele>,
    pub points_a_confirmer: Vec<PointAConfirmer>,
}

impl Referentiel {
    pub fn salle(&self, id: &str) -> Option<&Unite> {
        self.salles.iter().find(|u| u.id == id)
    }
    pub fn extension(&self, id: &str) -> Option<&Unite> {
        self.extensions.iter().find(|u| u.id == id)
    }
    pub fn equipe(&self, id: &str) -> Option<&Unite> {
        self.equipes.iter().find(|u| u.id == id)
    }
    pub fn gestionnaire(&self, id: &str) -> Option<&Gestionnaire> {
        self.gestionnaires.iter().find(|g| g.id == id)
    }
    pub fn modele_batiment(&self, id: &str) -> Option<&Modele> {
        self.modeles_batiments.iter().find(|m| m.id == id)
    }
    pub fn modele_organisation(&self, id: &str) -> Option<&Modele> {
        self.modeles_organisations.iter().find(|m| m.id == id)
    }
    pub fn communaute(&self, id: &str) -> Option<&CategorieCommunaute> {
        self.communautes.iter().find(|c| c.id == id)
    }
    pub fn point(&self, code: &str) -> Option<&PointAConfirmer> {
        self.points_a_confirmer.iter().find(|p| p.code == code)
    }
    pub fn livre(&self, code: &str) -> Option<&LivreSource> {
        self.sources.iter().find(|l| l.code == code)
    }

    /// Valeurs d'achat et de production d'un capital (Table 2-1, p. 78). `None` pour les po.
    pub fn valeur(&self, r: Ressource) -> Option<&ValeurCapital> {
        self.capitaux.iter().find(|v| v.ressource == r)
    }

    /// Catégorie officielle proposée pour une population (Guide du maître, PC-11 pour 20).
    pub fn categorie_pour_population(&self, population: u32) -> Option<&CategorieCommunaute> {
        self.communautes.iter().find(|c| {
            population >= c.population_min && c.population_max.is_none_or(|max| population <= max)
        })
    }

    /// Texte de référence consultable : « Guide de campagne… p. 102 — Labyrinthe ».
    pub fn reference(&self, s: &Source) -> String {
        let titre = self
            .livre(&s.livre)
            .map(|l| l.titre.as_str())
            .unwrap_or(&s.livre);
        match s.page {
            Some(p) => format!("{titre}, p. {p} — {}", s.section),
            None => format!("{titre} — {}", s.section),
        }
    }
}
