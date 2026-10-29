//! Choix génériques des classes : grâces du paladin, talents du roublard, ennemis jurés du rôdeur.
//! Données : `data/core/choix_de_classe.json` (textes du Manuel des joueurs fournis par l'utilisateur).

use crate::donnees::Referentiel;
use crate::fiche::{Fiche, Probleme};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ChoixDeClasse {
    pub grace_paladin: GracePaladin,
    pub talents_roublard: TalentsRoublard,
    pub ennemis_jures_rodeur: EnnemisJures,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GracePaladin {
    pub premier_niveau: u32,
    pub choix_tous_les: u32,
    pub listes: Vec<ListeGraces>,
    /// Grâce → grâce qu'il faut avoir déjà choisie.
    pub prerequis_de_grace: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListeGraces {
    pub a_partir_du: u32,
    pub graces: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TalentsRoublard {
    pub talents: Vec<String>,
    pub talents_de_maitre: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EnnemisJures {
    pub choix_aux_niveaux: Vec<u32>,
    pub types: Vec<String>,
    pub bonus_de_base: i32,
}

/// Nom sans l'astérisque qui marque un talent ajoutant un effet à l'attaque sournoise.
fn nom_sans_etoile(nom: &str) -> &str {
    nom.trim_end_matches('*')
}

impl ChoixDeClasse {
    /// Nombre de grâces à choisir au niveau de paladin `niveau` (une tous les 3 niveaux, dès le 3e).
    pub fn nombre_graces(&self, niveau: u32) -> u32 {
        let g = &self.grace_paladin;
        if g.choix_tous_les == 0 || niveau < g.premier_niveau {
            return 0;
        }
        (niveau - g.premier_niveau) / g.choix_tous_les + 1
    }

    /// Niveau de paladin à partir duquel la grâce `nom` est accessible, s'il existe.
    pub fn niveau_de_grace(&self, nom: &str) -> Option<u32> {
        self.grace_paladin
            .listes
            .iter()
            .find(|l| l.graces.iter().any(|g| g == nom))
            .map(|l| l.a_partir_du)
    }

    /// Nombre de choix d'ennemi juré acquis au niveau de rôdeur `niveau`.
    pub fn nombre_ennemis_jures(&self, niveau: u32) -> u32 {
        self.ennemis_jures_rodeur
            .choix_aux_niveaux
            .iter()
            .filter(|n| **n <= niveau)
            .count() as u32
    }

    /// Vrai si le talent (de roublard ou de maître) existe, astérisque ignorée.
    pub fn talent_connu(&self, nom: &str) -> bool {
        let t = &self.talents_roublard;
        let n = nom_sans_etoile(nom);
        t.talents
            .iter()
            .chain(&t.talents_de_maitre)
            .any(|x| nom_sans_etoile(x) == n)
    }
}

impl Fiche {
    /// Problèmes des choix génériques de classe : grâces, talents et ennemis jurés.
    /// Les grâces et les ennemis jurés doivent être tous choisis ; les talents ne sont
    /// vérifiés que par leur nom (leurs conditions de niveau ne sont pas saisies).
    pub(crate) fn problemes_choix_de_classe(&self, r: &Referentiel) -> Vec<Probleme> {
        let c = &r.choix;
        let mut p = Vec::new();

        let niveau_paladin = self.niveau_de_classe("paladin");
        let attendus = c.nombre_graces(niveau_paladin);
        let choisis = self.graces.len() as u32;
        if attendus != choisis {
            p.push(Probleme::GracesAChoisir { attendus, choisis });
        }
        for g in &self.graces {
            match c.niveau_de_grace(g) {
                None => p.push(Probleme::GraceInconnue(g.clone())),
                Some(requis) if requis > niveau_paladin => p.push(Probleme::GraceNonDisponible {
                    nom: g.clone(),
                    niveau: requis,
                }),
                Some(_) => {}
            }
            if let Some(pre) = c.grace_paladin.prerequis_de_grace.get(g) {
                if !self.graces.contains(pre) {
                    p.push(Probleme::GracePrerequisManquante {
                        nom: g.clone(),
                        requis: pre.clone(),
                    });
                }
            }
        }

        for t in &self.talents_roublard {
            if !c.talent_connu(t) {
                p.push(Probleme::TalentInconnu(t.clone()));
            }
        }

        let attendus = c.nombre_ennemis_jures(self.niveau_de_classe("rodeur"));
        let choisis = self.ennemis_jures.len() as u32;
        if attendus != choisis {
            p.push(Probleme::EnnemisJuresAChoisir { attendus, choisis });
        }
        for e in &self.ennemis_jures {
            if !c.ennemis_jures_rodeur.types.contains(e) {
                p.push(Probleme::EnnemiJureInconnu(e.clone()));
            }
        }
        p
    }
}
