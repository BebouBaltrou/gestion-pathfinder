// Données d'exemple pour l'aperçu sans Tauri (MOCK=1) : captures d'écran et essais de mise en page.
// Retiré de la version de l'application (le contenu n'est utilisé que si __MOCK__ est vrai).

import type { ApercuFin, ApercuSort, AventureVue, CombatVue, PjAventure, PjCombat } from "./aventure-types";

const sortsConnus = [
  { nomEn: "Light", nom: "Lumière", niveau: 0 },
  { nomEn: "Acid Splash", nom: "Aspersion d'acide", niveau: 0 },
  { nomEn: "Magic Missile", nom: "Projectile magique", niveau: 1 },
  { nomEn: "Mage Armor", nom: "Armure de mage", niveau: 1 },
  { nomEn: "Shield", nom: "Bouclier", niveau: 1 },
  { nomEn: "Resist Energy", nom: "Résistance à l'énergie", niveau: 2 },
  { nomEn: "Fireball", nom: "Boule de feu", niveau: 3 },
  { nomEn: "Haste", nom: "Hâte", niveau: 3 },
];

export const pjMialee = (pv = 23): PjAventure => ({
  id: 1,
  nom: "Mialee",
  niveau: 5,
  pv,
  pvMax: 31,
  temporaires: 4,
  nonLetaux: 3,
  sante: pv > 0 ? "conscient" : "hors de combat",
  santeCode: pv > 0 ? "conscient" : "hors_de_combat",
  etats: [{ id: "shaken", nom: "Secoué" }],
  effets: [
    {
      id: 1,
      nom: "Armure de mage",
      source: "sort Armure de mage, lancé par Mialee",
      duree: "5 heure(s)",
      restant: "4 h 12 min",
      fraction: 0.84,
      details: ["+4 armure (CA)"],
      etats: [],
    },
    {
      id: 2,
      nom: "Hâte",
      source: "sort Hâte, lancé par Seoni",
      duree: "5 round(s)",
      restant: "1 min",
      fraction: 0.2,
      details: ["+1 sans type (attaques)", "+1 esquive (CA)", "+1 esquive (Réflexes)"],
      etats: [],
    },
    {
      id: 3,
      nom: "Terreur",
      source: "capacité d'un monstre",
      duree: "10 minute(s)",
      restant: "7 min 30 s",
      fraction: 0.75,
      details: [],
      etats: ["Secoué"],
    },
  ],
  emplacements: [
    { niveau: 1, restants: 4, total: 6, dd: 14 },
    { niveau: 2, restants: 5, total: 5, dd: 15 },
    { niveau: 3, restants: 2, total: 4, dd: 16 },
  ],
  sortsConnus,
  ressources: [{ cle: "Acidic Ray", nom: "Rayon acide", unite: "utilisations", restantes: 4, max: 6, source: "lignage aberrant, Acidic Ray (niveau 1)" }],
  inventaire: [
    { id: 1, nom: "Potion de soins légers", genre: "potion", genreCode: "potion", sort: "Soins légers", quantite: 2, charges: null, parJour: null, niveauDeLanceur: 1 },
    { id: 2, nom: "Baguette de projectile magique", genre: "baguette", genreCode: "baguette", sort: "Projectile magique", quantite: 1, charges: [34, 50], parJour: null, niveauDeLanceur: 3 },
    { id: 3, nom: "Flèches", genre: "munition", genreCode: "munition", sort: null, quantite: 17, charges: null, parJour: null, niveauDeLanceur: 0 },
  ],
  degatsContinus: false,
  initiative: 2,
  caster: true,
  niveauDeLanceur: 5,
  echecArcaniquePct: 20,
});

const pjValeros = (): PjAventure => ({
  id: 2,
  nom: "Valéros",
  niveau: 5,
  pv: 12,
  pvMax: 44,
  temporaires: 0,
  nonLetaux: 0,
  sante: "conscient",
  santeCode: "conscient",
  etats: [],
  effets: [],
  emplacements: [],
  sortsConnus: [],
  ressources: [],
  inventaire: [],
  degatsContinus: false,
  initiative: 1,
  caster: false,
  niveauDeLanceur: 0,
  echecArcaniquePct: 0,
});

const pjKyra = (): PjAventure => ({
  ...pjValeros(),
  id: 3,
  nom: "Kyra",
  pv: -4,
  pvMax: 33,
  sante: "mourant",
  santeCode: "mourant",
  etats: [
    { id: "dying", nom: "Mourant" },
    { id: "unconscious", nom: "Inconscient" },
  ],
  initiative: 2,
});

export const aventureExemple = (): AventureVue => ({
  actif: true,
  horloge: { jour: 24, heure: 14, minute: 5, texte: "jour 24, 14:05", enCombat: false },
  survieActive: false,
  degatsMassifs: false,
  personnages: [pjMialee(), pjValeros()],
  disponibles: [[3, "Kyra"]],
  alertes: [
    { id: 9, nom: "Bénédiction", personnage: "Valéros", source: "sort Bénédiction, lancé par Kyra", raison: "a pris fin", quand: "jour 24, 14:00:00", dissipe: false },
  ],
  demandes: [],
  journal: [
    { quand: "jour 24, 14:05:00", texte: "Mialee reçoit 5 dégât(s) létaux : 23/31 PV, 3 non létaux." },
    { quand: "jour 24, 14:00:00", texte: "« Bénédiction » (Valéros) a pris fin." },
    { quand: "jour 24, 13:30:00", texte: "Mialee se repose 0 h." },
  ],
});

