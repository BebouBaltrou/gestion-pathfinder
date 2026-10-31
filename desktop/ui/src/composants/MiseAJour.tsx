// Vérification de nouvelle version (GitHub). Ne télécharge rien : elle indique seulement qu'une version plus récente existe.
// Désactivable dans les Paramètres ; le choix est mémorisé sur cet ordinateur.

import { useState } from "react";

export const VERSION = "1.2.0";
const DEPOT = "BebouBaltrou/gestion-pathfinder";
const CLE = "gestion-pathfinder.maj-auto";

export type Nouvelle = { version: string; page: string };

export function miseAJourActive(): boolean {
  try {
    return window.localStorage.getItem(CLE) !== "non";
  } catch {
    return true;
  }
}

function enregistrerMiseAJour(active: boolean) {
  try {
    window.localStorage.setItem(CLE, active ? "oui" : "non");
  } catch {
    /* sans stockage, le choix ne dure que jusqu'à la fermeture */
  }
}

/** Compare deux numéros « 1.2.0 » ; renvoie vrai si `a` est plus récent que `b`. */
export function plusRecente(a: string, b: string): boolean {
  const pa = a.split(".").map((n) => parseInt(n, 10) || 0);
  const pb = b.split(".").map((n) => parseInt(n, 10) || 0);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const x = pa[i] ?? 0;
    const y = pb[i] ?? 0;
    if (x !== y) return x > y;
  }
  return false;
}

/** Interroge GitHub. Renvoie la version plus récente, ou null si à jour ou injoignable. */
export async function chercherMiseAJour(): Promise<Nouvelle | null> {
  try {
    const r = await fetch(`https://api.github.com/repos/${DEPOT}/releases/latest`, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!r.ok) return null;
    const j = (await r.json()) as { tag_name?: string; html_url?: string };
    const version = (j.tag_name ?? "").replace(/^v/, "");
    if (!version || !plusRecente(version, VERSION)) return null;
    return { version, page: j.html_url ?? `https://github.com/${DEPOT}/releases` };
  } catch {
    return null;
  }
}

/** Section des Paramètres : interrupteur et vérification manuelle. */
export function MiseAJour() {
  const [active, setActive] = useState(miseAJourActive);
  const [etat, setEtat] = useState("");

  const basculer = (v: boolean) => {
    setActive(v);
    enregistrerMiseAJour(v);
  };

  const verifier = async () => {
    setEtat("Recherche…");
    const n = await chercherMiseAJour();
    setEtat(n ? `Version ${n.version} disponible : ${n.page}` : `Version ${VERSION} : à jour (ou vérification impossible sans connexion).`);
  };

  return (
    <section className="section">
      <h2>Mises à jour</h2>
      <p className="doux">
        Version installée : {VERSION}. Au lancement, l'application regarde seulement si une version plus récente existe sur GitHub
        et l'indique ; rien n'est téléchargé ni installé.
      </p>
      <label className="doux">
        <input type="checkbox" checked={active} onChange={(e) => basculer(e.target.checked)} /> Vérifier les mises à jour automatiquement
      </label>{" "}
      <button className="bouton discret" onClick={verifier}>
        Vérifier maintenant
      </button>
      {etat && <p className="doux">{etat}</p>}
    </section>
  );
}
