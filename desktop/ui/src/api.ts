// Pont vers le cœur Rust (commandes Tauri). Les types reprennent ceux de crates/vues.

import type {
  ActionCombat,
  ActionDef,
  ApercuFin,
  ApercuSort,
  AventureVue,
  CombatVue,
  Duree,
  EvenementAventure,
  HistoriqueVue,
  Recompenses,
  ReculVue,
  Validation,
  XpVue,
  RegleOptionnelle,
} from "./aventure-types";
import type {
  ApercuImport,
  ApercuMontee,
  CatalogueCreation,
  EssaiMontee,
  EvenementFiche,
  FicheDonnees,
  FicheVue,
  LignePersonnage,
  PrestigeDonVue,
  Montee,
  RegleInfo,
} from "./fiche-types";

export type Somme = { pa: number; texte: string };
export type Capitaux = { marchandises: number; influence: number; travail: number; magie: number };

export type Entete = {
  campagne: string;
  jour: number;
  date: string;
  peutAnnuler: boolean;
  peutRetablir: boolean;
  derniereAction: string | null;
  pastilles: [string, string][];
};

export type FichierCampagne = { nom: string; chemin: string; ouverte: boolean };

export type EtatApp = {
  entete: Entete | null;
  campagnes: FichierCampagne[];
  dossier: string;
  simulation: [number, number] | null;
  combatEnCours: boolean;
};

export type Retour = { ok: boolean; alerte: boolean; message: string };

export type CarteLocalite = {
  id: number;
  nom: string;
  genre: string;
  categorie: string;
  tresorerie: Somme;
  salairesJour: Somme | null;
  capitaux: Capitaux;
  recettesEnAttente: number;
  absence: string | null;
  batiments: number;
  organisations: number;
  employes: number;
  presents: string[];
  enRoute: string[];
};

export type Alerte = { texte: string; grave: boolean; page: string };

export type Travail = {
  titre: string;
  detail: string;
  avancement: number;
  joursRestants: number | null;
  fin: string | null;
  page: string;
};

export type TestPrestige = { localite: number; nom: string; dd: number };
export type CartePersonnage = { id: number; nom: string; compte: Somme; lieu: string | null };

export type TableauDeBord = {
  date: string;
  jour: number;
  tresorerieLocalites: Somme;
  comptesPersonnages: Somme;
  capitaux: Capitaux;
  employes: number;
  employesAVenir: number;
  transferts: number;
  localites: CarteLocalite[];
  alertes: Alerte[];
  travaux: Travail[];
  prestige: TestPrestige[];
  personnages: CartePersonnage[];
};

// ------------------------------------------------------------------ localités

export type SaisieLocalite = {
  nom: string;
  genre: string;
  population: string;
  officielle: boolean;
  categorieOfficielle: string;
  categoriePersoNom: string;
  categoriePersoLimite: string;
  notes: string;
};

/** Localité prête à l'emploi (villes de Varisie du wiki), formulaire déjà rempli. */
export type ModeleLocalite = {
  id: string;
  nom: string;
  libelle: string;
  officiel: boolean;
  remarque: string | null;
  source: string;
  saisie: SaisieLocalite;
};

export type ChoixCategorie = {
  id: string;
  libelle: string;
  nom: string;
  limite: number;
  sourcePopulation: string;
  sourceLimite: string;
  pointAConfirmer: string | null;
};

export type FicheLocalite = {
  id: number;
  nom: string;
  genre: string;
  population: number | null;
  categorie: string;
  personnalisee: boolean;
  tresorerie: Somme;
  capitaux: Capitaux;
  transferts: string[];
  notes: string;
  saisie: SaisieLocalite;
};

export type Choix = { id: number; libelle: string };

export type PageLocalites = {
  localites: FicheLocalite[];
  personnages: Choix[];
  categories: ChoixCategorie[];
  modeles: ModeleLocalite[];
};

export type FichePersonnage = {
  id: number;
  nom: string;
  compte: Somme;
  localite: number | null;
  lieu: string;
  prestige: number | null;
};

export type PagePersonnages = { personnages: FichePersonnage[]; localites: Choix[] };

// ------------------------------------------------------------------ bâtiments

export type RessourceId = "po" | "marchandises" | "influence" | "travail" | "magie";

export type Cible = { genre: "batiment" | "salle" | "organisation" | "equipe"; id: number };

export type EtatVente = {
  cible: Cible;
  libelle: string;
  enCours: string | null;
  valeur: string;
  produit: string;
  influenceDispo: number;
};

export type PageBatiments = {
  groupes: { localite: string; batiments: { id: number; nom: string; enTravaux: number }[] }[];
  localites: Choix[];
  modeles: { id: string; libelle: string; personnel: boolean }[];
  salles: { id: string; nom: string; libelle: string }[];
  payeurs: Choix[];
};

export type Acquisition = "plan" | "possede" | "construire" | "acheterCapitaux" | "acheterBatiment";

export type SaisieBatiment = {
  localite: number | null;
  modele: string | null;
  nom: string;
  classement: string;
  acquisition: Acquisition;
  facteur: number;
  seulementManquants: boolean;
  prix: string;
  payeur: number | null;
};

export type ApercuCreation = {
  composition: string;
  personnel: boolean;
  source: string;
  besoin: Capitaux;
  aAcheter: Capitaux;
  prixCapitauxPo: number;
  prixLivrePo: number;
  tresorerie: string;
};

