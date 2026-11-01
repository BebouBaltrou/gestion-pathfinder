import { sansAccents } from "./texte";
// Uniquement pour l'aperçu sans Tauri (MOCK=1) : un inventaire d'exemple qui réagit aux commandes,
// avec plusieurs centaines d'objets pour juger de la fluidité. Retiré de la version de l'application.

import type {
  CommandeInventaire,
  EmplacementVue,
  InventaireVue,
  ModeleObjetVue,
  ObjetVue,
  PageInventaire,
  PoidsVue,
} from "./inventaire-types";

type Retour = { ok: boolean; alerte: boolean; message: string };
const ok = (message: string): Retour => ({ ok: true, alerte: false, message });
const non = (message: string): Retour => ({ ok: false, alerte: false, message });

const EMPLACEMENTS: EmplacementVue[] = [
  ["tete", "Tête", "Casque, couvre-chef", 1],
  ["visage", "Visage", "Lunettes, masque", 1],
  ["cou", "Cou", "Amulette, collier", 1],
  ["epaules", "Épaules", "Cape, manteau", 1],
  ["torse", "Torse", "Veste, chemise", 1],
  ["corps", "Corps", "Robe, armure", 1],
  ["taille", "Taille", "Ceinture", 1],
  ["poignets", "Poignets", "Bracelets, brassards", 1],
  ["mains", "Mains", "Gants", 1],
  ["anneau", "Doigts", "Anneau", 2],
  ["pieds", "Pieds", "Bottes", 1],
  ["main_directrice", "Mains tenues", "Main directrice", 1],
  ["main_non_directrice", "Mains tenues", "Main non directrice", 1],
  ["sans_emplacement", "Sans emplacement", "Objets sans emplacement", 0],
].map(([id, zone, nom, capacite]) => ({ id, zone, nom, capacite }) as EmplacementVue);

const poids = (grammes: number): PoidsVue => ({
  grammes,
  texte:
    grammes < 1000
      ? `${grammes} g`
      : `${(grammes / 1000).toLocaleString("fr-FR", { maximumFractionDigits: 3 })} kg`,
});

type Base = { nom: string; type: string; g: number; pc: number; slots?: string[]; mains?: number };
const CATALOGUE: Base[] = [
  { nom: "Épée longue", type: "arme", g: 2000, pc: 1500, slots: ["main_directrice", "main_non_directrice"] },
  { nom: "Épée à deux mains", type: "arme", g: 4000, pc: 5000, slots: ["main_directrice", "main_non_directrice"], mains: 2 },
  { nom: "Dague", type: "arme", g: 500, pc: 200, slots: ["main_directrice", "main_non_directrice"] },
  { nom: "Arc long", type: "arme", g: 1500, pc: 7500, slots: ["main_directrice", "main_non_directrice"], mains: 2 },
  { nom: "Flèches (20)", type: "munition", g: 1500, pc: 100 },
  { nom: "Armure de cuir", type: "armure", g: 7500, pc: 1000, slots: ["corps"] },
  { nom: "Cotte de mailles", type: "armure", g: 20000, pc: 15000, slots: ["corps"] },
  { nom: "Targe", type: "bouclier", g: 2500, pc: 500, slots: ["main_non_directrice"] },
  { nom: "Torche", type: "materiel", g: 500, pc: 1 },
  { nom: "Corde en chanvre, 15 m", type: "materiel", g: 5000, pc: 100 },
  { nom: "Rations de survie (par jour)", type: "nourriture", g: 500, pc: 50 },
  { nom: "Outils de cambrioleur", type: "outil", g: 500, pc: 3000 },
  { nom: "Habit de voyageur", type: "vetement", g: 2500, pc: 100 },
  { nom: "Anneau de protection (+1)", type: "objet-magique", g: 0, pc: 200000, slots: ["anneau"] },
  { nom: "Cape de résistance (+2)", type: "objet-magique", g: 0, pc: 400000, slots: ["epaules"] },
  { nom: "Bottes elfiques", type: "objet-magique", g: 0, pc: 250000, slots: ["pieds"] },
  { nom: "Bille de force", type: "objet-merveilleux", g: 0, pc: 300000, slots: ["sans_emplacement"] },
  { nom: "Potion de soins légers", type: "materiel", g: 0, pc: 5000 },
  { nom: "Gemme taillée", type: "materiel", g: 0, pc: 10000 },
];

