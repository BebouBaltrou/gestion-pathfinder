// Recharge d'un bâton (Core p. 491) : le joueur choisit le bâton ; le cœur calcule le coût
// (emplacement du niveau du sort le plus haut du bâton) et l'enregistre.

import { useState } from "react";
import { api } from "../../api";
import type { ObjetVue } from "../../aventure-types";
import { Dialogue } from "../../composants/jeu";

export function DialogueRechargeBaton({
  pjNom,
  pjId,
  batons,
  agir,
  onFermer,
}: {
  pjNom: string;
  pjId: number;
  batons: ObjetVue[];
  agir: (p: Promise<{ ok: boolean; alerte: boolean; message: string }>) => Promise<unknown>;
  onFermer: () => void;
}) {
  const [choisi, setChoisi] = useState<number | null>(batons[0]?.id ?? null);

  const recharger = async () => {
    if (choisi === null) return;
    const r = await agir(api.aventureRechargerBaton({ pj: pjId, objet: choisi }));
    if (r) onFermer();
  };

  return (
    <Dialogue
      titre={`Recharger un bâton — ${pjNom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button className="bouton principal" disabled={choisi === null} onClick={recharger}>Recharger</button>
        </>
      }
    >
      <p className="aide">Une seule recharge par jour. Le coût (un emplacement de sort) est calculé et retiré automatiquement.</p>
      {batons.map((b) => (
        <label key={b.id}>
          <input type="radio" name="baton" checked={choisi === b.id} onChange={() => setChoisi(b.id)} />{" "}
          {b.nom} · {b.charges ? `${b.charges[0]}/${b.charges[1]} charges` : ""}
        </label>
      ))}
    </Dialogue>
  );
}