export type Avancement = {
  capitauxFaits: number;
  capitauxTotal: number;
  jours: number;
  duree: number;
  estimation: string;
  manque: string | null;
};

export type FicheExtension = {
  reference: string;
  nom: string;
  statutTexte: string;
  aConstruire: boolean;
  lancable: boolean;
  description: string;
  detail: string;
  prixPo: number;
  manque: string | null;
};

export type ExtensionPossible = { id: string; nom: string; aide: string; saisieRequise: boolean; detail: string };

export type FicheSalle = {
  id: number;
  nom: string;
  typeSalle: string | null;
  statut: "a_construire" | "en_travaux" | "construite";
  statutTexte: string;
  brisee: boolean;
  source: string;
  infos: string;
  pointAConfirmer: string | null;
  admises: RessourceId[];
  ressource: RessourceId | null;
  revenusActifs: boolean;
  chantier: Avancement | null;
  constructible: boolean;
  prixAchatPo: number | null;
  ameliorations: { id: string; nom: string; detail: string | null }[];
  vente: EtatVente | null;
  reparation: [string, string | null] | null;
  extensions: FicheExtension[];
  extensionsPossibles: ExtensionPossible[];
};

export type FicheEtage = {
  id: number;
  nom: string;
  sequentiel: boolean;
  dureeRestante: number;
  reste: Capitaux;
  salles: FicheSalle[];
};

export type FicheBatiment = {
  id: number;
  nom: string;
  classement: string;
  notes: string;
  modele: string | null;
  localite: number;
  lieu: string;
  reste: Capitaux;
  dureeRestante: number;
  bonus: { ressource: RessourceId; valeur: number }[];
  equipes: string;
  gestionnaires: string;
  etagesSuccessifs: boolean;
  vente: EtatVente;
  etages: FicheEtage[];
  tresorerie: string;
  payeurs: Choix[];
};

/** Action sur un bâtiment, une salle ou une extension (le cœur la traduit en événement). */
export type ActionBatiment =
  | { type: "etagesSuccessifs"; batiment: number; actif: boolean }
  | { type: "modeEtage"; batiment: number; etage: number; sequentiel: boolean }
  | { type: "ajouterEtage"; batiment: number; nom: string }
  | { type: "ajouterSalle"; batiment: number; etage: number; reference: string; deja: boolean }
  | { type: "reglerSalle"; salle: number; ressource: RessourceId | null; revenusActifs: boolean; brisee: boolean }
  | { type: "lancerConstruction"; salle: number; facteur: number }
  | { type: "acheterSalle"; salle: number; payeur: number | null }
  | { type: "sallePossedee"; salle: number }
  | { type: "retirerSalle"; salle: number }
  | { type: "ameliorer"; salle: number; vers: string; facteur: number }
  | { type: "reparer"; salle: number; enCapitaux: boolean; payeur: number | null }
  | { type: "lancerExtension"; salle: number; reference: string; facteur: number }
  | { type: "acheterExtension"; salle: number; reference: string; payeur: number | null }
  | { type: "extensionPossedee"; salle: number; reference: string }
  | { type: "retirerExtension"; salle: number; reference: string }
  | {
      type: "ajouterExtension";
      salle: number;
      reference: string;
      deja: boolean;
      cout: Capitaux | null;
      prix: string;
      duree: string;
    }
  | { type: "annulerVente"; cible: Cible }
  | { type: "abandonner"; cible: Cible; sansAcheteur: boolean }
  | { type: "garderSansAcheteur"; cible: Cible };

// ------------------------------------------------------------------ organisations

export type PageOrganisations = {
  groupes: { localite: string; organisations: { id: number; nom: string; equipes: number; effectif: number }[] }[];
  localites: Choix[];
  modeles: { id: string; libelle: string; personnel: boolean }[];
  equipes: { id: string; nom: string; libelle: string }[];
  payeurs: Choix[];
};

export type FicheEquipe = {
  id: number;
  nom: string;
  typeEquipe: string | null;
  statut: "a_recruter" | "en_recrutement" | "recrutee" | "en_delocalisation";
  statutTexte: string;
  ailleurs: string | null;
  effectif: number;
  source: string;
  infos: string;
  pointAConfirmer: string | null;
  organisation: number;
  lieu: number | null;
  lieux: Choix[];
  moitieSansBatiment: boolean;
  admises: RessourceId[];
  ressource: RessourceId | null;
  revenusActifs: boolean;
  chantier: Avancement | null;
  constructible: boolean;
  prixAchatPo: number | null;
  ameliorations: { id: string; nom: string; detail: string | null }[];
  vente: EtatVente | null;
  delocalisable: boolean;
  prixDeloc: string;
};

export type FicheOrganisation = {
  id: number;
  nom: string;
  classement: string;
  modele: string | null;
  siege: string;
  localite: number;
  effectif: string;
  reste: Capitaux;
  bonus: { ressource: RessourceId; valeur: number }[];
  gestionnaires: string;
  vente: EtatVente;
  equipes: FicheEquipe[];
  tresorerie: string;
  payeurs: Choix[];
  organisations: Choix[];
  localites: Choix[];
  jetUnique: boolean;
  equipesRecrutees: number;
  prixDelocOrganisation: string;
};

