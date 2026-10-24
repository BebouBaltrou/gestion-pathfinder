//! Gabarit de la feuille « Pathfinder 1 — fiche française remplissable » (6 pages) :
//! transforme les champs bruts du PDF en valeurs lues, avec leur nom de champ d'origine.

use crate::lecteur::Champs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaracLue {
    pub nom: &'static str,
    pub id: &'static str,
    /// Colonne « Val » (score total écrit sur la feuille).
    pub val: Option<i32>,
    pub base: Option<i32>,
    pub alteration: Option<i32>,
    pub modificateur: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompetenceLue {
    /// Clé du moteur (`bluff`, `knowledge:arcana`, `craft:alchimie`…).
    pub cle: String,
    /// Libellé tel qu'imprimé sur la feuille (+ sous-type écrit à la main).
    pub libelle: String,
    pub champ: String,
    pub total: Option<i32>,
    pub rangs: Option<i32>,
    /// Bonus de compétence de classe (+3) inscrit.
    pub classe: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Feuille {
    pub champs_lus: usize,
    pub nom: String,
    pub race: String,
    pub classes_et_niveaux: String,
    /// Autre case de la feuille qui répète classe et niveau (`ChampG.1`).
    pub classe_et_niveau_bis: String,
    pub caracs: Vec<CaracLue>,
    pub reflexes: Option<i32>,
    pub vigueur: Option<i32>,
    pub volonte: Option<i32>,
    pub base_reflexes: Option<i32>,
    pub base_vigueur: Option<i32>,
    pub base_volonte: Option<i32>,
    pub pv_max: Option<i32>,
    pub initiative: Option<i32>,
    pub bba_texte: String,
    pub ca: Option<i32>,
    pub ca_contact: Option<i32>,
    pub ca_depourvu: Option<i32>,
    pub competences: Vec<CompetenceLue>,
    pub sorts: Vec<String>,
    pub notes_dons: String,
    pub richesse_po: Option<i32>,
}

/// Entier lu dans une case (`+13`, `-2` ; `8/3` donne `8`).
pub fn entier(s: &str) -> Option<i32> {
    let t = s.trim().trim_start_matches('+');
    let t = t.split('/').next()?.trim();
    t.parse().ok()
}

fn texte(c: &Champs, nom: &str) -> String {
    match c.get(nom).map(|s| s.trim()) {
        Some("undefined") | None => String::new(),
        Some(s) => s.to_string(),
    }
}

fn nombre(c: &Champs, nom: &str) -> Option<i32> {
    c.get(nom).and_then(|s| entier(s))
}

/// (ligne, colonne, clé du moteur, libellé imprimé, champ « sous-type » écrit à la main).
type Ligne = (
    usize,
    usize,
    &'static str,
    &'static str,
    Option<&'static str>,
);

const LIGNES: &[Ligne] = &[
    (0, 0, "acrobatics", "Acrobaties", None),
    (0, 1, "climb", "Escalade", None),
    (1, 0, "spellcraft", "Art de la magie", None),
    (1, 1, "sleight_of_hand", "Escamotage", None),
    (2, 0, "craft", "Artisanat", Some("Compdom.0")),
    (2, 1, "appraise", "Estimation", None),
    (3, 0, "craft", "Artisanat", Some("Compdom.1")),
    (3, 1, "escape_artist", "Évasion", None),
    (4, 0, "bluff", "Bluff", None),
    (4, 1, "intimidate", "Intimidation", None),
    (
        5,
        0,
        "knowledge:dungeoneering",
        "Connaissances (exploration souterraine)",
        None,
    ),
    (5, 1, "linguistics", "Linguistique", None),
    (
        6,
        0,
        "knowledge:local",
        "Connaissances (folklore local)",
        None,
    ),
    (6, 1, "swim", "Natation", None),
    (
        7,
        0,
        "knowledge:geography",
        "Connaissances (géographie)",
        None,
    ),
    (7, 1, "perception", "Perception", None),
    (8, 0, "knowledge:history", "Connaissances (histoire)", None),
    (8, 1, "heal", "Premiers secours", None),
    (
        9,
        0,
        "knowledge:engineering",
        "Connaissances (ingénierie)",
        None,
    ),
    (9, 1, "profession", "Profession", Some("Compdom.3")),
    (10, 0, "knowledge:arcana", "Connaissances (mystères)", None),
    (10, 1, "profession", "Profession", Some("Compdom.4")),
    (11, 0, "knowledge:nature", "Connaissances (nature)", None),
    (11, 1, "sense_motive", "Psychologie", None),
    (
        12,
        0,
        "knowledge:nobility",
        "Connaissances (noblesse)",
        None,
    ),
    (12, 1, "perform", "Représentation", Some("Compdom.5")),
    (13, 0, "knowledge:planes", "Connaissances (plans)", None),
    (13, 1, "perform", "Représentation", Some("Compdom.6")),
    (
        14,
        0,
        "knowledge:religion",
        "Connaissances (religion)",
        None,
    ),
    (14, 1, "perform", "Représentation", Some("Compdom.7")),
    (15, 1, "perform", "Représentation", Some("Compdom.8.0")),
    (16, 0, "disguise", "Déguisement", None),
    (16, 1, "perform", "Représentation", Some("Compdom.8.1")),
    (17, 0, "diplomacy", "Diplomatie", None),
    (17, 1, "disable_device", "Sabotage", None),
    (18, 0, "stealth", "Discrétion", None),
    (18, 1, "survival", "Survie", None),
    (19, 0, "handle_animal", "Dressage", None),
    (
        19,
        1,
        "use_magic_device",
        "Utilisation d'objets magiques",
        None,
    ),
    (20, 0, "ride", "Équitation", None),
    (20, 1, "fly", "Vol", None),
];

const CARACS: [(&str, &str); 6] = [
    ("FOR", "for"),
    ("DEX", "dex"),
    ("CON", "con"),
    ("INT", "int"),
    ("SAG", "wis"),
    ("CHA", "cha"),
];

pub fn lire_feuille(c: &Champs) -> Feuille {
    let mut f = Feuille {
        champs_lus: c.len(),
        nom: texte(c, "Nom"),
        race: texte(c, "Race"),
        classes_et_niveaux: texte(c, "Classes&Niveaux"),
        classe_et_niveau_bis: texte(c, "ChampG.1"),
        ..Default::default()
    };
    for (i, (nom, id)) in CARACS.iter().enumerate() {
        let n = i + 1;
        f.caracs.push(CaracLue {
            nom,
            id,
            val: nombre(c, &format!("Val{n}")),
            base: nombre(c, &format!("Base{n}")),
            alteration: nombre(c, &format!("Alter{n}")),
            modificateur: nombre(c, &format!("Mod{n}")),
        });
    }
    // Sur la feuille : JdS1 = Réflexes, JdS2 = Vigueur, JdS3 = Volonté (ordre d'impression).
    f.reflexes = nombre(c, "JdS1");
    f.vigueur = nombre(c, "JdS2");
    f.volonte = nombre(c, "JdS3");
    f.base_reflexes = nombre(c, "JdSB1");
    f.base_vigueur = nombre(c, "JdSB2");
    f.base_volonte = nombre(c, "JdSB3");
    f.pv_max = nombre(c, "MaxPV");
    f.initiative = nombre(c, "Init");
    f.bba_texte = texte(c, "Combat.0.0");
    f.ca = nombre(c, "CombatCA.0");
    f.ca_contact = nombre(c, "CombatCA.1");
    f.ca_depourvu = nombre(c, "CombatCA.2");
    f.richesse_po = nombre(c, "Richesse.1.0");

    for &(ligne, col, cle, libelle, sous) in LIGNES {
        let champ = format!("Compétences.{ligne}.{col}");
        let total = nombre(c, &champ);
        let rangs = nombre(c, &format!("Comprg.{ligne}.{col}"));
        let classe = nombre(c, &format!("Compcla.{ligne}.{col}"));
        let sous_type = sous.map(|s| texte(c, s)).unwrap_or_default();
        // Ligne que la feuille n'a pas remplie : le moteur la calculerait, rien à comparer.
        if total.is_none() && rangs.is_none() && classe.is_none() {
            continue;
        }
        let (cle_complete, lib) = if sous.is_some() {
            let s = if sous_type.is_empty() {
                "?".to_string()
            } else {
                sous_type.to_lowercase()
            };
            (format!("{cle}:{s}"), format!("{libelle} ({sous_type})"))
        } else {
            (cle.to_string(), libelle.to_string())
        };
        f.competences.push(CompetenceLue {
            cle: cle_complete,
            libelle: lib,
            champ,
            total,
            rangs,
            classe,
        });
    }

    // Sorts connus : cases `Sorts.1.0.N` de la forme « 1 Armure de mage ».
    for n in 0..40 {
        let s = texte(c, &format!("Sorts.1.0.{n}"));
        if !s.is_empty() {
            f.sorts.push(s);
        }
    }
    f.notes_dons = texte(c, "Notes.2.3");
    f
}
