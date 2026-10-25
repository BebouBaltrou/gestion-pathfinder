import { useEffect, useMemo, useState } from "react";
import type { Outils } from "../App";
import { api } from "../api";
import { Markdown } from "../composants/Markdown";
import { ouvrirLien, prendreDemande } from "../lien-interne";
import {
  GENRES,
  SOURCES_CHRONO,
  heureReelle,
  type CritereNotes,
  type GenreNote,
  type LigneChrono,
  type NoteVue,
  type PageNotes,
} from "../notes-types";
import "../notes.css";

type Onglet = GenreNote | "chrono";
type Brouillon = { id: number | null; titre: string; texte: string; etiquettes: string; rubrique: string };

const brouillonVide = (rubrique = "evenement"): Brouillon => ({ id: null, titre: "", texte: "", etiquettes: "", rubrique });
const etiquettesDe = (s: string) => s.split(/[,;]/).map((e) => e.trim()).filter(Boolean);

/** Feuille de session, journal de campagne structuré, notes de MJ, et la chronologie qui les réunit. */
export function Notes({ outils }: { outils: Outils }) {
  const [page, setPage] = useState<PageNotes | null>(null);
  const [onglet, setOnglet] = useState<Onglet>("session");
  const [recherche, setRecherche] = useState("");
  const [rubrique, setRubrique] = useState("");
  const [etiquette, setEtiquette] = useState("");
  const [epingles, setEpingles] = useState(false);
  const [liste, setListe] = useState<NoteVue[]>([]);
  const [choisie, setChoisie] = useState<number | null>(null);
  const [edition, setEdition] = useState<Brouillon | null>(null);
  const [rapide, setRapide] = useState("");
  const [rubriqueRapide, setRubriqueRapide] = useState("evenement");
  const [confirmer, setConfirmer] = useState(false);
  const [sources, setSources] = useState<string[]>([]);
  const [chrono, setChrono] = useState<LigneChrono[]>([]);

  useEffect(() => {
    api.pageNotes().then(setPage, () => setPage(null));
  }, [outils.version]);

  // Une note citée dans un lien interne s'ouvre à l'arrivée sur la page.
  useEffect(() => {
    const cible = prendreDemande("note");
    if (cible) {
      setOnglet("session");
      setChoisie(Number(cible));
    }
  }, []);

  const critere = useMemo<CritereNotes>(
    () => ({
      genre: onglet === "chrono" ? null : onglet,
      rubrique: onglet === "journal" && rubrique ? rubrique : null,
      etiquette: etiquette || null,
      texte: recherche.trim() || null,
      epinglesSeulement: epingles,
    }),
    [onglet, rubrique, etiquette, recherche, epingles],
  );

  useEffect(() => {
    if (onglet === "chrono") {
      api.chronologie({ sources, texte: recherche.trim() || null }).then(setChrono, () => setChrono([]));
    } else {
      api.listeNotes(critere).then(setListe, () => setListe([]));
    }
  }, [critere, onglet, sources, recherche, outils.version]);

  useEffect(() => {
    setChoisie(null);
    setEdition(null);
    setConfirmer(false);
  }, [onglet]);

  const note = liste.find((n) => n.id === choisie) ?? null;
  const rubriques = page?.rubriques ?? [];
  const horloge = page?.horloge;

  const lien = (genre: string, cible: string) => {
    if (genre === "note") {
      setOnglet("session");
      setChoisie(Number(cible));
      return;
    }
    ouvrirLien(genre, cible, outils.aller);
  };

  const noterVite = async () => {
    if (!rapide.trim() || onglet === "chrono") return;
    const r = await outils.agir(api.creerNote(onglet, onglet === "journal" ? rubriqueRapide : null, "", rapide, []));
    if (r?.ok) setRapide("");
  };

  const enregistrer = async () => {
    if (!edition || onglet === "chrono") return;
    const rub = onglet === "journal" ? edition.rubrique : null;
    const r =
      edition.id === null
        ? await outils.agir(api.creerNote(onglet, rub, edition.titre, edition.texte, etiquettesDe(edition.etiquettes)))
        : await outils.agir(api.modifierNote(edition.id, rub, edition.titre, edition.texte, etiquettesDe(edition.etiquettes)));
    if (r?.ok) setEdition(null);
  };

  const genre = GENRES.find((g) => g.id === onglet);

  return (
    <div className="page page-notes">
      <div className="titre-page">
        <h1>Notes</h1>
        <p>
          Prenez des notes pendant la partie : feuille de session, journal de campagne par rubrique, et ce que le MJ a dit.
          {horloge && (
            <>
              {" "}
              Horloge de campagne : <strong>{horloge.date}</strong>
              {horloge.heure && (
                <>
                  {" "}
                  à <strong>{horloge.heure}</strong>
                </>
              )}
              .
            </>
          )}
        </p>
      </div>

      <div className="groupe-boutons onglets-notes" role="tablist" aria-label="Genre de notes">
        {GENRES.map((g) => (
          <button key={g.id} role="tab" aria-selected={onglet === g.id} className={"bouton" + (onglet === g.id ? " principal" : "")} onClick={() => setOnglet(g.id)}>
            {g.nom}
          </button>
        ))}
        <button role="tab" aria-selected={onglet === "chrono"} className={"bouton" + (onglet === "chrono" ? " principal" : "")} onClick={() => setOnglet("chrono")}>
          Chronologie
        </button>
      </div>
      {genre && <p className="doux">{genre.aide}</p>}

      <div className="barre-filtres">
        <input className="champ" type="search" placeholder="Rechercher dans les notes…" aria-label="Rechercher dans les notes" value={recherche} onChange={(e) => setRecherche(e.target.value)} />
        {onglet === "journal" && (
          <select className="champ" aria-label="Rubrique" value={rubrique} onChange={(e) => setRubrique(e.target.value)}>
            <option value="">Toutes les rubriques</option>
            {rubriques.map((r) => (
              <option key={r.id} value={r.id}>{r.nom}</option>
            ))}
          </select>
        )}
        {onglet !== "chrono" && (
          <>
            <select className="champ" aria-label="Étiquette" value={etiquette} onChange={(e) => setEtiquette(e.target.value)}>
              <option value="">Toutes les étiquettes</option>
              {(page?.etiquettes ?? []).map((e) => (
                <option key={e} value={e}>#{e}</option>
              ))}
            </select>
            <label className="case-a-cocher">
              <input type="checkbox" checked={epingles} onChange={(e) => setEpingles(e.target.checked)} />
              Épinglées seulement
            </label>
            <button className="bouton" onClick={() => outils.agir(api.exporterNotes(critere))} title="Enregistrer ces notes en Markdown ou en texte">
              Exporter
            </button>
          </>
        )}
      </div>

      {onglet === "chrono" ? (
        <div className="chronologie">
          <fieldset className="sources-chrono">
            <legend>Journaux à montrer (chacun reste distinct à la source)</legend>
            {SOURCES_CHRONO.map(([id, nom]) => (
              <label key={id} className="case-a-cocher">
                <input
                  type="checkbox"
                  checked={sources.length === 0 || sources.includes(id)}
                  onChange={(e) => {
                    const actuelles = sources.length === 0 ? SOURCES_CHRONO.map(([s]) => s) : sources;
                    const suite = e.target.checked ? [...actuelles, id] : actuelles.filter((s) => s !== id);
                    setSources(suite.length === SOURCES_CHRONO.length ? [] : suite);
                  }}
                />
                {nom}
              </label>
            ))}
          </fieldset>
          <ol className="lignes-chrono">
            {chrono.length === 0 && <li className="doux">Rien à afficher.</li>}
            {chrono.map((l, i) => (
              <li key={i} className={"ligne-chrono " + l.source}>
                <span className="quand">
                  {l.date}
                  {l.heure && ` · ${l.heure}`}
                </span>
                <span className="source mini">{l.sourceNom}</span>
                {l.note !== null ? (
                  <button className="lien-interne" onClick={() => lien("note", String(l.note))}>{l.texte}</button>
                ) : (
                  <span>{l.texte}</span>
                )}
              </li>
            ))}
          </ol>
        </div>
      ) : (
        <div className="disposition-notes">
          <div className="colonne-liste-notes">
            <div className="saisie-rapide">
              <textarea
                className="champ zone large"
                rows={2}
                placeholder={onglet === "mj" ? "Ce que le MJ vient de dire… (Ctrl+Entrée pour ajouter)" : "Noter vite… (Ctrl+Entrée pour ajouter)"}
                aria-label="Noter vite"
                value={rapide}
                onChange={(e) => setRapide(e.target.value)}
                onKeyDown={(e) => e.key === "Enter" && (e.ctrlKey || e.metaKey) && noterVite()}
              />
              <div className="ligne-action">
                {onglet === "journal" && (
                  <select className="champ" aria-label="Rubrique de la note" value={rubriqueRapide} onChange={(e) => setRubriqueRapide(e.target.value)}>
                    {rubriques.map((r) => (
                      <option key={r.id} value={r.id}>{r.nom}</option>
                    ))}
                  </select>
                )}
                <button className="bouton principal" disabled={!rapide.trim()} onClick={noterVite}>Ajouter</button>
                <button
                  className="bouton"
                  onClick={() => {
                    setEdition(brouillonVide(rubriqueRapide));
                    setChoisie(null);
                  }}
                >
                  Note détaillée…
                </button>
              </div>
            </div>
            <ul className="liste-notes" aria-label="Notes">
              {liste.length === 0 && <li className="doux">Aucune note.</li>}
              {liste.map((n) => (
                <li key={n.id}>
                  <button
                    className={"carte-note" + (n.id === choisie ? " choisie" : "")}
                    onClick={() => {
                      setChoisie(n.id);
                      setEdition(null);
                      setConfirmer(false);
                    }}
                  >
                    <span className="titre-note">
                      {n.epingle && <span aria-label="Épinglée">📌 </span>}
                      {n.titre || n.texte.split("\n")[0]}
                    </span>
                    <span className="meta-note">
                      {n.date}
                      {n.heure && ` · ${n.heure}`}
                      {n.rubriqueNom && ` · ${n.rubriqueNom}`}
                    </span>
                    {n.etiquettes.length > 0 && <span className="etiquettes">{n.etiquettes.map((e) => `#${e}`).join(" ")}</span>}
                  </button>
                </li>
              ))}
            </ul>
          </div>

          <section className="lecteur-note" aria-label="Note">
            {edition ? (
              <div className="editeur-note">
                <input className="champ large" placeholder="Titre" aria-label="Titre" value={edition.titre} onChange={(e) => setEdition({ ...edition, titre: e.target.value })} />
                {onglet === "journal" && (
                  <select className="champ" aria-label="Rubrique" value={edition.rubrique} onChange={(e) => setEdition({ ...edition, rubrique: e.target.value })}>
                    {rubriques.map((r) => (
                      <option key={r.id} value={r.id}>{r.nom}</option>
                    ))}
                  </select>
                )}
                <textarea
                  className="champ zone large"
                  rows={12}
                  placeholder="Texte en Markdown. Lien interne : [[objet:12|Épée longue]], [[note:3]]"
                  aria-label="Texte"
                  value={edition.texte}
                  onChange={(e) => setEdition({ ...edition, texte: e.target.value })}
                />
                <input className="champ large" placeholder="Étiquettes, séparées par des virgules" aria-label="Étiquettes" value={edition.etiquettes} onChange={(e) => setEdition({ ...edition, etiquettes: e.target.value })} />
                <div className="ligne-action">
                  <button className="bouton principal" onClick={enregistrer}>Enregistrer</button>
                  <button className="bouton" onClick={() => setEdition(null)}>Annuler</button>
                </div>
              </div>
            ) : note ? (
              <article>
                <h3>{note.titre || "(sans titre)"}</h3>
                <p className="doux">
                  {note.genreNom}
                  {note.rubriqueNom && ` · ${note.rubriqueNom}`} · {note.date}
                  {note.heure && ` à ${note.heure}`} · prise le {heureReelle(note.reel)}
                </p>
                <Markdown texte={note.texte} onLien={lien} />
                {note.etiquettes.length > 0 && <p className="etiquettes">{note.etiquettes.map((e) => `#${e}`).join(" ")}</p>}
                <div className="ligne-action">
                  <button
                    className="bouton"
                    onClick={() =>
                      setEdition({
                        id: note.id,
                        titre: note.titre,
                        texte: note.texte,
                        etiquettes: note.etiquettes.join(", "),
                        rubrique: note.rubrique ?? "evenement",
                      })
                    }
                  >
                    Modifier
                  </button>
                  <button className="bouton" onClick={() => outils.agir(api.epinglerNote(note.id, !note.epingle))}>
                    {note.epingle ? "Désépingler" : "Épingler"}
                  </button>
                  {confirmer ? (
                    <>
                      <button
                        className="bouton danger"
                        onClick={() => {
                          outils.agir(api.supprimerNote(note.id));
                          setConfirmer(false);
                          setChoisie(null);
                        }}
                      >
                        Oui, supprimer
                      </button>
                      <button className="bouton" onClick={() => setConfirmer(false)}>Annuler</button>
                    </>
                  ) : (
                    <button className="bouton danger" onClick={() => setConfirmer(true)}>Supprimer</button>
                  )}
                </div>
              </article>
            ) : (
              <p className="doux">Choisissez une note pour la lire, ou ajoutez-en une.</p>
            )}
          </section>
        </div>
      )}
    </div>
  );
}
