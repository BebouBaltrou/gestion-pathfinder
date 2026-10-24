// Essai de bout en bout lancé uniquement par la compilation sur GitHub
// (variable GESTION_PATHFINDER_AUTOTEST) : vérifie le pont interface ↔ cœur Rust.
import { api } from "./api";
import { act, ev } from "./evenements-jeu";
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
    verifier(
      (await api.creerBatiment({
        localite: id, modele: null, nom: "Auberge", classement: "auberge", acquisition: "plan",
        facteur: 1, seulementManquants: false, prix: "", payeur: null,
      })).ok,
      "création d'un bâtiment vide",
    );
    const pb = await api.pageBatiments();
    const idb = pb?.groupes[0]?.batiments[0]?.id ?? 0;
    verifier(idb > 0 && (pb?.salles.length ?? 0) > 0, "page Bâtiments : liste et salles du livre");
    const fb = await api.ficheBatiment(idb);
    verifier(fb?.etages.length === 1, "fiche du bâtiment : un rez-de-chaussée");
    verifier(
      (await api.actionBatiment({ type: "ajouterSalle", batiment: idb, etage: fb?.etages[0]?.id ?? 0,
        reference: pb?.salles[0]?.id ?? "", deja: true })).ok,
      "ajout d'une salle déjà construite",
    );
    verifier((await api.ficheBatiment(idb))?.etages[0]?.salles[0]?.statut === "construite", "salle construite");
    verifier(
      (await api.creerOrganisation({
        localite: id, modele: null, nom: "Guilde", classement: "guilde", acquisition: "plan",
        facteur: 1, seulementManquants: false, prix: "", payeur: null,
      })).ok,
      "création d'une organisation vide",
    );
    const po = await api.pageOrganisations();
    const ido = po?.groupes[0]?.organisations[0]?.id ?? 0;
    verifier(ido > 0 && (po?.equipes.length ?? 0) > 0, "page Organisations : liste et équipes du livre");
    verifier(
      (await api.actionOrganisation({ type: "ajouterEquipe", organisation: ido, reference: po?.equipes[0]?.id ?? "", deja: true })).ok,
      "ajout d'une équipe déjà recrutée",
    );
    verifier((await api.ficheOrganisation(ido))?.equipes[0]?.statut === "recrutee", "équipe recrutée");
    verifier(
      (await api.actionGestionnaire({ type: "embaucher", reference: null, nom: "Ameiko", salaire: "3", localite: id })).ok,
      "embauche d'un gestionnaire",
    );
    verifier((await api.pageGestionnaires())?.gestionnaires.length === 1, "page Gestionnaires : liste");
    const vr = await api.vueRecettes({ repartition: [], jets: [], conserves: { marchandises: 0, influence: 0, travail: 0, magie: 0 }, payeur: null });
    verifier(vr?.localite === id && (vr?.joursEnAttente ?? -1) >= 0, "page Recettes : calcul de la localité");
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
    verifier((await api.accorderNiveauDeDepart(pj.id, 5)).ok && ((await api.listePersonnages())[0].fiche?.niveauxEnAttente ?? 0) === 4, "niveau de départ 5 : quatre niveaux à monter");
    verifier((await api.annuler()).ok && ((await api.listePersonnages())[0].fiche?.niveauxEnAttente ?? 0) === 1, "niveau de départ annulé");
    verifier((await api.accorderNiveauDeDepart(pj.id, 25)).ok === false, "niveau de départ hors limites refusé");
    const ap = await api.apercuMontee(pj.id, "ensorceleur");
    verifier(ap !== null && ap.niveauDePersonnage === 2, "aperçu de montée de niveau");
    verifier((await api.reglesCampagne()).length > 5, "registre des règles");
    verifier((await api.changerRegle("pj.vitesse", "lente")).ok, "règle modifiée");
    verifier(((await api.listePersonnages())[0].fiche?.niveauxEnAttente ?? 9) === 0, "vitesse lente : plus de niveau en attente");
    verifier((await api.catalogueCreation()).dons.length === 176, "catalogue de création");
    // ---- aventure et combat
    verifier((await api.aventureEvenement(ev.entree(pj.id))).ok === false, "aventure : rien avant le démarrage");
    verifier((await api.aventureDemarrer(8, 0, false, false)).ok, "aventure démarrée");
    verifier((await api.aventureEvenement(ev.entree(pj.id))).ok, "le personnage rejoint l'aventure");
    verifier((await api.aventureEvenement(ev.pas("heure1"))).ok, "une heure passe");
    const a1 = await api.aventureVue();
    verifier(a1.actif && a1.horloge?.heure === 9 && a1.personnages.length === 1, `aventure : 09:00, un personnage (${a1.horloge?.texte})`);
    verifier(a1.personnages[0].caster && a1.personnages[0].sortsConnus.some((x) => x.nomEn === "Mage Armor"), "sorts connus du personnage");
    const ca0 = (await api.fichePersonnage(pj.id))?.ca.total ?? 0;
    verifier(
      (
        await api.aventureEvenement(
          ev.effet({ pj: pj.id, nom: "Bouclier test", source: "essai", duree: { minutes: 10 }, effets: [{ cible: "ca", type_bonus: "bouclier", valeur: 4, source: "essai" }] }),
        )
      ).ok,
      "effet temporaire appliqué",
    );
    verifier((await api.fichePersonnage(pj.id))?.ca.total === ca0 + 4, `CA avec l'effet (${ca0} + 4)`);
    verifier((await api.aventureEvenement(ev.pas("minutes10"))).ok, "dix minutes passent");
    const a2 = await api.aventureVue();
    verifier(a2.alertes.length === 1 && a2.personnages[0].effets.length === 0, "l'effet a expiré : alerte");
    verifier((await api.fichePersonnage(pj.id))?.ca.total === ca0, "CA revenue après l'expiration");
    verifier((await api.aventureEvenement(ev.alerteConfirmee(a2.alertes[0].id))).ok, "alerte confirmée");
    // combat
    verifier((await api.combatDemarrer([pj.id])).ok, "combat commencé");
    verifier((await api.etatApp()).combatEnCours === true, "campagne verrouillée pendant le combat");
    verifier((await api.avancer(1)).ok === false, "pas de jour d'intermède pendant un combat");
    verifier((await api.combatAction(act.initiative(pj.id, 12))).ok, "initiative saisie");
    verifier((await api.combatAction(act.demarrer(null))).ok, "combat démarré");
    const ap2 = await api.combatApercuSort(pj.id, "Mage Armor");
    verifier(ap2.niveauEmplacement === 1 && ap2.dd === 14 && ap2.dureeSuggeree !== null, `aperçu du sort (DD ${ap2.dd}, emplacement ${ap2.niveauEmplacement})`);
    const sansMunition = await api.combatAction(
      act.attaquer({ pj: pj.id, typeAction: "simple", libelle: "Tir", tirs: 1, munition: null, aDistanceAvecMunitions: true }),
    );
    verifier(sansMunition.ok === false, "attaque à distance refusée sans munition");
    verifier((await api.combatAction(act.lancer({ pj: pj.id, sort: "Mage Armor", cibles: [pj.id], echecArcaniqueD100: 90 }))).ok, "sort lancé sur soi");
    const c1 = await api.combatVue();
    verifier((c1?.personnages[0].effets.length ?? 0) === 1 && c1?.personnages[0].sortsLances === 1, "effet du sort créé dans le combat");
    verifier((await api.combatAction(act.aventure(ev.degats(pj.id, 3, true)))).ok, "dégâts subis en combat");
    verifier((await api.combatAction(act.tourSuivant())).ok, "tour suivant");
    verifier((await api.combatAnnuler()).ok, "annulation dans le combat");
    verifier((await api.combatAction(act.tourSuivant())).ok, "tour suivant (après annulation)");
    const fin = await api.combatApercuFin();
    verifier(fin.roundsJoues === 1 && fin.personnages.length === 1, "aperçu de fin de combat");
    verifier((await api.combatAction(act.degatsInfliges(pj.id, 11))).ok, "dégâts infligés notés à la main");
    const calcXp = await api.combatXp(["1", "1/2"], 2);
    verifier(calcXp.total === 600 && calcXp.parPersonnage === 300, `XP d'après les FP (${calcXp.total} / ${calcXp.parPersonnage})`);
    verifier((await api.combatXp(["99"], 1).then(() => false, () => true)), "FP inconnu refusé");
    verifier((await api.catalogueRegles()).some((r) => r.id === "degats_massifs" && r.source.includes("189")), "registre des règles optionnelles");
    const xpAvant = (await api.listePersonnages())[0].fiche?.xp ?? 0;
    const poAvant = (await api.listePersonnages())[0].compte.pa;
    const potion = {
      nom: "Potion de soins légers",
      genre: "potion" as const,
      sort: "Cure Light Wounds",
      niveau_du_sort: 1,
      niveau_de_lanceur: 1,
      arcanique: false,
      quantite: 2,
      charges: null,
      par_jour: null,
      notes: "",
    };
    verifier(
      (
        await api.combatTerminer({ genre: "tout" }, [], {
          xp: [[pj.id, calcXp.parPersonnage]],
          motif: "essai",
          or: [[pj.id, 5]],
          objets: [[pj.id, potion]],
        })
      ).ok,
      "combat validé avec XP, or et objet",
    );
    verifier((await api.etatApp()).combatEnCours === false, "campagne déverrouillée");
    const fiche1 = (await api.listePersonnages())[0];
    verifier((fiche1.fiche?.xp ?? 0) === xpAvant + 300, "XP reportés sur la fiche");
    verifier(fiche1.compte.pa === poAvant + 50, "or ajouté au compte du personnage");
    verifier((await api.aventureVue()).personnages[0].inventaire.some((o) => o.nom === "Potion de soins légers" && o.quantite === 2), "objet ajouté à l'inventaire");
    const histo = await api.combatHistorique();
    verifier(histo.combats.length === 1 && histo.combats[0].rounds === 1 && histo.statistiques[0].degatsInfliges === 11, "historique et statistiques du combat");
    const expo = await api.combatExporter(1);
    verifier(expo.length === 2 && expo[0].endsWith(".md") && expo[1].endsWith(".json"), "export du combat (Markdown et JSON)");
    // une seule annulation défait le combat et toutes ses récompenses
    verifier((await api.annuler()).ok, "annuler le combat validé");
    verifier((await api.listePersonnages())[0].fiche?.xp === xpAvant && (await api.combatHistorique()).combats.length === 0, "tout est défait (XP, historique)");
    verifier((await api.retablir()).ok, "rétablir le combat validé");
    const a3 = await api.aventureVue();
    verifier(a3.personnages[0].effets.length === 1 && a3.personnages[0].pv === a3.personnages[0].pvMax - 3, "état d'aventure mis à jour par le combat");
    verifier((await api.aventureEvenement(ev.repos([pj.id], 8, false, false))).ok, "repos de 8 heures");
    verifier((await api.avancer(1)).ok, "un jour d'intermède pendant l'aventure");
    verifier((await api.annuler()).ok, "annuler");
    verifier((await api.aventureTerminer()).ok, "aventure terminée");
    await pause(1500);
    const titre = document.querySelector("h1")?.textContent ?? "";
    verifier(titre.length > 0, `interface affichée (titre : « ${titre} »)`);
  } catch (e) {
    lignes.push(`ÉCHEC exception : ${String(e)}`);
  }
  const ok = lignes.every((l) => l.startsWith("OK"));
  await invoke("autotest_fin", { rapport: `${ok ? "AUTOTEST OK" : "AUTOTEST ÉCHEC"}\n${lignes.join("\n")}\n` });
}
