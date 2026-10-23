// Essai de bout en bout lancé uniquement par la compilation sur GitHub
// (variable GESTION_PATHFINDER_AUTOTEST) : vérifie le pont interface ↔ cœur Rust.
import { api } from "./api";
import type { FicheDonnees } from "./fiche-types";

const FICHE_EXEMPLE: FicheDonnees = {
  nom: "Lyra",
  race: "elfe",
  bonus_racial_libre: null,
  scores_base: { For: 8, Dex: 14, Con: 12, Int: 10, Sag: 10, Cha: 16 },
  dommages: {},
  drain: {},
  augmentations: [],
  niveaux: [
    {
      classe: "ensorceleur",
      de_vie_obtenu: 6,
      rangs: { bluff: 1, "knowledge:arcana": 1, spellcraft: 1 },
      predilection: "PointDeVie",
      dons: ["Toughness"],
    },
  ],
  classes_de_predilection: ["ensorceleur"],
  lignage: "draconic",
  connaissance_de_lignage: null,
  effets: [],
  sorts_connus: ["Acid Splash", "Detect Magic", "Light", "Mage Hand", "Magic Missile", "Mage Armor"],
  equipement: { armure: null, bouclier: null, armes: [], objets: [], charge_declaree: "Legere" },
  etats: [],
  temporaires: [],
};

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
    verifier((await api.simulation("demarrer")).ok, "démarrer une simulation");
    await api.avancer(1);
    verifier((await api.etatApp()).simulation?.[1] === 1, "simulation : un jour d'avance");
    verifier((await api.simulation("abandonner")).ok, "abandonner la simulation");
    verifier((await api.etatApp()).campagnes.some((c) => c.nom === nom && c.ouverte), "campagne enregistrée");
    // ---- personnages et fiches
    verifier((await api.creerPersonnage("Lyra", 100)).ok, "création d'un personnage");
    const pj = (await api.listePersonnages())[0];
    verifier(pj?.nom === "Lyra" && pj.fiche === null, "personnage sans fiche");
    verifier((await api.creerFiche(pj.id, { ...FICHE_EXEMPLE, race: "inconnue" })).ok === false, "fiche invalide refusée");
    verifier((await api.creerFiche(pj.id, FICHE_EXEMPLE)).ok, "création de la fiche");
    const f1 = await api.fichePersonnage(pj.id);
    verifier(f1?.ca.total === 13 && f1.pv.total === 7, `fiche calculée (CA ${f1?.ca.total}, PV ${f1?.pv.total})`);
    verifier((await api.listePersonnages())[0].prestige === 4, "Prestige calculé (niveau + CHA)");
    verifier((await api.evenementFiche(pj.id, { Equiper: { emplacement: "armure", objet: "Chainmail" } })).ok, "armure équipée");
    verifier((await api.fichePersonnage(pj.id))?.ca.total === 18, "CA avec cotte de mailles (DEX plafonné)");
    verifier((await api.evenementFiche(pj.id, { EtatAjoute: "shaken" })).ok, "état secoué");
    verifier((await api.fichePersonnage(pj.id))?.vigueur.total === -2, "vigueur −2 (secoué)");
    verifier((await api.evenementFiche(pj.id, { EtatAjoute: "bidon" })).ok === false, "état inconnu refusé");
    verifier((await api.evenementFiche(pj.id, { XpGagnes: { nombre: 2000, motif: "essai" } })).ok, "XP gagnés");
    verifier(((await api.listePersonnages())[0].fiche?.niveauxEnAttente ?? 0) === 1, "niveau en attente");
    const ap = await api.apercuMontee(pj.id, "ensorceleur");
    verifier(ap !== null && ap.niveauDePersonnage === 2, "aperçu de montée de niveau");
    verifier((await api.reglesCampagne()).length > 5, "registre des règles");
    verifier((await api.changerRegle("pj.vitesse", "lente")).ok, "règle modifiée");
    verifier(((await api.listePersonnages())[0].fiche?.niveauxEnAttente ?? 9) === 0, "vitesse lente : plus de niveau en attente");
    verifier((await api.catalogueCreation()).dons.length === 176, "catalogue de création");
    await pause(1500);
    const titre = document.querySelector("h1")?.textContent ?? "";
    verifier(titre.length > 0, `interface affichée (titre : « ${titre} »)`);
  } catch (e) {
    lignes.push(`ÉCHEC exception : ${String(e)}`);
  }
  const ok = lignes.every((l) => l.startsWith("OK"));
  await invoke("autotest_fin", { rapport: `${ok ? "AUTOTEST OK" : "AUTOTEST ÉCHEC"}\n${lignes.join("\n")}\n` });
}
