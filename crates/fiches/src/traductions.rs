//! Textes français du référentiel Core (`data/core/fr/*.json`).
//!
//! Ce sont des traductions **non officielles** des textes de l'Archives of Nethys (licence OGL),
//! faites pour l'application ; le joueur peut en corriger les noms d'après le Manuel des joueurs.

use serde_json::Value;

/// Lots de traduction des sorts (`data/core/fr/sorts_NN.json`).
const LOTS_SORTS: [&str; 30] = [
    include_str!("../../../data/core/fr/sorts_01.json"),
    include_str!("../../../data/core/fr/sorts_02.json"),
    include_str!("../../../data/core/fr/sorts_03.json"),
    include_str!("../../../data/core/fr/sorts_04.json"),
    include_str!("../../../data/core/fr/sorts_05.json"),
    include_str!("../../../data/core/fr/sorts_06.json"),
    include_str!("../../../data/core/fr/sorts_07.json"),
    include_str!("../../../data/core/fr/sorts_08.json"),
    include_str!("../../../data/core/fr/sorts_09.json"),
    include_str!("../../../data/core/fr/sorts_10.json"),
    include_str!("../../../data/core/fr/sorts_11.json"),
    include_str!("../../../data/core/fr/sorts_12.json"),
    include_str!("../../../data/core/fr/sorts_13.json"),
    include_str!("../../../data/core/fr/sorts_14.json"),
    include_str!("../../../data/core/fr/sorts_15.json"),
    include_str!("../../../data/core/fr/sorts_16.json"),
    include_str!("../../../data/core/fr/sorts_17.json"),
    include_str!("../../../data/core/fr/sorts_18.json"),
    include_str!("../../../data/core/fr/sorts_19.json"),
    include_str!("../../../data/core/fr/sorts_20.json"),
    include_str!("../../../data/core/fr/sorts_21.json"),
    include_str!("../../../data/core/fr/sorts_22.json"),
    include_str!("../../../data/core/fr/sorts_23.json"),
    include_str!("../../../data/core/fr/sorts_24.json"),
    include_str!("../../../data/core/fr/sorts_25.json"),
    include_str!("../../../data/core/fr/sorts_26.json"),
    include_str!("../../../data/core/fr/sorts_27.json"),
    include_str!("../../../data/core/fr/sorts_28.json"),
    include_str!("../../../data/core/fr/sorts_29.json"),
    include_str!("../../../data/core/fr/sorts_30.json"),
];

/// Lots de traduction des dons (`data/core/fr/dons_NN.json`).
const LOTS_DONS: [&str; 6] = [
    include_str!("../../../data/core/fr/dons_01.json"),
    include_str!("../../../data/core/fr/dons_02.json"),
    include_str!("../../../data/core/fr/dons_03.json"),
    include_str!("../../../data/core/fr/dons_04.json"),
    include_str!("../../../data/core/fr/dons_05.json"),
    include_str!("../../../data/core/fr/dons_06.json"),
];

/// Un don en français.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DonFr {
    pub nom: String,
    pub resume: Option<String>,
    pub prerequis: Option<String>,
    pub avantage: Option<String>,
    pub normal: Option<String>,
    pub special: Option<String>,
}

/// Un sort en français, assemblé à partir du catalogue anglais et des fichiers de traduction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortFr {
    pub nom: String,
    pub ecole: String,
    pub sous_ecole: Option<String>,
    pub descripteurs: Vec<String>,
    pub temps_d_incantation: Option<String>,
    pub composantes: Option<String>,
    pub portee: Option<String>,
    pub cible: Option<String>,
    pub duree: Option<String>,
    pub jet_de_sauvegarde: Option<String>,
    pub resistance_a_la_magie: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct Traductions {
    races: Value,
    competences: Value,
    ensorceleur: Value,
    sorts: std::collections::BTreeMap<String, SortFr>,
    dons: std::collections::BTreeMap<String, DonFr>,
}

