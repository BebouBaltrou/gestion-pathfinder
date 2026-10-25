// Liens internes des notes : « [[objet:12|Épée longue]] » ouvre l'objet dans l'inventaire,
// « [[note:3]] » ouvre une note. Les autres genres (sort, lieu, combat…) restent du texte cliquable
// sans destination pour l'instant.

import type { Page } from "./App";

export type DemandeLien = { genre: string; cible: string };

let demande: DemandeLien | null = null;

/** Mémorise la destination et change de page ; la page d'arrivée la reprend avec `prendreDemande`. */
export function ouvrirLien(genre: string, cible: string, aller: (p: Page) => void): boolean {
  if (genre === "objet") {
    demande = { genre, cible };
    aller("inventaire");
    return true;
  }
  if (genre === "note") {
    demande = { genre, cible };
    aller("notes");
    return true;
  }
  return false;
}

/** Reprend (une seule fois) la demande qui vise ce genre de destination. */
export function prendreDemande(genre: string): string | null {
  if (demande?.genre !== genre) return null;
  const cible = demande.cible;
  demande = null;
  return cible;
}
