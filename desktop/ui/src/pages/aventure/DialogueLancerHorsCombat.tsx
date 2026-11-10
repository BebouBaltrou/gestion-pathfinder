// Sort lancé hors combat : le joueur choisit un sort connu (ou préparé), saisit les dés.
// Le cœur contrôle l'emplacement, le sort préparé et le DD, puis enregistre tout d'un bloc.

import { useState } from "react";
import { api } from "../../api";
import type { PjAventure } from "../../aventure-types";
import { Dialogue } from "../../composants/jeu";

export function DialogueLancerHorsCombat({
  pj,
  agir,
  onFermer,
}: {
  pj: PjAventure;
  agir: (p: Promise<{ ok: boolean; alerte: boolean; message: string }>) => Promise<unknown>;
  onFermer: () => void;
}) {
  const [sort, setSort] = useState<string | null>(pj.sortsConnus[0]?.nomEn ?? null);
  const [resultat, setResultat] = useState<number | null>(null);
  const [d100, setD100] = useState<number | null>(null);

  const lancer = async () => {
    if (!sort) return;
    const r = await agir(
      api.aventureLancerSort({ pj: pj.id, sort, cibles: [pj.id], resultatDe: resultat, echecArcaniqueD100: d100 }),
    );
    if (r) onFermer();
  };

  return (
    <Dialogue
      large
      titre={`Lancer un sort — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button className="bouton principal" disabled={!sort} onClick={lancer}>Lancer</button>
        </>
      }
    >
      {pj.sortsConnus.length === 0 ? (
        <p className="aide">Aucun sort connu ou préparé pour ce personnage.</p>
      ) : (
        <div className="rangee-boutons">
          {pj.sortsConnus.map((s) => (
            <button key={s.nomEn} className={"bouton" + (s.nomEn === sort ? " principal" : "")} onClick={() => setSort(s.nomEn)}>
              {s.nom} (niveau {s.niveau})
            </button>
          ))}
        </div>
      )}
      <label>
        Total des dés (soins ou dégâts, facultatif)
        <input type="number" value={resultat ?? ""} onChange={(e) => setResultat(e.target.value === "" ? null : Number(e.target.value))} />
      </label>
      <label>
        Dé d'échec arcanique (d100, si le sort est profane avec armure)
        <input type="number" min={1} max={100} value={d100 ?? ""} onChange={(e) => setD100(e.target.value === "" ? null : Number(e.target.value))} />
      </label>
    </Dialogue>
  );
}
