//! Référentiels du Core Rulebook (races, compétences, classes), chargés depuis `data/core/*.json`.
//! Les fichiers sont produits par `outils/importer_aon.js` et embarqués dans l'application.

use crate::effet::{Effet, TypeBonus};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SourceLivre {
    pub livre: String,
    pub page: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffetDonnee {
    pub cible: String,
    #[serde(rename = "type")]
    pub type_bonus: TypeBonus,
    pub valeur: i32,
    pub source: String,
}

impl EffetDonnee {
    pub fn vers_effet(&self) -> Effet {
        Effet::new(&self.cible, self.type_bonus, self.valeur, &self.source)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraitTexte {
    pub nom_en: String,
    pub texte_en: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Race {
    pub id: String,
    pub nom: String,
    pub nom_en: String,
    pub taille: String,
    pub vitesse: u32,
    #[serde(default)]
    pub vitesse_jamais_reduite: bool,
    pub vision: String,
    /// Ajustements fixes : identifiant de caractéristique (`for`, `dex`…) et valeur.
    pub ajustements: Vec<(String, i32)>,
    /// Bonus à placer au choix du joueur (humain, demi-elfe, demi-orque).
    pub choix_libre: Option<i32>,
    pub langues: Vec<String>,
    pub langues_choix: String,
    #[serde(default)]
    pub bonus_don: u32,
    #[serde(default)]
    pub rang_supplementaire: u32,
    #[serde(default)]
    pub deux_classes_de_predilection: bool,
    /// Don imposé au niveau 1 (demi-elfe : Skill Focus), à choisir parmi ses variantes.
    pub don_impose: Option<String>,
    pub effets: Vec<EffetDonnee>,
    pub source: SourceLivre,
    pub traits: Vec<TraitTexte>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Competence {
    pub id: String,
    pub nom_en: String,
    pub caracteristique: String,
    pub formation_requise: bool,
    pub malus_d_armure: bool,
    pub sous_types: bool,
    pub source: SourceLivre,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProgressionNiveau {
    pub niveau: u32,
    /// Bonus de base à l'attaque (plusieurs valeurs : attaques itératives).
    pub bba: Vec<i32>,
    pub vigueur: i32,
    pub reflexes: i32,
    pub volonte: i32,
    pub special_en: String,
    /// Sorts par jour de niveau 1 à 9 (`None` : aucun de ce niveau).
    pub sorts_par_jour: Vec<Option<u32>>,
    /// Sorts connus de niveau 0 à 9.
    #[serde(default)]
    pub sorts_connus: Vec<Option<u32>>,
    /// Lanceurs qui préparent leurs sorts : oraisons (niveau 0) préparées chaque jour.
    #[serde(default)]
    pub oraisons: Option<u32>,
    /// Un emplacement de sort de domaine en plus pour chaque niveau de sort accessible (1 à 9).
    #[serde(default)]
    pub domaine: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SortLignage {
    pub nom_en: String,
    pub niveau_classe: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pouvoir {
    pub nom_en: String,
    #[serde(rename = "type")]
    pub genre: String,
    pub niveau_classe: Option<u32>,
    pub texte_en: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Lignage {
    pub id: String,
    pub nom_en: String,
    pub source: SourceLivre,
    pub competence_de_classe_en: String,
    pub sorts_bonus: Vec<SortLignage>,
    pub dons_bonus_en: Vec<String>,
    pub arcane_en: String,
    pub pouvoirs: Vec<Pouvoir>,
    pub choix_en: Vec<String>,
}

impl Lignage {
    /// Le don fait-il partie de la liste de dons supplémentaires de ce lignage ?
    /// Une entrée avec précision (« Skill Focus (Fly) ») impose cette précision ; une entrée
    /// sans précision accepte le don quel que soit son choix.
    pub fn autorise_don(&self, don: &str) -> bool {
        let base = crate::catalogue::nom_de_base(don);
        self.dons_bonus_en.iter().any(|e| {
            if e.contains(" (") {
                e.trim().eq_ignore_ascii_case(don.trim())
            } else {
                e.trim().eq_ignore_ascii_case(base)
            }
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Incantation {
    #[serde(rename = "type")]
    pub genre: String,
    pub mode: String,
    pub caracteristique: String,
    /// Échange d’un sort connu : à partir de ce niveau de classe, puis tous les `echange_pas` niveaux.
    pub echange_niveau_min: Option<u32>,
    pub echange_pas: Option<u32>,
}

impl Incantation {
    /// Le lanceur choisit et prépare ses sorts à l'avance (prêtre, druide) au lieu d'en connaître
    /// un nombre limité (ensorceleur).
    pub fn prepare(&self) -> bool {
        self.mode.starts_with("prépar")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Classe {
    pub id: String,
    pub nom: String,
    pub nom_en: String,
    pub source: SourceLivre,
    pub de_vie: u32,
    pub rangs_par_niveau: i32,
    pub caracteristique_rangs: String,
    pub competences_de_classe_en: Vec<String>,
    pub incantation: Option<Incantation>,
    /// Don obtenu gratuitement au niveau 1 (ensorceleur : Eschew Materials).
    pub don_bonus_niveau_1: Option<String>,
    pub progression: Vec<ProgressionNiveau>,
    #[serde(default)]
    pub lignages: Vec<Lignage>,
    /// Nombre de domaines à choisir (prêtre : 2), avec dieu et alignement.
    #[serde(default)]
    pub domaines_choisis: u32,
    /// Canalisation d'énergie (prêtre).
    #[serde(default)]
    pub canalisation: bool,
    /// Canalisation du paladin : énergie positive, niveau effectif de prêtre = niveau de paladin,
    /// chaque canalisation coûte 2 utilisations d'imposition des mains (moitié du niveau + Charisme).
    #[serde(default)]
    pub canalisation_par_imposition: bool,
    /// Identifiants des domaines que le « pacte avec la nature » du druide peut donner.
    #[serde(default)]
    pub pacte_domaines: Vec<String>,
    /// Pouvoirs de classe utilisables un nombre de fois par jour (forme animale…).
    #[serde(default)]
    pub pouvoirs: Vec<PouvoirClasse>,
    /// Texte du livre pour chaque capacité de classe (nom en minuscules → texte).
    #[serde(default)]
    pub textes_capacites: std::collections::BTreeMap<String, String>,
    /// Bonus chiffrés accordés par la classe (ex. instinct naturel du druide).
    #[serde(default)]
    pub effets: Vec<EffetClasse>,
    /// Listes de dons que la classe permet de prendre comme dons supplémentaires (moine).
    /// Les dons de ces listes s'obtiennent même sans en remplir les conditions.
    #[serde(default)]
    pub dons_supplementaires: Vec<ListeDonsSupplementaires>,
}

/// Dons de la liste, ouverts à partir du niveau de classe `a_partir_du` (cumulatifs).
#[derive(Debug, Clone, Deserialize)]
pub struct ListeDonsSupplementaires {
    pub a_partir_du: u32,
    pub source: String,
    /// Style de combat concerné (rôdeur : « tir_a_distance » ou « deux_armes »). Absent : toutes.
    #[serde(default)]
    pub style: Option<String>,
    pub dons_en: Vec<String>,
}

impl Classe {
    /// Dons de la classe ouverts au niveau de classe `nc`, sans condition, pour ce style de combat.
    pub fn dons_supplementaires_pour(&self, nc: u32, style: Option<&str>) -> Vec<&str> {
        self.dons_supplementaires
            .iter()
            .filter(|l| l.a_partir_du <= nc)
            .filter(|l| l.style.as_deref().is_none_or(|s| Some(s) == style))
            .flat_map(|l| l.dons_en.iter().map(String::as_str))
            .collect()
    }
}

/// Pouvoir de classe limité par jour : une utilisation au niveau `niveau`, une de plus tous les
/// `pas` niveaux, à volonté au niveau `a_volonte`.
#[derive(Debug, Clone, Deserialize)]
pub struct PouvoirClasse {
    pub cle: String,
    pub nom: String,
    pub niveau: u32,
    pub pas: u32,
    pub a_volonte: Option<u32>,
    /// Nombre fixé par une formule au lieu de `pas` (barbare : rounds de rage).
    #[serde(default)]
    pub formule: Option<FormuleUtilisations>,
}

/// Utilisations = `base` + `par_niveau` × ((niveau de classe − `depart`) ÷ `diviseur`) + modificateur de `carac`.
#[derive(Debug, Clone, Deserialize)]
pub struct FormuleUtilisations {
    pub base: i32,
    pub par_niveau: i32,
    pub carac: String,
    /// Niveaux de classe non comptés (rage : 1, car la rage augmente à partir du niveau 2).
    #[serde(default)]
    pub depart: u32,
    /// Division du niveau (imposition des mains : moitié du niveau).
    #[serde(default = "diviseur_unite")]
    pub diviseur: u32,
}

fn diviseur_unite() -> u32 {
    1
}

/// Bonus chiffré d'une capacité de classe, acquis à partir d'un niveau de classe.
#[derive(Debug, Clone, Deserialize)]
pub struct EffetClasse {
    pub niveau_classe: u32,
    pub cible: String,
    #[serde(rename = "type")]
    pub type_bonus: TypeBonus,
    pub valeur: i32,
    pub source: String,
}

/// Ligne du tableau 3-1 (Core p. 31) : XP requis et gains du niveau.
#[derive(Debug, Clone, Deserialize)]
pub struct LigneProgression {
    pub niveau: u32,
    pub xp_lente: u32,
    pub xp_moyenne: u32,
    pub xp_rapide: u32,
    pub don: bool,
    pub augmentation: bool,
}

#[derive(Debug, Deserialize)]
struct FichierProgression {
    niveaux: Vec<LigneProgression>,
}

#[derive(Debug, Deserialize)]
struct FichierRaces {
    races: Vec<Race>,
}
#[derive(Debug, Deserialize)]
struct FichierCompetences {
    competences: Vec<Competence>,
}
#[derive(Debug, Deserialize)]
struct FichierClasse {
    classe: Classe,
}

/// Tout le référentiel Core chargé.
#[derive(Debug, Clone)]
pub struct Referentiel {
    pub races: Vec<Race>,
    pub competences: Vec<Competence>,
    pub classes: Vec<Classe>,
    pub progression: Vec<LigneProgression>,
    pub dons: Vec<crate::catalogue::Don>,
    pub sorts: Vec<crate::catalogue::Sort>,
    pub equipement: crate::equipement::CatalogueEquipement,
    pub etats: Vec<crate::etats::EtatDef>,
    pub objets_magiques: crate::objets_magiques::CatalogueMagique,
    pub domaines: Vec<crate::divin::Domaine>,
    pub divinites: Vec<crate::divin::Divinite>,
    pub formes: Vec<crate::formes::FormeAnimale>,
    pub compagnons: crate::compagnon::DonneesCompagnons,
    pub choix: crate::choix_classe::ChoixDeClasse,
    pub rage: crate::choix_classe::PouvoirsRage,
}

impl Referentiel {
    /// Référentiel Core chargé une seule fois et partagé (le charger coûte cher).
    pub fn partage() -> &'static Referentiel {
        static R: std::sync::OnceLock<Referentiel> = std::sync::OnceLock::new();
        R.get_or_init(Referentiel::core)
    }

    /// Référentiel Core embarqué dans l'application.
    pub fn core() -> Referentiel {
        Self::depuis_json(
            include_str!("../../../data/core/races.json"),
            include_str!("../../../data/core/competences.json"),
            &[
                include_str!("../../../data/core/ensorceleur.json"),
                include_str!("../../../data/core/pretre.json"),
                include_str!("../../../data/core/druide.json"),
                include_str!("../../../data/core/barbare.json"),
                include_str!("../../../data/core/barde.json"),
                include_str!("../../../data/core/guerrier.json"),
                include_str!("../../../data/core/moine.json"),
                include_str!("../../../data/core/paladin.json"),
                include_str!("../../../data/core/rodeur.json"),
                include_str!("../../../data/core/roublard.json"),
                include_str!("../../../data/core/magicien.json"),
            ],
            include_str!("../../../data/core/progression.json"),
        )
        .and_then(|r| {
            r.avec_catalogues(
                include_str!("../../../data/core/dons.json"),
                include_str!("../../../data/core/sorts.json"),
            )
        })
        .and_then(|r| {
            r.avec_equipement(
                include_str!("../../../data/core/armures.json"),
                include_str!("../../../data/core/armes.json"),
                include_str!("../../../data/core/materiel.json"),
                include_str!("../../../data/core/charge.json"),
                include_str!("../../../data/core/maitrises.json"),
                include_str!("../../../data/core/etats.json"),
            )
        })
        .and_then(|r| {
            r.avec_objets_magiques(include_str!("../../../data/core/objets_magiques.json"))
        })
        .and_then(|r| r.avec_compagnons(include_str!("../../../data/core/compagnons_animaux.json")))
        .and_then(|r| r.avec_formes(include_str!("../../../data/core/formes_animales.json")))
        .and_then(|r| r.avec_choix(include_str!("../../../data/core/choix_de_classe.json")))
        .and_then(|r| {
            r.avec_rage(
                include_str!("../../../data/core/pouvoirs_rage.json"),
                include_str!("../../../data/core/pouvoirs_rage_prerequis.json"),
            )
        })
        .and_then(|r| {
            r.avec_divinites(
                include_str!("../../../data/core/domaines.json"),
                include_str!("../../../data/core/divinites.json"),
            )
        })
        .expect("référentiel Core embarqué valide")
    }

    pub fn depuis_json(
        races: &str,
        competences: &str,
        classes: &[&str],
        progression: &str,
    ) -> Result<Referentiel, serde_json::Error> {
        let races: FichierRaces = serde_json::from_str(races)?;
        let competences: FichierCompetences = serde_json::from_str(competences)?;
        let mut cl = Vec::new();
        for c in classes {
            let f: FichierClasse = serde_json::from_str(c)?;
            cl.push(f.classe);
        }
        let progression: FichierProgression = serde_json::from_str(progression)?;
        Ok(Referentiel {
            progression: progression.niveaux,
            races: races.races,
            competences: competences.competences,
            classes: cl,
            dons: Vec::new(),
            sorts: Vec::new(),
            equipement: Default::default(),
            etats: Vec::new(),
            objets_magiques: Default::default(),
            domaines: Vec::new(),
            divinites: Vec::new(),
            formes: Vec::new(),
            compagnons: Default::default(),
            choix: Default::default(),
            rage: Default::default(),
        })
    }

    /// Ajoute les pouvoirs de rage du barbare et leurs prérequis.
    pub fn avec_rage(
        mut self,
        pouvoirs: &str,
        prerequis: &str,
    ) -> Result<Referentiel, serde_json::Error> {
        self.rage = crate::choix_classe::PouvoirsRage::depuis_json(pouvoirs, prerequis)?;
        Ok(self)
    }

    /// Ajoute les choix génériques des classes (grâces, talents, ennemis jurés).
    pub fn avec_choix(mut self, choix: &str) -> Result<Referentiel, serde_json::Error> {
        self.choix = serde_json::from_str(choix)?;
        Ok(self)
    }

    /// Ajoute les catalogues de dons et de sorts.
    pub fn avec_catalogues(
        mut self,
        dons: &str,
        sorts: &str,
    ) -> Result<Referentiel, serde_json::Error> {
        let d: crate::catalogue::FichierDons = serde_json::from_str(dons)?;
        let s: crate::catalogue::FichierSorts = serde_json::from_str(sorts)?;
        self.dons = d.dons;
        self.sorts = s.sorts;
        Ok(self)
    }

    /// Ajoute les domaines et les divinités du prêtre.
    pub fn avec_divinites(
        mut self,
        domaines: &str,
        divinites: &str,
    ) -> Result<Referentiel, serde_json::Error> {
        self.domaines = serde_json::from_str::<crate::divin::FichierDomaines>(domaines)?.domaines;
        self.divinites =
            serde_json::from_str::<crate::divin::FichierDivinites>(divinites)?.divinites;
        Ok(self)
    }

    /// Ajoute la table et la liste des compagnons animaux du druide.
    pub fn avec_compagnons(mut self, json: &str) -> Result<Referentiel, serde_json::Error> {
        self.compagnons = serde_json::from_str(json)?;
        Ok(self)
    }

    /// Ajoute les formes de la forme animale du druide.
    pub fn avec_formes(mut self, json: &str) -> Result<Referentiel, serde_json::Error> {
        self.formes = serde_json::from_str::<crate::formes::FichierFormes>(json)?.formes;
        Ok(self)
    }

    pub fn forme(&self, id: &str) -> Option<&crate::formes::FormeAnimale> {
        self.formes.iter().find(|f| f.id == id)
    }

    pub fn domaine(&self, id: &str) -> Option<&crate::divin::Domaine> {
        self.domaines.iter().find(|d| d.id == id)
    }

    pub fn divinite(&self, id: &str) -> Option<&crate::divin::Divinite> {
        self.divinites.iter().find(|d| d.id == id)
    }

    /// Ajoute les objets magiques à bonus chiffré.
    pub fn avec_objets_magiques(mut self, json: &str) -> Result<Referentiel, serde_json::Error> {
        self.objets_magiques = crate::objets_magiques::CatalogueMagique::depuis_json(json)?;
        Ok(self)
    }

    /// Ajoute l'équipement (armures, armes, matériel, charge, maîtrises) et les états.
    pub fn avec_equipement(
        mut self,
        armures: &str,
        armes: &str,
        materiel: &str,
        charge: &str,
        maitrises: &str,
        etats: &str,
    ) -> Result<Referentiel, serde_json::Error> {
        self.equipement = crate::equipement::CatalogueEquipement::depuis_json(
            armures, armes, materiel, charge, maitrises,
        )?;
        self.etats = serde_json::from_str::<crate::etats::FichierEtats>(etats)?.etats;
        Ok(self)
    }

    pub fn etat(&self, id: &str) -> Option<&crate::etats::EtatDef> {
        self.etats.iter().find(|e| e.id == id)
    }

    pub fn don(&self, nom_en: &str) -> Option<&crate::catalogue::Don> {
        self.dons
            .iter()
            .find(|d| d.nom_en.eq_ignore_ascii_case(nom_en))
    }

    /// Sort du catalogue d'après un nom écrit autrement (« greater teleport », « burning hands* »,
    /// « summon monster IX ») comme dans les listes de sorts de lignage.
    pub fn sort_approx(&self, nom: &str) -> Option<&crate::catalogue::Sort> {
        let cible = forme_canonique(nom);
        self.sorts
            .iter()
            .find(|s| forme_canonique(&s.nom_en) == cible)
    }

    pub fn sort(&self, nom_en: &str) -> Option<&crate::catalogue::Sort> {
        self.sorts
            .iter()
            .find(|s| s.nom_en.eq_ignore_ascii_case(nom_en))
    }

    /// Ligne du tableau de progression pour un niveau de personnage (1 à 20).
    pub fn ligne_progression(&self, niveau: u32) -> Option<&LigneProgression> {
        self.progression.iter().find(|l| l.niveau == niveau)
    }

    pub fn race(&self, id: &str) -> Option<&Race> {
        self.races.iter().find(|r| r.id == id)
    }
    pub fn classe(&self, id: &str) -> Option<&Classe> {
        self.classes.iter().find(|c| c.id == id)
    }
    pub fn competence(&self, id: &str) -> Option<&Competence> {
        self.competences.iter().find(|c| c.id == id)
    }
}

/// Forme comparable d'un nom de sort : minuscules, sans astérisque ni précision entre
/// parenthèses ni virgule, mots triés (« Teleport, Greater » = « greater teleport »),
/// chiffres romains de « Summon Monster » en chiffres arabes.
fn forme_canonique(nom: &str) -> String {
    let nom = nom.split(" (").next().unwrap_or(nom);
    let nom = nom.replace(['*', ','], "").to_lowercase();
    let mut mots: Vec<String> = nom.split_whitespace().map(String::from).collect();
    if mots.len() == 3 && mots[0] == "summon" && mots[1] == "monster" {
        let romain = ["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix"];
        if let Some(p) = romain.iter().position(|r| *r == mots[2]) {
            mots[2] = (p + 1).to_string();
        }
    }
    mots.sort();
    mots.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_referentiel_embarque_se_charge() {
        let r = Referentiel::core();
        assert_eq!(r.races.len(), 7);
        assert_eq!(r.competences.len(), 26);
        let e = r.classe("ensorceleur").unwrap();
        assert_eq!(e.progression.len(), 20);
        assert_eq!(e.lignages.len(), 10);
        for l in &e.lignages {
            assert_eq!(l.sorts_bonus.len(), 9, "{}", l.id);
        }
    }

    #[test]
    fn chaque_donnee_a_sa_source() {
        let r = Referentiel::core();
        assert!(r.races.iter().all(|x| x.source.page.is_some()));
        assert!(r.competences.iter().all(|x| x.source.page.is_some()));
        let e = r.classe("ensorceleur").unwrap();
        assert!(e.lignages.iter().all(|l| l.source.page.is_some()));
    }
}
