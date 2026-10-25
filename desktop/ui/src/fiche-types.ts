// Types des fiches de personnage : reprennent crates/vues/src/personnage.rs (camelCase)
// et les structures sérialisées de crates/fiches (snake_case, tel quel).

import type { Somme } from "./api";

export type Carac = "For" | "Dex" | "Con" | "Int" | "Sag" | "Cha";
export const CARACS: Carac[] = ["For", "Dex", "Con", "Int", "Sag", "Cha"];
export const NOMS_CARAC: Record<Carac, string> = {
  For: "Force",
  Dex: "Dextérité",
  Con: "Constitution",
  Int: "Intelligence",
  Sag: "Sagesse",
  Cha: "Charisme",
};

export type Terme = { libelle: string; valeur: number; compte: boolean };
export type ValeurVue = { total: number; explication: string; termes: Terme[] };

export type ResumeFiche = {
  race: string;
  classes: string;
  niveau: number;
  pv: number;
  ca: number;
  xp: number;
  niveauAtteignable: number;
  niveauxEnAttente: number;
};

export type LignePersonnage = {
  id: number;
  nom: string;
  compte: Somme;
  lieu: string | null;
  prestige: number | null;
  prestigeCalcule: boolean;
  acolyteDe: string | null;
  fiche: ResumeFiche | null;
};

export type TexteVue = { nom: string; texte: string };
export type CaracVue = { id: string; nom: string; score: ValeurVue; modificateur: number };
export type CompetenceVue = {
  cle: string;
  nom: string;
  valeur: ValeurVue;
  rangs: number;
  deClasse: boolean;
  formationRequise: boolean;
  caracteristique: string;
};
export type DonVue = { nomEn: string; nom: string; resume: string | null; avantage: string | null; source: string | null; noteLivre: string | null; inutilisable: string | null };
export type EmplacementVue = {
  niveauDeSort: number;
  base: number | null;
  bonus: number;
  parJour: number | null;
  restants: number | null;
  connus: number;
  dd: number;
};
export type SortVue = { nomEn: string; nom: string; niveau: number; ecole: string; deLignage: boolean };
export type PouvoirDomaineVue = { nom: string; genre: string; niveau: number; actif: boolean; utilisations: number | null; texte: string };
export type SortDomaineVue = { niveau: number; nom: string; nomEn: string | null; note: string | null; accessible: boolean };
export type DomaineVue = { id: string; nom: string; description: string; pouvoirs: PouvoirDomaineVue[]; sorts: SortDomaineVue[] };
export type CanalisationVue = { positive: boolean | null; des: number; dd: number; utilisations: number };
export type DivinVue = {
  divinite: string | null;
  alignement: string | null;
  armeDePredilection: string | null;
  domaines: DomaineVue[];
  canalisation: CanalisationVue | null;
  energieAChoisir: boolean;
};
export type SortPrepareVue = { niveau: number; nomEn: string; nom: string; domaine: boolean; utilise: boolean };
export type CapacitePreparation = { niveau: number; normaux: number; domaine: number; restants: number | null };
export type PreparationVue = {
  classeId: string;
  classe: string;
  capacites: CapacitePreparation[];
  prepares: SortPrepareVue[];
  sortsDeDomaine: SortDomaineVue[];
  interdits: string[];
  conversion: boolean | null;
};
/** Un emplacement de sort préparé, tel que le journal du cœur le reçoit. */
export type SortPrepare = { niveau: number; sort: string; domaine: boolean };
export type IncantateurVue = {
  classe: string;
  niveauDeLanceur: number;
  emplacements: EmplacementVue[];
  sorts: SortVue[];
};
export type ArmeVue = {
  nomEn: string;
  nom: string;
  maitrise: boolean;
  attaque: ValeurVue;
  degats: string;
  critique: string | null;
  portee: string | null;
  typeDegats: string | null;
};
export type ObjetVue = { nomEn: string; nom: string; quantite: number };
export type EquipementVue = {
  armure: ObjetVue | null;
  bouclier: ObjetVue | null;
  armes: ArmeVue[];
  objets: ObjetVue[];
  poidsTotal: number;
  poidsInconnu: string[];
  charge: string;
  limites: [number, number, number] | null;
  dexMax: string | null;
  malusDArmure: number;
  sourceMalus: string;
  echecSortsPct: number;
  nonMaitrises: string[];
};
export type EtatVue = { id: string; nom: string; texte: string; note: string | null };
export type TemporaireVue = { nom: string; roundsRestants: number | null; effets: string[] };
export type IdentiteVue = {
  nom: string;
  race: string;
  classes: string;
  lignage: string | null;
  niveau: number;
  xp: number;
  xpSuivant: number | null;
  niveauAtteignable: number;
  niveauxEnAttente: number;
  prestige: number | null;
  prestigeCalcule: boolean;
  compte: Somme;
};

