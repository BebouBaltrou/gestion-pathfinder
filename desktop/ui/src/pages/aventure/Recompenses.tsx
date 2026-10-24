// Récompenses de fin de combat : XP (total saisi ou calculé d'après les FP des créatures vaincues),
// or ramassé et objets trouvés. Elles entrent dans la campagne avec le combat, en un seul bloc annulable.

import { useEffect, useState } from "react";
import { api } from "../../api";
import type { PjCombat, Recompenses } from "../../aventure-types";
import { SaisieNombre } from "../../composants/jeu";
import { DialogueObjet } from "./dialogues";
import type { Consommable } from "../../evenements-jeu";

export type EtatRecompenses = {
  xpTotal: number | null;
  motif: string;
  or: Record<number, number | null>;
  objets: [number, Consommable][];
};

export const recompensesVides = (): EtatRecompenses => ({ xpTotal: null, motif: "", or: {}, objets: [] });

/** Convertit l'état saisi en message pour le cœur ; `destinataires` : personnages qui reçoivent l'XP. */
export function versRecompenses(e: EtatRecompenses, destinataires: number[], parPersonnage: number): Recompenses {
  return {
    xp: parPersonnage > 0 ? destinataires.map((id) => [id, parPersonnage] as [number, number]) : [],
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
  onPartXp,
}: {
  pjs: PjCombat[];
  destinataires: number[];
  etat: EtatRecompenses;
  onChange: (e: EtatRecompenses) => void;
  onPartXp: (xp: number) => void;
}) {
  const [mode, setMode] = useState<"total" | "fp">("total");
  const [fps, setFps] = useState<string[]>([]);
  const [catalogue, setCatalogue] = useState<string[]>([]);
  const [calcul, setCalcul] = useState<{ total: number; parPersonnage: number } | null>(null);
  const [erreur, setErreur] = useState<string | null>(null);
  const [ajoutObjet, setAjoutObjet] = useState(false);
  useEffect(() => {
    api.catalogueFp().then(setCatalogue);
  }, []);

  const n = Math.max(1, destinataires.length);
  // part de chaque personnage, calculée par le cœur
  useEffect(() => {
    setErreur(null);
    if (mode === "fp") {
      if (fps.length === 0) {
        setCalcul(null);
        onPartXp(0);
        return;
      }
      api
        .combatXp(fps, n)
        .then((x) => {
          setCalcul(x);
          onPartXp(x.parPersonnage);
        })
        .catch((e) => setErreur(String(e)));
    } else if (etat.xpTotal !== null && etat.xpTotal > 0) {
      const part = Math.floor(etat.xpTotal / n);
      setCalcul({ total: etat.xpTotal, parPersonnage: part });
      onPartXp(part);
    } else {
      setCalcul(null);
      onPartXp(0);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode, fps, etat.xpTotal, n]);

  return (
    <div className="bloc">
      <h3>Récompenses (facultatif)</h3>
      <label className="champ-g">
        <span>Motif (pour l'historique des fiches)</span>
        <input className="champ-jeu" value={etat.motif} onChange={(e) => onChange({ ...etat, motif: e.target.value })} placeholder="combat dans la crypte" />
      </label>

      <div className="bloc">
        <h3>Expérience</h3>
        <div className="rangee-boutons">
          <label className="interrupteur">
            <input type="radio" name="xp" checked={mode === "total"} onChange={() => setMode("total")} />
            Je saisis le total d'XP
          </label>
          <label className="interrupteur">
            <input type="radio" name="xp" checked={mode === "fp"} onChange={() => setMode("fp")} />
            Calculer d'après les FP des créatures vaincues (tableau du Core p. 397)
          </label>
        </div>
        {mode === "total" ? (
          <SaisieNombre etiquette="XP à partager" min={0} max={10_000_000} valeur={etat.xpTotal} onChange={(v) => onChange({ ...etat, xpTotal: v })} />
        ) : (
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
          </div>
        )}
        {erreur && <div className="avertissement">{erreur}</div>}
        {calcul && (
          <p>
            Total : <strong>{calcul.total} XP</strong>, soit <strong>{calcul.parPersonnage} XP</strong> pour chacun des {n} personnage(s) concerné(s)
            {" "}<span className="doux">(méthode exacte : le total est divisé par le nombre de personnages, arrondi vers le bas)</span>.
          </p>
        )}
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
