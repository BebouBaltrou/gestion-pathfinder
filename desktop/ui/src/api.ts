// Pont vers le cœur Rust (commandes Tauri). Les types reprennent ceux de crates/vues.

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
  creerCampagne: (nom: string) => appeler<Retour>("creer_campagne", { nom }),
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
      };
      return Promise.resolve(p);
    }
    default:
      return Promise.resolve(ok(`(aperçu) ${cmd}`));
  }
}
