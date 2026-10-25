//! Équipement du Core Rulebook (armures, boucliers, armes, matériel), charge portée et maîtrises.
//!
//! Données : `data/core/{armures,armes,materiel,charge,maitrises}.json`, relevées sur Archives of
//! Nethys par `outils/importer_equipement.js` puis nettoyées par `outils/finaliser_equipement.js`.
//! Les noms français sont des traductions non officielles.

use crate::donnees::SourceLivre;
use crate::effet::{Effet, TypeBonus};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Cout {
    pub valeur: f64,
    /// `gp`, `sp`, `cp` ou `pp`.
    pub unite: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenreProtection {
    Legere,
    Intermediaire,
    Lourde,
    Bouclier,
    /// Pointes d'armure, gantelet de verrouillage, pointes de bouclier.
    Accessoire,
}

impl GenreProtection {
    pub fn nom(self) -> &'static str {
        match self {
            GenreProtection::Legere => "armure légère",
            GenreProtection::Intermediaire => "armure intermédiaire",
            GenreProtection::Lourde => "armure lourde",
            GenreProtection::Bouclier => "bouclier",
            GenreProtection::Accessoire => "accessoire",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArmureDef {
    pub nom_en: String,
    #[serde(default)]
    pub nom: String,
    pub source: SourceLivre,
    pub cout: Option<Cout>,
    pub poids_lb: Option<f64>,
    pub bonus_armure: Option<i32>,
    pub dex_max: Option<i32>,
    pub malus_armure: Option<i32>,
    pub echec_sorts_pct: Option<u32>,
    pub genre: GenreProtection,
}

impl ArmureDef {
    pub fn est_pavois(&self) -> bool {
        self.nom_en == "Tower"
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArmeDef {
    pub nom_en: String,
    #[serde(default)]
    pub nom: String,
    pub source: SourceLivre,
    pub cout: Option<Cout>,
    pub poids_lb: Option<f64>,
    pub degats_en: Option<String>,
    pub critique_en: Option<String>,
    pub portee_en: Option<String>,
    pub type_en: Option<String>,
    pub special_en: Option<String>,
    pub categorie_en: Option<String>,
    /// `Simple`, `Martial` ou `Exotic` ; absent pour les munitions et les engins de siège.
    pub maniement_en: Option<String>,
    pub groupes_en: Option<String>,
}

impl ArmeDef {
    /// Arme que l'on manie (pas une munition ni un engin de siège).
    pub fn est_maniable(&self) -> bool {
        self.maniement_en.is_some()
    }

    pub fn est_a_distance(&self) -> bool {
        self.categorie_en.as_deref() == Some("Ranged")
    }

    /// Arme à distance qui ajoute le bonus de FOR aux dégâts (armes de jet et fronde).
    pub fn ajoute_la_force_aux_degats(&self) -> bool {
        self.est_a_distance()
            && self
                .groupes_en
                .as_deref()
                .is_some_and(|g| g.contains("Thrown"))
    }

    pub fn est_a_deux_mains(&self) -> bool {
        self.categorie_en.as_deref() == Some("Two-Handed")
    }

    /// Dés de dégâts pour une créature de taille M (« 1d6 (small), 1d8 (medium) » → « 1d8 »).
    pub fn degats_moyens(&self) -> Option<String> {
        let t = self.degats_en.as_deref()?;
        match t.find(" (medium)") {
            Some(fin) => {
                let debut = t[..fin].rfind(", ").map(|i| i + 2).unwrap_or(0);
                Some(t[debut..fin].to_string())
            }
            None => Some(t.to_string()),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ObjetDef {
    pub nom_en: String,
    #[serde(default)]
    pub nom: String,
    pub source: SourceLivre,
    pub cout: Option<Cout>,
    pub poids_lb: Option<f64>,
    pub categorie_en: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LigneCharge {
    pub force: i32,
    pub legere: u32,
    pub intermediaire_max: u32,
    pub lourde: u32,
}

/// Tableaux 7-4 et 7-5 du Core (p. 169) : charge selon la Force, vitesse réduite.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TableCharge {
    pub lignes: Vec<LigneCharge>,
    /// (vitesse de base min, max, vitesse réduite), en pieds.
    pub vitesse_reduite: Vec<(u32, u32, u32)>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Charge {
    #[default]
    Legere,
    Intermediaire,
    Lourde,
    /// Au-delà de la charge lourde : le personnage titube (Core p. 169).
    Surcharge,
}

impl Charge {
    pub fn nom(self) -> &'static str {
        match self {
            Charge::Legere => "légère",
            Charge::Intermediaire => "intermédiaire",
            Charge::Lourde => "lourde",
            Charge::Surcharge => "surcharge",
        }
    }
}

impl TableCharge {
    /// Limites (légère, intermédiaire, lourde) en livres pour un score de Force.
    /// Au-delà de 29 : ligne 20-29 de même chiffre des unités, ×4 par tranche de 10.
    pub fn limites(&self, force: i32) -> Option<(f64, f64, f64)> {
        if force < 1 {
            return None;
        }
        let (reference, paliers) = if force <= 29 {
            (force, 0)
        } else {
            let r = 20 + force % 10;
            (r, (force - r) / 10)
        };
        let l = self.lignes.iter().find(|l| l.force == reference)?;
        let m = 4f64.powi(paliers);
        Some((
            f64::from(l.legere) * m,
            f64::from(l.intermediaire_max) * m,
            f64::from(l.lourde) * m,
        ))
    }

    pub fn charge(&self, force: i32, poids: f64) -> Charge {
        match self.limites(force) {
            None => Charge::Surcharge,
            Some((l, m, h)) => {
                if poids <= l {
                    Charge::Legere
                } else if poids <= m {
                    Charge::Intermediaire
                } else if poids <= h {
                    Charge::Lourde
                } else {
                    Charge::Surcharge
                }
            }
        }
    }

    pub fn vitesse_reduite(&self, base: u32) -> u32 {
        self.vitesse_reduite
            .iter()
            .find(|(a, b, _)| base >= *a && base <= *b)
            .map(|x| x.2)
            .unwrap_or(base)
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MaitrisesClasse {
    /// `simple`, `martial`, `exotic`.
    pub armes: Vec<String>,
    pub armures: Vec<GenreProtection>,
    pub boucliers: bool,
    pub bouclier_pavois: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MaitrisesRace {
    /// Armes maîtrisées (noms anglais du catalogue).
    pub armes: Vec<String>,
    /// Mots dont une arme exotique est traitée comme martiale (« elven »…).
    pub assimile_martiales: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DonneesMaitrises {
    pub classes: BTreeMap<String, MaitrisesClasse>,
    pub races: BTreeMap<String, MaitrisesRace>,
}

/// Maîtrises d'un personnage : classes, race et dons réunis.
#[derive(Debug, Clone, Default)]
pub struct Maitrises {
    pub categories: Vec<String>,
    /// Noms d'armes en minuscules.
    pub armes: Vec<String>,
    pub armures: Vec<GenreProtection>,
    pub boucliers: bool,
    pub pavois: bool,
    pub assimile_martiales: Vec<String>,
}

impl Maitrises {
    pub fn arme(&self, a: &ArmeDef) -> bool {
        let Some(cat) = a.maniement_en.as_deref() else {
            return true;
        };
        let nom = a.nom_en.to_lowercase();
        let mut cat = cat.to_lowercase();
        if cat == "exotic"
            && self
                .assimile_martiales
                .iter()
                .any(|m| nom.contains(m.as_str()))
        {
            cat = "martial".into();
        }
        self.categories.contains(&cat)
            || self
                .armes
                .iter()
                .any(|n| *n == nom || format!("{n}s") == nom || *n == format!("{nom}s"))
    }

    pub fn armure(&self, a: &ArmureDef) -> bool {
        match a.genre {
            GenreProtection::Accessoire => true,
            GenreProtection::Bouclier if a.est_pavois() => self.pavois,
            GenreProtection::Bouclier => self.boucliers,
            g => self.armures.contains(&g),
        }
    }
}

/// Tout le catalogue d'équipement.
#[derive(Debug, Clone, Default)]
pub struct CatalogueEquipement {
    pub armures: Vec<ArmureDef>,
    pub armes: Vec<ArmeDef>,
    pub materiel: Vec<ObjetDef>,
    pub charge: TableCharge,
    pub maitrises: DonneesMaitrises,
}

#[derive(Deserialize)]
struct FichierArmures {
    armures: Vec<ArmureDef>,
}
#[derive(Deserialize)]
struct FichierArmes {
    armes: Vec<ArmeDef>,
}
#[derive(Deserialize)]
struct FichierMateriel {
    materiel: Vec<ObjetDef>,
}

fn meme_nom(a_en: &str, a_fr: &str, cherche: &str) -> bool {
    a_en.eq_ignore_ascii_case(cherche) || (!a_fr.is_empty() && a_fr.eq_ignore_ascii_case(cherche))
}

impl CatalogueEquipement {
    pub fn depuis_json(
        armures: &str,
        armes: &str,
        materiel: &str,
        charge: &str,
        maitrises: &str,
    ) -> Result<CatalogueEquipement, serde_json::Error> {
        Ok(CatalogueEquipement {
            armures: serde_json::from_str::<FichierArmures>(armures)?.armures,
            armes: serde_json::from_str::<FichierArmes>(armes)?.armes,
            materiel: serde_json::from_str::<FichierMateriel>(materiel)?.materiel,
            charge: serde_json::from_str(charge)?,
            maitrises: serde_json::from_str(maitrises)?,
        })
    }

    pub fn armure(&self, nom: &str) -> Option<&ArmureDef> {
        self.armures
            .iter()
            .find(|a| meme_nom(&a.nom_en, &a.nom, nom))
    }
    pub fn arme(&self, nom: &str) -> Option<&ArmeDef> {
        self.armes.iter().find(|a| meme_nom(&a.nom_en, &a.nom, nom))
    }
    pub fn objet(&self, nom: &str) -> Option<&ObjetDef> {
        self.materiel
            .iter()
            .find(|a| meme_nom(&a.nom_en, &a.nom, nom))
    }

    /// Poids d'un objet porté, quelle que soit sa liste : `Some(None)` si le poids n'est pas connu.
    pub fn poids_de(&self, nom: &str) -> Option<Option<f64>> {
        if let Some(o) = self.objet(nom) {
            return Some(o.poids_lb);
        }
        if let Some(a) = self.arme(nom) {
            return Some(a.poids_lb);
        }
        self.armure(nom).map(|a| a.poids_lb)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjetPorte {
    pub nom_en: String,
    pub quantite: u32,
}

/// Ce que le personnage porte. Les noms sont ceux du catalogue (anglais).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Equipement {
    pub armure: Option<String>,
    pub bouclier: Option<String>,
    /// Armes maniables portées (leur poids compte dans la charge).
    pub armes: Vec<String>,
    /// Autres objets : matériel, munitions…
    pub objets: Vec<ObjetPorte>,
    /// Charge indiquée par le joueur. Pour l'instant le poids n'est pas converti en charge
    /// automatiquement (décision du 23/10/2026) ; le poids total reste affiché à titre indicatif.
    pub charge_declaree: Charge,
    /// Bonus d'altération (magique) de l'armure, du bouclier ou d'une arme portés, par nom
    /// du catalogue (anglais). Absent : pas de bonus.
    pub bonus_magiques: BTreeMap<String, i32>,
    /// Anneaux, capes, ceintures… dont le joueur saisit lui-même les effets.
    pub objets_magiques: Vec<ObjetMagique>,
}

/// Bonus d'altération maximal d'une armure, d'un bouclier ou d'une arme (Core, p. 468-469).
pub const BONUS_MAGIQUE_MAX: i32 = 5;

/// Objet magique porté : le nom et les effets chiffrés saisis par le joueur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjetMagique {
    pub nom: String,
    pub effets: Vec<Effet>,
}

impl Equipement {
    /// Bonus d'altération de l'armure, du bouclier ou de l'arme `nom_en` (0 si aucun).
    pub fn bonus_magique(&self, nom_en: &str) -> i32 {
        self.bonus_magiques.get(nom_en).copied().unwrap_or(0)
    }

    pub fn ajouter_objet(&mut self, nom_en: &str, quantite: u32) {
        match self.objets.iter_mut().find(|o| o.nom_en == nom_en) {
            Some(o) => o.quantite += quantite,
            None => self.objets.push(ObjetPorte {
                nom_en: nom_en.to_string(),
                quantite,
            }),
        }
    }

    /// Retire `quantite` exemplaires ; `false` si on n'en porte pas autant.
    pub fn retirer_objet(&mut self, nom_en: &str, quantite: u32) -> bool {
        let Some(i) = self.objets.iter().position(|o| o.nom_en == nom_en) else {
            return false;
        };
        if self.objets[i].quantite < quantite {
            return false;
        }
        self.objets[i].quantite -= quantite;
        if self.objets[i].quantite == 0 {
            self.objets.remove(i);
        }
        true
    }
}

/// Ce dont l'évaluation de l'équipement a besoin hors de l'équipement lui-même.
pub struct ContexteEquipement<'a> {
    pub force: i32,
    pub vitesse_base: u32,
    /// Les nains ne sont pas ralentis par l'armure ni la charge.
    pub vitesse_jamais_reduite: bool,
    pub maitrises: &'a Maitrises,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BilanEquipement {
    pub poids_total: f64,
    /// Objets portés dont le catalogue ne donne pas le poids (comptés pour 0).
    pub poids_inconnu: Vec<String>,
    /// Limites légère / intermédiaire / lourde pour la Force actuelle.
    pub limites: Option<(f64, f64, f64)>,
    pub charge: Charge,
    /// Plafond du bonus de DEX à la CA et ce qui l'impose ; `None` : aucun.
    pub dex_max: Option<(i32, String)>,
    /// Malus aux compétences à malus d'armure (négatif ou 0) et ce qui l'impose.
    pub malus_d_armure: (i32, String),
    /// Échec des sorts profanes (armure + bouclier), plafonné à 100.
    pub echec_sorts_pct: u32,
    pub vitesse: u32,
    /// Effets sur la CA (armure, bouclier).
    pub effets: Vec<Effet>,
    /// Armures et boucliers portés sans maîtrise (leur malus s'applique aux attaques).
    pub non_maitrises: Vec<String>,
    pub malus_attaque_non_maitrise: i32,
}

impl Equipement {
    pub fn evaluer(&self, cat: &CatalogueEquipement, ctx: &ContexteEquipement) -> BilanEquipement {
        let armure = self
            .armure
            .as_deref()
            .and_then(|n| cat.armure(n))
            .filter(|a| {
                matches!(
                    a.genre,
                    GenreProtection::Legere
                        | GenreProtection::Intermediaire
                        | GenreProtection::Lourde
                )
            });
        let bouclier = self
            .bouclier
            .as_deref()
            .and_then(|n| cat.armure(n))
            .filter(|a| a.genre == GenreProtection::Bouclier);

        // Poids total : armure, bouclier, armes, objets.
        let mut poids = 0.0;
        let mut inconnus = Vec::new();
        let mut compter = |nom: &str, p: Option<Option<f64>>, quantite: u32| match p {
            Some(Some(p)) => poids += p * f64::from(quantite),
            _ => inconnus.push(nom.to_string()),
        };
        if let Some(a) = armure {
            compter(&a.nom, Some(a.poids_lb), 1);
        }
        if let Some(b) = bouclier {
            compter(&b.nom, Some(b.poids_lb), 1);
        }
        for n in &self.armes {
            compter(n, cat.poids_de(n), 1);
        }
        for o in &self.objets {
            compter(&o.nom_en, cat.poids_de(&o.nom_en), o.quantite);
        }

        let charge = self.charge_declaree;

        // Plafond de DEX : le plus bas entre armure, bouclier et charge (« ne pas cumuler »).
        let mut dex_max: Option<(i32, String)> = None;
        let mut proposer = |v: Option<i32>, d: String| {
            if let Some(v) = v {
                if dex_max.as_ref().is_none_or(|(m, _)| v < *m) {
                    dex_max = Some((v, d));
                }
            }
        };
        if let Some(a) = armure {
            proposer(a.dex_max, a.nom.clone());
        }
        if let Some(b) = bouclier {
            proposer(b.dex_max, b.nom.clone());
        }
        match charge {
            Charge::Intermediaire => proposer(Some(3), "charge intermédiaire".into()),
            Charge::Lourde => proposer(Some(1), "charge lourde".into()),
            Charge::Surcharge => proposer(Some(0), "surcharge".into()),
            Charge::Legere => {}
        }

        // Malus d'armure : armure + bouclier s'additionnent ; la charge ne se cumule pas (le pire).
        let malus_equipement = armure.and_then(|a| a.malus_armure).unwrap_or(0)
            + bouclier.and_then(|b| b.malus_armure).unwrap_or(0);
        let malus_charge = match charge {
            Charge::Legere => 0,
            Charge::Intermediaire => -3,
            Charge::Lourde | Charge::Surcharge => -6,
        };
        let malus_d_armure = if malus_charge < malus_equipement {
            (malus_charge, format!("charge {}", charge.nom()))
        } else {
            let mut noms = Vec::new();
            if let Some(a) = armure {
                noms.push(a.nom.as_str());
            }
            if let Some(b) = bouclier {
                noms.push(b.nom.as_str());
            }
            (malus_equipement, noms.join(" + "))
        };

        let echec = armure.and_then(|a| a.echec_sorts_pct).unwrap_or(0)
            + bouclier.and_then(|b| b.echec_sorts_pct).unwrap_or(0);

        // Vitesse : l'armure intermédiaire ou lourde et la charge réduisent ; on prend la pire.
        let mut vitesse = ctx.vitesse_base;
        if !ctx.vitesse_jamais_reduite {
            let reduite = cat.charge.vitesse_reduite(ctx.vitesse_base);
            if armure.is_some_and(|a| {
                matches!(
                    a.genre,
                    GenreProtection::Intermediaire | GenreProtection::Lourde
                )
            }) {
                vitesse = vitesse.min(reduite);
            }
            match charge {
                Charge::Intermediaire | Charge::Lourde => vitesse = vitesse.min(reduite),
                Charge::Surcharge => vitesse = vitesse.min(5),
                Charge::Legere => {}
            }
        }

        let mut effets = Vec::new();
        if let Some(a) = armure {
            if let Some(v) = a.bonus_armure.filter(|v| *v != 0) {
                effets.push(Effet::new("ca", TypeBonus::Armure, v, &a.nom));
            }
            let m = self.bonus_magique(&a.nom_en);
            if m != 0 {
                effets.push(Effet::new(
                    "ca",
                    TypeBonus::Alteration,
                    m,
                    &format!("{} +{m}", a.nom),
                ));
            }
        }
        if let Some(b) = bouclier {
            if let Some(v) = b.bonus_armure.filter(|v| *v != 0) {
                effets.push(Effet::new("ca", TypeBonus::Bouclier, v, &b.nom));
            }
            let m = self.bonus_magique(&b.nom_en);
            if m != 0 {
                effets.push(Effet::new(
                    "ca",
                    TypeBonus::Alteration,
                    m,
                    &format!("{} +{m}", b.nom),
                ));
            }
        }

        let mut non_maitrises = Vec::new();
        let mut malus_attaque = 0;
        for p in [armure, bouclier].into_iter().flatten() {
            if !ctx.maitrises.armure(p) {
                non_maitrises.push(p.nom.clone());
                malus_attaque += p.malus_armure.unwrap_or(0);
            }
        }

        BilanEquipement {
            poids_total: poids,
            poids_inconnu: inconnus,
            limites: cat.charge.limites(ctx.force),
            charge,
            dex_max,
            malus_d_armure,
            echec_sorts_pct: echec.min(100),
            vitesse,
            effets,
            non_maitrises,
            malus_attaque_non_maitrise: malus_attaque,
        }
    }
}