let prochainId = 1;
const propre = new Map<number, number>(); // poids propre d'un exemplaire, en grammes (avec la quantité)
const conteneurs = new Map<number, { capacite: number | null; compte: boolean }>();
let inventaires: InventaireVue[] = [];
let sansInventaire = [{ id: 3, nom: "Kyra" }];
const jeux = new Map<number, Map<string, [number, string[]][]>>();

function nouvelObjet(racine: InventaireVue, base: Base, quantite: number, parent: number): ObjetVue {
  const id = prochainId++;
  propre.set(id, base.g * quantite);
  const o: ObjetVue = {
    id,
    nom: base.nom,
    typeObjet: base.type,
    quantite,
    parent,
    estConteneur: false,
    poids: poids(0),
    contenu: null,
    capacite: null,
    contenuCompte: true,
    valeurUnite: { pc: base.pc, texte: `${base.pc} pc` },
    note: "",
    source: "Exemple",
    colle: false,
    equipe: [],
    emplacementsPossibles: base.slots ?? [],
    proprietes: base.mains ? [["mains", String(base.mains)]] : [],
  };
  racine.objets.push(o);
  return o;
}

function nouveauConteneur(racine: InventaireVue, nom: string, g: number, parent: number, capacite: number | null, compte = true) {
  const o = nouvelObjet(racine, { nom, type: "materiel", g, pc: 100 }, 1, parent);
  o.estConteneur = true;
  conteneurs.set(o.id, { capacite, compte });
  return o;
}

/** Recalcule les poids : propre + contenu qui compte, du plus profond vers la racine. */
function recalculer() {
  for (const inv of inventaires) {
    const enfants = new Map<number, ObjetVue[]>();
    for (const o of inv.objets) enfants.set(o.parent ?? -1, [...(enfants.get(o.parent ?? -1) ?? []), o]);
    const total = (o: ObjetVue): number => {
      const c = conteneurs.get(o.id);
      const interne = (enfants.get(o.id) ?? []).reduce((s, e) => s + total(e), 0);
      if (c) {
        o.contenu = poids(interne);
        o.capacite = c.capacite === null ? null : poids(c.capacite);
        o.contenuCompte = c.compte;
      }
      const g = (propre.get(o.id) ?? 0) + (c && !c.compte ? 0 : interne);
      o.poids = poids(g);
      return g;
    };
    const racines = enfants.get(inv.racine) ?? [];
    const g = racines.reduce((s, o) => s + total(o), 0);
    inv.poids = poids(g);
    inv.valeur = {
      pc: inv.objets.reduce((s, o) => s + (o.valeurUnite?.pc ?? 0) * o.quantite, 0),
      texte: "",
    };
    inv.valeur.texte = `${Math.floor(inv.valeur.pc / 100).toLocaleString("fr-FR")} po`;
    if (inv.charge) {
      const lb = g / 500;
      const [l, m, h] = inv.charge.limitesLb;
      inv.charge = { ...inv.charge, poidsLb: lb, nom: lb <= l ? "légère" : lb <= m ? "intermédiaire" : lb <= h ? "lourde" : "surcharge" };
    }
  }
}

