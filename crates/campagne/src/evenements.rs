//! Événements du journal. Chaque modification de la campagne est un événement immuable ;
//! l'état est recalculé en rejouant le journal (annuler/rétablir, audit).

use crate::modele::*;
use domaine::{CoutCapitaux, Montant};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Evenement {
    PersonnageCree {
        id: IdPersonnage,
        nom: String,
        compte_initial: Montant,
        prestige: Option<i32>,
    },
    PersonnageModifie {
        id: IdPersonnage,
        nom: String,
        prestige: Option<i32>,
    },
    /// Le PJ change de localité (ou quitte toute localité : `None`).
    PersonnageDeplace {
        id: IdPersonnage,
        vers: Option<IdLocalite>,
    },
    LocaliteCreee {
        id: IdLocalite,
        nom: String,
        genre: String,
        population: Option<u32>,
        categorie: CategorieLocalite,
        notes: String,
    },
    LocaliteModifiee {
        id: IdLocalite,
        nom: String,
        genre: String,
        population: Option<u32>,
        categorie: CategorieLocalite,
        notes: String,
    },
    /// Compte du PJ → trésorerie de la localité.
    InjectionPo {
        pj: IdPersonnage,
        localite: IdLocalite,
        montant: Montant,
    },
    /// Trésorerie de la localité → compte du PJ.
    RetraitPo {
        localite: IdLocalite,
        pj: IdPersonnage,
        montant: Montant,
    },
    /// Gain ou dépense du PJ hors de l'application (butin, achats…).
    AjustementCompte {
        pj: IdPersonnage,
        delta: Montant,
        motif: String,
    },
    /// Correction manuelle des soldes d'une localité (récompense du MJ, erreur de saisie…).
    AjustementLocalite {
        localite: IdLocalite,
        delta_po: Montant,
        /// Ajouts de capitaux.
        ajout: CoutCapitaux,
        /// Retraits de capitaux.
        retrait: CoutCapitaux,
        motif: String,
    },
    TransfertEnvoye {
        id: IdTransfert,
        de: IdLocalite,
        vers: IdLocalite,
        lot: Lot,
        duree_jours: u32,
    },
    JoursAvances {
        nombre: u32,
    },
    // ---- bâtiments ----
    BatimentCree {
        id: IdBatiment,
        localite: IdLocalite,
        nom: String,
        classement: String,
        notes: String,
        etages_successifs: bool,
        modele: Option<String>,
    },
    BatimentModifie {
        id: IdBatiment,
        nom: String,
        classement: String,
        notes: String,
        etages_successifs: bool,
    },
    EtageAjoute {
        batiment: IdBatiment,
        id: IdEtage,
        nom: String,
        mode: ModeChantier,
    },
    EtageModifie {
        batiment: IdBatiment,
        id: IdEtage,
        nom: String,
        mode: ModeChantier,
    },
    /// Ajoute une salle à un étage, à construire ou déjà construite.
    SalleAjoutee {
        id: IdSalle,
        batiment: IdBatiment,
        etage: IdEtage,
        reference: String,
        nom: Option<String>,
        deja_construite: bool,
    },
    SalleModifiee {
        id: IdSalle,
        etage: IdEtage,
        nom: Option<String>,
        ressources: Vec<domaine::Ressource>,
        revenus_actifs: bool,
        brisee: bool,
    },
    SalleRetiree {
        id: IdSalle,
    },
    /// Lance la construction (facteur 1 à 4 : accélération en Travail, GC p. 95).
    ConstructionLancee {
        salle: IdSalle,
        facteur: u32,
    },
    /// Achète la salle terminée. `prix` : prix saisi (par défaut le prix imprimé) ;
    /// `payeur` : par défaut la trésorerie de la localité du bâtiment.
    SalleAchetee {
        salle: IdSalle,
        #[cfg_attr(feature = "serde", serde(default))]
        payeur: Option<Payeur>,
        #[cfg_attr(feature = "serde", serde(default))]
        prix: Option<Montant>,
    },
    /// Paiement en po (ex. bâtiment acheté directement, prix saisi par le joueur).
    Paiement {
        payeur: Payeur,
        montant: Montant,
        motif: String,
    },
    /// Achat de capitaux à leur valeur d'achat (Table 2-1, GC p. 78), versés dans les
    /// réserves de la localité.
    CapitauxAchetes {
        localite: IdLocalite,
        payeur: Payeur,
        capitaux: CoutCapitaux,
    },
    AmeliorationLancee {
        salle: IdSalle,
        vers: String,
        facteur: u32,
    },
    /// Plusieurs événements appliqués d'un bloc (ex. bâtiment créé depuis un modèle).
    Groupe {
        libelle: String,
        evenements: Vec<Evenement>,
    },
    /// Réglage de la limite des dépenses (PC-14) : `false` = une limite par localité (défaut),
    /// `true` = une limite par PJ présent, additionnées. Journalisé pour que le changement
    /// ne s'applique qu'à partir de ce jour et reste annulable.
    ReglageLimiteDepense {
        par_pj: bool,
    },
    Note {
        texte: String,
    },
}

/// Entrée du journal : numéro d'ordre et jour de jeu où l'événement a eu lieu.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entree {
    pub numero: u64,
    pub jour: u32,
    pub evenement: Evenement,
}
