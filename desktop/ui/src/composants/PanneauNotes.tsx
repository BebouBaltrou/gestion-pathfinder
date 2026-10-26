import { useEffect, useRef, useState } from "react";
import type { Outils } from "../App";
import { api } from "../api";
import { ouvrirLien } from "../lien-interne";
import { GENRES, type GenreNote, type NoteVue, type PageNotes } from "../notes-types";
import { Markdown } from "./Markdown";

/**
 * Panneau de notes repliable, accessible depuis toutes les vues (Alt+N) : saisie rapide pendant
 * la partie et dernières notes du genre choisi.
 */
export function PanneauNotes(props: { outils: Outils; onFermer: () => void }) {
  const { outils, onFermer } = props;
  const [genre, setGenre] = useState<GenreNote>("session");
  const [texte, setTexte] = useState("");
  const [rubrique, setRubrique] = useState("evenement");
  const [page, setPage] = useState<PageNotes | null>(null);
  const [recentes, setRecentes] = useState<NoteVue[]>([]);
  const zone = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    zone.current?.focus();
  }, []);

  useEffect(() => {
    api.pageNotes().then(setPage, () => setPage(null));
    api.listeNotes({ genre }).then((l) => setRecentes(l.slice(0, 8)), () => setRecentes([]));
  }, [genre, outils.version]);

  const ajouter = async () => {
    if (!texte.trim()) return;
    const r = await outils.agir(api.creerNote(genre, genre === "journal" ? rubrique : null, "", texte, []));
    if (r?.ok) setTexte("");
  };

  return (
    <aside className="panneau-notes" aria-label="Notes de la partie">
      <header>
        <h2>Notes</h2>
        <button className="bouton discret" onClick={onFermer} aria-label="Fermer le panneau de notes">Fermer (Alt+N)</button>
      </header>
      {page?.horloge && (
        <p className="doux">
          {page.horloge.date}
          {page.horloge.heure && ` · ${page.horloge.heure}`}
        </p>
      )}
      <div className="groupe-boutons" role="tablist" aria-label="Genre de notes">
        {GENRES.map((g) => (
          <button key={g.id} role="tab" aria-selected={genre === g.id} className={"bouton" + (genre === g.id ? " principal" : "")} onClick={() => setGenre(g.id)}>
            {g.id === "session" ? "Session" : g.id === "journal" ? "Journal" : "MJ"}
          </button>
        ))}
      </div>
      <textarea
        ref={zone}
        className="champ zone large"
        rows={4}
        placeholder={genre === "mj" ? "Ce que le MJ vient de dire…" : "Noter vite… (Ctrl+Entrée pour ajouter)"}
        aria-label="Nouvelle note"
        value={texte}
        onChange={(e) => setTexte(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && (e.ctrlKey || e.metaKey) && ajouter()}
      />
      <div className="ligne-action">
        {genre === "journal" && (
          <select className="champ" aria-label="Rubrique" value={rubrique} onChange={(e) => setRubrique(e.target.value)}>
            {(page?.rubriques ?? []).map((r) => (
              <option key={r.id} value={r.id}>{r.nom}</option>
            ))}
          </select>
        )}
        <button className="bouton principal" disabled={!texte.trim()} onClick={ajouter}>Ajouter</button>
        <button className="bouton" onClick={() => { outils.aller("notes"); onFermer(); }}>Toutes les notes</button>
      </div>
      <ul className="recentes">
        {recentes.length === 0 && <li className="doux">Aucune note pour l'instant.</li>}
        {recentes.map((n) => (
          <li key={n.id}>
            <div className="meta-note">
              {n.epingle && <span aria-label="Épinglée">📌 </span>}
              {n.date}
              {n.heure && ` · ${n.heure}`}
              {n.rubriqueNom && ` · ${n.rubriqueNom}`}
            </div>
            {n.titre && <strong>{n.titre}</strong>}
            <Markdown texte={n.texte} onLien={(g, c) => ouvrirLien(g, c, outils.aller)} />
          </li>
        ))}
      </ul>
    </aside>
  );
}