function generer() {
  const lyra: InventaireVue = {
    racine: prochainId++,
    proprietaire: "Lyra",
    typeProprietaire: "personnage",
    refProprietaire: "PJ-1",
    poids: poids(0),
    valeur: { pc: 0, texte: "" },
    objets: [],
    jeux: [],
    charge: { nom: "légère", poidsLb: 0, limitesLb: [33, 66, 100] },
  };
  inventaires = [lyra];
  const sac = nouveauConteneur(lyra, "Sac à dos (vide)", 1000, lyra.racine, 25000);
  const etui = nouveauConteneur(lyra, "Étui à parchemins", 250, sac.id, 2000);
  const sacSansFond = nouveauConteneur(lyra, "Sac sans fond (1er modèle)", 1000, lyra.racine, 150000, false);
  nouvelObjet(lyra, CATALOGUE[5], 1, lyra.racine);
  nouvelObjet(lyra, CATALOGUE[7], 1, lyra.racine);
  nouvelObjet(lyra, CATALOGUE[0], 1, lyra.racine);
  nouvelObjet(lyra, CATALOGUE[13], 1, lyra.racine);
  nouvelObjet(lyra, CATALOGUE[14], 1, lyra.racine);
  nouvelObjet(lyra, CATALOGUE[8], 3, sac.id);
  nouvelObjet(lyra, CATALOGUE[10], 5, sac.id);
  nouvelObjet(lyra, CATALOGUE[4], 1, etui.id);
  // Plusieurs centaines d'objets pour juger de la fluidité (liste virtualisée, glisser-déposer).
  for (let i = 0; i < 800; i++) {
    const base = CATALOGUE[(i * 7 + 3) % CATALOGUE.length];
    const o = nouvelObjet(lyra, { ...base, nom: `${base.nom} n°${i + 1}` }, 1 + (i % 4), i % 3 === 0 ? sacSansFond.id : lyra.racine);
    o.emplacementsPossibles = base.slots ?? [];
  }
  const valeros: InventaireVue = {
    racine: prochainId++,
    proprietaire: "Valeros",
    typeProprietaire: "personnage",
    refProprietaire: "PJ-2",
    poids: poids(0),
    valeur: { pc: 0, texte: "" },
    objets: [],
    jeux: [],
    charge: { nom: "légère", poidsLb: 0, limitesLb: [76, 153, 230] },
  };
  inventaires.push(valeros);
  nouvelObjet(valeros, CATALOGUE[6], 1, valeros.racine);
  nouvelObjet(valeros, CATALOGUE[1], 1, valeros.racine);
  recalculer();
}
let pret = false;

function occupation(inv: InventaireVue, slot: string) {
  return inv.objets.filter((o) => o.equipe.includes(slot)).length;
}

