//! Entités d'une campagne : personnages, localités, transferts en cours.

use domaine::{CoutCapitaux, Montant, TypeCapital};

macro_rules! identifiant {
    ($nom:ident, $prefixe:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $nom(pub u32);
        impl std::fmt::Display for $nom {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, concat!($prefixe, "{}"), self.0)
            }
        }
    };
}
identifiant!(IdPersonnage, "PJ-");
identifiant!(IdLocalite, "L-");
identifiant!(IdTransfert, "T-");

/// Catégorie de communauté retenue pour une localité.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CategorieLocalite {
    /// Catégorie officielle du référentiel (id : « village », « metropole »…).
    Officielle { id: String },
    /// Lieu non couvert par une catégorie officielle : nom et limite définis par l'utilisateur.
    Personnalisee {
        nom: String,
        limite_depense_jour: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Localite {
    pub id: IdLocalite,
    pub nom: String,
    /// Libellé libre : ville, village, lieu-dit, avant-poste…
    pub genre: String,
    pub population: Option<u32>,
    pub categorie: CategorieLocalite,
    /// Trésorerie en po de la localité ; peut devenir négative (affichée en rouge).
    pub tresorerie: Montant,
    /// Réserves de capitaux ; jamais négatives.
    pub capitaux: CoutCapitaux,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Personnage {
    pub id: IdPersonnage,
    pub nom: String,
    /// Compte personnel en po, distinct des trésoreries des localités.
    pub compte: Montant,
    /// Localité où se trouve le PJ (usure des capitaux et des entreprises, GC p. 82).
    pub localite: Option<IdLocalite>,
    /// Valeur de Prestige, ou niveau + modificateur de Charisme (limite de recrutement, GC p. 93).
    pub prestige: Option<i32>,
}

/// Ce qui est transféré : des po ou un capital. L'Influence ne peut pas être déplacée
/// Qui paie un achat en po : la trésorerie d'une localité ou le compte d'un PJ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Payeur {
    Localite(IdLocalite),
    Personnage(IdPersonnage),
}

/// (GC p. 81 « Déplacer des capitaux » ; décision de l'utilisateur).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Lot {
    Po(Montant),
    Capital(TypeCapital, u32),
}

impl std::fmt::Display for Lot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lot::Po(m) => write!(f, "{m}"),
            Lot::Capital(c, n) => write!(f, "{n} {c}"),
        }
    }
}

/// Transfert entre localités, en route jusqu'au jour d'arrivée.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transfert {
    pub id: IdTransfert,
    pub de: IdLocalite,
    pub vers: IdLocalite,
    pub lot: Lot,
    pub jour_depart: u32,
    pub jour_arrivee: u32,
}

// ---------------------------------------------------------------- bâtiments

identifiant!(IdBatiment, "B-");
identifiant!(IdEtage, "E-");
identifiant!(IdSalle, "S-");

/// Construction des salles d'un même étage (option personnalisée, le livre ne précise rien).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModeChantier {
    /// Toutes les salles en travaux avancent en même temps.
    Parallele,
    /// Une seule salle à la fois, dans l'ordre de l'étage.
    Sequentiel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Etage {
    pub id: IdEtage,
    pub nom: String,
    pub mode: ModeChantier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NatureChantier {
    Construction,
    /// Amélioration vers une autre salle du référentiel (GC p. 95).
    Amelioration {
        vers: String,
    },
}

/// Travaux en cours sur une salle : capitaux à dépenser et durée à écouler (GC p. 85).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Chantier {
    pub nature: NatureChantier,
    /// Coût total, accélération comprise.
    pub cout: CoutCapitaux,
    pub depense: CoutCapitaux,
    /// Durée totale en jours, accélération comprise.
    pub duree: u32,
    pub jours_ecoules: u32,
    /// Facteur d'accélération 1 à 4 (GC p. 95).
    pub facteur: u32,
    pub jour_debut: u32,
}

impl Chantier {
    pub fn reste(&self) -> CoutCapitaux {
        let mut r = CoutCapitaux::default();
        for c in TypeCapital::TOUS {
            *r.get_mut(c) = self.cout.get(c).saturating_sub(self.depense.get(c));
        }
        r
    }
    pub fn termine(&self) -> bool {
        self.reste().unites() == 0 && self.jours_ecoules >= self.duree
    }
    /// Jours restants au mieux : temps restant, ou capitaux restants ÷ limite quotidienne.
    pub fn jours_restants_estimes(&self, limite: u32) -> u32 {
        let temps = self.duree.saturating_sub(self.jours_ecoules);
        let capitaux = if limite == 0 {
            if self.reste().unites() > 0 {
                u32::MAX
            } else {
                0
            }
        } else {
            self.reste().unites().div_ceil(limite)
        };
        temps.max(capitaux)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatutSalle {
    AConstruire,
    EnTravaux(Chantier),
    Construite,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SalleBatie {
    pub id: IdSalle,
    pub batiment: IdBatiment,
    pub etage: IdEtage,
    /// Id de la salle dans le référentiel (« bar », « forge »…).
    pub reference: String,
    /// Nom donné par l'utilisateur (facultatif).
    pub nom: Option<String>,
    pub statut: StatutSalle,
    /// Ressources que la salle contribue à générer, cochées parmi celles qu'elle admet.
    pub ressources: Vec<domaine::Ressource>,
    /// Génération de recettes activée.
    pub revenus_actifs: bool,
    /// Salle brisée : moitié des recettes (GC p. 92).
    pub brisee: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Batiment {
    pub id: IdBatiment,
    pub localite: IdLocalite,
    pub nom: String,
    /// Classement libre (auberge, temple, entrepôt…).
    pub classement: String,
    pub notes: String,
    /// Étages dans l'ordre (rez-de-chaussée d'abord).
    pub etages: Vec<Etage>,
    /// Étages construits l'un après l'autre (sinon en même temps). Option personnalisée.
    pub etages_successifs: bool,
    /// Modèle du référentiel d'origine, s'il y en a un.
    pub modele: Option<String>,
}
