// Récompenses de fin de combat : XP saisi par le joueur pour chaque personnage (avec une aide de calcul
// d'après les FP des créatures vaincues), or ramassé et objets trouvés. Elles entrent dans la campagne
// avec le combat, en un seul bloc annulable.

import { useEffect, useState } from "react";
import { api } from "../../api";
import type { PjCombat, Recompenses } from "../../aventure-types";
import { SaisieNombre } from "../../composants/jeu";
import { DialogueObjet } from "./dialogues";
import type { Consommable } from "../../evenements-jeu";

export type EtatRecompenses = {
  xp: Record<number, number | null>;
  motif: string;
  or: Record<number, number | null>;
  objets: [number, Consommable][];
};

export const recompensesVides = (): EtatRecompenses => ({ xp: {}, motif: "", or: {}, objets: [] });

/** Convertit l'état saisi en message pour le cœur ; `destinataires` : personnages concernés par le combat. */
export function versRecompenses(e: EtatRecompenses, destinataires: number[]): Recompenses {
  return {
    xp: destinataires.map((id) => [id, e.xp[id] ?? 0] as [number, number]).filter(([, v]) => v > 0),
    motif: e.motif,
    or: Object.entries(e.or)
      .filter(([, v]) => v !== null && v !== 0)
      .map(([id, v]) => [Number(id), v as number] as [number, number]),
    objets: e.objets,
  };
}

export function BlocRecompenses({
  pjs,
  destinataires,
  etat,
  onChange,
}: {
  pjs: PjCombat[];
  destinataires: number[];
  etat: EtatRecompenses;
  onChange: (e: EtatRecompenses) => void;
}) {
  const [fps, setFps] = useState<string[]>([]);
  const [catalogue, setCatalogue] = useState<string[]>([]);
  const [calcul, setCalcul] = useState<{ total: number; parPersonnage: number } | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);
  const [ajoutObjet, setAjoutObjet] = useState(false);
  useEffect(() => {
    api.catalogueFp().then(setCatalogue);
  }, []);

  const concernes = pjs.filter((p) => destinataires.includes(p.id));
  const n = Math.max(1, concernes.length);
  // aide facultative : le cœur calcule le total d'XP des créatures vaincues (Core p. 397)
  useEffect(() => {
    setErreur(null);
    if (fps.length === 0) {
      setCalcul(null);
      return;
    }
    api
      .combatXp(fps, n)
      .then(setCalcul)
      .catch((e) => setErreur(String(e)));
  }, [fps, n]);

  return (
    <div className="bloc">
      <h3>Récompenses (facultatif)</h3>
      <label className="champ-g">
        <span>Motif (pour l'historique des fiches)</span>
        <input className="champ-jeu" value={etat.motif} onChange={(e) => onChange({ ...etat, motif: e.target.value })} placeholder="combat dans la crypte" />
      </label>

      <div className="bloc">
        <h3>Expérience gagnée par personnage</h3>
        <div className="champs">
          {concernes.map((p) => (
            <SaisieNombre key={p.id} etiquette={p.nom} min={0} max={10_000_000} valeur={etat.xp[p.id] ?? null} onChange={(v) => onChange({ ...etat, xp: { ...etat.xp, [p.id]: v } })} />
          ))}
        </div>
        <details>
          <summary>Aide : calculer d'après les FP des créatures vaincues (tableau du Core p. 397)</summary>
          <div className="bloc">
            {fps.map((fp, i) => (
              <div className="rangee-boutons" key={i}>
                <select className="champ-jeu" style={{ maxWidth: 160 }} value={fp} onChange={(e) => setFps(fps.map((x, j) => (j === i ? e.target.value : x)))} aria-label="FP de la créature">
                  {catalogue.map((c) => (
                    <option key={c} value={c}>FP {c}</option>
                  ))}
                </select>
                <button className="bouton mini danger" onClick={() => setFps(fps.filter((_, j) => j !== i))}>Retirer</button>
              </div>
            ))}
            <div className="rangee-boutons">
              <button className="bouton mini" onClick={() => setFps([...fps, "1"])}>+ Une créature vaincue</button>
            </div>
            {erreur && <div className="avertissement">{erreur}</div>}
            {calcul && (
              <p>
                Total des créatures : <strong>{calcul.total} XP</strong>, soit <strong>{calcul.parPersonnage} XP</strong> par personnage si on partage entre {n}{" "}
                <span className="doux">(simple suggestion, arrondie vers le bas : c'est toi qui décides de ce que chacun reçoit)</span>.{" "}
                <button className="bouton mini" onClick={() => onChange({ ...etat, xp: Object.fromEntries(concernes.map((p) => [p.id, calcul.parPersonnage])) })}>
                  Reporter cette part à chacun
                </button>
              </p>
            )}
          </div>
        </details>
      </div>

      <div className="bloc">
        <h3>Or ramassé (en po, pour le compte du personnage)</h3>
        <div className="champs">
          {pjs.map((p) => (
            <SaisieNombre key={p.id} etiquette={p.nom} min={0} max={10_000_000} valeur={etat.or[p.id] ?? null} onChange={(v) => onChange({ ...etat, or: { ...etat.or, [p.id]: v } })} />
          ))}
        </div>
      </div>

      <div className="bloc">
        <h3>Objets trouvés</h3>
        {etat.objets.length === 0 && <span className="doux">Aucun.</span>}
        {etat.objets.map(([pj, o], i) => (
          <div className="ligne-objet" key={i}>
            <span className="nom">
              {o.nom} <span className="doux">→ {pjs.find((p) => p.id === pj)?.nom}</span>
            </span>
            <button className="bouton mini danger" onClick={() => onChange({ ...etat, objets: etat.objets.filter((_, j) => j !== i) })}>Retirer</button>
          </div>
        ))}
        <div className="rangee-boutons">
          <button className="bouton mini" onClick={() => setAjoutObjet(true)}>+ Ajouter un objet</button>
        </div>
      </div>
      {ajoutObjet && (
        <DialogueObjet
          pjs={pjs}
          pjId={pjs[0].id}
          envoyer={async (e) => {
            const o = (e as { ObjetAjoute?: { pj: number; objet: Consommable } }).ObjetAjoute;
            if (o) onChange({ ...etat, objets: [...etat.objets, [o.pj, o.objet]] });
          }}
          onFermer={() => setAjoutObjet(false)}
        />
      )}
    </div>
  );
}