impl Traductions {
    /// Traductions chargées une seule fois et partagées.
    pub fn partage() -> &'static Traductions {
        static T: std::sync::OnceLock<Traductions> = std::sync::OnceLock::new();
        T.get_or_init(Traductions::core)
    }

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
            sorts: Self::charger_sorts(),
            dons: Self::charger_dons(),
        }
    }

    fn charger_dons() -> std::collections::BTreeMap<String, DonFr> {
        let noms: Value =
            serde_json::from_str(include_str!("../../../data/core/fr/dons_noms.json"))
                .expect("dons_noms.json valide");
        let texte = |v: &Value| v.as_str().map(str::to_string);
        let mut out = std::collections::BTreeMap::new();
        for lot in LOTS_DONS {
            let lot: Value = serde_json::from_str(lot).expect("lot de dons valide");
            let Some(objet) = lot.as_object() else {
                continue;
            };
            for (nom_en, t) in objet {
                out.insert(
                    nom_en.clone(),
                    DonFr {
                        nom: texte(&noms["noms"][nom_en]).unwrap_or_else(|| nom_en.clone()),
                        resume: texte(&t["resume"]),
                        prerequis: texte(&t["prerequis"]),
                        avantage: texte(&t["avantage"]),
                        normal: texte(&t["normal"]),
                        special: texte(&t["special"]),
                    },
                );
            }
        }
        out
    }

    /// Don en français d'après son nom anglais du catalogue (sans son choix éventuel).
    pub fn don(&self, nom_en: &str) -> Option<&DonFr> {
        self.dons.get(crate::catalogue::nom_de_base(nom_en))
    }

    fn charger_sorts() -> std::collections::BTreeMap<String, SortFr> {
        let noms: Value =
            serde_json::from_str(include_str!("../../../data/core/fr/sorts_noms.json"))
                .expect("sorts_noms.json valide");
        let champs: Value =
            serde_json::from_str(include_str!("../../../data/core/fr/sorts_champs.json"))
                .expect("sorts_champs.json valide");
        let texte = |v: &Value| v.as_str().map(str::to_string);
        let mut out = std::collections::BTreeMap::new();
        for lot in LOTS_SORTS {
            let lot: Value = serde_json::from_str(lot).expect("lot de sorts valide");
            let Some(objet) = lot.as_object() else {
                continue;
            };
            for (nom_en, t) in objet {
                let c = &champs[nom_en];
                out.insert(
                    nom_en.clone(),
                    SortFr {
                        nom: texte(&noms["noms"][nom_en]).unwrap_or_else(|| nom_en.clone()),
                        ecole: texte(&c["ecole"]).unwrap_or_default(),
                        sous_ecole: texte(&c["sous_ecole"]),
                        descripteurs: c["descripteurs"]
                            .as_array()
                            .map(|a| a.iter().filter_map(texte).collect())
                            .unwrap_or_default(),
                        temps_d_incantation: texte(&c["temps_d_incantation"]),
                        // les composantes avec détail matériel sont dans le lot, sinon la forme normalisée
                        composantes: texte(&t["composantes"]).or_else(|| texte(&c["composantes"])),
                        portee: texte(&c["portee"]),
                        cible: texte(&t["cible"]),
                        duree: texte(&c["duree"]),
                        jet_de_sauvegarde: texte(&c["jet_de_sauvegarde"]),
                        resistance_a_la_magie: texte(&c["resistance_a_la_magie"]),
                        description: texte(&t["description"]).unwrap_or_default(),
                    },
                );
            }
        }
        out
    }

    /// Sort en français d'après son nom anglais du catalogue.
    pub fn sort(&self, nom_en: &str) -> Option<&SortFr> {
        self.sorts.get(nom_en)
    }

    /// Nombre de sorts traduits.
    pub fn nombre_de_sorts(&self) -> usize {
        self.sorts.len()
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
    fn tous_les_sorts_de_l_ensorceleur_et_de_lignage_sont_traduits() {
        let r = Referentiel::core();
        let t = Traductions::core();
        let mut manquants = Vec::new();
        for s in &r.sorts {
            if s.niveau_pour("sorcerer").is_some() && t.sort(&s.nom_en).is_none() {
                manquants.push(s.nom_en.clone());
            }
        }
        let ens = r.classe("ensorceleur").unwrap();
        for l in &ens.lignages {
            for b in &l.sorts_bonus {
                if let Some(s) = r.sort_approx(&b.nom_en) {
                    if t.sort(&s.nom_en).is_none() {
                        manquants.push(s.nom_en.clone());
                    }
                }
            }
        }
        assert!(manquants.is_empty(), "sorts non traduits : {manquants:?}");
        // champs mécaniques présents pour chaque sort traduit
        for s in &r.sorts {
            if let Some(fr) = t.sort(&s.nom_en) {
                assert!(!fr.ecole.is_empty(), "{}", s.nom_en);
                assert!(!fr.description.is_empty(), "{}", s.nom_en);
                assert_eq!(
                    fr.temps_d_incantation.is_some(),
                    s.temps_d_incantation_en.is_some(),
                    "{}",
                    s.nom_en
                );
                assert_eq!(fr.cible.is_some(), s.cible_en.is_some(), "{}", s.nom_en);
                assert_eq!(fr.portee.is_some(), s.portee_en.is_some(), "{}", s.nom_en);
            }
        }
        let magic = t.sort("Magic Missile").unwrap();
        assert_eq!(magic.nom, "Projectile magique");
        assert_eq!(magic.ecole, "évocation");
        assert_eq!(magic.portee.as_deref(), Some("moyenne (30 m + 3 m/niveau)"));
        assert_eq!(magic.composantes.as_deref(), Some("V, G"));
    }

    #[test]
    fn tous_les_dons_sont_traduits() {
        let r = Referentiel::core();
        let t = Traductions::core();
        for d in &r.dons {
            let fr = t
                .don(&d.nom_en)
                .unwrap_or_else(|| panic!("don non traduit : {}", d.nom_en));
            // « Endurance » s’écrit pareil dans les deux langues
            if d.nom_en != "Endurance" {
                assert_ne!(fr.nom, d.nom_en, "nom non traduit : {}", d.nom_en);
            }
            assert_eq!(fr.resume.is_some(), !d.resume_en.is_empty(), "{}", d.nom_en);
            assert_eq!(
                fr.prerequis.is_some(),
                d.prerequis_en.is_some(),
                "{}",
                d.nom_en
            );
            assert_eq!(
                fr.avantage.is_some(),
                d.avantage_en.is_some(),
                "{}",
                d.nom_en
            );
            assert_eq!(fr.normal.is_some(), d.normal_en.is_some(), "{}", d.nom_en);
            assert_eq!(fr.special.is_some(), d.special_en.is_some(), "{}", d.nom_en);
        }
        assert_eq!(t.don("Power Attack").unwrap().nom, "Attaque en puissance");
        assert_eq!(
            t.don("Weapon Focus (longsword)").unwrap().nom,
            "Arme de prédilection"
        );
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
