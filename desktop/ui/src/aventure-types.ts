// Types du mode aventure et du mode combat : ils reprennent ceux de crates/vues/src/aventure.rs
// (affichage) et les actions du moteur (crates/aventure, crates/combat) telles que Tauri les échange.

export type Horloge = { jour: number; heure: number; minute: number; texte: string; enCombat: boolean };

export type EffetVue = {
  id: number;
  nom: string;
  source: string;
  duree: string;
  restant: string | null;
  fraction: number | null;
  details: string[];
  etats: string[];
};

export type AlerteVue = {
  id: number;
  nom: string;
  personnage: string;
  source: string;
  raison: string;
  quand: string;
  dissipe: boolean;
};

export type EmplacementVue = { niveau: number; restants: number; total: number; dd: number };
export type RessourceVue = { cle: string; nom: string; unite: string; restantes: number; max: number; source: string };
export type ObjetVue = {
  id: number;
  nom: string;
  genre: string;
  genreCode: "potion" | "parchemin" | "baguette" | "baton" | "charges" | "munition" | "autre";
  sort: string | null;
  quantite: number;
  charges: [number, number] | null;
  parJour: [number, number] | null;
  niveauDeLanceur: number;
};
export type SortConnuVue = { nomEn: string; nom: string; niveau: number };
export type EtatVue = { id: string; nom: string };

export type SanteCode = "conscient" | "chancelant" | "hors_de_combat" | "inconscient" | "mourant" | "stable" | "mort";

export type PjAventure = {
  id: number;
  nom: string;
  niveau: number;
  pv: number;
  pvMax: number;
  temporaires: number;
  nonLetaux: number;
  sante: string;
  santeCode: SanteCode;
  etats: EtatVue[];
  effets: EffetVue[];
  emplacements: EmplacementVue[];
  sortsConnus: SortConnuVue[];
  ressources: RessourceVue[];
  inventaire: ObjetVue[];
  degatsContinus: boolean;
  initiative: number;
  caster: boolean;
  niveauDeLanceur: number;
  echecArcaniquePct: number;
};

export type DemandeVue = {
  pj: number;
  nom: string;
  genre: "stabilisation" | "massifs" | "soif" | "faim";
  texte: string;
  dd: number | null;
};

export type LigneJournal = { quand: string; texte: string };

export type AventureVue = {
  actif: boolean;
  horloge: Horloge | null;
  survieActive: boolean;
  degatsMassifs: boolean;
  personnages: PjAventure[];
  disponibles: [number, string][];
  alertes: AlerteVue[];
  demandes: DemandeVue[];
  journal: LigneJournal[];
};

export type InitiativeVue = {
  id: number;
  nom: string;
  total: number;
  d20: number;
  modificateur: number;
  departage: number | null;
  actif: boolean;
  joue: boolean;
};

export type EconomieVue = {
  simple: boolean;
  mouvement: boolean;
  complexe: boolean;
  rapide: boolean;
  pas: boolean;
  limite: boolean;
};

export type PjCombat = PjAventure & {
  economie: EconomieVue;
  degatsSubis: number;
  soinsRecus: number;
  sortsLances: number;
};

export type CombatVue = {
  phase: "initiative" | "enCours";
  round: number;
  surprise: boolean;
  actif: number | null;
  initiatives: InitiativeVue[];
  sansInitiative: [number, string][];
  egalites: string[][];
  personnages: PjCombat[];
  demandes: DemandeVue[];
  alertes: AlerteVue[];
  journal: [number, string][];
  peutAnnuler: boolean;
  peutRetablir: boolean;
  horloge: Horloge;
  roundsJoues: number;
};

export type Duree =
  | "instantanee"
  | "permanente"
  | { rounds: number }
  | { minutes: number }
  | { heures: number }
  | { jours: number }
  | "concentration"
  | "jusqua_decharge"
  | { jusqua_prochain_tour: "debut" | "fin" }
  | { lie_a_un_evenement: string };

export type ApercuSort = {
  nomEn: string;
  nom: string;
  niveauDuSort: number;
  niveauEffectif: number;
  niveauEmplacement: number;
  restants: number | null;
  dd: number;
  niveauDeLanceur: number;
  typeAction: string;
  verbal: boolean;
  gestuel: boolean;
  echecArcaniquePct: number;
  ddLancementDefensif: number;
  bonusConcentration: number;
  dureeTexte: string | null;
  dureeSuggeree: Duree | null;
  dureeSuggereeTexte: string | null;
  portee: string | null;
  jetDeSauvegarde: string | null;
  description: string;
  effetAutomatique: boolean;
  choixEnergie: boolean;
  notes: string | null;
};

export type BilanPj = {
  id: number;
  nom: string;
  pvAvant: number;
  pvApres: number;
  degatsSubis: number;
  soinsRecus: number;
  sortsLances: number;
};

export type DissipationVue = { id: number; nom: string; personnage: string; source: string; duree: string; perdu: string | null };
export type ArrondiVue = { nom: string; personnage: string; avant: string; apres: string };

export type ApercuFin = {
  roundsJoues: number;
  personnages: BilanPj[];
  dissipes: DissipationVue[];
  arrondis: ArrondiVue[];
  persistants: [string, string, string | null][];
};

export type TypeAction = "simple" | "mouvement" | "complexe" | "rapide" | "immediate" | "libre" | "aucune";

export type ActionDef = {
  id: string;
  nom_en: string;
  nom: string;
  type: TypeAction | "variable";
  deplacement: boolean;
};

export type Effet = { cible: string; type_bonus: string; valeur: number; source: string };
export type Protection =
  | { Immunite: { type_degats: string } }
  | { Resistance: { type_degats: string; valeur: number } }
  | { Reduction: { valeur: number } };

/** Corps d'un `EvenementAventure` (JSON serde du moteur). */
export type EvenementAventure = Record<string, unknown> | string;
/** Corps d'une `ActionCombat` (JSON serde du moteur). */
export type ActionCombat = Record<string, unknown> | string;

export type Validation = { genre: "tout" } | { genre: "partiel"; pjs: number[] } | { genre: "abandon" };

// ------------------------------------------------------------ petits utilitaires d'affichage

export const dureeEnTexte = (d: Duree): string => {
  if (typeof d === "string") {
    return (
      { instantanee: "instantanée", permanente: "permanente", concentration: "concentration", jusqua_decharge: "jusqu'à décharge" } as Record<
        string,
        string
      >
    )[d];
  }
  if ("rounds" in d) return `${d.rounds} round(s)`;
  if ("minutes" in d) return `${d.minutes} minute(s)`;
  if ("heures" in d) return `${d.heures} heure(s)`;
  if ("jours" in d) return `${d.jours} jour(s)`;
  if ("jusqua_prochain_tour" in d) return d.jusqua_prochain_tour === "debut" ? "jusqu'au début de son prochain tour" : "jusqu'à la fin de son prochain tour";
  return d.lie_a_un_evenement;
};

export type ReculVue = {
  reactivables: [number, string, string][];
  crees: [number, string, string][];
  avantMinuit: boolean;
};
