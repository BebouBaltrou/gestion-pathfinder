// Fabriques des événements d'aventure et des actions de combat, au format JSON du moteur
// (serde : variantes à étiquette externe). Aucune règle ici : on construit seulement le message.

import type { ActionCombat, Duree, Effet, EvenementAventure, Protection, TypeAction } from "./aventure-types";

export const ROUNDS_PAR_MINUTE = 10;
export const ROUNDS_PAR_HEURE = 600;
export const ROUNDS_PAR_JOUR = 14400;

/** Instant de campagne (rounds depuis le jour 1 à 00:00). */
export const instantDe = (jour: number, heure: number, minute: number) =>
  (jour - 1) * ROUNDS_PAR_JOUR + heure * ROUNDS_PAR_HEURE + minute * ROUNDS_PAR_MINUTE;

export type PasHoraire = "minutes5" | "minutes10" | "minutes30" | "heure1";

export type Consommable = {
  nom: string;
  genre: "potion" | "parchemin" | "baguette" | "baton" | "charges" | "munition" | "autre";
  sort: string | null;
  niveau_du_sort: number;
  niveau_de_lanceur: number;
  arcanique: boolean;
  quantite: number;
  charges: { restantes: number; max: number } | null;
  par_jour: { restantes: number; max: number } | null;
  notes: string;
};

export const ev = {
  entree: (pj: number): EvenementAventure => ({ Entree: { pj } }),
  pas: (pas: PasHoraire): EvenementAventure => ({ Pas: { pas } }),
  avancer: (rounds: number): EvenementAventure => ({ AvancerTemps: { rounds } }),
  regleHeure: (instant: number, reactiver: number[] = [], retirerCrees = false): EvenementAventure => ({
    RegleHeure: { instant, reactiver, retirer_crees: retirerCrees },
  }),
  degats: (pj: number, nombre: number, letaux = true): EvenementAventure => ({ Degats: { pj, nombre, letaux } }),
  degatsTypes: (pj: number, nombre: number, typeDegats: string, contourneRd: boolean, letaux = true): EvenementAventure => ({
    DegatsTypes: { pj, nombre, type_degats: typeDegats, contourne_rd: contourneRd, letaux },
  }),
  soin: (pj: number, nombre: number): EvenementAventure => ({ Soin: { pj, nombre } }),
  pvTemp: (pj: number, nombre: number): EvenementAventure => ({ PvTemporaires: { pj, nombre } }),
  stabilisation: (pj: number, d20: number): EvenementAventure => ({ Stabilisation: { pj, d20 } }),
  degatsMassifs: (pj: number, d20: number): EvenementAventure => ({ DegatsMassifs: { pj, d20 } }),
  degatsContinus: (pj: number, actif: boolean): EvenementAventure => ({ DegatsContinus: { pj, actif } }),
  effet: (p: {
    pj: number;
    nom: string;
    source: string;
    duree: Duree;
    effets?: Effet[];
    etats?: string[];
    soinParRound?: number;
    protections?: Protection[];
  }): EvenementAventure => ({
    EffetAjoute: {
      pj: p.pj,
      nom: p.nom,
      source: p.source,
      duree: p.duree,
      effets: p.effets ?? [],
      etats: p.etats ?? [],
      soin_par_round: p.soinParRound ?? 0,
      protections: p.protections ?? [],
    },
  }),
  effetRetire: (id: number): EvenementAventure => ({ EffetRetire: { id } }),
  effetRetabli: (id: number, duree: Duree): EvenementAventure => ({ EffetRetabli: { id, duree } }),
  alerteConfirmee: (id: number): EvenementAventure => ({ AlerteConfirmee: { id } }),
  repos: (pjs: number[], heures: number, alitementComplet: boolean, interrompu: boolean): EvenementAventure => ({
    Repos: { pjs, heures, alitement_complet: alitementComplet, interrompu },
  }),
  objetAjoute: (pj: number, objet: Consommable): EvenementAventure => ({ ObjetAjoute: { pj, objet } }),
  objetRetire: (pj: number, id: number): EvenementAventure => ({ ObjetRetire: { pj, id } }),
  objetUtilise: (pj: number, id: number, charges = 1): EvenementAventure => ({ ObjetUtilise: { pj, id, charges } }),
  objetRecupere: (pj: number, id: number, nombre: number): EvenementAventure => ({ ObjetRecupere: { pj, id, nombre } }),
  ressourceUtilisee: (pj: number, nom: string, quantite = 1): EvenementAventure => ({ RessourceUtilisee: { pj, nom, quantite } }),
  ressourceRendue: (pj: number, nom: string, quantite = 1): EvenementAventure => ({ RessourceRendue: { pj, nom, quantite } }),
};

