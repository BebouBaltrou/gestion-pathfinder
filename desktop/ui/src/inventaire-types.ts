// Types de l'inventaire et de l'équipement : ils reprennent ceux de crates/vues/src/inventaire.rs.

export type Somme = { pc: number; texte: string };
export type PoidsVue = { grammes: number; texte: string };

export type ObjetVue = {
  id: number;
  nom: string;
  typeObjet: string;
  quantite: number;
  /** Conteneur qui le contient (la racine pour un objet posé « sur soi »). */
  parent: number | null;
  estConteneur: boolean;
  poids: PoidsVue;
  contenu: PoidsVue | null;
  capacite: PoidsVue | null;
  contenuCompte: boolean;
  valeurUnite: Somme | null;
  note: string;
  source: string;
  colle: boolean;
  equipe: string[];
  emplacementsPossibles: string[];
  proprietes: [string, string][];
};

export type ChargeVue = { nom: string; poidsLb: number; limitesLb: [number, number, number] };

export type InventaireVue = {
  racine: number;
  proprietaire: string;
  typeProprietaire: string;
  refProprietaire: string;
  poids: PoidsVue;
  valeur: Somme;
  objets: ObjetVue[];
  jeux: string[];
  charge: ChargeVue | null;
};

export type EmplacementVue = { id: string; zone: string; nom: string; capacite: number };
export type PersonnageVue = { id: number; nom: string };

export type PageInventaire = {
  inventaires: InventaireVue[];
  emplacements: EmplacementVue[];
  sansInventaire: PersonnageVue[];
};

export type ModeleObjetVue = {
  id: string;
  nom: string;
  typeObjet: string;
  valeur: Somme | null;
  poids: PoidsVue | null;
  source: string;
};

/** Commandes du moteur (serde, enum à étiquette externe) que l'interface sait envoyer. */
export type CommandeInventaire =
  | { Deplacer: { id: number; vers: number } }
  | { Retirer: { id: number } }
  | { Scinder: { id: number; quantite: number } }
  | { Fusionner: { garde: number; absorbe: number } }
  | { Equiper: { id: number; emplacement: string } }
  | { Deequiper: { id: number } }
  | { DefinirColle: { id: number; colle: boolean } }
  | { EnregistrerJeu: { racine: number; nom: string } }
  | { AppliquerJeu: { racine: number; nom: string } };

/** Noms lisibles des types d'objets du catalogue. */
export const NOMS_TYPES: Record<string, string> = {
  arme: "Arme",
  munition: "Munition",
  armure: "Armure",
  bouclier: "Bouclier",
  "supplement-armure": "Supplément d'armure",
  materiel: "Matériel",
  substance: "Substance",
  outil: "Outil",
  vetement: "Vêtement",
  nourriture: "Nourriture et boisson",
  harnachement: "Harnachement",
  "objet-magique": "Objet magique",
  anneau: "Anneau",
  sceptre: "Sceptre",
  baton: "Bâton",
  "armure-magique": "Armure magique",
  "bouclier-magique": "Bouclier magique",
  "arme-magique": "Arme magique",
  "objet-merveilleux": "Objet merveilleux",
  divers: "Divers",
};

export const nomDuType = (t: string) => NOMS_TYPES[t] ?? t;

const TYPES_MAGIQUES = new Set([
  "objet-magique",
  "anneau",
  "sceptre",
  "baton",
  "armure-magique",
  "bouclier-magique",
  "arme-magique",
  "objet-merveilleux",
]);
export const estMagique = (t: string) => TYPES_MAGIQUES.has(t);