function commande(c: CommandeInventaire): Retour {
  const trouve = (id: number) => {
    for (const inv of inventaires) {
      const o = inv.objets.find((x) => x.id === id);
      if (o) return { inv, o };
    }
    return null;
  };
  if ("Deplacer" in c) {
    const t = trouve(c.Deplacer.id);
    const vers = c.Deplacer.vers;
    if (!t) return non("Objet inconnu");
    if (t.o.estConteneur) {
      let p: number | null = vers;
      while (p !== null && p !== t.inv.racine) {
        if (p === t.o.id) return non("Un conteneur ne peut pas entrer dans son contenu");
        p = t.inv.objets.find((x) => x.id === p)?.parent ?? null;
      }
    }
    const cible = conteneurs.get(vers);
    if (cible?.capacite != null) {
      const dedans = t.inv.objets.filter((x) => x.parent === vers).reduce((s, x) => s + x.poids.grammes, 0);
      if (dedans + t.o.poids.grammes > cible.capacite) return non("Capacité du conteneur dépassée");
    }
    t.o.parent = vers;
    recalculer();
    return ok(`${t.o.nom} déplacé`);
  }
  if ("Retirer" in c) {
    const t = trouve(c.Retirer.id);
    if (!t) return non("Objet inconnu");
    if (t.o.colle && t.o.equipe.length) return non("Cet objet est collé : il ne se retire pas");
    if (t.inv.objets.some((x) => x.parent === t.o.id)) return non("Le conteneur n'est pas vide");
    t.inv.objets = t.inv.objets.filter((x) => x.id !== t.o.id);
    recalculer();
    return ok(`${t.o.nom} jeté`);
  }
  if ("Scinder" in c) {
    const t = trouve(c.Scinder.id);
    const q = c.Scinder.quantite;
    if (!t || q <= 0 || q >= t.o.quantite) return non("Quantité invalide");
    const unite = (propre.get(t.o.id) ?? 0) / t.o.quantite;
    propre.set(t.o.id, unite * (t.o.quantite - q));
    t.o.quantite -= q;
    const neuf = { ...t.o, id: prochainId++, quantite: q, equipe: [] as string[] };
    propre.set(neuf.id, unite * q);
    t.inv.objets.push(neuf);
    recalculer();
    return ok("Pile scindée");
  }
  if ("Fusionner" in c) {
    const a = trouve(c.Fusionner.garde);
    const b = trouve(c.Fusionner.absorbe);
    if (!a || !b || a.o.nom !== b.o.nom || a.o.parent !== b.o.parent) return non("Ces objets ne se fusionnent pas");
    propre.set(a.o.id, (propre.get(a.o.id) ?? 0) + (propre.get(b.o.id) ?? 0));
    a.o.quantite += b.o.quantite;
    a.inv.objets = a.inv.objets.filter((x) => x.id !== b.o.id);
    recalculer();
    return ok("Piles réunies");
  }
  if ("Equiper" in c) {
    const t = trouve(c.Equiper.id);
    const slot = c.Equiper.emplacement;
    if (!t) return non("Objet inconnu");
    if (t.o.equipe.length) return non("Déjà équipé");
    if (!t.o.emplacementsPossibles.includes(slot)) return non("Cet objet ne va pas dans cet emplacement");
    const zone = EMPLACEMENTS.find((e) => e.id === slot)!.zone;
    const deux = t.o.proprietes.some(([k, v]) => k === "mains" && v === "2");
    const voulus = deux ? EMPLACEMENTS.filter((e) => e.zone === zone).map((e) => e.id) : [slot];
    for (const s of voulus) {
      const cap = EMPLACEMENTS.find((e) => e.id === s)!.capacite;
      if (cap > 0 && occupation(t.inv, s) >= cap) return non(`L'emplacement ${s} est occupé`);
    }
    t.o.equipe = voulus;
    return ok(`${t.o.nom} équipé`);
  }
  if ("Deequiper" in c) {
    const t = trouve(c.Deequiper.id);
    if (!t) return non("Objet inconnu");
    if (t.o.colle) return non("Cet objet est collé : il ne se retire pas");
    t.o.equipe = [];
    return ok(`${t.o.nom} déséquipé`);
  }
  if ("DefinirColle" in c) {
    const t = trouve(c.DefinirColle.id);
    if (!t) return non("Objet inconnu");
    t.o.colle = c.DefinirColle.colle;
    return ok(t.o.colle ? "Objet maudit : collé" : "Objet décollé");
  }
  if ("EnregistrerJeu" in c) {
    const inv = inventaires.find((i) => i.racine === c.EnregistrerJeu.racine);
    if (!inv) return non("Inventaire inconnu");
    const m = jeux.get(inv.racine) ?? new Map();
    m.set(c.EnregistrerJeu.nom, inv.objets.filter((o) => o.equipe.length).map((o) => [o.id, [...o.equipe]]));
    jeux.set(inv.racine, m);
    inv.jeux = [...m.keys()];
    return ok(`Jeu « ${c.EnregistrerJeu.nom} » enregistré`);
  }
  const inv = inventaires.find((i) => i.racine === c.AppliquerJeu.racine);
  const jeu = jeux.get(c.AppliquerJeu.racine)?.get(c.AppliquerJeu.nom);
  if (!inv || !jeu) return non("Jeu d'équipement inconnu");
  for (const o of inv.objets) if (!o.colle) o.equipe = [];
  for (const [id, slots] of jeu) {
    const o = inv.objets.find((x) => x.id === id);
    if (o && !o.equipe.length) o.equipe = slots;
  }
  return ok(`Jeu « ${c.AppliquerJeu.nom} » appliqué`);
}

