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

        let niveau_barbare = self.niveau_de_classe("barbare");
        let attendus = PouvoirsRage::attendus(niveau_barbare);
        let choisis = self.pouvoirs_rage.len() as u32;
        if attendus != choisis {
            p.push(Probleme::PouvoirsRageAChoisir { attendus, choisis });
        }
        let mut vus: Vec<(FamilleRage, String)> = Vec::new();
        for nom in &self.pouvoirs_rage {
            if let Some((fam, g)) = groupe_rage(nom) {
                if vus.iter().any(|(f2, g2)| *f2 == fam && *g2 != g) {
                    p.push(Probleme::PouvoirRageGroupeDifferent { nom: nom.clone() });
                }
                if !vus.contains(&(fam, g.clone())) {
                    vus.push((fam, g));
                }
            }
        }
        for (i, nom) in self.pouvoirs_rage.iter().enumerate() {
            let rage = &r.rage;
            if self.pouvoirs_rage[..i].contains(nom) {
                p.push(Probleme::PouvoirRageEnDouble(nom.clone()));
            }
            let Some(&nt) = rage.niveau_de_table.get(nom) else {
                p.push(Probleme::PouvoirRageInconnu(nom.clone()));
                continue;
            };
            if nt > niveau_barbare {
                p.push(Probleme::PouvoirRageNonDisponible {
                    nom: nom.clone(),
                    niveau: nt,
                });
            }
            if let Some(pr) = rage.prerequis.get(nom) {
                if pr.niveau > niveau_barbare {
                    p.push(Probleme::PouvoirRageNonDisponible {
                        nom: nom.clone(),
                        niveau: pr.niveau,
                    });
                }
                for requis in &pr.pouvoirs {
                    if !self.pouvoirs_rage.contains(requis) {
                        p.push(Probleme::PouvoirRagePrerequisManquant {
                            nom: nom.clone(),
                            requis: requis.clone(),
                        });
                    }
                }
            }
        }
        p
    }
}

/// Pouvoirs de rage du barbare : table par niveau, source (icône MJRA) et prérequis.
/// Données : `data/core/pouvoirs_rage.json` et `data/core/pouvoirs_rage_prerequis.json`.
#[derive(Debug, Clone, Default)]
pub struct PouvoirsRage {
    /// Nom du pouvoir → niveau de barbare de la table (1, 4, 6…).
    pub niveau_de_table: BTreeMap<String, u32>,
    /// Pouvoir issu d'un livre avancé : nom → source (`APG` = icône MJRA).
    pub sources: BTreeMap<String, String>,
    pub prerequis: BTreeMap<String, PrerequisPouvoir>,
    /// Texte du livre de chaque pouvoir affiché.
    pub textes: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PrerequisPouvoir {
    #[serde(default)]
    pub pouvoirs: Vec<String>,
    #[serde(default)]
    pub niveau: u32,
}

#[derive(Deserialize)]
struct FichierPouvoirsRage {
    niveaux: Vec<NiveauRage>,
    sources: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct NiveauRage {
    niveau: u32,
    pouvoirs: Vec<String>,
}

#[derive(Deserialize)]
struct FichierTextesRage {
    textes: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct FichierPrerequisRage {
    prerequis: BTreeMap<String, PrerequisPouvoir>,
}

impl PouvoirsRage {
    pub fn depuis_json(
        pouvoirs: &str,
        prerequis: &str,
        textes: &str,
    ) -> Result<Self, serde_json::Error> {
        let t: FichierPouvoirsRage = serde_json::from_str(pouvoirs)?;
        let p: FichierPrerequisRage = serde_json::from_str(prerequis)?;
        let x: FichierTextesRage = serde_json::from_str(textes)?;
        let mut niveau_de_table = BTreeMap::new();
        for n in &t.niveaux {
            for nom in &n.pouvoirs {
                niveau_de_table.insert(nom.clone(), n.niveau);
            }
        }
        Ok(PouvoirsRage {
            niveau_de_table,
            sources: t.sources,
            prerequis: p.prerequis,
            textes: x.textes,
        })
    }

    /// Pouvoirs à choisir au niveau de barbare `niveau` : un au niveau 2, puis un tous les deux niveaux.
    pub fn attendus(niveau: u32) -> u32 {
        if niveau < 2 {
            0
        } else {
            (niveau - 2) / 2 + 1
        }
    }

    /// Vrai si le pouvoir est dans la table, de niveau atteint, et que ses prérequis sont pris.
    pub fn accessible(&self, nom: &str, niveau: u32, choisis: &[String]) -> bool {
        let Some(&nt) = self.niveau_de_table.get(nom) else {
            return false;
        };
        if nt > niveau {
            return false;
        }
        self.prerequis
            .get(nom)
            .is_none_or(|p| p.niveau <= niveau && p.pouvoirs.iter().all(|r| choisis.contains(r)))
    }
}

/// Famille d'un pouvoir de rage : totémique (un seul groupe autorisé) ou de sang (un seul groupe).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FamilleRage {
    Totem,
    Sang,
}

/// Groupe d'un pouvoir de rage. Totémique : mot-clé du nom (animal, draconique…).
/// De sang : le nom sans « Sang » ni « mineur/majeur » (infernal, abyssal…).
pub fn groupe_rage(nom: &str) -> Option<(FamilleRage, String)> {
    const TOTEMS: [&str; 7] = [
        "animal",
        "draconique",
        "chaos",
        "fiélon",
        "spirituel",
        "serpent-monde",
        "ruche",
    ];
    let bas = nom.to_lowercase();
    if let Some(reste) = bas.strip_prefix("sang ") {
        let groupe = reste
            .trim_end_matches(" mineur")
            .trim_end_matches(" majeur")
            .trim_start_matches("de ")
            .to_string();
        return Some((FamilleRage::Sang, groupe));
    }
    if ["totem", "totémique", "serpent-monde", "ruche"]
        .iter()
        .any(|m| bas.contains(m))
    {
        return TOTEMS
            .iter()
            .find(|t| bas.contains(*t))
            .map(|t| (FamilleRage::Totem, t.to_string()));
    }
    None
}
