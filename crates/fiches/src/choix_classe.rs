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
    pub environnements_rodeur: EnvironnementsRodeur,
    pub pacte_du_chasseur: PacteDuChasseur,
    pub entrainement_aux_armes_guerrier: EntrainementAuxArmes,
    pub moine_mains_nues: MoineMainsNues,
    pub pacte_divin_paladin: PacteDivin,
    pub compagnon_rodeur: SourceDeCompagnon,
    pub compagnon_destrier: SourceDeCompagnon,
    pub versatilite_artistique_barde: Versatilite,
    pub ecoles_magicien: EcolesMagicien,
    pub pacte_magique_magicien: PacteMagique,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EcolesMagicien {
    pub ecoles: Vec<EcoleDeMagie>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EcoleDeMagie {
    pub id: String,
    pub nom: String,
    pub pouvoirs: Vec<PouvoirDEcole>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PouvoirDEcole {
    pub nom: String,
    pub niveau: u32,
    pub texte: String,
    #[serde(default)]
    pub usage: Option<UsageDEcole>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageDEcole {
    #[serde(rename = "type")]
    pub genre: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PacteMagique {
    pub niveau: u32,
    pub formes: Vec<String>,
    pub objets_fetiches: Vec<String>,
    pub familiers: Vec<Familier>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Familier {
    pub nom: String,
    pub capacite: String,
    pub effet: Option<EffetFamilier>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffetFamilier {
    pub cible: String,
    pub valeur: i32,
}

impl EcolesMagicien {
    pub fn ecole(&self, id: &str) -> Option<&EcoleDeMagie> {
        self.ecoles.iter().find(|e| e.id == id)
    }
}

/// Nombre d'utilisations par jour d'un pouvoir d'école au niveau de magicien `niveau`
/// (`modificateur` : modificateur d'Intelligence).
pub fn usages_d_un_pouvoir_d_ecole(genre: &str, niveau: u32, modificateur: i32) -> i32 {
    let n = niveau as i32;
    match genre {
        "fois" => 3 + modificateur,
        "rounds" => n,
        "points" => 3 * n,
        "metres" => 9 * n,
        "metamagie" => 1 + (n - 8).max(0) / 2,
        _ => 0,
    }
}

/// Vrai si l'école est une spécialité (pas l'école universelle).
pub fn est_specialiste(fiche: &Fiche) -> bool {
    fiche.niveau_de_classe("magicien") >= 1
        && fiche.ecole.as_deref().is_some_and(|e| e != "universelle")
}

/// Effets chiffrés de l'école et du familier du magicien qui ne dépendent que du niveau.
pub fn effets_de_magicien(fiche: &Fiche, r: &Referentiel) -> Vec<crate::effet::Effet> {
    use crate::effet::{Effet, TypeBonus};
    let niveau = fiche.niveau_de_classe("magicien");
    let mut e = Vec::new();
    if niveau == 0 {
        return e;
    }
    match fiche.ecole.as_deref() {
        Some("enchantement") => {
            let bonus = (2 + niveau / 5).min(6) as i32;
            for cible in ["comp.bluff", "comp.diplomacy", "comp.intimidate"] {
                e.push(Effet::new(
                    cible,
                    TypeBonus::Alteration,
                    bonus,
                    "Sourire enchanteur",
                ));
            }
        }
        Some("divination") => {
            e.push(Effet::new(
                "init",
                TypeBonus::SansType,
                ((niveau / 2).max(1)) as i32,
                "Mis en garde",
            ));
        }
        _ => {}
    }
    if fiche.pacte_magique.as_deref() == Some("familier") {
        if let Some(nom) = fiche.familier.as_deref() {
            let effet = r
                .choix
                .pacte_magique_magicien
                .familiers
                .iter()
                .find(|x| x.nom == nom)
                .and_then(|x| x.effet.as_ref());
            if let Some(ef) = effet {
                e.push(Effet::new(
                    &ef.cible,
                    TypeBonus::SansType,
                    ef.valeur,
                    &format!("Familier ({nom})"),
                ));
            }
        }
    }
    e
}

/// Lignes chiffrées de l'école et du pacte magique du magicien.
pub fn lignes_du_magicien(fiche: &Fiche, r: &Referentiel) -> Vec<String> {
    let niveau = fiche.niveau_de_classe("magicien");
    let mut l = Vec::new();
    if niveau == 0 {
        return l;
    }
    let moitie = (niveau / 2).max(1);
    let cinq = niveau / 5;
    match fiche.ecole.as_deref() {
        Some("abjuration") => {
            let res = if niveau >= 20 {
                "immunité"
            } else if niveau >= 11 {
                "résistance 10"
            } else {
                "résistance 5"
            };
            l.push(format!(
                "Abjuration — Résistance : {res} à un type d'énergie choisi en préparant les sorts"
            ));
            l.push(format!("Abjuration — Champ de protection : bonus de parade de +{} à la CA des alliés dans 6 m", 1 + cinq));
            if niveau >= 6 {
                l.push(format!(
                    "Abjuration — Absorption d'énergie : {} points d'énergie absorbés par jour",
                    3 * niveau
                ));
            }
        }
        Some("divination") => {
            l.push(format!(
                "Divination — Chance du devin : bonus d'intuition de +{moitie} pendant 1 round"
            ));
            if niveau >= 20 {
                l.push(
                    "Divination — Mis en garde : l'initiative est considérée comme un 20 naturel"
                        .to_string(),
                );
            }
        }
        Some("enchantement") => {
            l.push(format!("Enchantement — Sourire enchanteur : +{} aux tests de Bluff, de Diplomatie et d'Intimidation (appliqué à la fiche)", (2 + cinq).min(6)));
        }
        Some("evocation") => {
            l.push(format!("Évocation — Sorts intenses : +{moitie} aux dégâts de chaque sort infligeant des dégâts"));
            l.push(format!(
                "Évocation — Projectile de force : 1d4 + {moitie} de dégâts de force"
            ));
        }
        Some("illusion") => {
            l.push(format!("Illusion — Illusions durables : {moitie} round(s) supplémentaire(s) après la fin de la concentration"));
        }
        Some("invocation") => {
            l.push(format!("Invocation — Maîtrise des invocations : +{moitie} round(s) de durée aux sorts de convocation"));
            l.push(format!(
                "Invocation — Fléchette d'acide : 1d6 + {} de dégâts d'acide",
                niveau / 2
            ));
        }
        Some("necromancie") => {
            l.push(format!(
                "Nécromancie — Toucher du tombeau : secoué {moitie} round(s)"
            ));
            if niveau >= 8 {
                let portee = if niveau >= 12 {
                    3 + 3 * (1 + (niveau - 12) / 4)
                } else {
                    3
                };
                l.push(format!("Nécromancie — Perception de la vie : vision aveugle de {portee} m pendant {niveau} round(s) par jour"));
            }
        }
        Some("transmutation") => {
            l.push(format!(
                "Transmutation — Amélioration physique : +{} (altération) à une caractéristique physique choisie en préparant les sorts{}",
                (1 + cinq).min(5),
                if niveau >= 20 { ", à deux caractéristiques" } else { "" }
            ));
            l.push(format!(
                "Transmutation — Poing télékinétique : 1d4 + {} de dégâts contondants",
                niveau / 2
            ));
        }
        _ => {}
    }
    if fiche.pacte_magique.as_deref() == Some("familier") {
        let ligne = r
            .choix
            .pacte_magique_magicien
            .familiers
            .iter()
            .find(|x| Some(x.nom.as_str()) == fiche.familier.as_deref());
        if let Some(fam) = ligne {
            l.push(format!("Familier ({}) : {}", fam.nom, fam.capacite));
            let (armure, int, special) = match niveau {
                1..=2 => (
                    1,
                    6,
                    "esquive extraordinaire, lien empathique, transfert de sorts, vigilance",
                ),
                3..=4 => (2, 7, "conduit"),
                5..=6 => (3, 8, "communication avec le maître"),
                7..=8 => (4, 9, "communication avec les animaux de la même espèce"),
                9..=10 => (5, 10, "—"),
                11..=12 => (6, 11, "résistance à la magie"),
                13..=14 => (7, 12, "scrutation sur le familier"),
                15..=16 => (8, 13, "—"),
                17..=18 => (9, 14, "—"),
                _ => (10, 15, "—"),
            };
            l.push(format!(
                "Familier : armure naturelle +{armure}, Intelligence {int}{} (capacités cumulatives, voir le texte)",
                if special == "—" { String::new() } else { format!(", nouveauté : {special}") }
            ));
            if niveau >= 11 {
                l.push(format!("Familier : résistance à la magie {}", niveau + 5));
            }
        }
    } else if let Some(objet) = fiche.objet_fetiche.as_deref() {
        l.push(format!("Objet fétiche ({objet}) : un sort du grimoire par jour sans préparation, hors écoles opposées"));
    }
    l
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Versatilite {
    pub premier_niveau: u32,
    pub puis_tous_les: u32,
    pub puis_a_partir_du: u32,
    pub formes: Vec<FormeDeRepresentation>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct FormeDeRepresentation {
    pub nom: String,
    pub competences: Vec<String>,
}

impl Versatilite {
    /// Formes de Représentation choisies au niveau de barde `niveau` : une au niveau 2, une de plus
    /// au niveau 6 puis tous les quatre niveaux.
    pub fn nombre(&self, niveau: u32) -> u32 {
        if self.premier_niveau == 0 || niveau < self.premier_niveau {
            return 0;
        }
        let plus = if self.puis_tous_les == 0 || niveau < self.puis_a_partir_du {
            0
        } else {
            (niveau - self.puis_a_partir_du) / self.puis_tous_les + 1
        };
        1 + plus
    }
}

/// Compagnon animal obtenu par une classe autre que le druide (rôdeur, paladin).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SourceDeCompagnon {
    pub niveau: u32,
    /// Ajouté au niveau de classe pour obtenir le niveau de druide effectif.
    pub decalage: i32,
    pub animaux: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PacteDivin {
    pub niveau: u32,
    pub formes: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MoineMainsNues {
    pub tranches: Vec<(u32, u32)>,
    pub petite: Vec<String>,
    pub moyenne: Vec<String>,
    pub grande: Vec<String>,
    /// Bonus d'attaque du déluge de coups, niveau par niveau (taille M).
    pub deluge: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EntrainementAuxArmes {
    pub premier_niveau: u32,
    pub tous_les: u32,
    pub groupes: Vec<GroupeDArmes>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GroupeDArmes {
    pub nom: String,
    pub armes: Vec<String>,
}

/// Nom d'arme comparable : minuscules, apostrophes droites, sans virgule.
pub fn nom_d_arme(nom: &str) -> String {
    nom.to_lowercase()
        .replace('’', "'")
        .replace(',', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EnvironnementsRodeur {
    pub choix_aux_niveaux: Vec<u32>,
    pub types: Vec<String>,
    pub bonus_de_base: i32,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PacteDuChasseur {
    pub niveau: u32,
    pub formes: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GracePaladin {
    pub premier_niveau: u32,
    pub choix_tous_les: u32,
    pub listes: Vec<ListeGraces>,
    /// Grâce → grâce qu'il faut avoir déjà choisie.
    pub prerequis_de_grace: BTreeMap<String, String>,
    /// Texte du livre de chaque grâce.
    pub textes: BTreeMap<String, String>,
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
    /// Talent → Intelligence minimale.
    pub prerequis_intelligence: BTreeMap<String, i32>,
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

    /// Nombre de choix d'environnement de prédilection acquis au niveau de rôdeur `niveau`.
    pub fn nombre_environnements(&self, niveau: u32) -> u32 {
        self.environnements_rodeur
            .choix_aux_niveaux
            .iter()
            .filter(|n| **n <= niveau)
            .count() as u32
    }

    /// Groupes d'armes à choisir au niveau de guerrier `niveau` (niveaux 5, 9, 13, 17).
    pub fn nombre_groupes_d_armes(&self, niveau: u32) -> u32 {
        let e = &self.entrainement_aux_armes_guerrier;
        if e.tous_les == 0 || niveau < e.premier_niveau {
            return 0;
        }
        (niveau - e.premier_niveau) / e.tous_les + 1
    }

    /// Bonus d'entraînement aux armes pour l'arme `nom` (nom français du catalogue) : le plus élevé
    /// des groupes choisis qui la contiennent. Le premier groupe choisi gagne +1 à chaque nouveau groupe.
    pub fn bonus_d_entrainement(&self, groupes_choisis: &[String], nom: &str) -> i32 {
        let cible = nom_d_arme(nom);
        let n = groupes_choisis.len() as i32;
        groupes_choisis
            .iter()
            .enumerate()
            .filter(|(_, g)| {
                self.entrainement_aux_armes_guerrier
                    .groupes
                    .iter()
                    .any(|x| &x.nom == *g && x.armes.iter().any(|a| nom_d_arme(a) == cible))
            })
            .map(|(i, _)| n - i as i32)
            .max()
            .unwrap_or(0)
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
        if self.talents_roublard.iter().any(|t| {
            regles
                .prerequis_intelligence
                .contains_key(nom_sans_etoile(t))
        }) {
            let intelligence = self.calculer(r).ok().and_then(|v| {
                v.scores
                    .get(&crate::caracteristiques::Carac::Int)
                    .map(|d| d.total)
            });
            if let Some(int) = intelligence {
                for t in &self.talents_roublard {
                    if let Some(&minimum) = regles.prerequis_intelligence.get(nom_sans_etoile(t)) {
                        if int < minimum {
                            p.push(Probleme::TalentIntelligenceInsuffisante {
                                nom: t.clone(),
                                minimum,
                            });
                        }
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

        let niveau_rodeur = self.niveau_de_classe("rodeur");
        let attendus = c.nombre_environnements(niveau_rodeur);
        let choisis = self.environnements.len() as u32;
        if attendus != choisis {
            p.push(Probleme::EnvironnementsAChoisir { attendus, choisis });
        }
        for (i, e) in self.environnements.iter().enumerate() {
            if !c.environnements_rodeur.types.contains(e) {
                p.push(Probleme::EnvironnementInconnu(e.clone()));
            }
            if self.environnements[..i].contains(e) {
                p.push(Probleme::EnvironnementEnDouble(e.clone()));
            }
        }
        let attendus = attendus.saturating_sub(1);
        let choisis = self.augmentations_environnements.len() as u32;
        if attendus != choisis {
            p.push(Probleme::AugmentationsEnvironnementsAChoisir { attendus, choisis });
        }
        for e in &self.augmentations_environnements {
            if !self.environnements.contains(e) {
                p.push(Probleme::EnvironnementInconnu(e.clone()));
            }
        }
        let pacte = &c.pacte_du_chasseur;
        if pacte.niveau > 0 && niveau_rodeur >= pacte.niveau {
            match &self.pacte_du_chasseur {
                None => p.push(Probleme::PacteDuChasseurManquant),
                Some(forme) if !pacte.formes.contains(forme) => {
                    p.push(Probleme::PacteDuChasseurInconnu(forme.clone()));
                }
                Some(_) => {}
            }
        }

        let attendus = c.nombre_groupes_d_armes(self.niveau_de_classe("guerrier"));
        let choisis = self.groupes_d_armes.len() as u32;
        if attendus != choisis {
            p.push(Probleme::GroupesDArmesAChoisir { attendus, choisis });
        }
        for (i, g) in self.groupes_d_armes.iter().enumerate() {
            if !c
                .entrainement_aux_armes_guerrier
                .groupes
                .iter()
                .any(|x| &x.nom == g)
            {
                p.push(Probleme::GroupeDArmesInconnu(g.clone()));
            }
            if self.groupes_d_armes[..i].contains(g) {
                p.push(Probleme::GroupeDArmesEnDouble(g.clone()));
            }
        }

        if self.niveau_de_classe("guerrier") >= 20 {
            match self.arme_de_maitrise.as_deref() {
                None => p.push(Probleme::ArmeDeMaitriseManquante),
                Some(a) => {
                    let cible = nom_d_arme(a);
                    let connue = r.equipement.armes.iter().any(|x| {
                        x.est_maniable()
                            && (nom_d_arme(&x.nom) == cible || nom_d_arme(&x.nom_en) == cible)
                    });
                    if !connue {
                        p.push(Probleme::ArmeDeMaitriseInconnue(a.to_string()));
                    }
                }
            }
        }

        let pacte = &c.pacte_divin_paladin;
        if pacte.niveau > 0 && self.niveau_de_classe("paladin") >= pacte.niveau {
            match &self.pacte_divin {
                None => p.push(Probleme::PacteDivinManquant),
                Some(forme) if !pacte.formes.contains(forme) => {
                    p.push(Probleme::PacteDivinInconnu(forme.clone()));
                }
                Some(_) => {}
            }
        }

        if let Some((source, niveau)) = compagnon_hors_druide(self, r) {
            match &self.compagnon {
                None => p.push(Probleme::CompagnonManquant),
                Some(comp) => {
                    if !source.animaux.contains(&comp.animal) {
                        p.push(Probleme::AnimalNonPermis(comp.animal.clone()));
                    }
                    let permises = crate::compagnon::augmentations_permises(&r.compagnons, niveau);
                    if comp.augmentations.len() as u32 > permises {
                        p.push(Probleme::AugmentationsDeTrop {
                            permises,
                            choisies: comp.augmentations.len() as u32,
                        });
                    }
                }
            }
        }

        let versatilite = &c.versatilite_artistique_barde;
        let attendus = versatilite.nombre(self.niveau_de_classe("barde"));
        let choisis = self.formes_de_representation.len() as u32;
        if attendus != choisis {
            p.push(Probleme::FormesDeRepresentationAChoisir { attendus, choisis });
        }
        for (i, forme) in self.formes_de_representation.iter().enumerate() {
            if !versatilite.formes.iter().any(|x| &x.nom == forme) {
                p.push(Probleme::FormeDeRepresentationInconnue(forme.clone()));
            }
            if self.formes_de_representation[..i].contains(forme) {
                p.push(Probleme::FormeDeRepresentationEnDouble(forme.clone()));
            }
        }

        if self.niveau_de_classe("magicien") >= 1 {
            let ecoles = &c.ecoles_magicien;
            match self.ecole.as_deref() {
                None => p.push(Probleme::EcoleManquante),
                Some(id) if ecoles.ecole(id).is_none() => {
                    p.push(Probleme::EcoleInconnue(id.to_string()));
                }
                Some(id) => {
                    let attendues = if id == "universelle" { 0 } else { 2 };
                    let choisies = self.ecoles_opposees.len() as u32;
                    if attendues != choisies {
                        p.push(Probleme::EcolesOpposeesAChoisir {
                            attendues,
                            choisies,
                        });
                    }
                    for (i, o) in self.ecoles_opposees.iter().enumerate() {
                        let valide = ecoles.ecole(o).is_some()
                            && o != "universelle"
                            && o != id
                            && !self.ecoles_opposees[..i].contains(o);
                        if !valide {
                            p.push(Probleme::EcoleOpposeeInvalide(o.clone()));
                        }
                    }
                }
            }
            let pacte = &c.pacte_magique_magicien;
            match self.pacte_magique.as_deref() {
                None => p.push(Probleme::PacteMagiqueManquant),
                Some(forme) if !pacte.formes.iter().any(|x| x == forme) => {
                    p.push(Probleme::PacteMagiqueInconnu(forme.to_string()));
                }
                Some("familier") => match self.familier.as_deref() {
                    None => p.push(Probleme::FamilierManquant),
                    Some(nom) if !pacte.familiers.iter().any(|x| x.nom == nom) => {
                        p.push(Probleme::FamilierInconnu(nom.to_string()));
                    }
                    Some(_) => {}
                },
                Some(_) => match self.objet_fetiche.as_deref() {
                    None => p.push(Probleme::ObjetFetichManquant),
                    Some(o) if !pacte.objets_fetiches.iter().any(|x| x == o) => {
                        p.push(Probleme::ObjetFetichInconnu(o.to_string()));
                    }
                    Some(_) => {}
                },
            }
        }

        if self.niveau_de_classe("magicien") >= 1 {
            let attendus = sorts_attendus_dans_le_grimoire(self, r);
            let choisis = self.grimoire.len() as u32;
            if choisis < attendus {
                p.push(Probleme::GrimoireIncomplet { attendus, choisis });
            }
            let maximal = niveau_maximal_de_sort(self, r);
            for (i, nom) in self.grimoire.iter().enumerate() {
                match r.sort(nom).and_then(|x| x.niveau_pour("wizard")) {
                    None | Some(0) => p.push(Probleme::SortDuGrimoireInconnu(nom.clone())),
                    Some(n) if n > maximal => p.push(Probleme::SortDuGrimoireInaccessible {
                        sort: nom.clone(),
                        niveau: n,
                    }),
                    Some(_) => {}
                }
                if self.grimoire[..i].contains(nom) {
                    p.push(Probleme::SortDuGrimoireEnDouble(nom.clone()));
                }
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
            "Inspiration vaillante : +{b} aux jets d'attaque et de dégâts avec des armes (aptitude), +{b} aux jets de sauvegarde contre la terreur et le charme (moral)"
        ));
    }
    if barde >= 3 {
        let b = 2 + (barde - 3) / 4;
        l.push(format!(
            "Inspiration talentueuse : +{b} aux tests d'une compétence au choix d'un allié (aptitude)"
        ));
    }
    if barde >= 9 {
        let cibles = (1 + (barde - 9) / 3).min(4);
        l.push(format!(
            "Inspiration glorieuse : {cibles} cible(s), 2 DV supplémentaires (d10), +2 aux jets d'attaque et +1 aux jets de Vigueur (aptitude)"
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
    for (i, e) in fiche.environnements.iter().enumerate() {
        if fiche.environnements[..i].contains(e) {
            continue;
        }
        let augmentations = fiche
            .augmentations_environnements
            .iter()
            .filter(|a| *a == e)
            .count() as u32;
        l.push(format!(
            "Environnement de prédilection ({e}) : +{} aux jets d'initiative et aux tests de Connaissances (géographie), Discrétion, Perception et Survie ; aucune trace laissée",
            2 + 2 * augmentations
        ));
    }
    if let Some(forme) = &fiche.pacte_du_chasseur {
        l.push(format!("Pacte du chasseur : {forme}"));
    }
    let roublard = fiche.niveau_de_classe("roublard");
    if roublard >= 1 {
        l.push(format!(
            "Recherche des pièges : +{} aux tests de Perception pour repérer les pièges et aux tests de Sabotage",
            (roublard / 2).max(1)
        ));
    }
    let barbare = fiche.niveau_de_classe("barbare");
    for (i, p) in fiche.pouvoirs_rage.iter().enumerate() {
        if fiche.pouvoirs_rage[..i].contains(p) {
            continue;
        }
        if let Some(v) = valeur_du_pouvoir_de_rage(p, barbare) {
            l.push(format!("Pouvoir de rage ({p}) : {v}"));
        }
    }
    // Les bonus de sens des pièges de classes différentes se cumulent (Manuel des joueurs).
    let sens = roublard / 3 + barbare / 3;
    if sens >= 1 {
        l.push(format!(
            "Sens des pièges : +{sens} aux jets de Réflexes contre les pièges et +{sens} à la CA (esquive) contre leurs attaques"
        ));
    }
    if barbare >= 7 {
        l.push(format!("Réduction de dégâts : {}/—", 1 + (barbare - 7) / 3));
    }
    let guerrier_niveau = fiche.niveau_de_classe("guerrier");
    if guerrier_niveau >= 3 {
        let n = (1 + (guerrier_niveau - 3) / 4).min(4);
        l.push(format!(
            "Entraînement aux armures : −{n} au malus d'armure, +{n} au bonus de DEX maximal (appliqués à la fiche)"
        ));
    }
    if guerrier_niveau >= 5 {
        let n = fiche.groupes_d_armes.len() as u32;
        for (i, g) in fiche.groupes_d_armes.iter().enumerate() {
            l.push(format!(
                "Entraînement aux armes ({g}) : +{} aux jets d'attaque et de dégâts avec ces armes (appliqué aux armes du groupe de la fiche)",
                n - i as u32
            ));
        }
    }
    if guerrier_niveau >= 20 {
        let arme = fiche.arme_de_maitrise.as_deref().unwrap_or("à choisir");
        l.push(format!(
            "Maîtrise des armes ({arme}) : critiques automatiquement confirmés, multiplicateur de dégâts +1 (appliqué à la fiche), ne peut plus être désarmé avec cette arme"
        ));
    }
    if guerrier_niveau >= 19 {
        l.push(
            "Maîtrise des armures : réduction de dégâts 5/— avec une armure ou un bouclier"
                .to_string(),
        );
    }
    let rodeur = fiche.niveau_de_classe("rodeur");
    if rodeur >= 1 {
        l.push(format!(
            "Pistage : +{} aux tests de Survie pour suivre des traces",
            (rodeur / 2).max(1)
        ));
    }
    if rodeur >= 11 {
        l.push(format!(
            "Proie du chasseur : bonus d'intuition de +{} aux jets d'attaque contre la proie, critiques confirmés automatiquement",
            if rodeur >= 19 { 4 } else { 2 }
        ));
    }
    for forme in &fiche.formes_de_representation {
        l.push(format!("Versatilité artistique ({forme}) : le bonus total de Représentation remplace celui de la compétence associée"));
    }
    if barde >= 5 {
        l.push(format!(
            "Maître du savoir : faire 10 aux compétences de Connaissances où le barde a des rangs ; faire 20 {} fois par jour (action simple)",
            (1 + (barde - 5) / 6).min(3)
        ));
    }
    if barde >= 7 {
        l.push(format!(
            "Représentation bardique : activer un effet demande {}",
            if barde >= 13 {
                "une action rapide"
            } else {
                "une action de mouvement"
            }
        ));
    }
    if barde >= 10 {
        l.push(format!(
            "Touche-à-tout : toutes les compétences peuvent être utilisées sans formation{}{}",
            if barde >= 16 {
                " ; toutes sont des compétences de classe"
            } else {
                ""
            },
            if barde >= 19 {
                " ; faire 10 à toutes les compétences"
            } else {
                ""
            }
        ));
    }
    if barde >= 2 {
        l.push("Érudition : +4 aux jets de sauvegarde contre les représentations bardiques, les effets de son et les effets de langage".to_string());
    }
    let paladin = fiche.niveau_de_classe("paladin");
    if paladin >= 5 {
        match fiche.pacte_divin.as_deref() {
            Some("arme liée") => l.push(format!(
                "Pacte divin (arme liée) : esprit céleste pendant {paladin} minute(s), bonus d'altération de +{} à l'arme (jusqu'à +5 cumulés avec ceux qu'elle a déjà), ou propriétés au même coût ; {} utilisation(s) par jour",
                (1 + (paladin - 5) / 3).min(6),
                1 + (paladin - 5) / 4
            )),
            Some("destrier") => l.push(format!(
                "Pacte divin (destrier) : compagnon animal de niveau de druide effectif {paladin}, appelé par une action complexe ; {} appel(s) par jour{}{}",
                1 + (paladin - 5) / 4,
                if paladin >= 11 { " ; archétype céleste" } else { "" },
                if paladin >= 15 { format!(" ; résistance à la magie {}", paladin + 11) } else { String::new() }
            )),
            _ => {}
        }
    }
    let moine = fiche.niveau_de_classe("moine");
    if moine >= 1 {
        let extra = if moine >= 15 {
            3
        } else if moine >= 8 {
            2
        } else {
            1
        };
        l.push(format!(
            "Déluge de coups : {extra} attaque(s) supplémentaire(s) lors d'une attaque à outrance, malus de −2 à toutes les attaques du round ; bonus de base à l'attaque égal au niveau de moine ({moine}) ; bonus de Force entier aux dégâts"
        ));
    }
    if barbare >= 14 {
        l.push("Volonté indomptable : +4 aux jets de Volonté contre les sorts d'Enchantement tant que le barbare est en rage (se cumule avec le bonus de moral de la rage)".to_string());
    }
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

/// Genre de l'armure portée, s'il y en a une et que le catalogue la connaît.
fn genre_de_l_armure(fiche: &Fiche, r: &Referentiel) -> Option<crate::equipement::GenreProtection> {
    fiche
        .equipement
        .armure
        .as_deref()
        .and_then(|n| r.equipement.armure(n))
        .map(|a| a.genre)
}

/// Vrai si le barbare profite du déplacement accéléré : aucune armure, légère ou intermédiaire,
/// et pas de charge lourde (Manuel des joueurs, Barbare).
pub fn deplacement_accelere_du_barbare(
    fiche: &Fiche,
    r: &Referentiel,
    charge: &crate::equipement::Charge,
) -> bool {
    use crate::equipement::{Charge, GenreProtection};
    !matches!(genre_de_l_armure(fiche, r), Some(GenreProtection::Lourde))
        && matches!(charge, Charge::Legere | Charge::Intermediaire)
}

/// Effets chiffrés qui dépendent d'un modificateur de caractéristique ou de l'équipement :
/// grâce divine du paladin (bonus de Charisme à tous les jets de sauvegarde, dès le niveau 2) et
/// bonus à la CA et au DMD du moine (Sagesse + bonus de niveau, sans armure, sans bouclier, sans
/// charge intermédiaire ou lourde).
pub fn effets_dependant_des_caracs(
    fiche: &Fiche,
    r: &Referentiel,
    modificateur: &dyn Fn(crate::caracteristiques::Carac) -> i32,
    charge: &crate::equipement::Charge,
) -> Vec<crate::effet::Effet> {
    use crate::caracteristiques::Carac;
    use crate::effet::{Effet, TypeBonus};
    let mut e = Vec::new();
    if fiche.niveau_de_classe("paladin") >= 2 {
        let cha = modificateur(Carac::Cha);
        if cha > 0 {
            for cible in ["jds.vig", "jds.ref", "jds.vol"] {
                e.push(Effet::new(cible, TypeBonus::SansType, cha, "Grâce divine"));
            }
        }
    }
    let moine = fiche.niveau_de_classe("moine");
    if moine >= 1
        && fiche.equipement.armure.is_none()
        && fiche.equipement.bouclier.is_none()
        && matches!(charge, crate::equipement::Charge::Legere)
    {
        let niveau = if moine >= 4 { 1 + (moine - 4) / 4 } else { 0 } as i32;
        let total = modificateur(Carac::Sag).max(0) + niveau;
        if total > 0 {
            for cible in ["ca", "dmd"] {
                e.push(Effet::new(
                    cible,
                    TypeBonus::SansType,
                    total,
                    "Bonus à la CA (moine)",
                ));
            }
        }
    }
    let _ = r;
    e
}

/// Entraînement aux armures du guerrier (Manuel des joueurs) : dès le niveau 3, le malus d'armure
/// aux compétences diminue de 1 et le bonus de DEX maximal augmente de 1, puis de 1 aux niveaux 7,
/// 11 et 15 (jusqu'à −4 et +4). Le guerrier se déplace à sa vitesse normale en armure
/// intermédiaire, et en armure lourde dès le niveau 7 (la charge reste prise en compte).
pub fn appliquer_entrainement_aux_armures(
    fiche: &Fiche,
    r: &Referentiel,
    bilan: &mut crate::equipement::BilanEquipement,
    vitesse_sans_armure: u32,
) {
    use crate::equipement::{Charge, GenreProtection};
    let niveau = fiche.niveau_de_classe("guerrier");
    let genre = genre_de_l_armure(fiche, r);
    if niveau < 3 || genre.is_none() {
        return;
    }
    let n = (1 + (niveau - 3) / 4).min(4) as i32;
    bilan.malus_d_armure.0 = (bilan.malus_d_armure.0 + n).min(0);
    if let Some((plafond, _)) = bilan.dex_max.as_mut() {
        *plafond += n;
    }
    let libre = match genre {
        Some(GenreProtection::Intermediaire) => true,
        Some(GenreProtection::Lourde) => niveau >= 7,
        _ => false,
    };
    if libre && matches!(bilan.charge, Charge::Legere) {
        bilan.vitesse = bilan.vitesse.max(vitesse_sans_armure);
    }
}

/// Lignes qui dépendent de la race du personnage : dégâts à mains nues du moine (tailles P et G).
pub fn lignes_dependant_de_la_race(fiche: &Fiche, r: &Referentiel) -> Vec<String> {
    let moine = fiche.niveau_de_classe("moine");
    if moine == 0 {
        return Vec::new();
    }
    let t = &r.choix.moine_mains_nues;
    let (colonne, taille) = match r.race(&fiche.race).map(|x| x.taille.as_str()) {
        Some("petite") => (&t.petite, "P"),
        Some("grande") => (&t.grande, "G"),
        _ => (&t.moyenne, "M"),
    };
    let mut lignes = Vec::new();
    if let Some(d) = t
        .tranches
        .iter()
        .position(|(a, b)| moine >= *a && moine <= *b)
        .and_then(|i| colonne.get(i))
    {
        lignes.push(format!("Dégâts à mains nues (taille {taille}) : {d}"));
    }
    if let Some(b) = t.deluge.get(moine as usize - 1) {
        lignes.push(format!(
            "Déluge de coups, jets d'attaque (table 3-11, taille M) : {b}"
        ));
    }
    lignes
}

/// Bonus de vitesse (en pieds) du déplacement accéléré du moine : +3 m (10 pieds) tous les 3
/// niveaux à partir du niveau 3, perdu avec une armure ou une charge intermédiaire ou lourde.
pub fn deplacement_accelere_du_moine(fiche: &Fiche, charge: &crate::equipement::Charge) -> u32 {
    let moine = fiche.niveau_de_classe("moine");
    if moine < 3
        || fiche.equipement.armure.is_some()
        || !matches!(charge, crate::equipement::Charge::Legere)
    {
        return 0;
    }
    10 * (moine / 3)
}

/// Bonus total de déplacement accéléré (barbare et moine), en pieds, avant la réduction due à
/// l'armure et à la charge.
pub fn bonus_de_deplacement_accelere(
    fiche: &Fiche,
    r: &Referentiel,
    charge: &crate::equipement::Charge,
) -> u32 {
    let barbare = if fiche.niveau_de_classe("barbare") >= 1
        && deplacement_accelere_du_barbare(fiche, r, charge)
    {
        10
    } else {
        0
    };
    barbare + deplacement_accelere_du_moine(fiche, charge)
}

/// Critique d'une arme dont le multiplicateur augmente de 1 (maîtrise des armes du guerrier) :
/// « 19-20/x2 » devient « 19-20/x3 », « x3/x4 » devient « x4/x5 ».
pub fn critique_majore(critique: &str) -> String {
    let mut sortie = String::new();
    let mut chars = critique.chars().peekable();
    while let Some(c) = chars.next() {
        sortie.push(c);
        if c == 'x' || c == '×' {
            let mut chiffres = String::new();
            while let Some(d) = chars.peek().copied().filter(char::is_ascii_digit) {
                chiffres.push(d);
                chars.next();
            }
            match chiffres.parse::<u32>() {
                Ok(n) => sortie.push_str(&(n + 1).to_string()),
                Err(_) => sortie.push_str(&chiffres),
            }
        }
    }
    sortie
}

/// Vrai si l'arme `nom` (nom français ou anglais du catalogue) est l'arme de maîtrise du guerrier.
pub fn est_l_arme_de_maitrise(fiche: &Fiche, def: &crate::equipement::ArmeDef) -> bool {
    fiche.niveau_de_classe("guerrier") >= 20
        && fiche.arme_de_maitrise.as_deref().is_some_and(|a| {
            let cible = nom_d_arme(a);
            nom_d_arme(&def.nom) == cible || nom_d_arme(&def.nom_en) == cible
        })
}

/// Valeur, au niveau de barbare `niveau`, du pouvoir de rage `nom` quand le texte du livre donne
/// un chiffre qui progresse avec le niveau (affichage seulement). Les formules « +1, +1 par tranche
/// de quatre niveaux » sont lues « 1 + niveau ÷ 4 » : lecture à valider.
pub fn valeur_du_pouvoir_de_rage(nom: &str, niveau: u32) -> Option<String> {
    let quatre = 1 + niveau / 4;
    Some(match nom {
        "Chasseur de sorcière" => format!(
            "+{quatre} aux jets de dégâts contre les créatures qui possèdent des sorts ou des pouvoirs magiques"
        ),
        "Coup puissant" => format!(
            "+{quatre} à un jet de dégâts (action rapide avant l'attaque, une fois par rage)"
        ),
        "Précision étonnante" => format!(
            "+{quatre} (moral) à un jet d'attaque (action rapide avant l'attaque, une fois par rage)"
        ),
        "Imprudente désinvolture" => {
            format!("−{quatre} à la CA pour +{quatre} aux jets d'attaque, si le barbare le choisit")
        }
        "Animal totem" => format!("bonus d'armure naturelle de +{quatre} en rage"),
        "Buveur prodigieux" => format!(
            "bonus d'esquive à la CA contre les attaques d'opportunité : +1 par dose d'alcool, jusqu'à +{}",
            niveau / 4
        ),
        "Posture défensive" => format!(
            "bonus d'esquive de +{} à la CA contre les attaques au corps à corps (rounds égaux au modificateur de Constitution, au moins 1)",
            1 + niveau / 6
        ),
        "Pic de force" => format!(
            "+{niveau} à un jet de Force, un test de manœuvre offensive ou la défense contre une manœuvre (une fois par rage)"
        ),
        "Résistance à l'énergie" => format!(
            "résistance de {} au type d'énergie choisi (acide, froid, électricité, feu ou son)",
            (niveau / 2).max(1)
        ),
        "Nouvelle vigueur" if niveau >= 4 => format!(
            "soigne {}d8 + modificateur de Constitution (une fois par jour, action simple, en rage)",
            (1 + (niveau - 4) / 4).min(5)
        ),
        "Réduction de dégâts accrue" => "réduction de dégâts +1/— par choix, tant que le barbare est en rage".to_string(),
        "Pas rapides" => "vitesse +1,50 m par choix (appliquée à la fiche en rage)".to_string(),
        "Escalade en rage" => format!("+{niveau} (altération) aux tests d'Escalade en rage (appliqué à la fiche)"),
        "Nage en rage" => format!("+{niveau} (altération) aux tests de Natation en rage (appliqué à la fiche)"),
        _ => return None,
    })
}

/// Compagnon animal du rôdeur ou destrier du paladin : animaux permis et niveau de druide
/// effectif, si le pacte choisi donne un compagnon et que le druide n'en a pas déjà un.
pub fn compagnon_hors_druide<'a>(
    fiche: &Fiche,
    r: &'a Referentiel,
) -> Option<(&'a SourceDeCompagnon, u32)> {
    if fiche.pacte.as_deref() == Some("compagnon") {
        return None;
    }
    let c = &r.choix;
    let rodeur = fiche.niveau_de_classe("rodeur");
    if fiche.pacte_du_chasseur.as_deref() == Some("compagnon animal")
        && rodeur >= c.compagnon_rodeur.niveau
    {
        let niveau = (rodeur as i32 + c.compagnon_rodeur.decalage).max(1) as u32;
        return Some((&c.compagnon_rodeur, niveau));
    }
    let paladin = fiche.niveau_de_classe("paladin");
    if fiche.pacte_divin.as_deref() == Some("destrier") && paladin >= c.compagnon_destrier.niveau {
        let niveau = (paladin as i32 + c.compagnon_destrier.decalage).max(1) as u32;
        return Some((&c.compagnon_destrier, niveau));
    }
    None
}

/// Nom de l'école dans le catalogue des sorts d'après l'identifiant d'école du magicien.
pub fn ecole_du_catalogue(id: &str) -> &'static str {
    match id {
        "abjuration" => "abjuration",
        "divination" => "divination",
        "enchantement" => "enchantment",
        "evocation" => "evocation",
        "illusion" => "illusion",
        "invocation" => "conjuration",
        "necromancie" => "necromancy",
        "transmutation" => "transmutation",
        _ => "universal",
    }
}

/// Écoles interdites du magicien spécialiste, avec les noms du catalogue des sorts.
pub fn ecoles_opposees_du_catalogue(fiche: &Fiche) -> Vec<&'static str> {
    fiche
        .ecoles_opposees
        .iter()
        .map(|e| ecole_du_catalogue(e))
        .collect()
}

/// Plus haut niveau de sort accessible au magicien à son niveau actuel (0 s'il n'a que des tours de magie).
pub fn niveau_maximal_de_sort(fiche: &Fiche, r: &Referentiel) -> u32 {
    let nc = fiche.niveau_de_classe("magicien") as usize;
    if nc == 0 {
        return 0;
    }
    r.classe("magicien")
        .and_then(|c| c.progression.get(nc - 1))
        .map_or(0, |l| {
            l.sorts_par_jour
                .iter()
                .rposition(Option::is_some)
                .map_or(0, |i| i as u32 + 1)
        })
}

/// Sorts que le grimoire doit contenir en plus des tours de magie : trois sorts de niveau 1, un de
/// plus par point de bonus d'Intelligence, puis deux nouveaux sorts à chaque niveau gagné.
pub fn sorts_attendus_dans_le_grimoire(fiche: &Fiche, r: &Referentiel) -> u32 {
    let nc = fiche.niveau_de_classe("magicien");
    if nc == 0 {
        return 0;
    }
    let bonus = fiche
        .calculer(r)
        .ok()
        .and_then(|v| {
            v.modificateurs
                .get(&crate::caracteristiques::Carac::Int)
                .copied()
        })
        .unwrap_or(0)
        .max(0) as u32;
    3 + bonus + 2 * (nc - 1)
}

/// Tours de magie du grimoire : tous ceux de la liste du magicien, sauf ceux d'une école interdite.
pub fn tours_de_magie_du_grimoire<'a>(fiche: &Fiche, r: &'a Referentiel) -> Vec<&'a str> {
    let interdites = ecoles_opposees_du_catalogue(fiche);
    r.sorts
        .iter()
        .filter(|x| x.niveau_pour("wizard") == Some(0))
        .filter(|x| !interdites.contains(&x.ecole.as_str()))
        .map(|x| x.nom_en.as_str())
        .collect()
}
