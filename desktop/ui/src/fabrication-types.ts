/** Entrée et sortie du calcul de fabrication (commande `fabrication_calculer`). */
export type EntreeFabrication = {
  prix_po: number;
  niveau_lanceur: number;
  conditions_manquantes: number;
  potion_ou_parchemin: boolean;
  acceleree: boolean;
  po_payes: number;
  marchandises: number;
  magie: number;
  jours_faits: number;
  jet: number | null;
};

/** Une journée de fabrication, enregistrée dans la campagne. */
export type EntreeJourFabrication = {
  cle: string;
  nom: string;
  localite: number;
  prix_po: number;
  niveau_lanceur: number;
  conditions_manquantes: number;
  potion_ou_parchemin: boolean;
  acceleree: boolean;
  po_payes: number;
  marchandises: number;
  magie: number;
  jet: number | null;
};

export type FabricationVue = {
  cle: string;
  nom: string;
  heures_requises: number;
  heures_faites: number;
  terminee: boolean;
  issue: "reussi" | "echec" | "maudit" | null;
};

export type SortieFabrication = {
  dd: number;
  heures_requises: number;
  jours_requis: number;
  jours_restants: number;
  terminee: boolean;
  fournitures_pa: number;
  paiement_pa: number;
  suffisant: boolean;
  issue: "reussi" | "echec" | "maudit" | null;
};