export type ActionOrganisation =
  | { type: "ajouterEquipe"; organisation: number; reference: string; deja: boolean }
  | { type: "affecterEquipe"; equipe: number; organisation: number; lieu: number | null }
  | { type: "reglerEquipe"; equipe: number; ressource: RessourceId | null; revenusActifs: boolean }
  | { type: "lancerRecrutement"; equipe: number; facteur: number }
  | { type: "acheterEquipe"; equipe: number; payeur: number | null }
  | { type: "equipePossedee"; equipe: number }
  | { type: "retirerEquipe"; equipe: number }
  | { type: "ameliorerEquipe"; equipe: number; vers: string; facteur: number }
  | { type: "delocaliser"; equipe: number; vers: number; preparation: string; voyage: string; payeur: number | null; prix: string }
  | {
      type: "delocaliserOrganisation";
      organisation: number;
      vers: number;
      preparation: string;
      voyage: string;
      payeur: number | null;
      prix: string;
    };

// ------------------------------------------------------------------ gestionnaires

export type CibleAffectation = { genre: "batiment" | "organisation"; id: number };
export type ChoixAffectation = { cible: CibleAffectation; libelle: string };

export type PageGestionnaires = {
  salaires: { localite: string; salairesJour: Somme; tresorerie: Somme; negative: boolean; joursDAvance: number | null }[];
  exemples: { id: string; nom: string; libelle: string; salaire: string; competences: string }[];
  localites: Choix[];
  gestionnaires: {
    id: number;
    nom: string;
    exemple: string | null;
    salaireJour: Somme;
    actif: boolean;
    localite: number;
    affectations: ChoixAffectation[];
    possibles: ChoixAffectation[];
  }[];
  noteSalaire: string;
};

export type ActionGestionnaire =
  | { type: "embaucher"; reference: string | null; nom: string; salaire: string; localite: number }
  | { type: "regler"; id: number; actif: boolean; localite: number }
  | { type: "renvoyer"; id: number }
  | { type: "affecter"; id: number; cible: CibleAffectation }
  | { type: "desaffecter"; id: number; cible: CibleAffectation };

// ------------------------------------------------------------------ historique, points, paramètres

export type LigneJournal = { numero: number; jour: number; date: string; texte: string };
export type PageHistorique = { annulees: LigneJournal[]; journal: LigneJournal[] };

export type PagePoints = {
  avertissementsMasques: boolean;
  controles: string[];
  total: number;
  ouverts: number;
  points: {
    code: string;
    ouvert: boolean;
    source: string;
    question: string;
    lectureAppliquee: string;
    decision: string | null;
  }[];
};

export type UniteVue = {
  id: string;
  nom: string;
  recettes: string;
  avantages: string | null;
  cout: string;
  prix: string;
  duree: string;
  taille: string;
  page: string;
  source: string;
  amelioreDe: string[];
  amelioreEn: string[];
  statut: "officiel" | "a_confirmer";
  point: string | null;
};

export type ModeleVue = {
  nom: string;
  page: string;
  cout: string;
  prix: string;
  composition: string;
  imprime: string | null;
  statut: "officiel" | "a_confirmer";
  point: string | null;
};

export type PageReferentiel = {
  salles: UniteVue[];
  equipes: UniteVue[];
  extensions: UniteVue[];
  gestionnaires: { nom: string; salaire: string; competences: string; page: string }[];
  modelesBatiments: ModeleVue[];
  modelesOrganisations: ModeleVue[];
  communautes: { nom: string; population: string; limite: string; statut: "officiel" | "a_confirmer"; point: string | null }[];
  capitaux: { ressource: RessourceId; nom: string; coutAchat: string; coutProduit: string }[];
};

export type SaisieCalendrier = {
  jourSemaine: number;
  jour: number;
  mois: number;
  annee: number;
  bissextile: "quatre" | "huit";
};

export type PageParametres = {
  campagne: string;
  reglages: { id: string; titre: string; aide: string; actif: boolean; statut: "officiel" | "personnalise" | null; point: string | null }[];
  calendrier: {
    jourSemaine: number;
    jour: number;
    mois: number;
    annee: number;
    bissextile: "quatre" | "huit";
    joursSemaine: string[];
    nomsMois: string[];
    apercu: string;
  };
};

// ------------------------------------------------------------------ recettes

export type CleU = { genre: "salle" | "equipe"; id: number };
export type Points = { ressource: RessourceId; points: number };
export type SaisieJet = { ressource: RessourceId; prendre10: boolean; des: string };

/** Saisie de la page Recettes ; les champs absents prennent leur valeur par défaut (calculée par le cœur). */
export type EntreeRecettes = {
  localite?: number | null;
  jours?: string | null;
  batiments?: number[] | null;
  organisations?: number[] | null;
  repartition: { cle: CleU; points: Points[] }[];
  jets: SaisieJet[];
  conserverTout?: boolean | null;
  conserves: Capitaux;
  payeur: number | null;
};

export type VueRecettes = {
  localites: Choix[];
  localite: number | null;
  joursEnAttente: number;
  jours: string;
  batiments: { id: number; nom: string; coche: boolean }[];
  organisations: { id: number; nom: string; coche: boolean }[];
  sansType: string[];
  repartitionStricte: boolean;
  unites: {
    cle: CleU;
    libelle: string;
    bonus: number;
    admises: RessourceId[];
    points: Points[];
    somme: number;
    defaut: Points[];
    precedente: Points[] | null;
  }[];
  tests: { ressource: RessourceId; prendre10: boolean; des: string; erreur: string | null }[];
  gratuits: boolean;
  conserverTout: boolean;
  payeurs: Choix[];
  tresorerie: string;
  bilan: {
    premierJour: { ressource: RessourceId; lignes: string; prendre10: boolean; formule: string }[];
    detailJours: string[];
    jours: number;
    poBrut: Somme;
    capitauxBruts: Capitaux;
    absence: string | null;
    poNet: Somme;
    capitauxNets: Capitaux;
    conservables: { capital: RessourceId; max: number; garde: number; coutUnitePo: number }[];
    coutConserves: Somme;
  } | null;
  erreur: string | null;
  encaissable: boolean;
};

