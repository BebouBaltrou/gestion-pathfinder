//! La fiche d'un personnage et le calcul de ses valeurs dérivées.
//!
//! La fiche ne stocke que des **choix** (scores de base, dés de vie obtenus, rangs placés…) ;
//! tout le reste est recalculé à chaque appel, donc un changement de CON ou d'INT se répercute
//! rétroactivement sur les points de vie et les rangs disponibles.

use crate::bonus::{calculer, Detail, Terme};
use crate::caracteristiques::{modificateur, Carac, Caracteristique};
use crate::donnees::{Classe, Competence, Referentiel};
use crate::effet::{Cible, Effet, TypeBonus};
use crate::equipement::{
    BilanEquipement, ContexteEquipement, Equipement, GenreProtection, Maitrises,
};
use crate::etats::EffetTemporaire;
use std::collections::BTreeMap;

/// Choix fait à chaque niveau pour la classe de prédilection (Core p. 31, « Favored Class »).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BonusPredilection {
    PointDeVie,
    Rang,
}

/// Un niveau pris (le niveau de personnage est le nombre de ces entrées ; le niveau de classe
/// est le nombre d'entrées de la même classe).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NiveauPris {
    pub classe: String,
    /// Valeur du dé de vie obtenue (au 1er niveau : le maximum), sans le modificateur de CON.
    pub de_vie_obtenu: u32,
    /// Rangs placés à ce niveau, par compétence (`acrobatics`, `knowledge:arcana`…).
    pub rangs: BTreeMap<String, u32>,
    pub predilection: Option<BonusPredilection>,
    /// Dons choisis à ce niveau (catalogue et prérequis : phase 4).
    #[cfg_attr(feature = "serde", serde(default))]
    pub dons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Fiche {
    pub nom: String,
    pub race: String,
    /// Caractéristique choisie pour le bonus libre (humain, demi-elfe, demi-orque).
    pub bonus_racial_libre: Option<Carac>,
    /// Scores avant ajustements raciaux.
    pub scores_base: BTreeMap<Carac, i32>,
    pub dommages: BTreeMap<Carac, i32>,
    pub drain: BTreeMap<Carac, i32>,
    /// Augmentations de caractéristique prises (niveaux 4, 8, 12, 16, 20) : niveau, caractéristique.
    pub augmentations: Vec<(u32, Carac)>,
    pub niveaux: Vec<NiveauPris>,
    /// Classes de prédilection (une, deux pour le demi-elfe).
    #[cfg_attr(feature = "serde", serde(default))]
    pub classes_de_predilection: Vec<String>,
    pub lignage: Option<String>,
    /// Sous-type choisi pour « Knowledge (any one) » du lignage arcanique.
    pub connaissance_de_lignage: Option<String>,
    /// Effets supplémentaires (objets, sorts, états…) ajoutés à ceux de la race.
    pub effets: Vec<Effet>,
    /// Sorts connus choisis (noms anglais du catalogue), hors sorts de lignage obtenus automatiquement.
    #[cfg_attr(feature = "serde", serde(default))]
    pub sorts_connus: Vec<String>,
    /// Ce que le personnage porte (armure, bouclier, armes, objets).
    #[cfg_attr(feature = "serde", serde(default))]
    pub equipement: Equipement,
    /// États actifs (identifiants de `data/core/etats.json`, ex. `shaken`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub etats: Vec<String>,
    /// Effets qui durent (sorts sur soi, potions…).
    #[cfg_attr(feature = "serde", serde(default))]
    pub temporaires: Vec<EffetTemporaire>,
    /// Divinité servie (identifiant de `data/core/divinites.json`), s'il y en a une.
    #[cfg_attr(feature = "serde", serde(default))]
    pub divinite: Option<String>,
    /// Alignement : `LB`, `NB`, `CB`, `LN`, `N`, `CN`, `LM`, `NM` ou `CM`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub alignement: Option<String>,
    /// Domaines choisis (identifiants de `data/core/domaines.json`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub domaines: Vec<String>,
    /// Énergie canalisée par un prêtre Neutre (sur l'axe Bien-Mal) : `positive` ou `negative`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub energie: Option<String>,
    /// Pacte avec la nature du druide : `domaine` (un des domaines permis, dans `domaines`) ou `compagnon`.
    #[cfg_attr(feature = "serde", serde(default))]
    pub pacte: Option<String>,
    /// Compagnon animal du druide (pacte `compagnon`).
    #[cfg_attr(feature = "serde", serde(default))]
    pub compagnon: Option<crate::compagnon::Compagnon>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probleme {
    DiviniteInconnue(String),
    AlignementManquant,
    AlignementInvalide(String),
    AlignementHorsDivinite {
        alignement: String,
        divinite: String,
    },
    DomainesAChoisir {
        attendus: u32,
        choisis: u32,
    },
    DomaineInconnu(String),
    DomaineEnDouble(String),
    DomaineHorsDivinite {
        domaine: String,
        divinite: String,
    },
    DomaineAlignement {
        domaine: String,
    },
    EnergieAChoisir,
    /// Pacte avec la nature du druide : choisir un domaine ou un compagnon animal.
    PacteManquant,
    PacteInvalide(String),
    DomaineHorsPacte {
        domaine: String,
    },
    CompagnonManquant,
    CompagnonInconnu(String),
    AugmentationsDeTrop {
        permises: u32,
        choisies: u32,
    },
    EquipementInconnu(String),
    MauvaisEmplacement {
        objet: String,
        attendu: &'static str,
    },
    ArmeNonManiable(String),
    /// Bonus magique hors de 1 à 5, ou sur un objet que le personnage ne porte pas.
    BonusMagiqueInvalide(String),
    EtatInconnu(String),
    SortInconnu(String),
    SortHorsListe {
        sort: String,
    },
    SortEnDouble(String),
    /// Sort déjà accordé automatiquement par le lignage : il ne se choisit pas une seconde fois.
    SortDeLignageChoisi(String),
    SortsAChoisir {
        classe: String,
        niveau_de_sort: u32,
        attendus: u32,
        choisis: u32,
    },
    CharismeInsuffisantPourSort {
        sort: String,
        niveau_de_sort: u32,
    },
    DonInconnu(String),
    DonEnDouble(String),
    DonImposeManquant(String),
    /// Le don supplémentaire de lignage doit venir de la liste du lignage.
    DonDeLignageManquant {
        niveau: u32,
        lignage: String,
    },
    DonsAuNiveau {
        niveau: u32,
        attendus: u32,
        choisis: u32,
    },
    RaceInconnue(String),
    ClasseInconnue(String),
    ChoixRacialManquant,
    LignageManquant,
    LignageInconnu(String),
    PremierNiveauPasAuMaximum {
        obtenu: u32,
        maximum: u32,
    },
    DeDeVieHorsLimites {
        niveau: u32,
        obtenu: u32,
        maximum: u32,
    },
    RangsDepasses {
        niveau: u32,
        places: u32,
        disponibles: u32,
    },
    RangAuDessusDuNiveau {
        competence: String,
        rangs: u32,
        niveau: u32,
    },
    AugmentationInvalide {
        niveau: u32,
    },
    CompetenceInconnue(String),
    /// Deux objets magiques portés au même emplacement du corps (deux anneaux sont permis).
    EmplacementOccupe {
        emplacement: String,
    },
    /// Armure ou bouclier interdit à la classe (druide : pas de métal).
    ObjetInterdit {
        classe: String,
        objet: String,
    },
}

impl Probleme {
    /// La fiche n'est pas fausse, seulement inachevée : il manque des dons ou des sorts à choisir.
    /// Une fiche importée d'un PDF peut être dans ce cas ; les dons et sorts se complètent ensuite.
    pub fn est_incomplet(&self) -> bool {
        match self {
            Probleme::DonsAuNiveau {
                attendus, choisis, ..
            }
            | Probleme::SortsAChoisir {
                attendus, choisis, ..
            }
            | Probleme::DomainesAChoisir { attendus, choisis } => choisis < attendus,
            Probleme::AlignementManquant
            | Probleme::EnergieAChoisir
            | Probleme::PacteManquant
            | Probleme::CompagnonManquant => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for Probleme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Probleme::RaceInconnue(r) => write!(f, "race inconnue : {r}"),
            Probleme::ClasseInconnue(c) => write!(f, "classe inconnue : {c}"),
            Probleme::ChoixRacialManquant => {
                write!(f, "choisir la caractéristique du bonus racial libre")
            }
            Probleme::LignageManquant => write!(f, "choisir un lignage d'ensorceleur"),
            Probleme::LignageInconnu(l) => write!(f, "lignage inconnu : {l}"),
            Probleme::PremierNiveauPasAuMaximum { obtenu, maximum } => {
                write!(
                    f,
                    "au 1er niveau le dé de vie est au maximum ({maximum}), pas {obtenu}"
                )
            }
            Probleme::DeDeVieHorsLimites {
                niveau,
                obtenu,
                maximum,
            } => write!(
                f,
                "niveau {niveau} : dé de vie {obtenu} hors de 1 à {maximum}"
            ),
            Probleme::RangsDepasses {
                niveau,
                places,
                disponibles,
            } => write!(
                f,
                "niveau {niveau} : {places} rangs placés pour {disponibles} disponibles"
            ),
            Probleme::RangAuDessusDuNiveau {
                competence,
                rangs,
                niveau,
            } => write!(
                f,
                "{competence} : {rangs} rangs, au-dessus du niveau {niveau} (rang maximum = niveau)"
            ),
            Probleme::AugmentationInvalide { niveau } => {
                write!(
                    f,
                    "augmentation de caractéristique impossible au niveau {niveau}"
                )
            }
            Probleme::CompetenceInconnue(c) => write!(f, "compétence inconnue : {c}"),
            Probleme::DiviniteInconnue(d) => write!(f, "divinité inconnue : {d}"),
            Probleme::AlignementManquant => write!(f, "choisir l'alignement du personnage"),
            Probleme::AlignementInvalide(a) => write!(f, "alignement invalide : {a}"),
            Probleme::AlignementHorsDivinite { alignement, divinite } => write!(
                f,
                "l'alignement {alignement} diffère de plus d'une catégorie de celui de {divinite}"
            ),
            Probleme::DomainesAChoisir { attendus, choisis } => {
                write!(f, "{attendus} domaine(s) à choisir, {choisis} choisi(s)")
            }
            Probleme::DomaineInconnu(d) => write!(f, "domaine inconnu : {d}"),
            Probleme::DomaineEnDouble(d) => write!(f, "domaine choisi deux fois : {d}"),
            Probleme::DomaineHorsDivinite { domaine, divinite } => {
                write!(f, "le domaine {domaine} n'est pas offert par {divinite}")
            }
            Probleme::DomaineAlignement { domaine } => {
                write!(f, "le domaine {domaine} est réservé à un alignement correspondant")
            }
            Probleme::PacteManquant => write!(
                f,
                "choisir le pacte avec la nature : un domaine ou un compagnon animal"
            ),
            Probleme::PacteInvalide(x) => write!(f, "pacte avec la nature invalide : {x}"),
            Probleme::DomaineHorsPacte { domaine } => write!(
                f,
                "le domaine {domaine} n'est pas offert par le pacte avec la nature"
            ),
            Probleme::CompagnonManquant => write!(f, "choisir le compagnon animal du druide"),
            Probleme::CompagnonInconnu(a) => write!(f, "animal compagnon inconnu : {a}"),
            Probleme::AugmentationsDeTrop { permises, choisies } => write!(
                f,
                "le compagnon n'a gagné que {permises} augmentation(s) de caractéristique, {choisies} choisie(s)"
            ),
            Probleme::EnergieAChoisir => {
                write!(f, "choisir l'énergie canalisée (positive ou négative)")
            }
            Probleme::ObjetInterdit { classe, objet } => write!(
                f,
                "{objet} est interdit à la classe {classe} : le porter fait perdre l'accès aux sorts et aux aptitudes surnaturelles ou magiques"
            ),
            Probleme::EmplacementOccupe { emplacement } => {
                write!(f, "deux objets magiques portés à l'emplacement « {emplacement} »")
            }
            Probleme::SortInconnu(s) => write!(f, "sort inconnu du catalogue : {s}"),
            Probleme::SortHorsListe { sort } => {
                write!(f, "{sort} n'est pas sur la liste de sorts de la classe")
            }
            Probleme::SortEnDouble(s) => write!(f, "sort choisi deux fois : {s}"),
            Probleme::SortDeLignageChoisi(s) => {
                write!(f, "{s} est déjà accordé par le lignage : choisir un autre sort")
            }
            Probleme::SortsAChoisir { classe, niveau_de_sort, attendus, choisis } => write!(
                f,
                "{classe} : {attendus} sort(s) connu(s) de niveau {niveau_de_sort} attendu(s), {choisis} choisi(s)"
            ),
            Probleme::CharismeInsuffisantPourSort { sort, niveau_de_sort } => write!(
                f,
                "{sort} : la caractéristique d'incantation doit valoir au moins {} (sort de niveau {niveau_de_sort})",
                10 + niveau_de_sort
            ),
            Probleme::DonInconnu(d) => write!(f, "don inconnu du catalogue : {d}"),
            Probleme::DonEnDouble(d) => write!(f, "don pris deux fois (non répétable) : {d}"),
            Probleme::DonDeLignageManquant { niveau, lignage } => write!(
                f,
                "niveau {niveau} : un des dons doit être un don du lignage {lignage}"
            ),
            Probleme::DonImposeManquant(d) => write!(f, "don imposé par la race manquant : {d}"),
            Probleme::EquipementInconnu(n) => write!(f, "équipement inconnu du catalogue : {n}"),
            Probleme::MauvaisEmplacement { objet, attendu } => {
                write!(f, "{objet} ne peut pas être porté comme {attendu}")
            }
            Probleme::ArmeNonManiable(n) => {
                write!(f, "{n} n'est pas une arme que l'on manie (munition ou engin de siège) : la ranger dans les objets")
            }
            Probleme::BonusMagiqueInvalide(n) => write!(
                f,
                "bonus magique invalide : {n} (de +1 à +{} sur une armure, un bouclier ou une arme portés)",
                crate::equipement::BONUS_MAGIQUE_MAX
            ),
            Probleme::EtatInconnu(n) => write!(f, "état inconnu : {n}"),
            Probleme::DonsAuNiveau { niveau, attendus, choisis } => {
                write!(f, "niveau {niveau} : {attendus} don(s) attendu(s), {choisis} choisi(s)")
            }
        }
    }
}

/// Bonus de sorts par jour pour un modificateur donné (Core p. 17, tableau 1-3) :
/// un sort de niveau N en plus si le modificateur est au moins N, puis un de plus par
/// tranche de 4 au-delà.
pub fn sorts_bonus(modificateur: i32, niveau_de_sort: u32) -> u32 {
    if niveau_de_sort == 0 || modificateur < niveau_de_sort as i32 {
        0
    } else {
        1 + (modificateur - niveau_de_sort as i32) as u32 / 4
    }
}

/// DD d'un sort : 10 + niveau du sort + modificateur de la caractéristique d'incantation de la
/// classe + 1 par don d'école applicable (École renforcée, École supérieure : ils se cumulent).
pub fn dd_sort(niveau_du_sort: u32, modificateur: i32, dons_ecole: u32) -> i32 {
    10 + niveau_du_sort as i32 + modificateur + dons_ecole as i32
}

/// Nombre de dons École renforcée / École supérieure pris pour `ecole` (nom anglais en minuscules,
/// comme dans le catalogue des sorts) parmi les dons choisis (« Spell Focus (evocation) »).
pub fn dons_ecole_pour(dons: &[String], ecole: &str) -> u32 {
    dons.iter()
        .filter(|d| {
            let d = d.to_lowercase();
            ["spell focus (", "greater spell focus ("].iter().any(|p| {
                d.strip_prefix(p)
                    .and_then(|s| s.strip_suffix(')'))
                    .is_some_and(|e| e.trim() == ecole)
            })
        })
        .count() as u32
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmplacementsDeSorts {
    pub niveau_de_sort: u32,
    /// Sorts par jour de la table de classe (`None` pour le niveau 0 : à volonté).
    pub base: Option<u32>,
    pub bonus: u32,
    /// `None` : à volonté (tours de magie) ou inaccessible.
    pub par_jour: Option<u32>,
    pub connus: u32,
    pub dd: i32,
    /// Emplacement de sort de domaine en plus (prêtre) : 0 ou 1.
    pub domaine: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incantateur {
    pub classe: String,
    pub niveau_de_lanceur: u32,
    pub emplacements: Vec<EmplacementsDeSorts>,
    /// Sorts de lignage obtenus automatiquement : nom (anglais) et niveau de classe d'obtention.
    pub sorts_de_lignage: Vec<(String, u32)>,
}

/// Canalisation d'énergie d'un prêtre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canalisation {
    /// `Some(true)` : énergie positive (soigne les vivants, blesse les morts-vivants) ; `Some(false)` :
    /// négative ; `None` : à choisir (prêtre Neutre).
    pub positive: Option<bool>,
    /// Nombre de d6 de soins ou de dégâts.
    pub des: u32,
    /// DD du jet de Volonté pour réduire les dégâts de moitié.
    pub dd: i32,
    /// Utilisations par jour (3 + modificateur de Charisme).
    pub utilisations: i32,
}

/// Un pouvoir qu'on peut utiliser un nombre de fois par jour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PouvoirJournalier {
    /// Identifiant stable : `canalisation`, `forme_animale`, `domaine:<id>:<rang>`.
    pub cle: String,
    pub nom: String,
    /// `None` : à volonté.
    pub max: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PouvoirVu {
    /// Identifiant du pouvoir journalier correspondant.
    pub cle: String,
    pub nom: String,
    pub genre: String,
    pub niveau: u32,
    /// Le niveau de prêtre est suffisant.
    pub actif: bool,
    /// Utilisations par jour, quand le livre en donne.
    pub utilisations: Option<i32>,
    pub texte: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortDomaineVu {
    pub niveau: u32,
    pub nom_fr: String,
    pub nom_en: Option<String>,
    pub note: Option<String>,
    /// Le personnage a accès à ce niveau de sort.
    pub accessible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomaineVu {
    pub id: String,
    pub nom: String,
    pub pouvoirs: Vec<PouvoirVu>,
    pub sorts: Vec<SortDomaineVu>,
}

/// Sort connu à remplacer, avec les sorts du même niveau proposés à sa place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemplacementSort {
    pub classe: String,
    pub ancien: String,
    pub niveau_de_sort: u32,
    pub propositions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilanRangs {
    pub niveau: u32,
    pub places: u32,
    pub disponibles: u32,
}

/// Une arme portée, avec son attaque et ses dégâts calculés.
#[derive(Debug, Clone, PartialEq)]
pub struct AttaqueArme {
    pub arme_en: String,
    pub nom: String,
    pub maitrise: bool,
    pub attaque: Detail,
    /// Dés de dégâts pour une créature de taille M.
    pub des: Option<String>,
    /// Bonus de dégâts (Force, états).
    pub bonus_degats: Detail,
    pub critique: Option<String>,
    pub portee: Option<String>,
    pub type_degats: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Valeurs {
    pub niveau_de_personnage: u32,
    pub scores: BTreeMap<Carac, Detail>,
    pub modificateurs: BTreeMap<Carac, i32>,
    pub pv_max: Detail,
    pub bba: Detail,
    /// Bonus d'attaque itératifs (ex. +10/+5 au niveau 20 de l'ensorceleur).
    pub bba_iteratif: Vec<i32>,
    pub vigueur: Detail,
    pub reflexes: Detail,
    pub volonte: Detail,
    pub initiative: Detail,
    pub ca: Detail,
    pub ca_contact: Detail,
    pub ca_pris_au_depourvu: Detail,
    pub bmo: Detail,
    pub dmd: Detail,
    pub attaque_melee: Detail,
    pub attaque_distance: Detail,
    pub vitesse: u32,
    pub equipement: BilanEquipement,
    pub armes: Vec<AttaqueArme>,
    /// Précisions des états actifs que le moteur ne calcule pas.
    pub notes_etats: Vec<String>,
    pub competences: BTreeMap<String, Detail>,
    pub rangs: Vec<BilanRangs>,
    pub incantateurs: Vec<Incantateur>,
    pub canalisation: Option<Canalisation>,
    pub domaines: Vec<DomaineVu>,
    /// Pouvoirs limités par jour (canalisation, pouvoirs de domaine, forme animale).
    pub pouvoirs: Vec<PouvoirJournalier>,
    /// Feuille du compagnon animal (pacte `compagnon` du druide).
    pub compagnon: Option<crate::compagnon::CompagnonCalcule>,
}

/// Comme `calculer`, mais plusieurs cibles à la fois (ex. une compétence et « toutes les compétences »),
/// empilées ensemble.
fn calculer_cibles(base: &[(&str, i32)], effets: &[Effet], cibles: &[&str]) -> Detail {
    let choisis: Vec<Effet> = effets
        .iter()
        .filter(|e| cibles.contains(&e.cible.0.as_str()))
        .map(|e| Effet {
            cible: Cible::new("x"),
            ..e.clone()
        })
        .collect();
    calculer(base, &choisis, &Cible::new("x"))
}

fn carac_depuis_id(id: &str) -> Option<Carac> {
    Carac::TOUTES
        .into_iter()
        .find(|c| c.abrege().eq_ignore_ascii_case(id))
}

/// Une compétence de classe correspond-elle à la clé (`knowledge:arcana`) ?
/// « Knowledge (arcana) » vise un sous-type ; « Craft » ou « Profession » seuls, tous.
fn est_de_classe(liste: &[String], cle: &str) -> bool {
    let (base, sous) = match cle.split_once(':') {
        Some((b, s)) => (b, Some(s)),
        None => (cle, None),
    };
    liste.iter().any(|entree| {
        let entree = entree.trim().trim_end_matches('.').to_lowercase();
        match entree.split_once(" (") {
            Some((nom, reste)) => {
                let s = reste.trim_end_matches(')');
                nom.replace(' ', "_") == base && sous == Some(s)
            }
            None => entree.replace(' ', "_") == base,
        }
    })
}

impl Fiche {
    pub fn niveau_de_personnage(&self) -> u32 {
        self.niveaux.len() as u32
    }

    pub fn niveau_de_classe(&self, classe: &str) -> u32 {
        self.niveaux.iter().filter(|n| n.classe == classe).count() as u32
    }

    /// Effets issus de la race, des augmentations de niveau et des ajouts manuels.
    fn tous_les_effets(&self, r: &Referentiel) -> Result<Vec<Effet>, Probleme> {
        let race = r
            .race(&self.race)
            .ok_or_else(|| Probleme::RaceInconnue(self.race.clone()))?;
        let mut e = Vec::new();
        for (id, v) in &race.ajustements {
            if let Some(c) = carac_depuis_id(id) {
                e.push(Effet {
                    cible: c.cible(),
                    type_bonus: TypeBonus::Racial,
                    valeur: *v,
                    source: format!("{} (ajustement racial)", race.nom),
                });
            }
        }
        if let Some(v) = race.choix_libre {
            let c = self
                .bonus_racial_libre
                .ok_or(Probleme::ChoixRacialManquant)?;
            e.push(Effet {
                cible: c.cible(),
                type_bonus: TypeBonus::Racial,
                valeur: v,
                source: format!("{} (bonus au choix)", race.nom),
            });
        }
        e.extend(race.effets.iter().map(|x| x.vers_effet()));
        for (niv, c) in &self.augmentations {
            e.push(Effet {
                cible: c.cible(),
                type_bonus: TypeBonus::SansType,
                valeur: 1,
                source: format!("augmentation du niveau {niv}"),
            });
        }
        e.extend(self.effets.iter().cloned());
        // Bonus chiffrés des capacités de classe, une fois le niveau de classe atteint.
        let mut classes_vues: Vec<&str> = Vec::new();
        for n in &self.niveaux {
            if classes_vues.contains(&n.classe.as_str()) {
                continue;
            }
            classes_vues.push(&n.classe);
            if let Some(c) = r.classe(&n.classe) {
                let nc = self.niveau_de_classe(&n.classe);
                e.extend(
                    c.effets
                        .iter()
                        .filter(|x| x.niveau_classe <= nc)
                        .map(|x| Effet::new(&x.cible, x.type_bonus, x.valeur, &x.source)),
                );
            }
        }
        // Objets magiques portés : leur bonus chiffré, avec son type (altération, parade…).
        for o in &self.equipement.objets {
            if let Some(m) = r.objets_magiques.objet(&o.nom_en) {
                e.extend(m.effets);
            }
        }
        for id in &self.etats {
            if let Some(etat) = r.etat(id) {
                e.extend(etat.effets_chiffres());
            }
        }
        for t in &self.temporaires {
            e.extend(t.effets.iter().cloned());
        }
        for o in &self.equipement.objets_magiques {
            e.extend(o.effets.iter().cloned());
        }
        Ok(e)
    }

    /// Maîtrises d'armes et d'armures : classes, race et dons.
    pub fn maitrises(&self, r: &Referentiel) -> Maitrises {
        let mut m = Maitrises::default();
        let mut vues: Vec<&str> = Vec::new();
        for n in &self.niveaux {
            if vues.contains(&n.classe.as_str()) {
                continue;
            }
            vues.push(&n.classe);
            if let Some(c) = r.equipement.maitrises.classes.get(&n.classe) {
                m.categories.extend(c.armes.iter().cloned());
                m.armes
                    .extend(c.armes_nommees.iter().map(|a| a.to_lowercase()));
                m.armures.extend(c.armures.iter().copied());
                m.boucliers |= c.boucliers;
                m.pavois |= c.bouclier_pavois;
            }
            // Le prêtre sait manier l'arme de prédilection de son dieu.
            if r.classe(&n.classe).is_some_and(|c| c.domaines_choisis > 0) {
                if let Some(arme) = self
                    .divinite
                    .as_deref()
                    .and_then(|d| r.divinite(d))
                    .and_then(|d| d.arme_de_predilection_en.as_deref())
                {
                    m.armes.push(arme.to_lowercase());
                }
            }
        }
        if let Some(rc) = r.equipement.maitrises.races.get(&self.race) {
            m.armes.extend(rc.armes.iter().map(|a| a.to_lowercase()));
            m.assimile_martiales
                .extend(rc.assimile_martiales.iter().cloned());
        }
        for d in self
            .dons_automatiques(r)
            .into_iter()
            .chain(self.dons_choisis())
        {
            let choix = d
                .split_once(" (")
                .map(|(_, c)| c.trim_end_matches(')').to_lowercase());
            match crate::catalogue::nom_de_base(&d) {
                "Armor Proficiency, Light" => m.armures.push(GenreProtection::Legere),
                "Armor Proficiency, Medium" => m.armures.push(GenreProtection::Intermediaire),
                "Armor Proficiency, Heavy" => m.armures.push(GenreProtection::Lourde),
                "Shield Proficiency" => m.boucliers = true,
                "Tower Shield Proficiency" => m.pavois = true,
                "Simple Weapon Proficiency" => m.categories.push("simple".into()),
                "Martial Weapon Proficiency" | "Exotic Weapon Proficiency" => {
                    m.armes.extend(choix);
                }
                _ => {}
            }
        }
        m
    }

    /// Problèmes d'équipement et d'états (noms inconnus, mauvais emplacement).
    pub fn problemes_equipement(&self, r: &Referentiel) -> Vec<Probleme> {
        let cat = &r.equipement;
        let mut p = Vec::new();
        let eq = &self.equipement;
        if let Some(n) = &eq.armure {
            match cat.armure(n) {
                None => p.push(Probleme::EquipementInconnu(n.clone())),
                Some(a)
                    if !matches!(
                        a.genre,
                        GenreProtection::Legere
                            | GenreProtection::Intermediaire
                            | GenreProtection::Lourde
                    ) =>
                {
                    p.push(Probleme::MauvaisEmplacement {
                        objet: n.clone(),
                        attendu: "armure",
                    })
                }
                _ => {}
            }
        }
        if let Some(n) = &eq.bouclier {
            match cat.armure(n) {
                None => p.push(Probleme::EquipementInconnu(n.clone())),
                Some(a) if a.genre != GenreProtection::Bouclier => {
                    p.push(Probleme::MauvaisEmplacement {
                        objet: n.clone(),
                        attendu: "bouclier",
                    })
                }
                _ => {}
            }
        }
        for n in &eq.armes {
            match cat.arme(n) {
                None => p.push(Probleme::EquipementInconnu(n.clone())),
                Some(a) if !a.est_maniable() => p.push(Probleme::ArmeNonManiable(n.clone())),
                _ => {}
            }
        }
        // Armures et boucliers que la classe interdit (druide : pas de métal).
        for n in &self.niveaux {
            let Some(m) = r.equipement.maitrises.classes.get(&n.classe) else {
                continue;
            };
            let nom_classe = r
                .classe(&n.classe)
                .map(|c| c.nom.clone())
                .unwrap_or_default();
            if let (Some(permises), Some(a)) = (&m.armures_permises, &eq.armure) {
                if !permises.iter().any(|x| x.eq_ignore_ascii_case(a)) {
                    p.push(Probleme::ObjetInterdit {
                        classe: nom_classe.clone(),
                        objet: a.clone(),
                    });
                }
            }
            if let (Some(permis), Some(b)) = (&m.boucliers_permis, &eq.bouclier) {
                if !permis.iter().any(|x| x.eq_ignore_ascii_case(b)) {
                    p.push(Probleme::ObjetInterdit {
                        classe: nom_classe,
                        objet: b.clone(),
                    });
                }
            }
        }
        let mut occupes: BTreeMap<String, u32> = BTreeMap::new();
        for o in &eq.objets {
            match r.objets_magiques.objet(&o.nom_en) {
                Some(m) => {
                    if let Some(e) = m.emplacement {
                        *occupes.entry(e).or_insert(0) += o.quantite;
                    }
                }
                None if cat.poids_de(&o.nom_en).is_none() => {
                    p.push(Probleme::EquipementInconnu(o.nom_en.clone()))
                }
                None => {}
            }
        }
        // Un seul objet par emplacement du corps, sauf deux anneaux.
        for (emplacement, n) in occupes {
            let permis = if emplacement == "anneau" { 2 } else { 1 };
            if n > permis {
                p.push(Probleme::EmplacementOccupe { emplacement });
            }
        }
        for (nom, bonus) in &eq.bonus_magiques {
            let porte = eq.armure.as_ref() == Some(nom)
                || eq.bouclier.as_ref() == Some(nom)
                || eq.armes.contains(nom);
            if !porte || !(1..=crate::equipement::BONUS_MAGIQUE_MAX).contains(bonus) {
                p.push(Probleme::BonusMagiqueInvalide(nom.clone()));
            }
        }
        for e in &self.etats {
            if r.etat(e).is_none() {
                p.push(Probleme::EtatInconnu(e.clone()));
            }
        }
        p
    }

    fn caracteristique(&self, c: Carac) -> Caracteristique {
        Caracteristique {
            base: self.scores_base.get(&c).copied().unwrap_or(10),
            dommages: self.dommages.get(&c).copied().unwrap_or(0),
            drain: self.drain.get(&c).copied().unwrap_or(0),
        }
    }

    fn lignage_de<'a>(&self, classe: &'a Classe) -> Option<&'a crate::donnees::Lignage> {
        let id = self.lignage.as_ref()?;
        classe.lignages.iter().find(|l| &l.id == id)
    }

    /// Sorts que le lignage accorde déjà à ce niveau de `classe` (noms anglais).
    fn sorts_accordes_par_le_lignage(&self, classe: &Classe) -> Vec<String> {
        let nc = self.niveau_de_classe(&classe.id);
        self.lignage_de(classe)
            .map(|l| {
                l.sorts_bonus
                    .iter()
                    .filter(|s| s.niveau_classe <= nc)
                    .map(|s| s.nom_en.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Sorts connus à remplacer (en double, ou déjà accordés par le lignage), chacun avec
    /// les sorts du même niveau, de la liste de la classe, qu'on peut prendre à la place.
    /// Les sorts de tous les niveaux du lignage sont écartés des propositions : un sort
    /// que le lignage accordera plus tard ferait un doublon (lecture à valider).
    pub fn remplacements_de_sorts(&self, r: &Referentiel) -> Vec<RemplacementSort> {
        let mut res = Vec::new();
        for n in &self.niveaux {
            if res.iter().any(|x: &RemplacementSort| x.classe == n.classe) {
                continue;
            }
            let Some(classe) = r.classe(&n.classe) else {
                continue;
            };
            let cle = classe.nom_en.to_lowercase();
            let accordes = self.sorts_accordes_par_le_lignage(classe);
            let tous_lignage: Vec<&str> = self
                .lignage_de(classe)
                .map(|l| l.sorts_bonus.iter().map(|s| s.nom_en.as_str()).collect())
                .unwrap_or_default();
            let mut pris: Vec<String> = Vec::new();
            let mut a_remplacer: Vec<(String, u32)> = Vec::new();
            for s in &self.sorts_connus {
                let double = pris.iter().any(|v| v.eq_ignore_ascii_case(s));
                let lignage = accordes.iter().any(|a| a.eq_ignore_ascii_case(s));
                if double || lignage {
                    if let Some(sn) = r.sort(s).and_then(|d| d.niveau_pour(&cle)) {
                        a_remplacer.push((s.clone(), sn));
                    }
                } else {
                    pris.push(s.clone());
                }
            }
            for (ancien, niveau) in a_remplacer {
                let propositions = r
                    .sorts
                    .iter()
                    .filter(|d| d.niveau_pour(&cle) == Some(niveau))
                    .filter(|d| !pris.iter().any(|v| v.eq_ignore_ascii_case(&d.nom_en)))
                    .filter(|d| {
                        !tous_lignage
                            .iter()
                            .any(|a| a.eq_ignore_ascii_case(&d.nom_en))
                    })
                    .map(|d| d.nom_en.clone())
                    .collect();
                res.push(RemplacementSort {
                    classe: n.classe.clone(),
                    ancien,
                    niveau_de_sort: niveau,
                    propositions,
                });
            }
        }
        res
    }

    /// Compétences de classe de tout le personnage, avec celle du lignage.
    pub fn competences_de_classe(&self, r: &Referentiel) -> Vec<String> {
        let mut v = Vec::new();
        let mut vues = Vec::new();
        for n in &self.niveaux {
            if vues.contains(&n.classe) {
                continue;
            }
            vues.push(n.classe.clone());
            if let Some(c) = r.classe(&n.classe) {
                v.extend(c.competences_de_classe_en.iter().cloned());
                if let Some(l) = self.lignage_de(c) {
                    let comp = &l.competence_de_classe_en;
                    if comp.contains("(any one)") {
                        if let Some(sous) = &self.connaissance_de_lignage {
                            v.push(format!("Knowledge ({sous})"));
                        }
                    } else {
                        v.push(comp.clone());
                    }
                }
            }
        }
        // Compétences accordées par les domaines choisis.
        for id in &self.domaines {
            if let Some(d) = r.domaine(id) {
                v.extend(d.competences_de_classe_en.iter().cloned());
            }
        }
        v
    }

    /// Rangs disponibles au niveau `n` (index 0 = niveau 1), avec le modificateur d'INT actuel.
    fn rangs_disponibles(&self, r: &Referentiel, n: usize, mod_int: i32) -> u32 {
        let niveau = &self.niveaux[n];
        let base = r
            .classe(&niveau.classe)
            .map(|c| c.rangs_par_niveau)
            .unwrap_or(0);
        let race = r
            .race(&self.race)
            .map(|x| x.rang_supplementaire)
            .unwrap_or(0);
        let pred = u32::from(niveau.predilection == Some(BonusPredilection::Rang));
        (base + mod_int).max(1) as u32 + race + pred
    }

    /// Problèmes qui empêchent la fiche d'être valide (assistant avec validation).
    pub fn valider(&self, r: &Referentiel) -> Vec<Probleme> {
        let mut p = Vec::new();
        let race = match r.race(&self.race) {
            Some(x) => x,
            None => return vec![Probleme::RaceInconnue(self.race.clone())],
        };
        if race.choix_libre.is_some() && self.bonus_racial_libre.is_none() {
            p.push(Probleme::ChoixRacialManquant);
        }
        let effets = match self.tous_les_effets(r) {
            Ok(e) => e,
            Err(e) => {
                p.push(e);
                return p;
            }
        };
        let mod_int = self
            .caracteristique(Carac::Int)
            .modificateur(Carac::Int, &effets);
        let mut cumul: BTreeMap<String, u32> = BTreeMap::new();
        for (i, n) in self.niveaux.iter().enumerate() {
            let niveau = i as u32 + 1;
            let classe = match r.classe(&n.classe) {
                Some(c) => c,
                None => {
                    p.push(Probleme::ClasseInconnue(n.classe.clone()));
                    continue;
                }
            };
            if i == 0 && n.de_vie_obtenu != classe.de_vie {
                p.push(Probleme::PremierNiveauPasAuMaximum {
                    obtenu: n.de_vie_obtenu,
                    maximum: classe.de_vie,
                });
            } else if n.de_vie_obtenu < 1 || n.de_vie_obtenu > classe.de_vie {
                p.push(Probleme::DeDeVieHorsLimites {
                    niveau,
                    obtenu: n.de_vie_obtenu,
                    maximum: classe.de_vie,
                });
            }
            let places: u32 = n.rangs.values().sum();
            let dispo = self.rangs_disponibles(r, i, mod_int);
            if places > dispo {
                p.push(Probleme::RangsDepasses {
                    niveau,
                    places,
                    disponibles: dispo,
                });
            }
            for (c, v) in &n.rangs {
                let base = c.split(':').next().unwrap_or(c);
                if r.competence(base).is_none() {
                    p.push(Probleme::CompetenceInconnue(c.clone()));
                }
                *cumul.entry(c.clone()).or_default() += v;
            }
            if classe.incantation.is_some() && !classe.lignages.is_empty() {
                match &self.lignage {
                    None => p.push(Probleme::LignageManquant),
                    Some(id) if !classe.lignages.iter().any(|l| &l.id == id) => {
                        p.push(Probleme::LignageInconnu(id.clone()))
                    }
                    _ => {}
                }
            }
        }
        let niveau = self.niveau_de_personnage();
        for (c, v) in cumul {
            if v > niveau {
                p.push(Probleme::RangAuDessusDuNiveau {
                    competence: c,
                    rangs: v,
                    niveau,
                });
            }
        }
        for (niv, _) in &self.augmentations {
            if !matches!(niv, 4 | 8 | 12 | 16 | 20) || *niv > niveau {
                p.push(Probleme::AugmentationInvalide { niveau: *niv });
            }
        }
        p.extend(self.valider_dons(r));
        p.extend(self.valider_sorts(r));
        p.extend(self.valider_divin(r));
        p.extend(self.problemes_equipement(r));
        p
    }

    /// Divinité, alignement, domaines et énergie des classes qui en demandent (prêtre).
    fn valider_divin(&self, r: &Referentiel) -> Vec<Probleme> {
        let mut p = Vec::new();
        // Pacte avec la nature (druide) : un domaine parmi ceux que la classe permet, ou un compagnon.
        if let Some(permis) = self
            .niveaux
            .iter()
            .filter_map(|n| r.classe(&n.classe))
            .map(|c| &c.pacte_domaines)
            .find(|d| !d.is_empty())
        {
            match self.pacte.as_deref() {
                None => p.push(Probleme::PacteManquant),
                Some("domaine") => {
                    if self.domaines.len() != 1 {
                        p.push(Probleme::DomainesAChoisir {
                            attendus: 1,
                            choisis: self.domaines.len() as u32,
                        });
                    }
                    for id in &self.domaines {
                        match r.domaine(id) {
                            None => p.push(Probleme::DomaineInconnu(id.clone())),
                            Some(d) if !permis.contains(id) => p.push(Probleme::DomaineHorsPacte {
                                domaine: d.nom.clone(),
                            }),
                            Some(_) => {}
                        }
                    }
                }
                Some("compagnon") => {
                    if !self.domaines.is_empty() {
                        p.push(Probleme::DomainesAChoisir {
                            attendus: 0,
                            choisis: self.domaines.len() as u32,
                        });
                    }
                    match &self.compagnon {
                        None => p.push(Probleme::CompagnonManquant),
                        Some(c) => {
                            if !r.compagnons.animaux.iter().any(|a| a.id == c.animal) {
                                p.push(Probleme::CompagnonInconnu(c.animal.clone()));
                            }
                            let nc = self
                                .niveaux
                                .iter()
                                .filter_map(|n| r.classe(&n.classe))
                                .find(|k| !k.pacte_domaines.is_empty())
                                .map(|k| self.niveau_de_classe(&k.id))
                                .unwrap_or(0);
                            let permises =
                                crate::compagnon::augmentations_permises(&r.compagnons, nc);
                            if c.augmentations.len() as u32 > permises {
                                p.push(Probleme::AugmentationsDeTrop {
                                    permises,
                                    choisies: c.augmentations.len() as u32,
                                });
                            }
                        }
                    }
                }
                Some(autre) => p.push(Probleme::PacteInvalide(autre.to_string())),
            }
        }
        let Some(attendus) = self
            .niveaux
            .iter()
            .filter_map(|n| r.classe(&n.classe))
            .map(|c| c.domaines_choisis)
            .find(|d| *d > 0)
        else {
            return p;
        };
        let dieu = match &self.divinite {
            Some(id) => {
                let d = r.divinite(id);
                if d.is_none() {
                    p.push(Probleme::DiviniteInconnue(id.clone()));
                }
                d
            }
            None => None,
        };
        match &self.alignement {
            None => p.push(Probleme::AlignementManquant),
            Some(a) if crate::divin::axes(a).is_none() => {
                p.push(Probleme::AlignementInvalide(a.clone()))
            }
            Some(a) => {
                if let Some(d) = dieu {
                    if !crate::divin::alignement_compatible(a, &d.alignement) {
                        p.push(Probleme::AlignementHorsDivinite {
                            alignement: a.clone(),
                            divinite: d.nom.clone(),
                        });
                    }
                }
            }
        }
        let mut vus: Vec<&str> = Vec::new();
        for id in &self.domaines {
            let Some(d) = r.domaine(id) else {
                p.push(Probleme::DomaineInconnu(id.clone()));
                continue;
            };
            if vus.contains(&id.as_str()) {
                p.push(Probleme::DomaineEnDouble(d.nom.clone()));
                continue;
            }
            vus.push(id);
            if let Some(dieu) = dieu {
                if !dieu.domaines.contains(id) {
                    p.push(Probleme::DomaineHorsDivinite {
                        domaine: d.nom.clone(),
                        divinite: dieu.nom.clone(),
                    });
                }
            }
            if let Some(a) = &self.alignement {
                if crate::divin::axes(a).is_some() && !crate::divin::domaine_permis(d, a) {
                    p.push(Probleme::DomaineAlignement {
                        domaine: d.nom.clone(),
                    });
                }
            }
        }
        if self.domaines.len() as u32 != attendus {
            p.push(Probleme::DomainesAChoisir {
                attendus,
                choisis: self.domaines.len() as u32,
            });
        }
        // Prêtre Neutre sur l'axe Bien-Mal : il choisit une fois pour toutes l'énergie canalisée.
        if let Some((_, 1)) = self.alignement.as_deref().and_then(crate::divin::axes) {
            if !matches!(self.energie.as_deref(), Some("positive" | "negative")) {
                p.push(Probleme::EnergieAChoisir);
            }
        }
        p
    }

    /// Dons obtenus sans choix (ensorceleur : Eschew Materials).
    pub fn dons_automatiques(&self, r: &Referentiel) -> Vec<String> {
        self.niveaux
            .first()
            .and_then(|n| r.classe(&n.classe))
            .and_then(|c| c.don_bonus_niveau_1.clone())
            .into_iter()
            .collect()
    }

    /// Tous les dons choisis, dans l'ordre d'acquisition.
    pub fn dons_choisis(&self) -> Vec<String> {
        self.niveaux.iter().flat_map(|n| n.dons.clone()).collect()
    }

    /// Nombre de dons supplémentaires de lignage (à prendre dans la liste du lignage) au niveau de
    /// personnage `i + 1` (index `i`).
    pub(crate) fn dons_de_lignage_a(&self, r: &Referentiel, i: usize) -> u32 {
        let Some(classe) = r.classe(&self.niveaux[i].classe) else {
            return 0;
        };
        let nc = self.niveaux[..=i]
            .iter()
            .filter(|n| n.classe == classe.id)
            .count();
        let bonus = nc >= 1
            && classe
                .progression
                .get(nc - 1)
                .is_some_and(|l| l.special_en.contains("Bloodline feat"));
        u32::from(bonus)
    }

    /// Nombre de dons à choisir au niveau de personnage `i + 1` (index `i`).
    pub(crate) fn dons_attendus_a(&self, r: &Referentiel, i: usize) -> u32 {
        let niveau = i as u32 + 1;
        if r.classe(&self.niveaux[i].classe).is_none() {
            return 0;
        }
        let mut n = self.dons_de_lignage_a(r, i);
        if r.ligne_progression(niveau).is_some_and(|l| l.don) {
            n += 1;
        }
        if niveau == 1 {
            n += r.race(&self.race).map(|x| x.bonus_don).unwrap_or(0);
        }
        n
    }

    fn valider_dons(&self, r: &Referentiel) -> Vec<Probleme> {
        let mut p = Vec::new();
        let mut vus: Vec<String> = Vec::new();
        for (i, n) in self.niveaux.iter().enumerate() {
            let attendus = self.dons_attendus_a(r, i);
            if attendus != n.dons.len() as u32 {
                p.push(Probleme::DonsAuNiveau {
                    niveau: i as u32 + 1,
                    attendus,
                    choisis: n.dons.len() as u32,
                });
            }
            let a_lignage = self.dons_de_lignage_a(r, i);
            if a_lignage > 0 {
                if let Some(l) = r.classe(&n.classe).and_then(|c| self.lignage_de(c)) {
                    let du_lignage = n.dons.iter().filter(|d| l.autorise_don(d)).count() as u32;
                    if du_lignage < a_lignage {
                        p.push(Probleme::DonDeLignageManquant {
                            niveau: i as u32 + 1,
                            lignage: l.nom_en.clone(),
                        });
                    }
                }
            }
            for d in &n.dons {
                let base = crate::catalogue::nom_de_base(d);
                match r.don(base) {
                    None => p.push(Probleme::DonInconnu(d.clone())),
                    Some(def) => {
                        if vus.iter().any(|v| v.eq_ignore_ascii_case(d)) && !def.repetable() {
                            p.push(Probleme::DonEnDouble(d.clone()));
                        }
                    }
                }
                vus.push(d.clone());
            }
        }
        if let (Some(imp), Some(premier)) = (
            r.race(&self.race).and_then(|x| x.don_impose.clone()),
            self.niveaux.first(),
        ) {
            if !premier
                .dons
                .iter()
                .any(|d| crate::catalogue::nom_de_base(d).eq_ignore_ascii_case(&imp))
            {
                p.push(Probleme::DonImposeManquant(imp));
            }
        }
        p
    }

    fn valider_sorts(&self, r: &Referentiel) -> Vec<Probleme> {
        let mut p = Vec::new();
        let mut classes: Vec<&str> = Vec::new();
        for n in &self.niveaux {
            if !classes.contains(&n.classe.as_str()) {
                classes.push(&n.classe);
            }
        }
        let effets = self.tous_les_effets(r).unwrap_or_default();
        for id in classes {
            let Some(classe) = r.classe(id) else { continue };
            let Some(inc) = &classe.incantation else {
                continue;
            };
            let nc = self.niveau_de_classe(id) as usize;
            let Some(ligne) = classe.progression.get(nc - 1) else {
                continue;
            };
            let cle = classe.nom_en.to_lowercase();
            let mut choisis = [0u32; 10];
            let mut vus: Vec<String> = Vec::new();
            let sorts_accordes = self.sorts_accordes_par_le_lignage(classe);
            let car = carac_depuis_id(&inc.caracteristique).unwrap_or(Carac::Cha);
            let score = self.caracteristique(car).score(car, &effets).total;
            for s in &self.sorts_connus {
                if vus.iter().any(|v| v.eq_ignore_ascii_case(s)) {
                    p.push(Probleme::SortEnDouble(s.clone()));
                    continue;
                }
                vus.push(s.clone());
                if sorts_accordes.iter().any(|a| a.eq_ignore_ascii_case(s)) {
                    p.push(Probleme::SortDeLignageChoisi(s.clone()));
                }
                let Some(def) = r.sort(s) else {
                    p.push(Probleme::SortInconnu(s.clone()));
                    continue;
                };
                match def.niveau_pour(&cle) {
                    None => p.push(Probleme::SortHorsListe { sort: s.clone() }),
                    Some(sn) => {
                        if sn <= 9 {
                            choisis[sn as usize] += 1;
                        }
                        if sn > 0 && score < 10 + sn as i32 {
                            p.push(Probleme::CharismeInsuffisantPourSort {
                                sort: s.clone(),
                                niveau_de_sort: sn,
                            });
                        }
                    }
                }
            }
            // Un lanceur qui prépare ses sorts ne « connaît » pas un nombre fixe de sorts : il
            // choisit chaque jour parmi toute sa liste.
            if inc.prepare() {
                continue;
            }
            for (sn, nb) in choisis.iter().enumerate() {
                let attendus = ligne.sorts_connus.get(sn).copied().flatten().unwrap_or(0);
                if attendus != *nb {
                    p.push(Probleme::SortsAChoisir {
                        classe: id.to_string(),
                        niveau_de_sort: sn as u32,
                        attendus,
                        choisis: *nb,
                    });
                }
            }
        }
        p
    }

    /// Calcule toutes les valeurs dérivées, avec leur explication.
    pub fn calculer(&self, r: &Referentiel) -> Result<Valeurs, Probleme> {
        let mut effets = self.tous_les_effets(r)?;
        let mut scores = BTreeMap::new();
        let mut mods = BTreeMap::new();
        for c in Carac::TOUTES {
            let car = self.caracteristique(c);
            let d = car.score(c, &effets);
            mods.insert(c, modificateur(d.total - car.dommages));
            scores.insert(c, d);
        }
        let m = |c: Carac| mods[&c];
        let niveau = self.niveau_de_personnage();

        // Équipement : armure, bouclier, charge portée, malus et vitesse.
        let maitrises = self.maitrises(r);
        let race = r.race(&self.race);
        let force =
            scores[&Carac::For].total - self.dommages.get(&Carac::For).copied().unwrap_or(0);
        let bilan = self.equipement.evaluer(
            &r.equipement,
            &ContexteEquipement {
                force,
                vitesse_base: race.map(|x| x.vitesse).unwrap_or(30),
                vitesse_jamais_reduite: race.is_some_and(|x| x.vitesse_jamais_reduite),
                maitrises: &maitrises,
            },
        );
        effets.extend(bilan.effets.iter().cloned());
        let (malus_armure, source_malus) = &bilan.malus_d_armure;
        if *malus_armure < 0 {
            for c in r.competences.iter().filter(|c| c.malus_d_armure) {
                effets.push(Effet::new(
                    &format!("comp.{}", c.id),
                    TypeBonus::SansType,
                    *malus_armure,
                    &format!("malus d'armure ({source_malus})"),
                ));
            }
        }
        if bilan.malus_attaque_non_maitrise < 0 {
            effets.push(Effet::new(
                "attaque",
                TypeBonus::SansType,
                bilan.malus_attaque_non_maitrise,
                &format!("sans maîtrise : {}", bilan.non_maitrises.join(", ")),
            ));
        }
        let etats_actifs: Vec<&crate::etats::EtatDef> =
            self.etats.iter().filter_map(|id| r.etat(id)).collect();
        // Bonus de DEX à la CA : plafonné par l'équipement, perdu par certains états.
        let mut dex_ca = m(Carac::Dex);
        let mut dex_libelle = String::from("DEX");
        if let Some((cap, source)) = &bilan.dex_max {
            if dex_ca > *cap {
                dex_ca = *cap;
                dex_libelle = format!("DEX (plafonné à {cap:+} par {source})");
            }
        }
        if dex_ca > 0 && etats_actifs.iter().any(|e| e.perd_bonus_dex_ca) {
            dex_ca = 0;
            dex_libelle = "DEX (bonus perdu : état)".to_string();
        }

        // Totaux de classe : BBA, jets de sauvegarde, plus les tables de chaque classe.
        let mut bba_classe = 0;
        let (mut vig, mut refl, mut vol) = (0, 0, 0);
        let mut iteratif: Vec<i32> = Vec::new();
        let mut classes_vues: Vec<String> = Vec::new();
        for n in &self.niveaux {
            if classes_vues.contains(&n.classe) {
                continue;
            }
            classes_vues.push(n.classe.clone());
            let classe = r
                .classe(&n.classe)
                .ok_or_else(|| Probleme::ClasseInconnue(n.classe.clone()))?;
            let nc = self.niveau_de_classe(&n.classe) as usize;
            let ligne = &classe.progression[nc - 1];
            bba_classe += ligne.bba[0];
            vig += ligne.vigueur;
            refl += ligne.reflexes;
            vol += ligne.volonte;
        }
        // Attaques itératives : un seul total de BBA, -5 par attaque supplémentaire (Core, BBA ≥ +6).
        let mut a = bba_classe;
        while a > 0 {
            iteratif.push(a);
            a -= 5;
            if iteratif.len() >= 4 {
                break;
            }
        }
        if iteratif.is_empty() {
            iteratif.push(bba_classe);
        }

        let bba = calculer(&[("classes", bba_classe)], &[], &Cible::new("bba"));
        let eff = |cible: &str| Cible::new(cible);
        let jds = |nom: &str, base: i32, c: Carac, cible: &str| {
            calculer(&[(nom, base), (c.abrege(), m(c))], &effets, &eff(cible))
        };
        let vigueur = jds("table de classe", vig, Carac::Con, "jds.vig");
        let reflexes = jds("table de classe", refl, Carac::Dex, "jds.ref");
        let volonte = jds("table de classe", vol, Carac::Sag, "jds.vol");
        let initiative = calculer(&[("DEX", m(Carac::Dex))], &effets, &eff("init"));

        let ca = calculer(
            &[("base", 10), (dex_libelle.as_str(), dex_ca)],
            &effets,
            &eff("ca"),
        );
        let sans = |exclus: &[TypeBonus]| -> Vec<Effet> {
            effets
                .iter()
                .filter(|e| !(e.valeur > 0 && exclus.contains(&e.type_bonus)))
                .cloned()
                .collect()
        };
        let ca_contact = calculer(
            &[("base", 10), (dex_libelle.as_str(), dex_ca)],
            &sans(&[TypeBonus::Armure, TypeBonus::Bouclier, TypeBonus::Naturel]),
            &eff("ca"),
        );
        let dex_dep = m(Carac::Dex).min(0);
        let ca_dep = calculer(
            &[("base", 10), ("DEX (malus seulement)", dex_dep)],
            &sans(&[TypeBonus::Esquive, TypeBonus::Parade]),
            &eff("ca"),
        );
        let bmo = calculer(
            &[("BBA", bba_classe), ("FOR", m(Carac::For))],
            &effets,
            &eff("bmo"),
        );
        let dmd = calculer(
            &[
                ("base", 10),
                ("BBA", bba_classe),
                ("FOR", m(Carac::For)),
                ("DEX", m(Carac::Dex)),
            ],
            &effets,
            &eff("dmd"),
        );
        let melee = calculer(
            &[("BBA", bba_classe), ("FOR", m(Carac::For))],
            &effets,
            &eff("attaque"),
        );
        let distance = calculer(
            &[("BBA", bba_classe), ("DEX", m(Carac::Dex))],
            &effets,
            &eff("attaque"),
        );

        // Points de vie : dé obtenu + CON (au moins 1 par niveau, à confirmer) + prédilection.
        let mut des = 0;
        let mut con_total = 0;
        let mut correction = 0;
        let mut pred = 0;
        for n in &self.niveaux {
            let brut = n.de_vie_obtenu as i32 + m(Carac::Con);
            des += n.de_vie_obtenu as i32;
            con_total += m(Carac::Con);
            if brut < 1 {
                correction += 1 - brut;
            }
            if n.predilection == Some(BonusPredilection::PointDeVie) {
                pred += 1;
            }
        }
        let mut termes = vec![
            Terme {
                libelle: "dés de vie".into(),
                valeur: des,
                compte: true,
            },
            Terme {
                libelle: format!("CON ({:+} × {niveau} niveaux)", m(Carac::Con)),
                valeur: con_total,
                compte: true,
            },
        ];
        if correction != 0 {
            termes.push(Terme {
                libelle: "minimum 1 PV par niveau".into(),
                valeur: correction,
                compte: true,
            });
        }
        if pred != 0 {
            termes.push(Terme {
                libelle: "classe de prédilection".into(),
                valeur: pred,
                compte: true,
            });
        }
        let extra = calculer(&[], &effets, &eff("pv"));
        termes.extend(extra.termes);
        let total = termes.iter().filter(|t| t.compte).map(|t| t.valeur).sum();
        let pv_max = Detail { total, termes };

        // Compétences.
        let de_classe = self.competences_de_classe(r);
        let mut rangs_totaux: BTreeMap<String, u32> = BTreeMap::new();
        for n in &self.niveaux {
            for (c, v) in &n.rangs {
                *rangs_totaux.entry(c.clone()).or_default() += v;
            }
        }
        let mut competences = BTreeMap::new();
        let mut cles: Vec<String> = r
            .competences
            .iter()
            .filter(|c| !c.sous_types)
            .map(|c| c.id.clone())
            .collect();
        for k in rangs_totaux.keys() {
            if !cles.contains(k) {
                cles.push(k.clone());
            }
        }
        for cle in cles {
            let base = cle.split(':').next().unwrap_or(&cle);
            let def: &Competence = match r.competence(base) {
                Some(d) => d,
                None => continue,
            };
            let carac = carac_depuis_id(&def.caracteristique).unwrap_or(Carac::Int);
            let rangs = rangs_totaux.get(&cle).copied().unwrap_or(0) as i32;
            let mut b = vec![("rangs", rangs), (carac.abrege(), m(carac))];
            if rangs >= 1 && est_de_classe(&de_classe, &cle) {
                b.push(("compétence de classe", 3));
            }
            let mut cibles = vec![format!("comp.{base}")];
            if cle != base {
                cibles.push(format!("comp.{cle}"));
            }
            cibles.push("comp.*".to_string());
            let refs: Vec<&str> = cibles.iter().map(String::as_str).collect();
            let d = calculer_cibles(&b, &effets, &refs);
            competences.insert(cle, d);
        }

        let rangs = (0..self.niveaux.len())
            .map(|i| BilanRangs {
                niveau: i as u32 + 1,
                places: self.niveaux[i].rangs.values().sum(),
                disponibles: self.rangs_disponibles(r, i, m(Carac::Int)),
            })
            .collect();

        // Incantation.
        let mut incantateurs = Vec::new();
        let mut vues: Vec<String> = Vec::new();
        for n in &self.niveaux {
            if vues.contains(&n.classe) {
                continue;
            }
            vues.push(n.classe.clone());
            let classe = r
                .classe(&n.classe)
                .ok_or_else(|| Probleme::ClasseInconnue(n.classe.clone()))?;
            let Some(inc) = &classe.incantation else {
                continue;
            };
            let car = carac_depuis_id(&inc.caracteristique).unwrap_or(Carac::Cha);
            let nc = self.niveau_de_classe(&n.classe);
            let ligne = &classe.progression[nc as usize - 1];
            let score = scores[&car].total;
            let mut emplacements = Vec::new();
            for sn in 0..=9u32 {
                let prepare = inc.prepare();
                let connus = if prepare && sn == 0 {
                    ligne.oraisons
                } else if prepare {
                    None
                } else {
                    ligne.sorts_connus.get(sn as usize).copied().flatten()
                };
                let base = if sn == 0 {
                    ligne.oraisons.filter(|_| prepare)
                } else {
                    ligne.sorts_par_jour.get(sn as usize - 1).copied().flatten()
                };
                if sn > 0 && base.is_none() {
                    continue;
                }
                if sn == 0 && connus.is_none() {
                    continue;
                }
                let peut = score >= 10 + sn as i32;
                let bonus = if peut { sorts_bonus(m(car), sn) } else { 0 };
                emplacements.push(EmplacementsDeSorts {
                    niveau_de_sort: sn,
                    base,
                    bonus,
                    par_jour: match (sn, base, peut) {
                        (0, _, _) => None,
                        (_, Some(b), true) => Some(b + bonus),
                        _ => Some(0),
                    },
                    connus: connus.unwrap_or(0),
                    dd: dd_sort(sn, m(car), 0),
                    domaine: u32::from(
                        (ligne.domaine || self.pacte.as_deref() == Some("domaine")) && sn > 0,
                    ),
                });
            }
            let sorts_de_lignage = self
                .lignage_de(classe)
                .map(|l| {
                    l.sorts_bonus
                        .iter()
                        .filter(|s| s.niveau_classe <= nc)
                        .map(|s| (s.nom_en.clone(), s.niveau_classe))
                        .collect()
                })
                .unwrap_or_default();
            incantateurs.push(Incantateur {
                classe: n.classe.clone(),
                niveau_de_lanceur: nc,
                emplacements,
                sorts_de_lignage,
            });
        }

        // Canalisation d'énergie et domaines (prêtre).
        let mut canalisation = None;
        let mut domaines_vus = Vec::new();
        let mut pouvoirs: Vec<PouvoirJournalier> = Vec::new();
        let mut compagnon_calcule = None;
        for id in &vues {
            let Some(classe) = r.classe(id) else { continue };
            let nc = self.niveau_de_classe(id);
            if !classe.pacte_domaines.is_empty() && self.pacte.as_deref() == Some("compagnon") {
                compagnon_calcule = self
                    .compagnon
                    .as_ref()
                    .and_then(|c| c.calculer(&r.compagnons, nc));
            }
            for p in classe.pouvoirs.iter().filter(|p| nc >= p.niveau) {
                pouvoirs.push(PouvoirJournalier {
                    cle: p.cle.clone(),
                    nom: p.nom.clone(),
                    max: if let Some(f) = &p.formule {
                        let carac = Carac::TOUTES
                            .into_iter()
                            .find(|c| c.abrege().eq_ignore_ascii_case(&f.carac));
                        let bonus = carac.map_or(0, m);
                        Some(f.base + f.par_niveau * (nc as i32 - 1) + bonus)
                    } else if p.a_volonte.is_some_and(|l| nc >= l) {
                        None
                    } else {
                        Some(1 + ((nc - p.niveau) / p.pas.max(1)) as i32)
                    },
                });
            }
            if classe.canalisation {
                let positive = match self.alignement.as_deref().and_then(crate::divin::axes) {
                    Some((_, 0)) => Some(true),
                    Some((_, 2)) => Some(false),
                    Some(_) => match self.energie.as_deref() {
                        Some("positive") => Some(true),
                        Some("negative") => Some(false),
                        _ => None,
                    },
                    None => None,
                };
                let utilisations = (3 + m(Carac::Cha)).max(0);
                pouvoirs.push(PouvoirJournalier {
                    cle: "canalisation".into(),
                    nom: "Canalisation d'énergie".into(),
                    max: Some(utilisations),
                });
                canalisation = Some(Canalisation {
                    positive,
                    des: 1 + nc.saturating_sub(1) / 2,
                    dd: 10 + (nc / 2) as i32 + m(Carac::Cha),
                    utilisations,
                });
            }
            if classe.domaines_choisis > 0 || self.pacte.as_deref() == Some("domaine") {
                let niveau_max = classe
                    .progression
                    .get(nc as usize - 1)
                    .map(|l| {
                        l.sorts_par_jour
                            .iter()
                            .rposition(|s| s.is_some())
                            .map(|i| i as u32 + 1)
                            .unwrap_or(0)
                    })
                    .unwrap_or(0);
                for did in &self.domaines {
                    let Some(d) = r.domaine(did) else { continue };
                    domaines_vus.push(DomaineVu {
                        id: d.id.clone(),
                        nom: d.nom.clone(),
                        pouvoirs: d
                            .pouvoirs
                            .iter()
                            .enumerate()
                            .map(|(k, p)| {
                                let utilisations = if nc >= p.niveau {
                                    p.utilisations(nc, m(Carac::Sag))
                                } else {
                                    None
                                };
                                let cle = format!("domaine:{}:{k}", d.id);
                                if let Some(max) = utilisations {
                                    pouvoirs.push(PouvoirJournalier {
                                        cle: cle.clone(),
                                        nom: p.nom.clone(),
                                        max: Some(max),
                                    });
                                }
                                PouvoirVu {
                                    cle,
                                    nom: p.nom.clone(),
                                    genre: p.genre.clone(),
                                    niveau: p.niveau,
                                    actif: nc >= p.niveau,
                                    utilisations,
                                    texte: p.texte.clone(),
                                }
                            })
                            .collect(),
                        sorts: d
                            .sorts_de_domaine
                            .iter()
                            .map(|s| SortDomaineVu {
                                niveau: s.niveau,
                                nom_fr: s.nom_fr.clone(),
                                nom_en: s.nom_en.clone(),
                                note: s.note.clone(),
                                accessible: s.niveau <= niveau_max,
                            })
                            .collect(),
                    });
                }
            }
        }

        let mut vitesse = bilan.vitesse;
        if etats_actifs.iter().any(|e| e.vitesse_moitie) {
            vitesse /= 2;
        }
        let notes_etats: Vec<String> = etats_actifs
            .iter()
            .filter_map(|e| e.note.as_ref().map(|n| format!("{} : {n}", e.nom)))
            .collect();
        let mut armes = Vec::new();
        for n in &self.equipement.armes {
            let Some(def) = r.equipement.arme(n).filter(|a| a.est_maniable()) else {
                continue;
            };
            let maitrise = maitrises.arme(def);
            let mut ef = effets.clone();
            let mut ef_degats = effets.clone();
            let magique = self.equipement.bonus_magique(&def.nom_en);
            if magique != 0 {
                let source = format!("{} +{magique}", def.nom);
                ef.push(Effet::new(
                    "attaque",
                    TypeBonus::Alteration,
                    magique,
                    &source,
                ));
                ef_degats.push(Effet::new(
                    "degats",
                    TypeBonus::Alteration,
                    magique,
                    &source,
                ));
            }
            if !maitrise {
                ef.push(Effet::new(
                    "attaque",
                    TypeBonus::SansType,
                    -4,
                    "arme non maîtrisée",
                ));
            }
            let (lib, val) = if def.est_a_distance() {
                ("DEX", m(Carac::Dex))
            } else {
                ("FOR", m(Carac::For))
            };
            let attaque = calculer(&[("BBA", bba_classe), (lib, val)], &ef, &eff("attaque"));
            let f = m(Carac::For);
            let force_degats = if def.est_a_distance() {
                if def.ajoute_la_force_aux_degats() {
                    f
                } else {
                    0
                }
            } else if def.est_a_deux_mains() && f > 0 {
                f * 3 / 2
            } else {
                f
            };
            let bonus_degats = calculer(&[("FOR", force_degats)], &ef_degats, &eff("degats"));
            armes.push(AttaqueArme {
                arme_en: def.nom_en.clone(),
                nom: def.nom.clone(),
                maitrise,
                attaque,
                des: def.degats_moyens(),
                bonus_degats,
                critique: def.critique_en.clone(),
                portee: def.portee_en.clone(),
                type_degats: def.type_en.clone(),
            });
        }
        Ok(Valeurs {
            niveau_de_personnage: niveau,
            scores,
            modificateurs: mods,
            pv_max,
            bba,
            bba_iteratif: iteratif,
            vigueur,
            reflexes,
            volonte,
            initiative,
            ca,
            ca_contact,
            ca_pris_au_depourvu: ca_dep,
            bmo,
            dmd,
            attaque_melee: melee,
            attaque_distance: distance,
            vitesse,
            equipement: bilan,
            armes,
            notes_etats,
            competences,
            rangs,
            incantateurs,
            canalisation,
            domaines: domaines_vus,
            pouvoirs,
            compagnon: compagnon_calcule,
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) use crate::exemple::elfe_ensorceleuse;

    #[test]
    fn dd_des_sorts_et_dons_d_ecole() {
        // 10 + niveau du sort + modificateur (+1 par don d'école)
        assert_eq!(dd_sort(3, 4, 0), 17);
        assert_eq!(dd_sort(0, -1, 0), 9);
        let dons: Vec<String> = [
            "Spell Focus (evocation)",
            "Greater Spell Focus (evocation)",
            "Spell Focus (illusion)",
            "Toughness",
        ]
        .map(String::from)
        .to_vec();
        assert_eq!(dons_ecole_pour(&dons, "evocation"), 2);
        assert_eq!(dons_ecole_pour(&dons, "illusion"), 1);
        assert_eq!(dons_ecole_pour(&dons, "necromancy"), 0);
        assert_eq!(dd_sort(3, 4, dons_ecole_pour(&dons, "evocation")), 19);
    }

    /// Complète les sorts connus pour que la fiche soit valide à son niveau actuel.
    pub(crate) fn completer_sorts(r: &Referentiel, f: &mut Fiche) {
        let classe = r.classe("ensorceleur").unwrap();
        let ligne = &classe.progression[f.niveau_de_classe("ensorceleur") as usize - 1];
        let mut choisis: Vec<String> = Vec::new();
        let lignage: Vec<String> = f
            .lignage_de(classe)
            .map(|l| l.sorts_bonus.iter().map(|s| s.nom_en.clone()).collect())
            .unwrap_or_default();
        for sn in 0..=9usize {
            let n = ligne.sorts_connus[sn].unwrap_or(0) as usize;
            choisis.extend(
                r.sorts
                    .iter()
                    .filter(|s| s.niveau_pour("sorcerer") == Some(sn as u32))
                    .filter(|s| !lignage.contains(&s.nom_en))
                    .take(n)
                    .map(|s| s.nom_en.clone()),
            );
        }
        f.sorts_connus = choisis;
    }

    /// Donne à chaque niveau les dons attendus (dons distincts, sans prérequis ni choix).
    pub(crate) fn completer_dons(r: &Referentiel, f: &mut Fiche) {
        let lignage_dons: Vec<String> = f
            .lignage
            .as_ref()
            .and_then(|id| {
                let c = r.classe(&f.niveaux[0].classe)?;
                c.lignages.iter().find(|l| &l.id == id)
            })
            .map(|l| l.dons_bonus_en.clone())
            .unwrap_or_default();
        // les dons réservés au lignage ne servent pas de dons ordinaires (pas de doublon)
        let mut pool = r
            .dons
            .iter()
            .filter(|d| !d.repetable() && d.prerequis.is_empty() && !d.a_un_choix())
            .filter(|d| !lignage_dons.iter().any(|l| l.starts_with(&d.nom_en)))
            .map(|d| d.nom_en.clone());
        let mut prochains_lignage = lignage_dons.clone().into_iter();
        for i in 0..f.niveaux.len() {
            let n = f.dons_attendus_a(r, i) as usize;
            let mut dons: Vec<String> = Vec::new();
            for _ in 0..f.dons_de_lignage_a(r, i) {
                dons.extend(prochains_lignage.next());
            }
            let reste = n - dons.len();
            let autres: Vec<String> = (&mut pool).take(reste).collect();
            dons.extend(autres);
            f.niveaux[i].dons = dons;
        }
    }

    #[test]
    fn niveau_1_elfe_ensorceleuse() {
        let r = Referentiel::core();
        let f = elfe_ensorceleuse();
        assert_eq!(f.valider(&r), vec![]);
        let v = f.calculer(&r).unwrap();
        // DEX 14+2 = 16 (+3), INT 10+2 = 12 (+1), CON 12-2 = 10 (0), CHA 16 (+3)
        assert_eq!(v.scores[&Carac::Dex].total, 16);
        assert_eq!(v.modificateurs[&Carac::Int], 1);
        assert_eq!(v.modificateurs[&Carac::Con], 0);
        // PV : d6 max + 0 CON + 1 prédilection
        assert_eq!(v.pv_max.total, 7);
        // JS : Vig 0+0, Réf 0+3, Vol +2+0
        assert_eq!(v.vigueur.total, 0);
        assert_eq!(v.reflexes.total, 3);
        assert_eq!(v.volonte.total, 2);
        assert_eq!(v.ca.total, 13);
        assert_eq!(v.initiative.total, 3);
        assert_eq!(v.vitesse, 30);
        // Sorts : 3 de niveau 1 + 1 bonus (CHA +3), DD 10+1+3
        let inc = &v.incantateurs[0];
        let n1 = inc
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == 1)
            .unwrap();
        assert_eq!(
            (n1.base, n1.bonus, n1.par_jour, n1.connus, n1.dd),
            (Some(3), 1, Some(4), 2, 14)
        );
        let n0 = &inc.emplacements[0];
        assert_eq!((n0.niveau_de_sort, n0.connus, n0.par_jour), (0, 4, None));
        // Rangs : 2 + INT(+1) = 3 disponibles, 3 placés
        assert_eq!(v.rangs[0].disponibles, 3);
        assert_eq!(v.rangs[0].places, 3);
        // Compétences : Spellcraft = 1 rang + INT 1 + 3 (de classe) ; Perception = SAG 0 + 2 racial.
        assert_eq!(v.competences["spellcraft"].total, 5);
        assert_eq!(v.competences["knowledge:arcana"].total, 5);
        assert_eq!(v.competences["perception"].total, 2);
        // Stealth non classe, sans rang : DEX +3 seulement.
        assert_eq!(v.competences["stealth"].total, 3);
    }

    #[test]
    fn l_explication_de_la_ca_cite_les_sources() {
        let r = Referentiel::core();
        let v = elfe_ensorceleuse().calculer(&r).unwrap();
        assert_eq!(v.ca.explication(), "13 = +10 base +3 DEX");
    }

    #[test]
    fn changer_con_recalcule_les_pv_retroactivement() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.niveaux.push(NiveauPris {
            classe: "ensorceleur".into(),
            de_vie_obtenu: 4,
            rangs: BTreeMap::new(),
            predilection: None,
            dons: vec![],
        });
        assert_eq!(f.calculer(&r).unwrap().pv_max.total, 7 + 4);
        f.scores_base.insert(Carac::Con, 16); // CON 14 après race : +2
                                              // niveau 1 : 6+2, niveau 2 : 4+2, +1 prédilection
        assert_eq!(f.calculer(&r).unwrap().pv_max.total, 8 + 6 + 1);
    }

    #[test]
    fn con_tres_basse_donne_au_moins_1_pv_par_niveau() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.scores_base.insert(Carac::Con, 3); // 3-2 = 1 -> -5
        f.niveaux[0].predilection = None;
        // dé 6 - 5 = 1
        assert_eq!(f.calculer(&r).unwrap().pv_max.total, 1);
        f.niveaux.push(NiveauPris {
            classe: "ensorceleur".into(),
            de_vie_obtenu: 2,
            rangs: BTreeMap::new(),
            predilection: None,
            dons: vec![],
        });
        assert_eq!(f.calculer(&r).unwrap().pv_max.total, 2); // chaque niveau compte au moins 1
    }

    #[test]
    fn changer_int_change_les_rangs_disponibles() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.scores_base.insert(Carac::Int, 6); // 6+2 = 8 : -1 -> 2-1 = 1 rang
        let p = f.valider(&r);
        assert_eq!(
            p,
            vec![Probleme::RangsDepasses {
                niveau: 1,
                places: 3,
                disponibles: 1
            }]
        );
    }

    #[test]
    fn dommages_de_caracteristique_baissent_le_modificateur_sans_toucher_le_score() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.dommages.insert(Carac::Cha, 4);
        let v = f.calculer(&r).unwrap();
        assert_eq!(v.scores[&Carac::Cha].total, 16);
        assert_eq!(v.modificateurs[&Carac::Cha], 1);
    }

    #[test]
    fn humain_exige_le_choix_du_bonus_libre() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.race = "humain".into();
        assert!(f.valider(&r).contains(&Probleme::ChoixRacialManquant));
        f.bonus_racial_libre = Some(Carac::Cha);
        let v = f.calculer(&r).unwrap();
        assert_eq!(v.scores[&Carac::Cha].total, 18);
        // « Skilled » : +1 rang par niveau
        assert_eq!(v.rangs[0].disponibles, 2 + 1);
    }

    #[test]
    fn petite_taille_ajuste_ca_attaque_bmo_dmd() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.race = "halfelin".into();
        let v = f.calculer(&r).unwrap();
        // DEX 14+2 = 16 (+3) ; CA = 10 + 3 + 1 (taille)
        assert_eq!(v.ca.total, 14);
        // FOR 8-2 = 6 : -2 ; BMO = 0 - 2 - 1 (taille)
        assert_eq!(v.bmo.total, -2 - 1);
        assert_eq!(v.dmd.total, 10 - 2 + 3 - 1);
        assert_eq!(v.attaque_melee.total, -2 + 1);
        // Chance du halfelin : +1 racial aux JS
        assert_eq!(v.volonte.total, 2 + 1);
    }

    #[test]
    fn table_des_sorts_bonus_du_livre() {
        // tableau 1-3 : (modificateur, [bonus aux niveaux 1..9])
        let lignes: [(i32, [u32; 9]); 8] = [
            (0, [0; 9]),
            (1, [1, 0, 0, 0, 0, 0, 0, 0, 0]),
            (4, [1, 1, 1, 1, 0, 0, 0, 0, 0]),
            (5, [2, 1, 1, 1, 1, 0, 0, 0, 0]),
            (8, [2, 2, 2, 2, 1, 1, 1, 1, 0]),
            (9, [3, 2, 2, 2, 2, 1, 1, 1, 1]),
            (12, [3, 3, 3, 3, 2, 2, 2, 2, 1]),
            (17, [5, 4, 4, 4, 4, 3, 3, 3, 3]),
        ];
        for (m, attendu) in lignes {
            for (i, a) in attendu.iter().enumerate() {
                assert_eq!(sorts_bonus(m, i as u32 + 1), *a, "mod {m} niveau {}", i + 1);
            }
        }
    }

    #[test]
    fn ensorceleur_niveau_20_correspond_aux_tables() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.niveaux = (1..=20)
            .map(|i| NiveauPris {
                classe: "ensorceleur".into(),
                de_vie_obtenu: if i == 1 { 6 } else { 4 },
                rangs: BTreeMap::new(),
                predilection: None,
                dons: vec![],
            })
            .collect();
        f.augmentations = vec![
            (4, Carac::Cha),
            (8, Carac::Cha),
            (12, Carac::Cha),
            (16, Carac::Cha),
            (20, Carac::Cha),
        ];
        completer_dons(&r, &mut f);
        completer_sorts(&r, &mut f);
        assert_eq!(f.valider(&r), vec![]);
        let v = f.calculer(&r).unwrap();
        assert_eq!(v.niveau_de_personnage, 20);
        assert_eq!(v.bba.total, 10);
        assert_eq!(v.bba_iteratif, vec![10, 5]);
        assert_eq!(
            (v.vigueur.total, v.reflexes.total - 3, v.volonte.total),
            (6, 6, 12)
        );
        // CHA 21 (+5) : sorts par jour 6 de base + bonus.
        let inc = &v.incantateurs[0];
        assert_eq!(inc.niveau_de_lanceur, 20);
        let n9 = inc
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == 9)
            .unwrap();
        assert_eq!(
            (n9.base, n9.bonus, n9.par_jour, n9.connus),
            (Some(6), 0, Some(6), 3)
        );
        let n1 = inc
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == 1)
            .unwrap();
        assert_eq!((n1.par_jour, n1.connus), (Some(8), 5));
        assert_eq!(inc.sorts_de_lignage.len(), 9);
    }

    #[test]
    fn cha_insuffisant_interdit_les_sorts_du_niveau() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.scores_base.insert(Carac::Cha, 10);
        let v = f.calculer(&r).unwrap();
        let n1 = v.incantateurs[0]
            .emplacements
            .iter()
            .find(|e| e.niveau_de_sort == 1)
            .unwrap();
        // il faut CHA 11 pour des sorts de niveau 1
        assert_eq!(n1.par_jour, Some(0));
    }

    #[test]
    fn augmentation_hors_niveau_est_refusee() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.augmentations.push((4, Carac::Cha));
        assert!(f
            .valider(&r)
            .contains(&Probleme::AugmentationInvalide { niveau: 4 }));
    }

    #[test]
    fn rang_maximum_egal_au_niveau() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.niveaux.push(NiveauPris {
            classe: "ensorceleur".into(),
            de_vie_obtenu: 4,
            rangs: [("spellcraft".to_string(), 2)].into_iter().collect(),
            predilection: None,
            dons: vec![],
        });
        // spellcraft : 1 + 2 = 3 rangs au niveau 2
        assert!(f.valider(&r).contains(&Probleme::RangAuDessusDuNiveau {
            competence: "spellcraft".into(),
            rangs: 3,
            niveau: 2
        }));
    }

    #[test]
    fn premier_niveau_au_maximum() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.niveaux[0].de_vie_obtenu = 3;
        assert!(f
            .valider(&r)
            .contains(&Probleme::PremierNiveauPasAuMaximum {
                obtenu: 3,
                maximum: 6
            }));
    }

    #[test]
    fn lignage_arcanique_connaissance_au_choix() {
        let r = Referentiel::core();
        let mut f = elfe_ensorceleuse();
        f.lignage = Some("arcane".into());
        f.connaissance_de_lignage = Some("history".into());
        f.niveaux[0].rangs.insert("knowledge:history".into(), 0);
        f.niveaux[0].rangs.insert("sense_motive".into(), 0);
        let v = f.calculer(&r).unwrap();
        assert!(v.competences.contains_key("knowledge:history"));
    }
}
