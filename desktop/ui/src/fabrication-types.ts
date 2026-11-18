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
