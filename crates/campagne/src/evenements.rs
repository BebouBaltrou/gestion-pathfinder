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
    /// Opération sur l'inventaire (objets, conteneurs, transferts).
    Inventaire(inventaire::Commande),
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
    /// Ajoute une extension à une salle (GC p. 108), à construire ou déjà faite. Pour le piège
    /// (coût « comme une salle piégée », sans valeur fixe), coût, prix et durée sont saisis.
    ExtensionAjoutee {
        salle: IdSalle,
        reference: String,
        deja_construite: bool,
        cout: Option<CoutCapitaux>,
        prix_po: Option<u32>,
        duree: Option<u32>,
    },
    /// Lance la construction d'une extension prévue (accélération en Travail).
    ExtensionLancee {
        salle: IdSalle,
        reference: String,
        facteur: u32,
    },
    /// Extension achetée terminée (prix saisi, par défaut celui du livre).
    ExtensionAchetee {
        salle: IdSalle,
        reference: String,
        payeur: Option<Payeur>,
        prix: Option<Montant>,
    },
    /// Retire une extension encore à construire (une extension faite est permanente).
    ExtensionRetiree {
        salle: IdSalle,
        reference: String,
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
    // ---- organisations, équipes, gestionnaires ----
    OrganisationCreee {
        id: IdOrganisation,
        localite: IdLocalite,
        nom: String,
        classement: String,
        notes: String,
        modele: Option<String>,
    },
    OrganisationModifiee {
        id: IdOrganisation,
        nom: String,
        classement: String,
        notes: String,
    },
    /// Ajoute une équipe à une organisation, dans la localité de l'organisation.
    EquipeAjoutee {
        id: IdEquipe,
        organisation: IdOrganisation,
        reference: String,
        nom: Option<String>,
        lieu: Option<IdBatiment>,
        deja_recrutee: bool,
    },
    EquipeModifiee {
        id: IdEquipe,
        nom: Option<String>,
        ressources: Vec<domaine::Ressource>,
        revenus_actifs: bool,
    },
    /// Change l'appartenance et/ou le lieu de travail d'une équipe (même localité).
    EquipeAffectee {
        id: IdEquipe,
        organisation: IdOrganisation,
        lieu: Option<IdBatiment>,
    },
    EquipeRetiree {
        id: IdEquipe,
    },
    /// Lance le recrutement (facteur 1 à 4 : accélération en Influence, GC p. 95).
    RecrutementLance {
        equipe: IdEquipe,
        facteur: u32,
    },
    /// Équipe acquise d'un coup : prix saisi (par défaut le prix imprimé), payeur au choix.
    EquipeAchetee {
        equipe: IdEquipe,
        payeur: Option<Payeur>,
        prix: Option<Montant>,
    },
    AmeliorationEquipeLancee {
        equipe: IdEquipe,
        vers: String,
        facteur: u32,
    },
    /// Délocalisation (GC p. 95) : prix payé (la moitié du prix de recrutement selon le livre),
    /// jours de préparation (1d6 par groupe, jet saisi) et de voyage (choisis par le joueur).
    DelocalisationLancee {
        equipe: IdEquipe,
        vers: IdLocalite,
        preparation: u32,
        voyage: u32,
        payeur: Payeur,
        prix: Montant,
    },
    GestionnaireEmbauche {
        id: IdGestionnaire,
        reference: Option<String>,
        nom: String,
        salaire: Montant,
        localite: IdLocalite,
    },
    GestionnaireModifie {
        id: IdGestionnaire,
        nom: String,
        salaire: Montant,
        localite: IdLocalite,
        affectations: Vec<Affectation>,
        actif: bool,
    },
    GestionnaireRenvoye {
        id: IdGestionnaire,
    },
    /// Réglage du salaire d'un gestionnaire affecté à plusieurs établissements :
    /// `false` = payé une fois par jour (défaut, PC-06), `true` = une fois par affectation.
    ReglageSalaireGestionnaire {
        par_affectation: bool,
    },
    /// Phase des recettes d'une localité (GC p. 84 et 94) sur `jours` jours se terminant
    /// aujourd'hui : un test par jour et par ressource, avec les affectations des unités.
    /// `conserves` : capitaux produits que le joueur garde (au plus ce que donnent les tests),
    /// payés au Coût produit (Table 2-1, GC p. 78) par `payeur` (défaut : la localité).
    RecettesPercues {
        localite: IdLocalite,
        jours: u32,
        affectations: Vec<(CleUnite, Vec<(domaine::Ressource, u32)>)>,
        /// Pour chaque ressource testée : un jet par jour (`None` = faire 10).
        jets: Vec<(domaine::Ressource, Vec<JetJour>)>,
        conserves: CoutCapitaux,
        payeur: Option<Payeur>,
        regles: ReglesTest,
    },
    /// Réglage personnalisé : aucune usure des capitaux ni des entreprises dans une localité
    /// qui a un gestionnaire payé.
    ReglageUsure {
        sans_usure_avec_gestionnaire: bool,
    },
    /// Test de prestige au retour après une longue absence (GC p. 82) : résultat noté.
    TestPrestigeResolu {
        localite: IdLocalite,
        reussi: bool,
    },
    /// Abandon d'un bien sans compensation (GC p. 84) : immédiat et définitif.
    AbandonActif {
        cible: Actif,
    },
    /// Mise en vente (GC p. 84) : acheteur trouvé au bout de `jours` (3d6 dans 75 % des cas,
    /// −1d6 par point d'Influence dépensé : jets saisis par le joueur) ; le produit (po et/ou
    /// capitaux, par défaut la moitié du prix ou du coût) arrive à ce moment-là.
    VenteLancee {
        cible: Actif,
        jours: u32,
        influence: u32,
        po: Montant,
        capitaux: CoutCapitaux,
    },
    VenteAnnulee {
        cible: Actif,
    },
    /// Réparation d'une salle brisée (GC p. 92) : po et/ou capitaux (réserves de la localité),
    /// en règle générale la moitié du prix de construction.
    SalleReparee {
        salle: IdSalle,
        payeur: Option<Payeur>,
        po: Montant,
        capitaux: CoutCapitaux,
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