export const aventureVide = (): AventureVue => ({
  actif: false,
  horloge: null,
  survieActive: false,
  degatsMassifs: false,
  personnages: [],
  disponibles: [
    [1, "Mialee"],
    [2, "Valéros"],
  ],
  alertes: [],
  demandes: [],
  journal: [],
});

const economie = (simple: boolean, mouvement: boolean) => ({ simple, mouvement, complexe: simple && mouvement, rapide: true, pas: mouvement, limite: false });

export const combatExemple = (): CombatVue => {
  const mialee: PjCombat = { ...pjMialee(), economie: economie(true, true), degatsSubis: 8, soinsRecus: 0, sortsLances: 1 };
  const valeros: PjCombat = { ...pjValeros(), economie: economie(false, false), degatsSubis: 14, soinsRecus: 0, sortsLances: 0 };
  const kyra: PjCombat = { ...pjKyra(), economie: economie(false, false), degatsSubis: 37, soinsRecus: 0, sortsLances: 0 };
  return {
    phase: "enCours",
    round: 3,
    surprise: false,
    actif: 1,
    initiatives: [
      { id: 1, nom: "Mialee", total: 18, d20: 16, modificateur: 2, departage: null, actif: true, joue: true },
      { id: 3, nom: "Kyra", total: 14, d20: 12, modificateur: 2, departage: null, actif: false, joue: true },
      { id: 2, nom: "Valéros", total: 9, d20: 8, modificateur: 1, departage: null, actif: false, joue: true },
    ],
    sansInitiative: [],
    egalites: [],
    personnages: [mialee, valeros, kyra],
    demandes: [{ pj: 3, nom: "Kyra", genre: "stabilisation", texte: "Jet de stabilisation : d20 + Constitution −4", dd: 10 }],
    alertes: [],
    journal: [
      [3, "Tour de Mialee."],
      [3, "Round 3."],
      [2, "Valéros : Attaque à outrance (action complexe)."],
      [2, "Kyra subit 9 dégât(s) létaux : -4/33 PV, 0 non létaux."],
      [1, "Mialee lance Armure de mage : emplacement de niveau 1, DD 14, niveau de lanceur 5."],
    ],
    peutAnnuler: true,
    peutRetablir: false,
    horloge: { jour: 24, heure: 14, minute: 5, texte: "jour 24, 14:05", enCombat: true },
    roundsJoues: 2,
  };
};

export const combatInitiative = (): CombatVue => ({
  ...combatExemple(),
  phase: "initiative",
  round: 1,
  actif: null,
  initiatives: [{ id: 1, nom: "Mialee", total: 18, d20: 16, modificateur: 2, departage: null, actif: false, joue: true }],
  sansInitiative: [
    [2, "Valéros"],
    [3, "Kyra"],
  ],
});

export const apercuSortExemple = (sort: string): ApercuSort => ({
  nomEn: sort,
  nom: sort === "Fireball" ? "Boule de feu" : sort === "Mage Armor" ? "Armure de mage" : sort,
  niveauDuSort: sort === "Fireball" ? 3 : 1,
  niveauEffectif: sort === "Fireball" ? 3 : 1,
  niveauEmplacement: sort === "Fireball" ? 3 : 1,
  restants: 2,
  dd: sort === "Fireball" ? 16 : 14,
  niveauDeLanceur: 5,
  typeAction: "action simple",
  verbal: true,
  gestuel: true,
  echecArcaniquePct: 20,
  ddLancementDefensif: sort === "Fireball" ? 21 : 17,
  bonusConcentration: 8,
  dureeTexte: sort === "Fireball" ? "instantanée" : "1 heure/niveau (D)",
  dureeSuggeree: sort === "Fireball" ? "instantanee" : { heures: 5 },
  dureeSuggereeTexte: sort === "Fireball" ? "instantanée" : "5 heure(s)",
  portee: sort === "Fireball" ? "longue (120 m + 12 m/niveau)" : "contact",
  jetDeSauvegarde: sort === "Fireball" ? "Réflexes, moitié" : "aucun (inoffensif)",
  description: "Description du sort tirée du livre (traduction non officielle).",
  effetAutomatique: sort === "Mage Armor",
  choixEnergie: false,
  notes: null,
});

export const apercuFinExemple = (): ApercuFin => ({
  roundsJoues: 3,
  personnages: [
    { id: 1, nom: "Mialee", pvAvant: 31, pvApres: 23, degatsSubis: 8, soinsRecus: 0, sortsLances: 2 },
    { id: 2, nom: "Valéros", pvAvant: 44, pvApres: 12, degatsSubis: 32, soinsRecus: 0, sortsLances: 0 },
    { id: 3, nom: "Kyra", pvAvant: 33, pvApres: -4, degatsSubis: 37, soinsRecus: 0, sortsLances: 0 },
  ],
  dissipes: [{ id: 4, nom: "Bénédiction", personnage: "Valéros", source: "sort Bénédiction, lancé par Kyra", duree: "20 round(s)", perdu: "17 round(s)" }],
  arrondis: [{ nom: "Hâte", personnage: "Mialee", avant: "7 min 24 s", apres: "7 min" }],
  persistants: [
    ["Armure de mage", "Mialee", "jour 24, 19:05:00"],
    ["Terreur", "Mialee", null],
  ],
});
