// Mises à jour : vérifie la dernière version publiée puis l'installe (module de mise à jour de Tauri).
// Désactivable dans les Paramètres ; le choix est mémorisé sur cet ordinateur.

import { useState } from "react";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";

declare const __BUILD_ANDROID__: number;
/** Numéro de la version Android installée (0 sur ordinateur). */
export const BUILD_ANDROID = __BUILD_ANDROID__;
const DEPOT = "BebouBaltrou/gestion-pathfinder";

export const VERSION = "1.2.30";
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

/**
 * Android : les versions sont publiées sur GitHub sous l'étiquette « android-N ».
 * « Mettre à jour » ouvre le téléchargement de l'APK dans le navigateur ; Android propose
 * ensuite de l'installer par-dessus (les campagnes sont conservées).
 */
async function chercherAndroid(): Promise<Nouvelle | null> {
  const r = await fetch(`https://api.github.com/repos/${DEPOT}/releases?per_page=30`);
  if (!r.ok) throw new Error(`GitHub a répondu ${r.status}`);
  const liste = (await r.json()) as { tag_name: string; assets: { name: string; browser_download_url: string }[] }[];
  let meilleure: Nouvelle | null = null;
  let max = BUILD_ANDROID;
  for (const rel of liste) {
    const m = /^android-(\d+)$/.exec(rel.tag_name);
    const apk = rel.assets.find((a) => a.name.endsWith(".apk"));
    if (!m || !apk || Number(m[1]) <= max) continue;
    max = Number(m[1]);
    meilleure = { version: `Android n°${max}`, installer: () => openUrl(apk.browser_download_url) };
  }
  return meilleure;
}

/** Interroge le dépôt de publication. Renvoie la version plus récente, ou null si à jour ou injoignable. */
export async function chercherMiseAJour(): Promise<Nouvelle | null> {
  if (BUILD_ANDROID > 0) return chercherAndroid().catch(() => null);
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
    if (BUILD_ANDROID > 0) {
      try {
        const n = await chercherAndroid();
        setNouvelle(n);
        setEtat(n ? `${n.version} disponible.` : `Android n°${BUILD_ANDROID} : à jour.`);
      } catch (erreur) {
        setNouvelle(null);
        setEtat(`Erreur de vérification : ${String(erreur)}`);
      }
      return;
    }
    try {
      const maj = await check();
      setNouvelle(
        maj
          ? {
              version: maj.version,
              installer: async () => {
                await maj.downloadAndInstall();
                await relaunch();
              },
            }
          : null,
      );
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
        Version installée : {VERSION}
        {BUILD_ANDROID > 0 && ` (Android n°${BUILD_ANDROID} : la mise à jour se télécharge, puis Android propose de l'installer)`}. Au lancement, l'application regarde si une version plus récente existe. Elle ne l'installe
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
