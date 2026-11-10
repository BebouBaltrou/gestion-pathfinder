// Convocation d'alliés naturels et formes (sorts de forme) en combat.
import { useEffect, useState } from "react";
import { api, Retour } from "../../api";
import type { ActionCombat, FormeOption, FormePouvoir, InvocationVue, OptionCreature, PjCombat } from "../../aventure-types";
import { Dialogue, SaisieNombre } from "../../composants/jeu";

/** Sorts de forme proposés, dans l'ordre du livre (nom anglais du catalogue). */
export const SORTS_DE_FORME = [
  "Beast Shape I",
  "Beast Shape II",
  "Beast Shape III",
  "Beast Shape IV",
  "Elemental Body I",
  "Elemental Body II",
  "Elemental Body III",
  "Elemental Body IV",
  "Plant Shape I",
  "Plant Shape II",
  "Plant Shape III",
];

/** Convocations : niveau du sort (1 à 9) donné sous son nom anglais du catalogue. */
export const nomConvocation = (niveau: number) => `Summon Nature's Ally ${niveau}`;

export function DialogueConvocation({
  pj,
  agir,
  onFermer,
}: {
  pj: PjCombat;
  agir: (a: ActionCombat) => Promise<Retour | null>;
  onFermer: () => void;
}) {
  const [niveau, setNiveau] = useState(1);
  const [options, setOptions] = useState<OptionCreature[]>([]);
  const [creature, setCreature] = useState<string>("");
  const [nombre, setNombre] = useState<number | null>(1);
  const [d20, setD20] = useState<number | null>(null);

  useEffect(() => {
    api.combatConvocations(niveau).then((o) => {
      setOptions(o);
      setCreature(o[0]?.nom ?? "");
    });
  }, [niveau]);

  const choisie = options.find((o) => o.nom === creature) ?? null;
  const pret = choisie !== null && nombre !== null && nombre >= (choisie?.nombreMin ?? 1) && nombre <= (choisie?.nombreMax ?? 1) && d20 !== null;

  const convoquer = async () => {
    if (!pret || !choisie) return;
    const r = await agir({ Convoquer: { lanceur: pj.id, sort: nomConvocation(niveau), creature: choisie.nom, nombre, d20 } });
    if (r?.ok) onFermer();
  };

  return (
    <Dialogue
      large
      titre={`Convoquer des alliés naturels — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Fermer</button>
          <button className="bouton principal" disabled={!pret} onClick={convoquer}>Convoquer</button>
        </>
      }
    >
      <div className="bloc">
        <label>
          Niveau du sort{" "}
          <select value={niveau} onChange={(e) => setNiveau(Number(e.target.value))}>
            {[1, 2, 3, 4, 5, 6, 7, 8, 9].map((n) => (
              <option key={n} value={n}>{`Summon Nature's Ally ${n}`}</option>
            ))}
          </select>
        </label>
        <p className="doux">Le druide peut utiliser ce sort à la place d'un sort préparé du même niveau (il consomme cet emplacement).</p>
      </div>
      <div className="bloc">
        <label>
          Créature{" "}
          <select value={creature} onChange={(e) => setCreature(e.target.value)}>
            {options.map((o) => (
              <option key={o.nom} value={o.nom}>{`${o.nom} (niveau ${o.niveau}, FP ${o.fp})`}</option>
            ))}
          </select>
        </label>
        {choisie && (
          <p className="doux">
            PV {choisie.pv} · CA {choisie.ca} · Init {choisie.init >= 0 ? `+${choisie.init}` : choisie.init} · nombre : de {choisie.nombreMin} à {choisie.nombreMax}
          </p>
        )}
        <SaisieNombre
          etiquette="Nombre de créatures"
          valeur={nombre}
          onChange={setNombre}
          min={choisie?.nombreMin ?? 1}
          max={choisie?.nombreMax ?? 1}
          rappel={choisie && choisie.nombreMax > 1 ? "Jet du dé : 1d3 ou 1d4+1 selon le niveau de la créature" : undefined}
        />
        <SaisieNombre etiquette="d20 d'initiative (groupe)" valeur={d20} onChange={setD20} min={1} max={20} />
      </div>
    </Dialogue>
  );
}

