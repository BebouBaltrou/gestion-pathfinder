// Résumé des changements par version, affiché une fois après une mise à jour.
// À chaque nouvelle version : ajouter une entrée EN TÊTE de la liste (la plus récente d'abord).

export type NotesVersion = { version: string; changements: string[] };

export const NOTES: NotesVersion[] = [
  {
    version: "1.2.33",
    changements: [
      "Barre de progression du téléchargement de la mise à jour, aux couleurs du thème choisi.",
      "Résumé des changements affiché une fois après chaque mise à jour.",
    ],
  },
  {
    version: "1.2.32",
    changements: ["Mise à jour automatique corrigée : les erreurs s'affichent et la bonne version est installée."],
  },
];

const CLE = "gestion-pathfinder.derniere-version";

function lire(): string | null {
  try {
    return window.localStorage.getItem(CLE);
  } catch {
    return null;
  }
}

/** Notes des versions arrivées depuis la dernière ouverture (vide au premier lancement ou sans changement). */
export function notesDepuisDerniereVersion(versionActuelle: string): NotesVersion[] {
  const vue = lire();
  try {
    window.localStorage.setItem(CLE, versionActuelle);
  } catch {
    /* sans stockage, le résumé ne s'affichera pas */
  }
  if (!vue || vue === versionActuelle) return [];
  const iActuelle = NOTES.findIndex((n) => n.version === versionActuelle);
  if (iActuelle < 0) return [];
  const iVue = NOTES.findIndex((n) => n.version === vue);
  return NOTES.slice(iActuelle, iVue < 0 ? iActuelle + 1 : iVue);
}
