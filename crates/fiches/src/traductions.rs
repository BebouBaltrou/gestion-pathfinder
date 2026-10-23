//! Textes français du référentiel Core (`data/core/fr/*.json`).
//!
//! Ce sont des traductions **non officielles** des textes de l'Archives of Nethys (licence OGL),
//! faites pour l'application ; le joueur peut en corriger les noms d'après le Manuel des joueurs.

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Traductions {
    races: Value,
    competences: Value,
    ensorceleur: Value,
}

impl Traductions {
    pub fn core() -> Traductions {
        Traductions {
            races: serde_json::from_str(include_str!("../../../data/core/fr/races.json"))
                .expect("races.json (fr) valide"),
            competences: serde_json::from_str(include_str!(
                "../../../data/core/fr/competences.json"
            ))
            .expect("competences.json (fr) valide"),
            ensorceleur: serde_json::from_str(include_str!(
                "../../../data/core/fr/ensorceleur.json"
            ))
            .expect("ensorceleur.json (fr) valide"),
        }
    }

    /// Nom et texte français d'un trait racial, d'après son nom anglais.
    pub fn trait_racial(&self, race: &str, nom_en: &str) -> Option<(&str, &str)> {
        let t = &self.races["races"][race]["traits"][nom_en];
        Some((t["nom"].as_str()?, t["texte"].as_str()?))
    }

    /// Nom français d'une compétence (`spellcraft` → « Art de la magie »).
    pub fn competence(&self, id: &str) -> Option<&str> {
        self.competences["competences"][id].as_str()
    }

    /// Nom français d'une compétence avec son sous-type : `knowledge:arcana` → « Connaissances (arcanes) ».
    pub fn competence_avec_sous_type(&self, cle: &str) -> Option<String> {
        match cle.split_once(':') {
            None => self.competence(cle).map(str::to_string),
            Some((base, sous)) => {
                let nom = self.competence(base)?;
                let sous = if base == "knowledge" {
                    self.competences["sous_types_connaissances"][sous]
                        .as_str()
                        .unwrap_or(sous)
                } else {
                    sous
                };
                Some(format!("{nom} ({sous})"))
            }
        }
    }

    pub fn lignage(&self, id: &str) -> Option<LignageFr<'_>> {
        let l = &self.ensorceleur["lignages"][id];
        Some(LignageFr {
            nom: l["nom"].as_str()?,
            competence_de_classe: l["competence_de_classe"].as_str()?,
            arcane: l["arcane"].as_str()?,
            choix: l["choix"].as_str(),
            valeur: l,
        })
    }

    /// Capacités de classe d'un niveau, du texte anglais du tableau (« Bloodline power, cantrips »).
    pub fn capacites_de_niveau(&self, special_en: &str) -> Vec<String> {
        special_en
            .split(", ")
            .filter(|s| !s.trim().is_empty() && *s != "—")
            .map(|s| {
                self.ensorceleur["classe"]["capacites_par_niveau"][s.trim().to_lowercase()]
                    .as_str()
                    .unwrap_or(s)
                    .to_string()
            })
            .collect()
    }

    /// Texte d'une règle de classe (« Spells », « Swap », « Bloodline »…).
    pub fn texte_de_classe(&self, cle: &str) -> Option<&str> {
        self.ensorceleur["classe"]["textes"][cle].as_str()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LignageFr<'a> {
    pub nom: &'a str,
    pub competence_de_classe: &'a str,
    pub arcane: &'a str,
    pub choix: Option<&'a str>,
    valeur: &'a Value,
}

impl<'a> LignageFr<'a> {
    /// Nom et texte français d'un pouvoir, d'après son nom anglais.
    pub fn pouvoir(&self, nom_en: &str) -> Option<(&'a str, &'a str)> {
        let p = &self.valeur["pouvoirs"][nom_en];
        Some((p["nom"].as_str()?, p["texte"].as_str()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::donnees::Referentiel;

    #[test]
    fn tout_le_referentiel_est_traduit() {
        let r = Referentiel::core();
        let t = Traductions::core();
        for race in &r.races {
            for tr in &race.traits {
                assert!(
                    t.trait_racial(&race.id, &tr.nom_en).is_some(),
                    "trait non traduit : {} / {}",
                    race.id,
                    tr.nom_en
                );
            }
        }
        for c in &r.competences {
            assert!(t.competence(&c.id).is_some(), "compétence : {}", c.id);
        }
        let ens = r.classe("ensorceleur").unwrap();
        for l in &ens.lignages {
            let fr = t
                .lignage(&l.id)
                .unwrap_or_else(|| panic!("lignage {}", l.id));
            for p in &l.pouvoirs {
                assert!(
                    fr.pouvoir(&p.nom_en).is_some(),
                    "pouvoir non traduit : {} / {}",
                    l.id,
                    p.nom_en
                );
            }
            if l.id == "draconic" || l.id == "elemental" {
                assert!(fr.choix.is_some());
            }
        }
        for niveau in &ens.progression {
            for (c, en) in t
                .capacites_de_niveau(&niveau.special_en)
                .iter()
                .zip(niveau.special_en.split(", ").filter(|x| *x != "—"))
            {
                assert_ne!(
                    c.to_lowercase(),
                    en.to_lowercase(),
                    "capacité non traduite : {en}"
                );
            }
        }
    }

    #[test]
    fn noms_de_competences() {
        let t = Traductions::core();
        assert_eq!(t.competence("spellcraft"), Some("Art de la magie"));
        assert_eq!(
            t.competence_avec_sous_type("knowledge:local").as_deref(),
            Some("Connaissances (folklore local)")
        );
        assert_eq!(
            t.competence_avec_sous_type("craft:armes").as_deref(),
            Some("Artisanat (armes)")
        );
    }

    #[test]
    fn capacites_du_niveau_1() {
        let t = Traductions::core();
        let c = t.capacites_de_niveau("Bloodline power, cantrips, eschew materials");
        assert_eq!(c.len(), 3);
        assert_eq!(c[0], "pouvoir de lignage");
    }
}