export const act = {
  initiative: (pj: number, d20: number): ActionCombat => ({ Initiative: { pj, d20 } }),
  departage: (pj: number, d20: number): ActionCombat => ({ Departage: { pj, d20 } }),
  demarrer: (surprise: number[] | null): ActionCombat => ({ Demarrer: { surprise } }),
  tourSuivant: (): ActionCombat => "TourSuivant",
  retarder: (pj: number, nouvelleInitiative: number): ActionCombat => ({
    Retarder: { pj, nouvelle_initiative: nouvelleInitiative },
  }),
  modifierInitiative: (pj: number, total: number): ActionCombat => ({ ModifierInitiative: { pj, total } }),
  preparer: (pj: number, action: string): ActionCombat => ({ Preparer: { pj, action } }),
  note: (texte: string): ActionCombat => ({ Note: texte }),
  degatsInfliges: (pj: number, nombre: number): ActionCombat => ({ DegatsInfliges: { pj, nombre } }),
  aventure: (e: EvenementAventure): ActionCombat => ({ Aventure: e }),
  agir: (pj: number, typeAction: TypeAction, libelle: string, deplacement = false): ActionCombat => ({
    Agir: { pj, type_action: typeAction, libelle, deplacement },
  }),
  attaquer: (p: {
    pj: number;
    typeAction: TypeAction;
    libelle: string;
    tirs: number;
    munition: number | null;
    aDistanceAvecMunitions: boolean;
    deplacement?: boolean;
  }): ActionCombat => ({
    Attaquer: {
      pj: p.pj,
      type_action: p.typeAction,
      libelle: p.libelle,
      tirs: p.tirs,
      munition: p.munition,
      a_distance_avec_munitions: p.aDistanceAvecMunitions,
      deplacement: p.deplacement ?? false,
    },
  }),
  lancer: (p: {
    pj: number;
    sort: string;
    metamagies?: string[];
    niveauHeighten?: number | null;
    defensifD20?: number | null;
    echecArcaniqueD100?: number | null;
    cibles?: number[];
    duree?: Duree | null;
    choixEnergie?: string | null;
    resultatDe?: number | null;
  }): ActionCombat => ({
    Lancer: {
      pj: p.pj,
      sort: p.sort,
      metamagies: p.metamagies ?? [],
      niveau_heighten: p.niveauHeighten ?? null,
      defensif_d20: p.defensifD20 ?? null,
      echec_arcanique_d100: p.echecArcaniqueD100 ?? null,
      cibles: p.cibles ?? [],
      duree: p.duree ?? null,
      choix_energie: p.choixEnergie ?? null,
      resultat_de: p.resultatDe ?? null,
    },
  }),
  concentration: (pj: number, d20: number, cause: Record<string, unknown> | string, niveauDuSort: number, effet: number | null): ActionCombat => ({
    Concentration: { pj, d20, cause, niveau_du_sort: niveauDuSort, effet },
  }),
  utiliserObjet: (p: {
    pj: number;
    objet: number;
    cibles?: number[];
    duree?: Duree | null;
    choixEnergie?: string | null;
    charges?: number;
    niveauDeLanceurD20?: number | null;
    accidentD20?: number | null;
    echecArcaniqueD100?: number | null;
    typeAction?: TypeAction | null;
    resultatDe?: number | null;
  }): ActionCombat => ({
    UtiliserObjet: {
      pj: p.pj,
      objet: p.objet,
      cibles: p.cibles ?? [],
      duree: p.duree ?? null,
      choix_energie: p.choixEnergie ?? null,
      charges: p.charges ?? 0,
      niveau_de_lanceur_d20: p.niveauDeLanceurD20 ?? null,
      accident_d20: p.accidentD20 ?? null,
      echec_arcanique_d100: p.echecArcaniqueD100 ?? null,
      type_action: p.typeAction ?? null,
      resultat_de: p.resultatDe ?? null,
    },
  }),
};

export const TYPES_DE_BONUS: [string, string][] = [
  ["alteration", "altération"],
  ["armure", "armure"],
  ["bouclier", "bouclier"],
  ["parade", "parade"],
  ["esquive", "esquive"],
  ["moral", "moral"],
  ["chance", "chance"],
  ["circonstance", "circonstance"],
  ["racial", "racial"],
  ["objet", "d'objet"],
  ["taille", "de taille"],
  ["naturel", "naturel"],
  ["perspicacite", "perspicacité"],
  ["competence", "de compétence"],
  ["sacre", "sacré"],
  ["profane", "profane"],
  ["sans_type", "sans type"],
];

export const CIBLES_D_EFFET: [string, string][] = [
  ["ca", "CA"],
  ["attaque", "Attaques"],
  ["degats", "Dégâts"],
  ["init", "Initiative"],
  ["bmo", "BMO"],
  ["dmd", "DMD"],
  ["jds.vig", "Vigueur"],
  ["jds.ref", "Réflexes"],
  ["jds.vol", "Volonté"],
  ["comp.*", "Toutes les compétences"],
  ["carac.for", "Force"],
  ["carac.dex", "Dextérité"],
  ["carac.con", "Constitution"],
  ["carac.int", "Intelligence"],
  ["carac.sag", "Sagesse"],
  ["carac.cha", "Charisme"],
];

export const TYPES_D_ENERGIE: string[] = ["acide", "froid", "électricité", "feu", "son"];