export function DialogueForme({
  pj,
  agir,
  onFermer,
}: {
  pj: PjCombat;
  agir: (a: ActionCombat) => Promise<Retour | null>;
  onFermer: () => void;
}) {
  const [sort, setSort] = useState(SORTS_DE_FORME[0]);
  const [formes, setFormes] = useState<FormeOption[]>([]);
  const [forme, setForme] = useState("");

  useEffect(() => {
    api.combatFormes(sort).then((f) => {
      setFormes(f);
      setForme(f[0]?.id ?? "");
    });
  }, [sort]);

  const choisie = formes.find((f) => f.id === forme) ?? null;

  const prendre = async () => {
    if (!choisie) return;
    const r = await agir({ Forme: { pj: pj.id, sort, forme: choisie.id } });
    if (r?.ok) onFermer();
  };

  return (
    <Dialogue
      large
      titre={`Prendre une forme — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Fermer</button>
          <button className="bouton principal" disabled={!choisie} onClick={prendre}>Prendre la forme</button>
        </>
      }
    >
      <div className="bloc">
        <label>
          Sort{" "}
          <select value={sort} onChange={(e) => setSort(e.target.value)}>
            {SORTS_DE_FORME.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
        </label>
        <label>
          Forme{" "}
          <select value={forme} onChange={(e) => setForme(e.target.value)}>
            {formes.map((f) => (
              <option key={f.id} value={f.id}>{f.nom}</option>
            ))}
          </select>
        </label>
        {choisie && <p className="doux">{choisie.texte}</p>}
      </div>
    </Dialogue>
  );
}

/** Fiche de chaque créature invoquée en combat : statistiques complètes. */
export function BlocInvocations({ invocations }: { invocations: InvocationVue[] }) {
  if (invocations.length === 0) return null;
  return (
    <div className="bloc">
      <h3>Créatures invoquées</h3>
      {invocations.map((i) => (
        <details key={i.id} className="carte">
          <summary>
            <strong>{i.nom}</strong> · niveau {i.niveauSort} du sort · PV {i.pv} · CA {i.ca} · Init {i.init >= 0 ? `+${i.init}` : i.init}
          </summary>
          <p className="doux">{i.lignes.map(([l, v]) => `${l ? l + " : " : ""}${v}`).join(" · ")}</p>
          <p>
            {i.sauvegardes.map(([k, v]) => `${k} ${v >= 0 ? "+" : ""}${v}`).join(", ")}
          </p>
          {i.attaques && <p>{i.attaques}</p>}
        </details>
      ))}
    </div>
  );
}

/** Pouvoir de forme animale du druide en combat : on choisit la forme, puis ses effets sont appliqués. */
export function DialogueFormeAnimale({
  pj,
  agir,
  onFermer,
}: {
  pj: PjCombat;
  agir: (a: ActionCombat) => Promise<Retour | null>;
  onFermer: () => void;
}) {
  const [formes, setFormes] = useState<FormePouvoir[]>([]);
  const [forme, setForme] = useState("");

  useEffect(() => {
    api.combatFormesAnimales(pj.id).then((f) => {
      setFormes(f);
      setForme(f[0]?.id ?? "");
    });
  }, [pj.id]);

  const choisie = formes.find((f) => f.id === forme) ?? null;

  const prendre = async () => {
    if (!choisie) return;
    const r = await agir({ FormeAnimale: { pj: pj.id, forme: choisie.id } });
    if (r?.ok) onFermer();
  };

  return (
    <Dialogue
      large
      titre={`Forme animale — ${pj.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Fermer</button>
          <button className="bouton principal" disabled={!choisie} onClick={prendre}>Prendre la forme</button>
        </>
      }
    >
      <div className="bloc">
        <label>
          Forme{" "}
          <select value={forme} onChange={(e) => setForme(e.target.value)}>
            {formes.map((f) => (
              <option key={f.id} value={f.id}>{`${f.nom} (druide niveau ${f.niveauDruide})`}</option>
            ))}
          </select>
        </label>
        {choisie && (
          <p className="doux">
            Taille : {choisie.taille ?? "non précisée"} · Pendant une heure par niveau de druide · consomme une utilisation du jour
          </p>
        )}
        {choisie && <p className="doux">{choisie.texte}</p>}
      </div>
    </Dialogue>
  );
}
