// Mises à jour : vérifie la dernière version publiée puis l'installe (module de mise à jour de Tauri).
// Désactivable dans les Paramètres ; le choix est mémorisé sur cet ordinateur.

import { useState } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export const VERSION = "1.2.12";
const CLE = "gestion-pathfinder.maj-auto";

export type Nouvelle = { version: string; installer: () => Promise<void> };

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
    return {
      version: maj.version,
      installer: async () => {
        await maj.downloadAndInstall();
        await relaunch();
      },
    };
  } catch {
    return null;
  }
}

/** Section des Paramètres : interrupteur, vérification et installation. */
export function MiseAJour() {
  const [active, setActive] = useState(miseAJourActive);
  const [nouvelle, setNouvelle] = useState<Nouvelle | null>(null);
  const [etat, setEtat] = useState("");

  const basculer = (v: boolean) => {
    setActive(v);
    enregistrerMiseAJour(v);
  };

  const verifier = async () => {
    setEtat("Recherche…");
    const n = await chercherMiseAJour();
    setNouvelle(n);
    setEtat(n ? `Version ${n.version} disponible.` : `Version ${VERSION} : à jour (ou vérification impossible sans connexion).`);
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
      {nouvelle && (
        <button className="bouton principal" onClick={() => nouvelle.installer()}>
          Mettre à jour
        </button>
      )}
      {etat && <p className="doux">{etat}</p>}
    </section>
  );
}
