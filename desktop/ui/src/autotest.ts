// Essai de bout en bout lancé uniquement par la compilation sur GitHub
// (variable GESTION_PATHFINDER_AUTOTEST) : vérifie le pont interface ↔ cœur Rust.
import { api } from "./api";

const pause = (ms: number) => new Promise((r) => setTimeout(r, ms));

export async function autotest(): Promise<void> {
  const invoke = (window as unknown as { __TAURI__?: { core: { invoke: (c: string, a?: object) => Promise<unknown> } } })
    .__TAURI__?.core.invoke;
  if (!invoke || !(await invoke("autotest_actif"))) return;
  const lignes: string[] = [];
  const verifier = (ok: boolean, quoi: string) => lignes.push(`${ok ? "OK " : "ÉCHEC "} ${quoi}`);
  try {
    const nom = `Autotest ${Date.now()}`;
    verifier((await api.creerCampagne(nom)).ok, "création d'une campagne");
    const e1 = await api.etatApp();
    verifier(e1.entete?.campagne === nom, "campagne ouverte");
    verifier((await api.avancer(3)).ok, "avancer de 3 jours");
    const t = await api.tableauDeBord();
    verifier(t?.jour === (e1.entete?.jour ?? 0) + 3, "tableau de bord : jour avancé");
    verifier((await api.annuler()).ok, "annuler");
    verifier((await api.etatApp()).entete?.jour === e1.entete?.jour, "jour revenu après annulation");
    const saisie = {
      nom: "Sandpoint", genre: "ville", population: "1240", officielle: false, categorieOfficielle: "",
      categoriePersoNom: "Essai", categoriePersoLimite: "3", notes: "",
    };
    verifier((await api.creerLocalite(saisie)).ok, "création d'une localité");
    const pl = await api.pageLocalites();
    verifier(pl?.localites.length === 1 && pl.localites[0].population === 1240, "page Localités : fiche créée");
    const id = pl?.localites[0]?.id ?? 0;
    verifier((await api.ajustement(id, "12,5", { marchandises: 2, influence: 0, travail: 0, magie: 0 },
      { marchandises: 0, influence: 0, travail: 0, magie: 0 }, "autotest")).ok, "ajustement de la localité");
    verifier((await api.pageLocalites())?.localites[0]?.tresorerie.pa === 125, "trésorerie après ajustement");
    verifier(!(await api.mouvementPo(true, 1, id, "abc")).ok, "montant illisible refusé");
    verifier((await api.simulation("demarrer")).ok, "démarrer une simulation");
    await api.avancer(1);
    verifier((await api.etatApp()).simulation?.[1] === 1, "simulation : un jour d'avance");
    verifier((await api.simulation("abandonner")).ok, "abandonner la simulation");
    verifier((await api.etatApp()).campagnes.some((c) => c.nom === nom && c.ouverte), "campagne enregistrée");
    await pause(1500);
    const titre = document.querySelector("h1")?.textContent ?? "";
    verifier(titre.length > 0, `interface affichée (titre : « ${titre} »)`);
  } catch (e) {
    lignes.push(`ÉCHEC exception : ${String(e)}`);
  }
  const ok = lignes.every((l) => l.startsWith("OK"));
  await invoke("autotest_fin", { rapport: `${ok ? "AUTOTEST OK" : "AUTOTEST ÉCHEC"}\n${lignes.join("\n")}\n` });
}