export type FicheVue = {
  identite: IdentiteVue;
  caracteristiques: CaracVue[];
  pv: ValeurVue;
  ca: ValeurVue;
  caContact: ValeurVue;
  caDepourvu: ValeurVue;
  initiative: ValeurVue;
  vigueur: ValeurVue;
  reflexes: ValeurVue;
  volonte: ValeurVue;
  bba: ValeurVue;
  attaquesIteratives: number[];
  bmo: ValeurVue;
  dmd: ValeurVue;
  vitesse: number;
  competences: CompetenceVue[];
  rangsParNiveau: [number, number, number][];
  dons: DonVue[];
  traits: TexteVue[];
  capacites: TexteVue[];
  divin: DivinVue | null;
  preparation: PreparationVue | null;
  incantateurs: IncantateurVue[];
  equipement: EquipementVue;
  etats: EtatVue[];
  temporaires: TemporaireVue[];
  notesEtats: string[];
  problemes: string[];
  historique: string[];
};

// ---------------------------------------------------------------- catalogue de création

export type RaceVue = {
  id: string;
  nom: string;
  taille: string;
  vitesse: number;
  ajustements: [string, number][];
  choixLibre: number | null;
  bonusDon: number;
  rangSupplementaire: number;
  donImpose: string | null;
  deuxClassesDePredilection: boolean;
  traits: TexteVue[];
};
export type LignageVue = {
  id: string;
  nom: string;
  competenceDeClasse: string;
  arcane: string;
  choixConnaissance: boolean;
  sorts: [string, number][];
  dons: string[];
};
export type ClasseVue = {
  id: string;
  nom: string;
  deVie: number;
  rangsParNiveau: number;
  incantation: boolean;
  caracteristiqueIncantation: string | null;
  lignages: LignageVue[];
  competencesDeClasse: string[];
  donBonusNiveau1: string | null;
  sortsConnus: (number | null)[][];
  prepare: boolean;
  domainesChoisis: number;
  canalisation: boolean;
  capacites: TexteVue[];
  maitrises: string;
};
export type DomaineCatalogue = {
  id: string;
  nom: string;
  description: string;
  divinites: string[];
  alignement: string | null;
  pouvoirs: PouvoirDomaineVue[];
  sorts: SortDomaineVue[];
};
export type DiviniteCatalogue = { id: string; nom: string; alignement: string; theme: string; domaines: string[]; armeDePredilection: string };
export type CompetenceCatalogue = {
  id: string;
  nom: string;
  caracteristique: string;
  formationRequise: boolean;
  malusDArmure: boolean;
  sousTypes: boolean;
};
export type DonCatalogue = {
  nomEn: string;
  nom: string;
  resume: string | null;
  prerequis: string | null;
  repetable: boolean;
  aUnChoix: boolean;
};
export type SortCatalogue = { nomEn: string; nom: string; ecole: string; niveau: number; resume: string; source: string | null; noteLivre: string | null; descripteurs: string[] };
export type EquipementCatalogue = { nomEn: string; nom: string; genre: string; poidsLb: number | null; source: string | null; noteLivre: string | null; description: string | null };
export type EtatCatalogue = { id: string; nom: string };
export type CatalogueCreation = {
  races: RaceVue[];
  classes: ClasseVue[];
  competences: CompetenceCatalogue[];
  dons: DonCatalogue[];
  /** Sorts de la liste de chaque classe (identifiant de classe) : on ne choisit que dans sa propre liste. */
  sortsParClasse: Record<string, SortCatalogue[]>;
  domaines: DomaineCatalogue[];
  divinites: DiviniteCatalogue[];
  armures: EquipementCatalogue[];
  armes: EquipementCatalogue[];
  materiel: EquipementCatalogue[];
  etats: EtatCatalogue[];
  coutAchat: [number, number][];
};

// ---------------------------------------------------------------- données de la fiche (cœur)

export type Charge = "Legere" | "Intermediaire" | "Lourde" | "Surcharge";

export type EffetDonnees = { cible: string; type_bonus: string; valeur: number; source: string };

export type NiveauPris = {
  classe: string;
  de_vie_obtenu: number;
  rangs: Record<string, number>;
  predilection: "PointDeVie" | "Rang" | null;
  dons: string[];
};

