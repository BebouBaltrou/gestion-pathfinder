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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Probleme {
    EquipementInconnu(String),
    MauvaisEmplacement {
        objet: String,
        attendu: &'static str,
    },
    ArmeNonManiable(String),
    EtatInconnu(String),
    SortInconnu(String),
    SortHorsListe {
        sort: String,
    },
    SortEnDouble(String),
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
            Probleme::SortInconnu(s) => write!(f, "sort inconnu du catalogue : {s}"),
            Probleme::SortHorsListe { sort } => {
                write!(f, "{sort} n'est pas sur la liste de sorts de la classe")
            }
            Probleme::SortEnDouble(s) => write!(f, "sort choisi deux fois : {s}"),
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
            Probleme::DonImposeManquant(d) => write!(f, "don imposé par la race manquant : {d}"),
            Probleme::EquipementInconnu(n) => write!(f, "équipement inconnu du catalogue : {n}"),
            Probleme::MauvaisEmplacement { objet, attendu } => {
                write!(f, "{objet} ne peut pas être porté comme {attendu}")
            }
            Probleme::ArmeNonManiable(n) => {
                write!(f, "{n} n'est pas une arme que l'on manie (munition ou engin de siège) : la ranger dans les objets")
            }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incantateur {
    pub classe: String,
    pub niveau_de_lanceur: u32,
    pub emplacements: Vec<EmplacementsDeSorts>,
    /// Sorts de lignage obtenus automatiquement : nom (anglais) et niveau de classe d'obtention.
    pub sorts_de_lignage: Vec<(String, u32)>,
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
        for id in &self.etats {
            if let Some(etat) = r.etat(id) {
                e.extend(etat.effets_chiffres());
            }
        }
        for t in &self.temporaires {
            e.extend(t.effets.iter().cloned());
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
                m.armures.extend(c.armures.iter().copied());
                m.boucliers |= c.boucliers;
                m.pavois |= c.bouclier_pavois;
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
        for o in &eq.objets {
            if cat.poids_de(&o.nom_en).is_none() {
                p.push(Probleme::EquipementInconnu(o.nom_en.clone()));
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

    /// Compétences de classe de tout le personnage, avec celle du lignage.
    fn competences_de_classe(&self, r: &Referentiel) -> Vec<String> {
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
        p.extend(self.problemes_equipement(r));
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

    /// Nombre de dons à choisir au niveau de personnage `i + 1` (index `i`).
    pub(crate) fn dons_attendus_a(&self, r: &Referentiel, i: usize) -> u32 {
        let niveau = i as u32 + 1;
        let Some(classe) = r.classe(&self.niveaux[i].classe) else {
            return 0;
        };
        let nc = self.niveaux[..=i]
            .iter()
            .filter(|n| n.classe == classe.id)
            .count() as u32;
        let mut n = 0;
        if r.ligne_progression(niveau).is_some_and(|l| l.don) {
            n += 1;
        }
        if niveau == 1 {
            n += r.race(&self.race).map(|x| x.bonus_don).unwrap_or(0);
        }
        if nc >= 1
            && classe
                .progression
                .get(nc as usize - 1)
                .is_some_and(|l| l.special_en.contains("Bloodline feat"))
        {
            n += 1;
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
            let car = carac_depuis_id(&inc.caracteristique).unwrap_or(Carac::Cha);
            let score = self.caracteristique(car).score(car, &effets).total;
            for s in &self.sorts_connus {
                if vus.iter().any(|v| v.eq_ignore_ascii_case(s)) {
                    p.push(Probleme::SortEnDouble(s.clone()));
                    continue;
                }
                vus.push(s.clone());
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
                let connus = ligne.sorts_connus.get(sn as usize).copied().flatten();
                let base = if sn == 0 {
                    None
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
                    dd: 10 + sn as i32 + m(car),
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

        let mut vitesse = bilan.vitesse;
        if etats_actifs.iter().any(|e| e.vitesse_moitie) {
            vitesse = vitesse / 2 / 5 * 5;
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
            let bonus_degats = calculer(&[("FOR", force_degats)], &effets, &eff("degats"));
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
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn elfe_ensorceleuse() -> Fiche {
        // Achat de 15 points : FOR 8, DEX 14, CON 12, INT 10, SAG 10, CHA 16 ; elfe : DEX+2 INT+2 CON-2.
        let scores = [
            (Carac::For, 8),
            (Carac::Dex, 14),
            (Carac::Con, 12),
            (Carac::Int, 10),
            (Carac::Sag, 10),
            (Carac::Cha, 16),
        ];
        let mut rangs = BTreeMap::new();
        rangs.insert("spellcraft".to_string(), 1);
        rangs.insert("bluff".to_string(), 1);
        rangs.insert("knowledge:arcana".to_string(), 1);
        Fiche {
            nom: "Lyra".into(),
            race: "elfe".into(),
            bonus_racial_libre: None,
            scores_base: scores.into_iter().collect(),
            dommages: BTreeMap::new(),
            drain: BTreeMap::new(),
            augmentations: vec![],
            niveaux: vec![NiveauPris {
                classe: "ensorceleur".into(),
                de_vie_obtenu: 6,
                rangs,
                predilection: Some(BonusPredilection::PointDeVie),
                dons: vec!["Toughness".into()],
            }],
            classes_de_predilection: vec!["ensorceleur".into()],
            lignage: Some("draconic".into()),
            connaissance_de_lignage: None,
            effets: vec![],
            equipement: Default::default(),
            etats: vec![],
            temporaires: vec![],
            sorts_connus: [
                "Acid Splash",
                "Detect Magic",
                "Light",
                "Mage Hand",
                "Magic Missile",
                "Mage Armor",
            ]
            .map(String::from)
            .to_vec(),
        }
    }

    /// Complète les sorts connus pour que la fiche soit valide à son niveau actuel.
    pub(crate) fn completer_sorts(r: &Referentiel, f: &mut Fiche) {
        let classe = r.classe("ensorceleur").unwrap();
        let ligne = &classe.progression[f.niveau_de_classe("ensorceleur") as usize - 1];
        let mut choisis: Vec<String> = Vec::new();
        for sn in 0..=9usize {
            let n = ligne.sorts_connus[sn].unwrap_or(0) as usize;
            choisis.extend(
                r.sorts
                    .iter()
                    .filter(|s| s.niveau_pour("sorcerer") == Some(sn as u32))
                    .take(n)
                    .map(|s| s.nom_en.clone()),
            );
        }
        f.sorts_connus = choisis;
    }

    /// Donne à chaque niveau les dons attendus (dons distincts, sans prérequis ni choix).
    pub(crate) fn completer_dons(r: &Referentiel, f: &mut Fiche) {
        let mut pool = r
            .dons
            .iter()
            .filter(|d| !d.repetable() && d.prerequis.is_empty() && !d.a_un_choix())
            .map(|d| d.nom_en.clone());
        for i in 0..f.niveaux.len() {
            let n = f.dons_attendus_a(r, i) as usize;
            f.niveaux[i].dons = (&mut pool).take(n).collect();
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
