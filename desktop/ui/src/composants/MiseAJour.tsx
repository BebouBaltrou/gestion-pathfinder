// Mises à jour : vérifie la dernière version publiée puis l'installe (module de mise à jour de Tauri).
// Désactivable dans les Paramètres ; le choix est mémorisé sur cet ordinateur.

import { useState } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export const VERSION = "1.2.33";
const CLE = "gestion-pathfinder.maj-auto";

export type Progression = { recu: number; total: number | null; fini: boolean };

export type Nouvelle = { version: string; installer: (suivi?: (p: Progression) => void) => Promise<void> };

/** Télécharge puis installe en signalant l'avancement. */
async function telecharger(
  maj: NonNullable<Awaited<ReturnType<typeof check>>>,
  suivi?: (p: Progression) => void,
) {
  let recu = 0;
  let total: number | null = null;
  await maj.downloadAndInstall((e) => {
    if (e.event === "Started") {
      total = e.data.contentLength ?? null;
      suivi?.({ recu: 0, total, fini: false });
    } else if (e.event === "Progress") {
      recu += e.data.chunkLength;
      suivi?.({ recu, total, fini: false });
    } else if (e.event === "Finished") {
      suivi?.({ recu, total, fini: true });
    }
  });
  await relaunch();
}

/** Barre de progression aux couleurs du thème (variables du thème actif). */
export function BarreProgression({ p }: { p: Progression }) {
  const pct = p.fini ? 100 : p.total ? Math.min(100, Math.round((p.recu / p.total) * 100)) : null;
  const mo = (n: number) => (n / 1048576).toFixed(1).replace(".", ",");
  return (
    <div className="maj-progression" role="progressbar" aria-valuenow={pct ?? undefined} aria-valuemin={0} aria-valuemax={100}>
      <div className="maj-piste">
        <div className={pct === null ? "maj-remplissage indetermine" : "maj-remplissage"} style={pct === null ? undefined : { width: `${pct}%` }} />
      </div>
      <span className="doux">
        {p.fini
          ? "Installation… l'application va redémarrer."
          : pct === null
            ? `Téléchargement… ${mo(p.recu)} Mo`
            : `Téléchargement… ${pct} % (${mo(p.recu)} / ${mo(p.total ?? 0)} Mo)`}
      </span>
    </div>
  );
}

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

/** Interroge le dépôt de publication. Renvoie la version plus récente, ou null si à jour ou injoignable. */
export async function chercherMiseAJour(): Promise<Nouvelle | null> {
  try {
    const maj = await check();
    if (!maj) return null;
    return { version: maj.version, installer: (suivi) => telecharger(maj, suivi) };
  } catch {
    return null;
  }
}

/** Section des Paramètres : interrupteur, vérification et installation. */
export function MiseAJour() {
  const [active, setActive] = useState(miseAJourActive);
  const [nouvelle, setNouvelle] = useState<Nouvelle | null>(null);
  const [etat, setEtat] = useState("");
  const [progression, setProgression] = useState<Progression | null>(null);

  const basculer = (v: boolean) => {
    setActive(v);
    enregistrerMiseAJour(v);
  };

  const verifier = async () => {
    setEtat("Recherche…");
    try {
      const maj = await check();
      setNouvelle(maj ? { version: maj.version, installer: (suivi) => telecharger(maj, suivi) } : null);
      setEtat(maj ? `Version ${maj.version} disponible.` : `Version ${VERSION} : à jour.`);
    } catch (erreur) {
      setNouvelle(null);
      setEtat(`Erreur de vérification : ${String(erreur)}`);
    }
  };

  return (
    <section className="section">
      <h2>Mises à jour</h2>
      <p className="doux">
        Version installée : {VERSION}. Au lancement, l'application regarde si une version plus récente existe. Elle ne l'installe
        que si tu cliques sur « Mettre à jour ».
      </p>
      <label className="doux">
        <input type="checkbox" checked={active} onChange={(e) => basculer(e.target.checked)} /> Vérifier les mises à jour automatiquement
      </label>{" "}
      <button className="bouton discret" onClick={verifier}>
        Vérifier maintenant
      </button>{" "}
      {nouvelle && !progression && (
        <button
          className="bouton principal"
          onClick={() => {
            setProgression({ recu: 0, total: null, fini: false });
            nouvelle.installer(setProgression).catch((erreur) => {
              setProgression(null);
              setEtat(`Échec de la mise à jour : ${String(erreur)}`);
            });
          }}
        >
          Mettre à jour
        </button>
      )}
      {progression && <BarreProgression p={progression} />}
      {etat && <p className="doux">{etat}</p>}
    </section>
  );
}