/** Ce qu'un transfert déplace (l'Influence ne se transfère pas, GC p. 81). */
export type RessourceTransfert = "po" | "marchandises" | "travail" | "magie";

declare const __MOCK__: boolean;

type Invoke = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

function pont(): Invoke {
  const t = (window as unknown as { __TAURI__?: { core: { invoke: Invoke } } }).__TAURI__;
  if (t) return t.core.invoke;
  if (__MOCK__) return simule;
  return () => Promise.reject(new Error("L'interface doit être lancée depuis l'application."));
}

const appeler = <T>(cmd: string, args?: Record<string, unknown>) => pont()(cmd, args) as Promise<T>;

export const api = {
  etatApp: () => appeler<EtatApp>("etat_app"),
  ouvrirCampagne: (chemin: string) => appeler<Retour>("ouvrir_campagne", { chemin }),
  creerCampagne: (nom: string, calendrier?: SaisieCalendrier) => appeler<Retour>("creer_campagne", { nom, calendrier: calendrier ?? null }),
  calendrierParDefaut: () => appeler<PageParametres["calendrier"]>("calendrier_par_defaut"),
  exporterCampagne: () => appeler<Retour>("exporter_campagne"),
  exporterRecapitulatif: () => appeler<Retour>("exporter_recapitulatif"),
  exporterHistorique: () => appeler<Retour>("exporter_historique"),
  importerCampagne: () => appeler<Retour>("importer_campagne"),
  ouvrirDossier: () => appeler<Retour>("ouvrir_dossier"),
  tableauDeBord: () => appeler<TableauDeBord | null>("tableau_de_bord"),
  avancer: (jours: number) => appeler<Retour>("avancer", { jours }),
  annuler: () => appeler<Retour>("annuler"),
  retablir: () => appeler<Retour>("retablir"),
  resoudrePrestige: (localite: number, reussi: boolean) =>
    appeler<Retour>("resoudre_prestige", { localite, reussi }),
  simulation: (action: "demarrer" | "abandonner" | "garder") =>
    appeler<Retour>("simulation", { action }),

  pageLocalites: () => appeler<PageLocalites | null>("page_localites"),
  categorieSuggeree: (population: string) => appeler<ChoixCategorie | null>("categorie_suggeree", { population }),
  lireMontant: (texte: string) => appeler<Somme | null>("lire_montant", { texte }),
  creerLocalite: (saisie: SaisieLocalite) => appeler<Retour>("creer_localite", { saisie }),
  modifierLocalite: (id: number, saisie: SaisieLocalite) => appeler<Retour>("modifier_localite", { id, saisie }),
  mouvementPo: (injection: boolean, pj: number, localite: number, montant: string) =>
    appeler<Retour>("mouvement_po", { injection, pj, localite, montant }),
  transfert: (de: number, vers: number, ressource: RessourceTransfert, quantite: string, duree: string) =>
    appeler<Retour>("transfert", { de, vers, ressource, quantite, duree }),
  ajustement: (localite: number, deltaPo: string, ajout: Capitaux, retrait: Capitaux, motif: string) =>
    appeler<Retour>("ajustement", { localite, deltaPo, ajout, retrait, motif }),


  pagePersonnages: () => appeler<PagePersonnages | null>("page_personnages"),
  modifierPersonnage: (id: number, nom: string, prestige: string) =>
    appeler<Retour>("modifier_personnage", { id, nom, prestige }),
  deplacerPersonnage: (id: number, vers: number | null) => appeler<Retour>("deplacer_personnage", { id, vers }),
  ajusterCompte: (pj: number, delta: string, motif: string) => appeler<Retour>("ajuster_compte", { pj, delta, motif }),

  pageBatiments: () => appeler<PageBatiments | null>("page_batiments"),
  apercuCreationBatiment: (saisie: SaisieBatiment) => appeler<ApercuCreation | null>("apercu_creation_batiment", { saisie }),
  creerBatiment: (saisie: SaisieBatiment) => appeler<Retour>("creer_batiment", { saisie }),
  ficheBatiment: (id: number) => appeler<FicheBatiment | null>("fiche_batiment", { id }),
  actionBatiment: (action: ActionBatiment) => appeler<Retour>("action_batiment", { action }),
  miseEnVente: (cible: Cible, jours: string, influence: number, po: string) =>
    appeler<Retour>("mise_en_vente", { cible, jours, influence, po }),
  enregistrerModeleBatiment: (id: number) => appeler<Retour>("enregistrer_modele_batiment", { id }),
  supprimerModeleBatiment: (id: string) => appeler<Retour>("supprimer_modele_batiment", { id }),

  pageOrganisations: () => appeler<PageOrganisations | null>("page_organisations"),
  apercuCreationOrganisation: (saisie: SaisieBatiment) =>
    appeler<ApercuCreation | null>("apercu_creation_organisation", { saisie }),
  creerOrganisation: (saisie: SaisieBatiment) => appeler<Retour>("creer_organisation", { saisie }),
  ficheOrganisation: (id: number) => appeler<FicheOrganisation | null>("fiche_organisation", { id }),
  actionOrganisation: (action: ActionOrganisation) => appeler<Retour>("action_organisation", { action }),
  enregistrerModeleOrganisation: (id: number) => appeler<Retour>("enregistrer_modele_organisation", { id }),
  supprimerModeleOrganisation: (id: string) => appeler<Retour>("supprimer_modele_organisation", { id }),

  pageGestionnaires: () => appeler<PageGestionnaires | null>("page_gestionnaires"),
  actionGestionnaire: (action: ActionGestionnaire) => appeler<Retour>("action_gestionnaire", { action }),

  pageHistorique: () => appeler<PageHistorique | null>("page_historique"),
  pagePoints: () => appeler<PagePoints>("page_points"),
  pageReferentiel: () => appeler<PageReferentiel>("page_referentiel"),
  pageParametres: () => appeler<PageParametres | null>("page_parametres"),
  regler: (id: string, actif: boolean) => appeler<Retour>("regler", { id, actif }),
  reglerCalendrier: (jourSemaine: number, jour: number, mois: number, annee: number, bissextile: "quatre" | "huit") =>
    appeler<Retour>("regler_calendrier", { jourSemaine, jour, mois, annee, bissextile }),

  vueRecettes: (entree: EntreeRecettes) => appeler<VueRecettes | null>("vue_recettes", { entree }),
  encaisserRecettes: (entree: EntreeRecettes) => appeler<Retour>("encaisser_recettes", { entree }),
  // ---- personnages et fiches
  listePersonnages: () => appeler<LignePersonnage[]>("liste_personnages"),
  fichePersonnage: (id: number) => appeler<FicheVue | null>("fiche_personnage", { id }),
  catalogueCreation: () => appeler<CatalogueCreation>("catalogue_creation"),
  apercuFiche: (fiche: FicheDonnees) => appeler<FicheVue>("apercu_fiche", { fiche }),
  creerPersonnage: (nom: string, po: number) => appeler<Retour>("creer_personnage", { nom, po }),
  creerFiche: (pj: number, fiche: FicheDonnees) => appeler<Retour>("creer_fiche", { pj, fiche }),
  choisirFichePdf: () => appeler<ApercuImport | null>("choisir_fiche_pdf"),
  importerFichePdf: (chemin: string, gardes: string[], po: number) =>
    appeler<Retour>("importer_fiche_pdf", { chemin, gardes, po }),
  prestigeDon: (pj: number) => appeler<PrestigeDonVue | null>("prestige_don", { pj }),
  prestigeEvenement: (evenement: unknown) => appeler<Retour>("prestige_evenement", { evenement }),
  recruterAcolyte: (maitre: number, nom: string) => appeler<Retour>("recruter_acolyte", { maitre, nom }),
  accorderNiveauDeDepart: (pj: number, niveau: number) => appeler<Retour>("accorder_niveau_de_depart", { pj, niveau }),
  evenementFiche: (pj: number, evenement: EvenementFiche) =>
    appeler<Retour>("evenement_fiche", { pj, evenement }),
  apercuMontee: (pj: number, classe: string) => appeler<ApercuMontee | null>("apercu_montee", { pj, classe }),
  essaiMontee: (pj: number, montee: Montee) => appeler<EssaiMontee | null>("essai_montee", { pj, montee }),
  reglesCampagne: () => appeler<RegleInfo[]>("regles_campagne"),
  changerRegle: (id: string, valeur: string) => appeler<Retour>("changer_regle", { id, valeur }),
  // ---- aventure
  aventureVue: () => appeler<AventureVue>("aventure_vue"),
  aventureDemarrer: (heure: number, minute: number, survie: boolean, degatsMassifs: boolean) =>
    appeler<Retour>("aventure_demarrer", { heure, minute, survie, degatsMassifs }),
  aventureTerminer: () => appeler<Retour>("aventure_terminer"),
  aventureEvenement: (evenement: EvenementAventure) => appeler<Retour>("aventure_evenement", { evenement }),
  aventureApercuRecul: (instant: number) => appeler<ReculVue | null>("aventure_apercu_recul", { instant }),
  aventureActualiserRessources: (pj: number) => appeler<Retour>("aventure_actualiser_ressources", { pj }),
  // ---- combat
  combatVue: () => appeler<CombatVue | null>("combat_vue"),
  combatDemarrer: (engages: number[]) => appeler<Retour>("combat_demarrer", { engages }),
  combatAction: (action: ActionCombat) => appeler<Retour>("combat_action", { action }),
  combatAnnuler: () => appeler<Retour>("combat_annuler"),
  combatRetablir: () => appeler<Retour>("combat_retablir"),
  combatApercuSort: (pj: number, sort: string, metamagies: string[] = [], niveauHeighten: number | null = null) =>
    appeler<ApercuSort>("combat_apercu_sort", { pj, sort, metamagies, niveauHeighten }),
  combatApercuFin: () => appeler<ApercuFin>("combat_apercu_fin"),
  combatTerminer: (validation: Validation, contestations: [number, Duree][], recompenses: Recompenses | null = null) =>
    appeler<Retour>("combat_terminer", { validation, contestations, recompenses }),
  combatXp: (fps: string[], personnages: number) => appeler<XpVue>("combat_xp", { fps, personnages }),
  catalogueFp: () => appeler<string[]>("catalogue_fp"),
  catalogueRegles: () => appeler<RegleOptionnelle[]>("catalogue_regles"),
  combatHistorique: () => appeler<HistoriqueVue>("combat_historique"),
  combatExporter: (numero: number) => appeler<string[]>("combat_exporter", { numero }),
  catalogueActions: () => appeler<ActionDef[]>("catalogue_actions"),
  catalogueEtats: () => appeler<[string, string][]>("catalogue_etats"),
};

