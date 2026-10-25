// Choix de l'apparence : le thème est un simple jeu de valeurs (jetons de couleur) mémorisé sur cet ordinateur.

import { useState } from "react";

export const THEMES: { id: string; nom: string; sous: string; couleurs: string[] }[] = [
  { id: "", nom: "Sombre actuel", sous: "L'apparence validée le 23/10/2026", couleurs: ["#0f1115", "#171a20", "#e9c46a", "#7fd1a8"] },
  { id: "parchemin", nom: "A · Parchemin arcanique", sous: "Sombre chaud, brun et ambre, esprit grimoire", couleurs: ["#1a1410", "#271e17", "#e0a84a", "#8fbf6a"] },
  { id: "nuit", nom: "B · Nuit de Golarion", sous: "Sombre froid, bleu nuit, esprit jeu vidéo", couleurs: ["#0a0d1a", "#121830", "#7aa2ff", "#4be0a0"] },
];

const CLE = "gestion-pathfinder.theme";

export function appliquerThemeEnregistre() {
  try {
    const t = window.localStorage.getItem(CLE) ?? "";
    if (t) document.documentElement.dataset.theme = t;
  } catch {
    /* pas de stockage : thème par défaut */
  }
}

export function Apparence() {
  const [choisi, setChoisi] = useState(() => document.documentElement.dataset.theme ?? "");
  const choisir = (id: string) => {
    setChoisi(id);
    if (id) document.documentElement.dataset.theme = id;
    else delete document.documentElement.dataset.theme;
    try {
      window.localStorage.setItem(CLE, id);
    } catch {
      /* sans stockage, le choix ne dure que jusqu'à la fermeture */
    }
  };
  return (
    <section className="section">
      <h2>Apparence</h2>
      <p className="doux">
        Les couleurs viennent de jetons de design : changer de thème ne change que ces valeurs. Les nouvelles pages (Aventure et combat)
        les suivent entièrement ; les anciennes pages seront reprises plus tard.
      </p>
      <div className="apparence" role="radiogroup" aria-label="Thème">
        {THEMES.map((t) => (
          <button key={t.id || "defaut"} role="radio" aria-checked={choisi === t.id} className={"pastille-theme" + (choisi === t.id ? " choisi" : "")} onClick={() => choisir(t.id)}>
            <span className="echantillon" aria-hidden="true">
              {t.couleurs.map((c) => (
                <i key={c} style={{ background: c }} />
              ))}
            </span>
            <strong>{t.nom}</strong>
            <span className="doux">{t.sous}</span>
          </button>
        ))}
      </div>
    </section>
  );
}
