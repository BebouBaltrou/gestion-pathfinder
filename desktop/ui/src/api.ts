// Pont vers le cœur Rust (commandes Tauri). Les types reprennent ceux de crates/vues.

import type {
  ApercuMontee,
  CatalogueCreation,
  EssaiMontee,
  EvenementFiche,
  FicheDonnees,
  FicheVue,
  LignePersonnage,
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
  creerCampagne: (nom: string) => appeler<Retour>("creer_campagne", { nom }),
  tableauDeBord: () => appeler<TableauDeBord | null>("tableau_de_bord"),
  avancer: (jours: number) => appeler<Retour>("avancer", { jours }),
  annuler: () => appeler<Retour>("annuler"),
  retablir: () => appeler<Retour>("retablir"),
  resoudrePrestige: (localite: number, reussi: boolean) =>
    appeler<Retour>("resoudre_prestige", { localite, reussi }),
  simulation: (action: "demarrer" | "abandonner" | "garder") =>
    appeler<Retour>("simulation", { action }),
  // ---- personnages et fiches
  listePersonnages: () => appeler<LignePersonnage[]>("liste_personnages"),
  fichePersonnage: (id: number) => appeler<FicheVue | null>("fiche_personnage", { id }),
  catalogueCreation: () => appeler<CatalogueCreation>("catalogue_creation"),
  apercuFiche: (fiche: FicheDonnees) => appeler<FicheVue>("apercu_fiche", { fiche }),
  creerPersonnage: (nom: string, po: number) => appeler<Retour>("creer_personnage", { nom, po }),
  creerFiche: (pj: number, fiche: FicheDonnees) => appeler<Retour>("creer_fiche", { pj, fiche }),
  evenementFiche: (pj: number, evenement: EvenementFiche) =>
    appeler<Retour>("evenement_fiche", { pj, evenement }),
  apercuMontee: (pj: number, classe: string) => appeler<ApercuMontee | null>("apercu_montee", { pj, classe }),
  essaiMontee: (pj: number, montee: Montee) => appeler<EssaiMontee | null>("essai_montee", { pj, montee }),
  reglesCampagne: () => appeler<RegleInfo[]>("regles_campagne"),
  changerRegle: (id: string, valeur: string) => appeler<Retour>("changer_regle", { id, valeur }),
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
    default:
      return Promise.resolve(ok(`(aperçu) ${cmd}`));
  }
}
