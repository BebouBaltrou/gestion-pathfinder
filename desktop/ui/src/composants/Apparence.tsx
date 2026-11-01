// Choix de l'apparence : le thème est un simple jeu de valeurs (jetons de couleur) mémorisé sur cet ordinateur.

import { useState } from "react";

export const THEMES: { id: string; nom: string; sous: string; couleurs: string[] }[] = [
  { id: "", nom: "Sombre actuel", sous: "L'apparence validée le 23/10/2026", couleurs: ["#0f1115", "#171a20", "#e9c46a", "#7fd1a8"] },
  { id: "parchemin", nom: "A · Parchemin arcanique", sous: "Sombre chaud, brun et ambre, esprit grimoire", couleurs: ["#1a1410", "#271e17", "#e0a84a", "#8fbf6a"] },
  { id: "nuit", nom: "B · Nuit de Golarion", sous: "Sombre froid, bleu nuit, esprit jeu vidéo", couleurs: ["#0a0d1a", "#121830", "#7aa2ff", "#4be0a0"] },
];

const CLE = "gestion-pathfinder.theme";

// Taille de l'affichage : « auto » suit la taille de la fenêtre (référence 1920 × 1080) ; les autres valeurs
// fixent un facteur d'agrandissement, utile sur un écran 1280 × 720 ou 2560 × 1440.
export const ECHELLES: { id: string; nom: string; facteur: number | null }[] = [
  { id: "auto", nom: "Automatique", facteur: null },
  { id: "compact", nom: "Compacte (1280 × 720)", facteur: 0.85 },
  { id: "standard", nom: "Standard (1920 × 1080)", facteur: 1 },
  { id: "grand", nom: "Grande (2560 × 1440)", facteur: 1.3 },
];

const CLE_ECHELLE = "gestion-pathfinder.echelle";

function facteurEchelle(id: string): number {
  const choix = ECHELLES.find((e) => e.id === id);
  if (!choix || choix.facteur === null) {
    return Math.min(1.4, Math.max(0.8, window.innerWidth / 1920));
  }
  return choix.facteur;
}

export function appliquerEchelle(id: string) {
  document.documentElement.style.zoom = String(facteurEchelle(id));
}

export function appliquerThemeEnregistre() {
  try {
    const t = window.localStorage.getItem(CLE) ?? "";
    if (t) document.documentElement.dataset.theme = t;
    appliquerEchelle(window.localStorage.getItem(CLE_ECHELLE) ?? "auto");
  } catch {
    /* pas de stockage : thème et taille par défaut */
    appliquerEchelle("auto");
  }
}

export function Apparence() {
  const [choisi, setChoisi] = useState(() => document.documentElement.dataset.theme ?? "");
  const [echelle, setEchelle] = useState(() => {
    try {
      return window.localStorage.getItem(CLE_ECHELLE) ?? "auto";
    } catch {
      return "auto";
    }
  });
  const choisirEchelle = (id: string) => {
    setEchelle(id);
    appliquerEchelle(id);
    try {
      window.localStorage.setItem(CLE_ECHELLE, id);
    } catch {
      /* sans stockage, le choix ne dure que jusqu'à la fermeture */
    }
  };
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
      <h2>Taille de l'affichage</h2>
      <p className="doux">Adapte la taille des textes et des cartes à votre écran. « Automatique » convient à la plupart des cas.</p>
      <div className="segments" role="radiogroup" aria-label="Taille de l'affichage">
        {ECHELLES.map((e) => (
          <button key={e.id} role="radio" aria-checked={echelle === e.id} className={echelle === e.id ? "actif" : ""} onClick={() => choisirEchelle(e.id)}>
            {e.nom}
          </button>
        ))}
      </div>
    </section>
  );
}