// ------------------------------------------------------------------ montants

/** « 1 052 po 3 pa » avec séparateur de milliers. */
export function formaterPo(pa: number): string {
  const signe = pa < 0 ? "−" : "";
  const abs = Math.abs(pa);
  const po = Math.trunc(abs / 10);
  const reste = abs % 10;
  const txtPo = po.toLocaleString("fr-FR");
  if (po === 0 && reste === 0) return "0 po";
  if (po === 0) return `${signe}${reste} pa`;
  if (reste === 0) return `${signe}${txtPo} po`;
  return `${signe}${txtPo} po ${reste} pa`;
}

// ------------------------------------------------------------------ données d'exemple
// Uniquement pour l'aperçu sans Tauri (MOCK=1) : retiré de la version de l'application.

// Aperçu seulement : valeurs factices, les vraies viennent du référentiel.
const CATEGORIES_EXEMPLE: ChoixCategorie[] = [
  { id: "village", libelle: "Village (exemple) — 4/jour", nom: "Village", limite: 4, sourcePopulation: "(aperçu)", sourceLimite: "(aperçu)", pointAConfirmer: null },
  { id: "petite-bourgade", libelle: "Petite bourgade (exemple) — 15/jour", nom: "Petite bourgade", limite: 15, sourcePopulation: "(aperçu)", sourceLimite: "(aperçu)", pointAConfirmer: null },
  { id: "grande-cite", libelle: "Grande cité (exemple) — 60/jour", nom: "Grande cité", limite: 60, sourcePopulation: "(aperçu)", sourceLimite: "(aperçu)", pointAConfirmer: "PC-11" },
];

