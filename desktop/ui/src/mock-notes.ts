import { sansAccents } from "./texte";
// Uniquement pour l'aperçu sans Tauri (MOCK=1) : des notes d'exemple qui réagissent aux commandes.
// Retiré de la version de l'application.

import type { CritereNotes, FiltreChrono, GenreNote, LigneChrono, NoteVue, PageNotes } from "./notes-types";

type Retour = { ok: boolean; alerte: boolean; message: string };
const ok = (message: string): Retour => ({ ok: true, alerte: false, message });
const non = (message: string): Retour => ({ ok: false, alerte: false, message });

const RUBRIQUES = [
  { id: "evenement", nom: "Événements" },
  { id: "pnj", nom: "PNJ" },
  { id: "lieu", nom: "Lieux" },
  { id: "quete", nom: "Quêtes" },
  { id: "indice", nom: "Indices" },
  { id: "promesse-dette", nom: "Promesses et dettes" },
];
const NOMS_GENRES: Record<GenreNote, string> = { session: "Feuille de session", journal: "Journal de campagne", mj: "Notes de MJ" };

let prochain = 1;
let notes: NoteVue[] = [];
let pret = false;

function nouvelle(genre: GenreNote, rubrique: string | null, titre: string, texte: string, etiquettes: string[], jour: number): NoteVue {
  return {
    id: prochain++,
    genre,
    genreNom: NOMS_GENRES[genre],
    rubrique,
    rubriqueNom: RUBRIQUES.find((r) => r.id === rubrique)?.nom ?? null,
    titre,
    texte,
    etiquettes,
    epingle: false,
    jour,
    date: `jour ${jour}`,
    heure: jour % 2 ? "14:30" : null,
    reel: 1_792_000_000 + jour * 600,
    mentions: [],
  };
}

function initialiser() {
  notes = [
    nouvelle("session", null, "Séance 3 : la crypte", "On a trouvé une **épée longue** : [[objet:2|Épée longue]].\n\n- Valeros a été blessé\n- Lyra garde la clef", ["crypte"], 14),
    nouvelle("journal", "pnj", "Ameiko Kaijitsu", "Aubergiste de Sandpoint. *Propriétaire* du Dragon rouillé.", ["sandpoint"], 12),
    nouvelle("journal", "promesse-dette", "Dette envers Ameiko", "On lui doit **50 po** pour la chambre.", ["argent"], 13),
    nouvelle("mj", null, "Le MJ a dit", "Le pont de la ville est piégé la nuit.", ["piège"], 14),
  ];
  notes[0].epingle = true;
}

function trier(l: NoteVue[]) {
  return [...l].sort((a, b) => Number(b.epingle) - Number(a.epingle) || b.jour - a.jour || b.id - a.id);
}

export function simulerNotes(cmd: string, args?: Record<string, unknown>): Promise<unknown> | undefined {
  if (!pret && /^(page_notes|liste_notes|chronologie|creer_note|modifier_note|epingler_note|supprimer_note|noter_objet|exporter_notes)$/.test(cmd)) {
    pret = true;
    initialiser();
  }
  switch (cmd) {
    case "page_notes": {
      const page: PageNotes = {
        rubriques: RUBRIQUES,
        etiquettes: [...new Set(notes.flatMap((n) => n.etiquettes))].sort(),
        horloge: { jour: 15, date: "juredi 15 Rova 4712 AR", heure: "14:45" },
        nombre: notes.length,
      };
      return Promise.resolve(page);
    }
    case "liste_notes": {
      const c = (args?.critere ?? {}) as CritereNotes;
      const mots = sansAccents(c.texte ?? "").split(/\s+/).filter(Boolean);
      const liste = notes.filter(
        (n) =>
          (!c.genre || n.genre === c.genre) &&
          (!c.rubrique || n.rubrique === c.rubrique) &&
          (!c.epinglesSeulement || n.epingle) &&
          (!c.etiquette || n.etiquettes.some((e) => sansAccents(e) === sansAccents(c.etiquette!))) &&
          mots.every((m) => sansAccents(`${n.titre} ${n.texte} ${n.etiquettes.join(" ")}`).includes(m)),
      );
      return Promise.resolve(trier(liste));
    }
    case "chronologie": {
      const f = (args?.filtre ?? { sources: [] }) as FiltreChrono;
      const lignes: LigneChrono[] = [
        ...notes.map((n) => ({
          source: `note-${n.genre}`,
          sourceNom: n.genreNom,
          jour: n.jour,
          date: n.date,
          heure: n.heure,
          texte: n.titre || n.texte,
          note: n.id,
        })),
        { source: "intermede", sourceNom: "Intermède", jour: 15, date: "jour 15", heure: null, texte: "Jour suivant — Bar terminée", note: null },
        { source: "inventaire", sourceNom: "Inventaire", jour: 14, date: "jour 14", heure: null, texte: "Épée longue équipée", note: null },
        { source: "aventure", sourceNom: "Aventure", jour: 14, date: "jour 14", heure: "16:05", texte: "Entrée dans la crypte", note: null },
      ];
      const mots = sansAccents(f.texte ?? "").split(/\s+/).filter(Boolean);
      return Promise.resolve(
        lignes
          .filter((l) => !f.sources.length || f.sources.includes(l.source))
          .filter((l) => mots.every((m) => sansAccents(l.texte).includes(m)))
          .sort((a, b) => b.jour - a.jour),
      );
    }
    case "creer_note": {
      const titre = String(args?.titre ?? "");
      const texte = String(args?.texte ?? "");
      const genre = args?.genre as GenreNote;
      if (!titre.trim() && !texte.trim()) return Promise.resolve(non("La note est vide : donnez-lui un titre ou un texte"));
      if (genre === "journal" && !args?.rubrique) return Promise.resolve(non("Une entrée du journal de campagne a besoin d'une rubrique"));
      notes.push(nouvelle(genre, (args?.rubrique as string | null) ?? null, titre, texte, (args?.etiquettes as string[]) ?? [], 15));
      return Promise.resolve(ok("Note ajoutée"));
    }
    case "modifier_note": {
      const n = notes.find((x) => x.id === args?.id);
      if (!n) return Promise.resolve(non("Note inconnue"));
      n.titre = String(args?.titre ?? "");
      n.texte = String(args?.texte ?? "");
      n.etiquettes = (args?.etiquettes as string[]) ?? [];
      n.rubrique = (args?.rubrique as string | null) ?? null;
      n.rubriqueNom = RUBRIQUES.find((r) => r.id === n.rubrique)?.nom ?? null;
      return Promise.resolve(ok("Note enregistrée"));
    }
    case "epingler_note": {
      const n = notes.find((x) => x.id === args?.id);
      if (n) n.epingle = Boolean(args?.epingle);
      return Promise.resolve(ok(n?.epingle ? "Note épinglée" : "Note désépinglée"));
    }
    case "supprimer_note":
      notes = notes.filter((n) => n.id !== args?.id);
      return Promise.resolve(ok("Note supprimée (annulable)"));
    case "noter_objet":
      return Promise.resolve(ok("Note de l'objet enregistrée"));
    case "exporter_notes":
      return Promise.resolve(ok("Notes exportées (aperçu)"));
    default:
      return undefined;
  }
}