export type FicheDonnees = {
  nom: string;
  race: string;
  bonus_racial_libre: Carac | null;
  scores_base: Record<Carac, number>;
  dommages: Partial<Record<Carac, number>>;
  drain: Partial<Record<Carac, number>>;
  augmentations: [number, Carac][];
  niveaux: NiveauPris[];
  classes_de_predilection: string[];
  lignage: string | null;
  connaissance_de_lignage: string | null;
  effets: EffetDonnees[];
  sorts_connus: string[];
  equipement: {
    armure: string | null;
    bouclier: string | null;
    armes: string[];
    objets: { nom_en: string; quantite: number }[];
    charge_declaree: Charge;
  };
  etats: string[];
  temporaires: { nom: string; effets: EffetDonnees[]; rounds_restants: number | null }[];
  divinite: string | null;
  alignement: string | null;
  domaines: string[];
  energie: "positive" | "negative" | null;
};

export type Montee = {
  classe: string;
  de_vie: number;
  rangs: Record<string, number>;
  predilection: "PointDeVie" | "Rang" | null;
  dons: string[];
  augmentation: Carac | null;
  sorts_appris: string[];
  echange: [string, string] | null;
};

export type ApercuMontee = {
  niveauDePersonnage: number;
  classe: string;
  niveauDeClasse: number;
  deVie: number;
  deViePropose: number | null;
  bba: [number, number];
  vigueur: [number, number];
  reflexes: [number, number];
  volonte: [number, number];
  donsAChoisir: number;
  /** Parmi ces dons, nombre à prendre dans la liste du lignage. */
  donsDeLignage: number;
  donsDeLignagePossibles: { nomEn: string; nom: string }[];
  augmentation: boolean;
  predilectionPossible: boolean;
  capacites: string[];
  sortsConnusGagnes: [number, number][];
  echangeDeSort: boolean;
  rangsAPlacer: number;
  niveauAtteint: boolean;
  niveauAtteignable: number;
};
export type EssaiMontee = { erreurs: string[]; apres: FicheVue | null };

/** Événements du journal d'une fiche (représentation JSON du cœur). */
export type EvenementFiche =
  | "Repos"
  | { XpGagnes: { nombre: number; motif: string } }
  | { JalonAccorde: { niveau: number } }
  | { MonteeDeNiveau: Montee }
  | { SortLance: { classe: string; niveau_de_sort: number; sort: string } }
  | { Preparation: { classe: string; sorts: SortPrepare[] } }
  | { SortConverti: { classe: string; niveau_sacrifie: number; sort: string } }
  | { Equiper: { emplacement: "armure" | "bouclier"; objet: string | null } }
  | { ArmeAjoutee: string }
  | { ArmeRetiree: string }
  | { ObjetAjoute: { nom_en: string; quantite: number } }
  | { ObjetRetire: { nom_en: string; quantite: number } }
  | { EtatAjoute: string }
  | { EtatRetire: string }
  | { EffetTemporaireAjoute: { nom: string; effets: EffetDonnees[]; rounds_restants: number | null } }
  | { EffetTemporaireRetire: string }
  | { TempsEcoule: { rounds: number } }
  | { ChargeDeclaree: Charge };

export type RegleInfo = {
  id: string;
  nom: string;
  description: string;
  officielle: boolean;
  impact: string;
  defaut: string;
  valeur: string;
  choix: [string, string][];
};

// ---- Importation d'une fiche PDF (desktop/src-tauri/src/import_fiche.rs) ----

export type GraviteImport = "accord" | "a_valider" | "ecart";

export type LigneImport = {
  cle: string;
  rubrique: string;
  libelle: string;
  champ: string;
  feuille: string;
  regles: string;
  explication: string;
  gravite: GraviteImport;
  /** « Garder » est possible pour cette ligne (valeur numérique que la fiche sait recevoir). */
  peut_garder: boolean;
  delta: number | null;
};

export type ApercuImport = {
  chemin: string;
  nom: string;
  niveau: number;
  pv: number;
  dons: string[];
  lignes: LigneImport[];
  non_verifie: string[];
};

// ---- don Prestige
export type ModificateurPrestige = { id: string; libelle: string; valeur: number; portee: "tous" | "acolyte" | "partisans"; cumulable: boolean; compte: number };
export type PrestigeDonVue = {
  actif: boolean;
  niveauMinimal: number;
  acolyte: { id: number; nom: string; niveau: number | null; xp: number | null; aUneFiche: boolean } | null;
  scoreAcolyte: number | null;
  scorePartisans: number | null;
  niveauTableau: number;
  niveauMaxAcolyte: number;
  modificateurs: ModificateurPrestige[];
  candidats: [number, string][];
  partisans: { index: number; niveau: number; nom: string }[];
  places: { niveau: number; permis: number; pris: number }[];
};