import { apercuFinExemple, apercuSortExemple, aventureExemple, aventureVide, combatExemple, combatInitiative } from "./mock-aventure";

let mockAventure = true;
let mockCombat: "non" | "initiative" | "encours" = "encours";
let jourSimule = 24;
let annulables = 3;

function simule(cmd: string, args?: Record<string, unknown>): Promise<unknown> {
  const ok = (message: string): Retour => ({ ok: true, alerte: false, message });
  const date = (j: number) => `${["lunedi", "marnedi", "mercantdi", "juredi", "ventdi", "astredi", "solardi"][j % 7]} ${j - 10} Rova 4712 AR`;
  switch (cmd) {
    case "etat_app":
      return Promise.resolve<EtatApp>({
        entete: {
          campagne: "L'Éveil des Seigneurs des Runes",
          jour: jourSimule,
          date: date(jourSimule),
          peutAnnuler: annulables > 0,
          peutRetablir: false,
          derniereAction: "Jour suivant",
          pastilles: [["recettes", `${jourSimule - 15} j`]],
        },
        campagnes: [
          { nom: "L'Éveil des Seigneurs des Runes", chemin: "/a.json", ouverte: true },
          { nom: "Essai", chemin: "/b.json", ouverte: false },
        ],
        dossier: "~/.local/share/GestionPathfinder/campagnes",
        simulation: null,
        combatEnCours: mockCombat !== "non",
      });
    case "avancer":
      jourSimule += Number(args?.jours ?? 1);
      annulables += 1;
      return Promise.resolve(ok("Jour suivant — Bar terminée (Le Dragon vert)"));
    case "annuler":
      jourSimule -= 1;
      annulables -= 1;
      return Promise.resolve(ok("Annulé : 1 jour écoulé"));
    case "tableau_de_bord": {
      const cap = (marchandises: number, influence: number, travail: number, magie: number) => ({ marchandises, influence, travail, magie });
      const t: TableauDeBord = {
        date: date(jourSimule),
        jour: jourSimule,
        tresorerieLocalites: { pa: 12843, texte: "1284 po 3 pa" },
        comptesPersonnages: { pa: 3120, texte: "312 po" },
        capitaux: cap(16, 9, 14, 2),
        employes: 23,
        employesAVenir: 5,
        transferts: 1,
        localites: [
          {
            id: 1, nom: "Sandpoint", genre: "ville", categorie: "Petite bourgade · limite 15/jour",
            tresorerie: { pa: 10523, texte: "1052 po 3 pa" }, salairesJour: { pa: 20, texte: "2 po" },
            capitaux: cap(12, 6, 9, 2), recettesEnAttente: jourSimule - 15, absence: null,
            batiments: 2, organisations: 1, employes: 18, presents: ["Seoni", "Valeros", "Kyra"], enRoute: [],
          },
          {
            id: 2, nom: "Magnimar", genre: "cité", categorie: "Grande cité · limite 60/jour",
            tresorerie: { pa: 2320, texte: "232 po" }, salairesJour: null,
            capitaux: cap(4, 3, 5, 0), recettesEnAttente: 0, absence: "Aucun PJ depuis 12 j — prochaine usure dans 2 j",
            batiments: 1, organisations: 0, employes: 5, presents: [], enRoute: ["3 Marchandises arrive le jour 27"],
          },
        ],
        alertes: [
          { texte: "Sandpoint : 9 jour(s) de recettes à encaisser", grave: false, page: "recettes" },
          { texte: "Magnimar : usure des capitaux dans 2 jour(s) (aucun PJ depuis 12 j)", grave: false, page: "personnages" },
          { texte: "Le Dragon vert — Cuisine : réserves vides pour 2 Travail", grave: true, page: "batiments" },
        ],
        travaux: [
          { titre: "Garde de Sandpoint — Archers", detail: "recrutement, reste 2 Influence", avancement: 0.6, joursRestants: 2, fin: date(jourSimule + 2), page: "organisations" },
          { titre: "Le Dragon vert — Bar", detail: "construction, reste 2 Marchandises, 1 Travail", avancement: 0.75, joursRestants: 4, fin: date(jourSimule + 4), page: "batiments" },
          { titre: "Le Dragon vert — Chambre à coucher : Ameublement", detail: "extension, reste 5 Marchandises, 3 Travail", avancement: 0.45, joursRestants: 11, fin: date(jourSimule + 11), page: "batiments" },
          { titre: "Le Dragon vert — Cuisine", detail: "construction, reste 2 Travail", avancement: 0.5, joursRestants: null, fin: null, page: "batiments" },
        ],
        prestige: [],
        personnages: [
          { id: 1, nom: "Seoni", compte: { pa: 1450, texte: "145 po" }, lieu: "Sandpoint" },
          { id: 2, nom: "Valeros", compte: { pa: 870, texte: "87 po" }, lieu: "Sandpoint" },
          { id: 3, nom: "Kyra", compte: { pa: 800, texte: "80 po" }, lieu: "Sandpoint" },
        ],
      };
      return Promise.resolve(t);
    }
    case "lire_montant": {
      const t = String(args?.texte ?? "").trim().replace(",", ".");
      const n = Number(t);
      return Promise.resolve(t && !Number.isNaN(n) ? { pa: Math.round(n * 10), texte: formaterPo(Math.round(n * 10)) } : null);
    }
    case "categorie_suggeree":
      return Promise.resolve(Number(String(args?.population ?? "").replace(/\s/g, "")) > 0 ? CATEGORIES_EXEMPLE[1] : null);
    case "page_localites": {
      const saisie = (nom: string, categorieOfficielle: string): SaisieLocalite => ({
        nom, genre: "ville", population: "", officielle: true, categorieOfficielle,
        categoriePersoNom: "", categoriePersoLimite: "", notes: "",
      });
      const p: PageLocalites = {
        localites: [
          {
            id: 1, nom: "Sandpoint", genre: "ville", population: 1240, categorie: "Petite bourgade · limite 15/jour",
            personnalisee: false, tresorerie: { pa: 10523, texte: "1052 po 3 pa" },
            capitaux: { marchandises: 12, influence: 6, travail: 9, magie: 2 },
            transferts: ["3 Marchandises part vers Magnimar — arrivée le jour 27 (astredi 17 Rova 4712 AR)"],
            notes: "Port de pêche de la Côte perdue.", saisie: { ...saisie("Sandpoint", "petite-bourgade"), population: "1240" },
          },
          {
            id: 2, nom: "Magnimar", genre: "cité", population: null, categorie: "Grande cité · limite 60/jour",
            personnalisee: false, tresorerie: { pa: 2320, texte: "232 po" },
            capitaux: { marchandises: 4, influence: 3, travail: 5, magie: 0 },
            transferts: ["3 Marchandises arrive de Sandpoint — arrivée le jour 27 (astredi 17 Rova 4712 AR)"],
            notes: "", saisie: saisie("Magnimar", "grande-cite"),
          },
          {
            id: 3, nom: "Tour de Thistletop", genre: "lieu-dit", population: null, categorie: "Avant-poste (personnalisée) · limite 2/jour",
            personnalisee: true, tresorerie: { pa: -40, texte: "-4 po" },
            capitaux: { marchandises: 0, influence: 0, travail: 1, magie: 0 },
            transferts: [], notes: "",
            saisie: { ...saisie("Tour de Thistletop", ""), officielle: false, categoriePersoNom: "Avant-poste", categoriePersoLimite: "2" },
          },
        ],
        personnages: [
          { id: 1, libelle: "Seoni (145 po)" },
          { id: 2, libelle: "Valeros (87 po)" },
        ],
        categories: CATEGORIES_EXEMPLE,
        modeles: [
          {
            id: "korvosa", nom: "Korvosa", libelle: "Korvosa — grande ville · 18 486 hab.", officiel: true,
            remarque: null, source: "Wiki Pathfinder-FR, Golarion.Korvosa",
            saisie: { ...saisie("Korvosa", "grande-ville"), genre: "grande ville", population: "18486", notes: "Alignement : LN." },
          },
          {
            id: "abken", nom: "Abken", libelle: "Abken — petite ville · population inconnue", officiel: false,
            remarque: "Pas de population ni de chiffres sur le wiki : population et catégorie à choisir.",
            source: "Wiki Pathfinder-FR, Golarion.Abken",
            saisie: { ...saisie("Abken", ""), genre: "petite ville" },
          },
        ],
      };
      return Promise.resolve(p);
    }
    case "page_personnages":
      return Promise.resolve({
        personnages: [{ id: 1, nom: "Lyra", compte: { pa: 1000, texte: "100 po" }, localite: 1, lieu: "Sandpoint", prestige: 4 }],
        localites: [{ id: 1, libelle: "Sandpoint" }],
      });
    case "liste_personnages":
      return Promise.resolve([
        { id: 1, nom: "Lyra", compte: { pa: 1000, texte: "100 po" }, lieu: "Sandpoint", prestige: 4, prestigeCalcule: true,
          fiche: { race: "Elfe", classes: "Ensorceleur 1", niveau: 1, pv: 7, ca: 13, xp: 0, niveauAtteignable: 1, niveauxEnAttente: 0 } },
        { id: 2, nom: "Valeros", compte: { pa: 870, texte: "87 po" }, lieu: null, prestige: null, prestigeCalcule: false, fiche: null },
      ]);
    case "fiche_personnage":
      return args?.id === 1 ? fetch("mock/fiche.json").then((r) => r.json()) : Promise.resolve(null);
    case "catalogue_creation":
      return fetch("mock/catalogue.json").then((r) => r.json());
    case "apercu_fiche":
      return fetch("mock/fiche.json").then((r) => r.json());
    case "regles_campagne":
      return Promise.resolve([
        { id: "pj.vitesse", nom: "Vitesse de progression", description: "Table d'expérience utilisée pour monter de niveau.", officielle: true,
          impact: "Fixe les XP nécessaires pour chaque niveau.", defaut: "moyenne", valeur: "moyenne",
          choix: [["lente", "Lente"], ["moyenne", "Moyenne"], ["rapide", "Rapide"]] },
        { id: "inter.repartition_stricte", nom: "Répartition stricte des bonus", description: "Une salle donne tout son bonus à un seul type de capital par jour.",
          officielle: false, impact: "Recettes des salles et des équipes.", defaut: "oui", valeur: "non", choix: [["oui", "Oui"], ["non", "Non"]] },
      ]);
    case "aventure_vue":
      return Promise.resolve(mockAventure ? aventureExemple() : aventureVide());
    case "aventure_demarrer":
      mockAventure = true;
      return Promise.resolve(ok("Aventure démarrée"));
    case "combat_vue":
      return Promise.resolve(mockCombat === "non" ? null : mockCombat === "initiative" ? combatInitiative() : combatExemple());
    case "combat_demarrer":
      mockCombat = "initiative";
      return Promise.resolve(ok("Combat commencé"));
    case "combat_action":
      if (JSON.stringify(args?.action).includes("Demarrer")) mockCombat = "encours";
      return Promise.resolve(ok("Action du combat"));
    case "combat_apercu_sort":
      return Promise.resolve(apercuSortExemple(String(args?.sort)));
    case "combat_apercu_fin":
      return Promise.resolve(apercuFinExemple());
    case "combat_terminer":
      mockCombat = "non";
      return Promise.resolve(ok("Combat validé"));
    case "prestige_don":
      return Promise.resolve(null);
    case "catalogue_regles":
      return Promise.resolve<RegleOptionnelle[]>([
        {
          id: "degats_massifs",
          nom: "Dégâts massifs",
          description: "Quand une seule attaque inflige au moins 50 PV et au moins la moitié des PV maximum sans tuer, un jet de Vigueur DD 15 est demandé ; un échec est la mort.",
          source: "Core Rulebook p. 189",
          par_defaut: false,
        },
      ]);
    case "catalogue_fp":
      return Promise.resolve(["1/8", "1/4", "1/2", "1", "2", "3", "4", "5"]);
    case "combat_xp":
      return Promise.resolve({ total: 1200, parPersonnage: Math.floor(1200 / Number(args?.personnages ?? 1)) });
    case "combat_historique":
      return Promise.resolve<HistoriqueVue>({
        totalRounds: 7,
        combats: [
          { numero: 1, jour: 22, heureDeFin: "10:30", rounds: 4, resume: "4 round(s), 3 sort(s) lancé(s)", bilan: [] },
          { numero: 2, jour: 24, heureDeFin: "14:05", rounds: 3, resume: "3 round(s), 2 sort(s) lancé(s)", bilan: [] },
        ],
        statistiques: [
          { id: 1, nom: "Mialee", combats: 2, degatsSubis: 18, soinsRecus: 6, degatsInfliges: 54, sortsLances: 5, objetsUtilises: 1 },
          { id: 2, nom: "Valéros", combats: 2, degatsSubis: 41, soinsRecus: 0, degatsInfliges: 96, sortsLances: 0, objetsUtilises: 0 },
        ],
      });
    case "combat_exporter":
      return Promise.resolve(["exports/campagne-combat-1.md", "exports/campagne-combat-1.json"]);
    case "catalogue_etats":
      return Promise.resolve([["shaken", "Secoué"], ["sickened", "Nauséeux"], ["fatigued", "Fatigué"], ["invisible", "Invisible"]]);
    case "catalogue_actions":
      return Promise.resolve([
        { id: "attack", nom_en: "Attack", nom: "Attaque", type: "simple", deplacement: false },
        { id: "move", nom_en: "Move", nom: "Se déplacer", type: "mouvement", deplacement: true },
        { id: "full_attack", nom_en: "Full attack", nom: "Attaque à outrance", type: "complexe", deplacement: false },
        { id: "five_foot_step", nom_en: "5-foot step", nom: "Pas de 1,5 m", type: "aucune", deplacement: false },
        { id: "cast_quickened", nom_en: "Cast a quickened spell", nom: "Lancer un sort accéléré", type: "rapide", deplacement: false },
        { id: "speak", nom_en: "Speak", nom: "Parler", type: "libre", deplacement: false },
      ] satisfies ActionDef[]);
    default:
      return Promise.resolve(ok(`(aperçu) ${cmd}`));
  }
}
