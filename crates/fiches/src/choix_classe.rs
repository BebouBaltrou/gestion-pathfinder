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
    /// Texte du livre de chaque talent (nom sans astérisque).
    pub textes: BTreeMap<String, String>,
    pub premier_niveau: u32,
    pub un_tous_les: u32,
    pub premier_niveau_de_maitre: u32,
    /// Talents que l'on peut prendre plusieurs fois.
    pub repetables: Vec<String>,
    /// Talent → talents qu'il faut déjà avoir.
    pub prerequis_de_talent: BTreeMap<String, Vec<String>>,
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

    /// Talents à choisir au niveau de roublard `niveau` (un au niveau 2, puis un tous les 2 niveaux).
    pub fn nombre_talents(&self, niveau: u32) -> u32 {
        let t = &self.talents_roublard;
        if t.un_tous_les == 0 || niveau < t.premier_niveau {
            return 0;
        }
        (niveau - t.premier_niveau) / t.un_tous_les + 1
    }

    /// Talents de maître permis au niveau de roublard `niveau` (ils remplacent un talent dès le niveau 10).
    pub fn talents_de_maitre_permis(&self, niveau: u32) -> u32 {
        let t = &self.talents_roublard;
        if t.un_tous_les == 0 || niveau < t.premier_niveau_de_maitre {
            return 0;
        }
        (niveau - t.premier_niveau_de_maitre) / t.un_tous_les + 1
    }

    /// Vrai si le talent est un talent de maître roublard.
    pub fn talent_de_maitre(&self, nom: &str) -> bool {
        let n = nom_sans_etoile(nom);
        self.talents_roublard
            .talents_de_maitre
            .iter()
            .any(|x| nom_sans_etoile(x) == n)
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

        let niveau_roublard = self.niveau_de_classe("roublard");
        let attendus = c.nombre_talents(niveau_roublard);
        let choisis = self.talents_roublard.len() as u32;
        if attendus != choisis {
            p.push(Probleme::TalentsAChoisir { attendus, choisis });
        }
        let regles = &c.talents_roublard;
        let mut maitres = 0;
        for (i, t) in self.talents_roublard.iter().enumerate() {
            if !c.talent_connu(t) {
                p.push(Probleme::TalentInconnu(t.clone()));
            }
            let sans = nom_sans_etoile(t);
            let repetable = regles.repetables.iter().any(|r| nom_sans_etoile(r) == sans);
            let deja = self.talents_roublard[..i]
                .iter()
                .any(|x| nom_sans_etoile(x) == sans);
            if deja && !repetable {
                p.push(Probleme::TalentEnDouble(t.clone()));
            }
            if c.talent_de_maitre(t) {
                maitres += 1;
            }
            if let Some(requis) = regles.prerequis_de_talent.get(sans) {
                for r in requis {
                    if !self
                        .talents_roublard
                        .iter()
                        .any(|x| nom_sans_etoile(x) == r)
                    {
                        p.push(Probleme::TalentPrerequisManquant {
                            nom: t.clone(),
                            requis: r.clone(),
                        });
                    }
                }
            }
        }
        let permis = c.talents_de_maitre_permis(niveau_roublard);
        if maitres > permis {
            p.push(Probleme::TalentsDeMaitreTropNombreux {
                permis,
                choisis: maitres,
            });
        }

        let attendus = c.nombre_ennemis_jures(self.niveau_de_classe("rodeur"));
        let choisis = self.ennemis_jures.len() as u32;
        if attendus != choisis {
            p.push(Probleme::EnnemisJuresAChoisir { attendus, choisis });
        }
        for (i, e) in self.ennemis_jures.iter().enumerate() {
            if !c.ennemis_jures_rodeur.types.contains(e) {
                p.push(Probleme::EnnemiJureInconnu(e.clone()));
            }
            if self.ennemis_jures[..i].contains(e) {
                p.push(Probleme::EnnemiJureEnDouble(e.clone()));
            }
        }
        // Dès le niveau 5, chaque nouveau choix s'accompagne d'une augmentation de +2 contre
        // l'un des ennemis jurés : une de moins que de choix d'ennemis.
        let attendus = attendus.saturating_sub(1);
        let choisis = self.augmentations_ennemis.len() as u32;
        if attendus != choisis {
            p.push(Probleme::AugmentationsEnnemisAChoisir { attendus, choisis });
        }
        for e in &self.augmentations_ennemis {
            if !self.ennemis_jures.contains(e) {
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
            let deja = self.pouvoirs_rage[..i].iter().filter(|x| *x == nom).count() as u32;
            if deja >= rage.repetables.get(nom).copied().unwrap_or(1) {
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
    /// Pouvoirs que l'on peut prendre plusieurs fois : nom → nombre maximal.
    pub repetables: BTreeMap<String, u32>,
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
    regles: ReglesRage,
}

#[derive(Deserialize)]
struct ReglesRage {
    #[serde(default)]
    repetables: BTreeMap<String, u32>,
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
            repetables: t.regles.repetables,
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

    /// Vrai si `nom` peut encore être pris, compte tenu des pouvoirs déjà choisis
    /// (une fois, ou plusieurs fois pour les pouvoirs répétables).
    pub fn encore_prenable(&self, nom: &str, choisis: &[String]) -> bool {
        let deja = choisis.iter().filter(|c| *c == nom).count() as u32;
        deja < self.repetables.get(nom).copied().unwrap_or(1)
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

/// Effets de la rage du barbare selon son niveau (Manuel des joueurs, Barbare : Rage de
/// berserker +4/+4/+2, Rage de grand berserker au niveau 11 +6/+3, Rage de maître berserker
/// au niveau 20 +8/+4), malus de −2 à la CA.
pub fn effets_de_rage(niveau: u32) -> Vec<crate::effet::Effet> {
    use crate::effet::{Effet, TypeBonus};
    if niveau == 0 {
        return Vec::new();
    }
    let (nom, carac, volonte) = if niveau >= 20 {
        ("Rage de maître berserker", 8, 4)
    } else if niveau >= 11 {
        ("Rage de grand berserker", 6, 3)
    } else {
        ("Rage de berserker", 4, 2)
    };
    vec![
        Effet::new("carac.for", TypeBonus::Moral, carac, nom),
        Effet::new("carac.con", TypeBonus::Moral, carac, nom),
        Effet::new("jds.vol", TypeBonus::Moral, volonte, nom),
        Effet::new("ca", TypeBonus::SansType, -2, nom),
    ]
}

/// Effets chiffrés des pouvoirs de rage choisis, tant que le barbare est en rage (Manuel des
/// joueurs : Escalade en rage et Nage en rage donnent un bonus d'altération égal au niveau).
pub fn effets_des_pouvoirs_de_rage(fiche: &Fiche) -> Vec<crate::effet::Effet> {
    use crate::effet::{Effet, TypeBonus};
    if !fiche.en_rage {
        return Vec::new();
    }
    let niveau = fiche.niveau_de_classe("barbare") as i32;
    let mut e = Vec::new();
    for (pouvoir, cible) in [
        ("Escalade en rage", "comp.climb"),
        ("Nage en rage", "comp.swim"),
    ] {
        if fiche.pouvoirs_rage.iter().any(|p| p == pouvoir) {
            e.push(Effet::new(cible, TypeBonus::Alteration, niveau, pouvoir));
        }
    }
    e
}

/// Bonus de vitesse (en pieds, 5 = 1,50 m) des pouvoirs de rage choisis : Pas rapides, jusqu'à trois fois.
pub fn bonus_de_vitesse_de_rage(fiche: &Fiche) -> u32 {
    if !fiche.en_rage {
        return 0;
    }
    5 * (fiche
        .pouvoirs_rage
        .iter()
        .filter(|p| *p == "Pas rapides")
        .count() as u32)
        .min(3)
}

/// Valeurs chiffrées des capacités de classe conditionnelles, à afficher (Manuel des joueurs).
pub fn capacites_chiffrees(fiche: &Fiche) -> Vec<String> {
    let mut l = Vec::new();
    let barde = fiche.niveau_de_classe("barde");
    if barde >= 1 {
        let b = 1 + (barde + 1) / 6;
        l.push(format!(
            "Inspiration vaillante : +{b} aux jets d'attaque et de dégâts avec des armes (compétence), +{b} aux jets de sauvegarde contre la terreur et le charme (moral)"
        ));
    }
    if barde >= 3 {
        let b = 2 + (barde - 3) / 4;
        l.push(format!(
            "Inspiration talentueuse : +{b} aux tests d'une compétence au choix d'un allié (compétence)"
        ));
    }
    if barde >= 9 {
        let cibles = (1 + (barde - 9) / 3).min(4);
        l.push(format!(
            "Inspiration glorieuse : {cibles} cible(s), 2 DV supplémentaires (d10), +2 aux jets d'attaque et +1 aux jets de Vigueur (compétence)"
        ));
    }
    if barde >= 15 {
        let cibles = 1 + (barde - 15) / 3;
        l.push(format!(
            "Inspiration héroïque : {cibles} cible(s), +4 aux jets de sauvegarde (moral) et +4 à la CA (esquive)"
        ));
    }
    for (i, t) in fiche.ennemis_jures.iter().enumerate() {
        if fiche.ennemis_jures[..i].contains(t) {
            continue;
        }
        let augmentations = fiche
            .augmentations_ennemis
            .iter()
            .filter(|a| *a == t)
            .count() as u32;
        l.push(format!(
            "Ennemi juré ({t}) : +{} aux tests de Bluff, Connaissances, Perception, Psychologie et Survie, et aux jets d'attaque et de dégâts contre ces créatures",
            2 + 2 * augmentations
        ));
    }
    if !fiche.ennemis_jures.is_empty() {
        l.push("Ennemis jurés : si une créature relève de deux familles, seul le bonus le plus élevé s'applique".to_string());
    }
    let roublard = fiche.niveau_de_classe("roublard");
    if roublard >= 1 {
        l.push(format!(
            "Recherche des pièges : +{} aux tests de Perception pour repérer les pièges et aux tests de Sabotage",
            (roublard / 2).max(1)
        ));
    }
    if roublard >= 3 {
        let b = roublard / 3;
        l.push(format!(
            "Sens des pièges : +{b} aux jets de Réflexes contre les pièges et +{b} à la CA (esquive) contre leurs attaques"
        ));
    }
    let moine = fiche.niveau_de_classe("moine");
    if moine >= 4 {
        let mut armes = vec!["armes magiques"];
        if moine >= 7 {
            armes.push("fer froid et argent");
        }
        if moine >= 10 {
            armes.push("armes Loyales");
        }
        if moine >= 16 {
            armes.push("adamantium");
        }
        l.push(format!(
            "Frappe ki (1 point de ki restant au moins) : attaques à mains nues considérées comme {}, pour ignorer la réduction de dégâts{}",
            armes.join(", "),
            if moine >= 16 { " ou la solidité des objets" } else { "" }
        ));
        l.push("Dépenser 1 point de ki (action rapide) : une attaque de plus dans un déluge de coups, ou +6 m de vitesse pendant 1 round, ou +4 à la CA (esquive) pendant 1 round".to_string());
    }
    let guerrier = fiche.niveau_de_classe("guerrier");
    if guerrier >= 2 {
        let b = 1 + (guerrier - 2) / 4;
        l.push(format!(
            "Courage : +{b} aux jets de sauvegarde contre la terreur"
        ));
    }
    l
}
