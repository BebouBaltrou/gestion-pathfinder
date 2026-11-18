// Choix de l'apparence : le thème est un simple jeu de valeurs (jetons de couleur) mémorisé sur cet ordinateur.

import { useState } from "react";
import { api } from "../api";

export const THEMES: { id: string; nom: string; sous: string; couleurs: string[] }[] = [
  { id: "carnet", nom: "Carnet de voyage", sous: "Clair, papier et encre sépia (par défaut)", couleurs: ["#f3ead6", "#faf3e2", "#8c2f1f", "#9a6b12"] },
  { id: "neon", nom: "Néon nocturne", sous: "Sombre, quadrillage et lueurs par capital", couleurs: ["#07080d", "#0e111c", "#2ee6d6", "#a66bff"] },
  { id: "", nom: "Sombre d'origine", sous: "L'apparence validée le 23/10/2026", couleurs: ["#0f1115", "#171a20", "#e9c46a", "#7fd1a8"] },
];

const THEME_PAR_DEFAUT = "carnet";

// Nouvelle clé (18/11/2026) : tout le monde repart sur le thème par défaut « Carnet de voyage ».
const CLE = "gestion-pathfinder.theme.v2";

// Taille de l'affichage : « auto » suit la taille de la fenêtre (référence 1920 × 1080) ; les autres valeurs
// fixent un facteur d'agrandissement, utile sur un écran 1280 × 720 ou 2560 × 1440.
export const ECHELLES: { id: string; nom: string; facteur: number | null }[] = [
  { id: "auto", nom: "Automatique", facteur: null },
  { id: "compact", nom: "Compacte (1280 × 720)", facteur: 0.85 },
  { id: "standard", nom: "Standard (1920 × 1080)", facteur: 1 },
  { id: "grand", nom: "Grande (2560 × 1440)", facteur: 1.3 },
];

const CLE_ECHELLE = "gestion-pathfinder.echelle";

// Vision des couleurs : remplace les couleurs des capitaux et des états (voir themes.css), dans chaque thème.
export const VISIONS: { id: string; nom: string; sous: string }[] = [
  { id: "", nom: "Normale", sous: "Couleurs d'origine du thème" },
  { id: "protanopie", nom: "Protanopie", sous: "Rouge peu ou pas perçu (environ 1 % des hommes) ; vaut aussi pour la protanomalie" },
  { id: "deuteranopie", nom: "Deutéranopie", sous: "Vert peu ou pas perçu (la forme la plus courante, environ 6 % des hommes avec la deutéranomalie)" },
  { id: "tritanopie", nom: "Tritanopie", sous: "Bleu peu ou pas perçu, bleu/vert et jaune/violet confondus (rare)" },
  { id: "achromatopsie", nom: "Achromatopsie", sous: "Aucune couleur perçue, seulement des gris (très rare)" },
];

const CLE_VISION = "gestion-pathfinder.daltonisme";

function appliquerVision(id: string) {
  if (id) document.documentElement.dataset.daltonisme = id;
  else delete document.documentElement.dataset.daltonisme;
}

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

function echelleEnregistree(): string {
  try {
    return window.localStorage.getItem(CLE_ECHELLE) ?? "auto";
  } catch {
    return "auto";
  }
}

export function appliquerThemeEnregistre() {
  try {
    const t = window.localStorage.getItem(CLE) ?? THEME_PAR_DEFAUT;
    if (t) document.documentElement.dataset.theme = t;
    appliquerVision(window.localStorage.getItem(CLE_VISION) ?? "");
  } catch {
    document.documentElement.dataset.theme = THEME_PAR_DEFAUT;
  }
  // La fenêtre n'a pas encore sa taille définitive au lancement : on recalcule l'échelle une fois chargée
  // et à chaque changement de taille, pour que « Automatique » reste juste sans passer par le réglage.
  const relancer = () => appliquerEchelle(echelleEnregistree());
  relancer();
  window.addEventListener("load", relancer);
  window.addEventListener("resize", relancer);
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
  const [vision, setVision] = useState(() => document.documentElement.dataset.daltonisme ?? "");
  const choisirVision = (id: string) => {
    setVision(id);
    appliquerVision(id);
    try {
      window.localStorage.setItem(CLE_VISION, id);
    } catch {
      /* sans stockage, le choix ne dure que jusqu'à la fermeture */
    }
  };
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
      <h2>Vision des couleurs</h2>
      <p className="doux">Adapte les couleurs des capitaux et des états à un daltonisme, dans les trois thèmes.</p>
      <div className="apparence" role="radiogroup" aria-label="Vision des couleurs">
        {VISIONS.map((v) => (
          <button key={v.id || "normale"} role="radio" aria-checked={vision === v.id} className={"pastille-theme" + (vision === v.id ? " choisi" : "")} onClick={() => choisirVision(v.id)}>
            <strong>{v.nom}</strong>
            <span className="doux">{v.sous}</span>
          </button>
        ))}
      </div>
      <h2>Taille de l'affichage</h2>
      <p className="doux">Adapte la taille des textes et des cartes à votre écran. « Automatique » convient à la plupart des cas.</p>
      <div className="ligne" style={{ gap: 12, marginBottom: 14 }}>
        <button type="button" className="bouton" onClick={() => api.basculerPleinEcran()}>
          Basculer en plein écran (F11)
        </button>
      </div>
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
