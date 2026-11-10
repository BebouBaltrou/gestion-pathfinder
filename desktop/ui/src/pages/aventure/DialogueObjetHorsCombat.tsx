// Utilisation d'un objet magique hors combat : le joueur saisit les dés à la main ; le cœur
// calcule soins, dégâts, effets et dépense, et enregistre le tout d'un bloc.

import { useState } from "react";
import { api } from "../../api";
import type { ObjetVue } from "../../aventure-types";
import { Dialogue } from "../../composants/jeu";

export function DialogueObjetHorsCombat({
  pjNom,
  pjId,
  objet,
  agir,
  onFermer,
}: {
  pjNom: string;
  pjId: number;
  objet: ObjetVue;
  agir: (p: Promise<{ ok: boolean; alerte: boolean; message: string }>) => Promise<unknown>;
  onFermer: () => void;
}) {
  const [resultat, setResultat] = useState<number | null>(null);
  const [cl, setCl] = useState<number | null>(null);
  const [charges, setCharges] = useState<number>(1);
  const baton = objet.genreCode === "baton";

  const utiliser = async () => {
    const r = await agir(
      api.aventureUtiliserObjet({
        pj: pjId,
        objet: objet.id,
        resultatDe: resultat,
        niveauDeLanceur: cl,
        charges: baton ? charges : 0,
      }),
    );
    if (r) onFermer();
  };

  return (
    <Dialogue
      titre={`Utiliser ${objet.nom} — ${pjNom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button className="bouton principal" onClick={utiliser}>Utiliser</button>
        </>
      }
    >
      <p className="aide">
        {objet.sort ? `Sort : ${objet.sort}` : "Sans sort automatique"} · niveau de lanceur {objet.niveauDeLanceur}
      </p>
      <label>
        Total des dés (soins ou dégâts, à saisir)
        <input type="number" value={resultat ?? ""} onChange={(e) => setResultat(e.target.value === "" ? null : Number(e.target.value))} />
      </label>
      <label>
        Niveau de lanceur (vide : celui de l'objet)
        <input type="number" min={1} value={cl ?? ""} onChange={(e) => setCl(e.target.value === "" ? null : Number(e.target.value))} />
      </label>
      {baton && (
        <label>
          Charges dépensées
          <input type="number" min={1} value={charges} onChange={(e) => setCharges(Math.max(1, Number(e.target.value)))} />
        </label>
      )}
    </Dialogue>
  );
}
