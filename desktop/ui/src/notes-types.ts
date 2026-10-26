// Types des notes : ils reprennent ceux de crates/vues/src/notes.rs.

export type GenreNote = "session" | "journal" | "mj";

export const GENRES: { id: GenreNote; nom: string; aide: string }[] = [
  { id: "session", nom: "Feuille de session", aide: "Ce que vous notez pendant la séance." },
  { id: "journal", nom: "Journal de campagne", aide: "Événements, PNJ, lieux, quêtes, indices, promesses ou dettes." },
  { id: "mj", nom: "Notes de MJ", aide: "Ce que le MJ a dit ou consigné à la table." },
];

export type RubriqueVue = { id: string; nom: string };
export type HorlogeVue = { jour: number; date: string; heure: string | null };
export type PageNotes = { rubriques: RubriqueVue[]; etiquettes: string[]; horloge: HorlogeVue; nombre: number };

export type MentionVue = { genre: string; cible: string; libelle: string };

export type NoteVue = {
  id: number;
  genre: GenreNote;
  genreNom: string;
  rubrique: string | null;
  rubriqueNom: string | null;
  titre: string;
  texte: string;
  etiquettes: string[];
  epingle: boolean;
  jour: number;
  date: string;
  heure: string | null;
  /** Heure réelle, secondes depuis 1970 (UTC). */
  reel: number;
  mentions: MentionVue[];
};

export type CritereNotes = {
  genre?: GenreNote | null;
  rubrique?: string | null;
  etiquette?: string | null;
  texte?: string | null;
  epinglesSeulement?: boolean;
};

export type LigneChrono = {
  source: string;
  sourceNom: string;
  jour: number;
  date: string;
  heure: string | null;
  texte: string;
  note: number | null;
};

export type FiltreChrono = { sources: string[]; texte?: string | null };

export const SOURCES_CHRONO: [string, string][] = [
  ["note-session", "Feuille de session"],
  ["note-journal", "Journal de campagne"],
  ["note-mj", "Notes de MJ"],
  ["intermede", "Intermède"],
  ["inventaire", "Inventaire"],
  ["aventure", "Aventure"],
];

/** Heure réelle lisible (« 25/10/2026 22:30 »). */
export function heureReelle(secondes: number): string {
  return new Date(secondes * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
}