function modeles(): ModeleObjetVue[] {
  return CATALOGUE.map((b, i) => ({
    id: `exemple:${i}`,
    nom: b.nom,
    typeObjet: b.type,
    valeur: { pc: b.pc, texte: `${b.pc} pc` },
    poids: poids(b.g),
    source: "Exemple",
  }));
}


/** Réponses aux commandes de l'inventaire ; `undefined` : ce n'est pas une commande d'inventaire. */
export function simulerInventaire(cmd: string, args?: Record<string, unknown>): Promise<unknown> | undefined {
  if (!pret && /^(page_inventaire|catalogue_objets|types_objets|inventaire_commande|creer_inventaire_personnage|ajouter_)/.test(cmd)) {
    pret = true;
    generer();
  }
  switch (cmd) {
    case "page_inventaire": {
      const page: PageInventaire = { inventaires, emplacements: EMPLACEMENTS, sansInventaire };
      return Promise.resolve(structuredClone(page));
    }
    case "catalogue_objets": {
      const mots = sansAccents(String(args?.requete ?? "")).split(/\s+/).filter(Boolean);
      const type = (args?.typeObjet as string | null) ?? null;
      return Promise.resolve(
        modeles().filter((m) => (!type || m.typeObjet === type) && mots.every((w) => sansAccents(m.nom).includes(w))),
      );
    }
    case "types_objets":
      return Promise.resolve([...new Set(CATALOGUE.map((b) => b.type))].sort());
    case "inventaire_commande":
      return Promise.resolve(commande(args?.commande as CommandeInventaire));
    case "creer_inventaire_personnage": {
      const pj = sansInventaire.find((p) => p.id === args?.pj);
      if (!pj) return Promise.resolve(non("Personnage inconnu"));
      sansInventaire = sansInventaire.filter((p) => p.id !== pj.id);
      inventaires.push({
        racine: prochainId++,
        proprietaire: pj.nom,
        typeProprietaire: "personnage",
        refProprietaire: `PJ-${pj.id}`,
        poids: poids(0),
        valeur: { pc: 0, texte: "0 po" },
        objets: [],
        jeux: [],
        charge: null,
      });
      return Promise.resolve(ok(`Inventaire de ${pj.nom} créé`));
    }
    case "ajouter_objet_catalogue":
    case "ajouter_conteneur":
    case "ajouter_objet_libre": {
      const dans = Number(args?.dans);
      const inv = inventaires.find((i) => i.racine === dans || i.objets.some((o) => o.id === dans));
      if (!inv) return Promise.resolve(non("Conteneur inconnu"));
      if (cmd === "ajouter_objet_libre") {
        const base = { nom: String(args?.nom), type: String(args?.typeObjet || "divers"), g: Number(args?.poidsGrammes ?? 0), pc: 0 };
        nouvelObjet(inv, base, Number(args?.quantite ?? 1), dans);
      } else {
        const base = CATALOGUE[Number(String(args?.modele).split(":")[1])] ?? CATALOGUE[8];
        if (cmd === "ajouter_conteneur") {
          nouveauConteneur(inv, base.nom, base.g, dans, (args?.capaciteGrammes as number | null) ?? null, !args?.extradimensionnel);
        } else {
          nouvelObjet(inv, base, Number(args?.quantite ?? 1), dans);
        }
      }
      recalculer();
      return Promise.resolve(ok("Objet ajouté"));
    }
    default:
      return undefined;
  }
}
