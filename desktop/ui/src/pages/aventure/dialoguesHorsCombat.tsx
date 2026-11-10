// Sorts et objets hors combat (phase aventure) : le cœur applique les règles et refuse ce qui
// n'est pas permis ; l'interface ne fait que recueillir les saisies (dés, cibles).

import { useState } from "react";
import { api } from "../../api";
import type { EvenementAventure, ObjetVue, PjAventure } from "../../aventure-types";
import { Dialogue, SaisieNombre } from "../../composants/jeu";
import { ev, type Consommable } from "../../evenements-jeu";

type Agir = (p: Promise<unknown>) => Promise<unknown>;

function Cibles({ pjs, valeur, onChange }: { pjs: PjAventure[]; valeur: number[]; onChange: (v: number[]) => void }) {
  return (
    <div className="champs">
      <span className="doux">Personnages visés</span>
      {pjs.map((p) => (
        <label key={p.id} className="interrupteur">
          <input
            type="checkbox"
            checked={valeur.includes(p.id)}
            onChange={(e) => onChange(e.target.checked ? [...valeur, p.id] : valeur.filter((x) => x !== p.id))}
          />
          {p.nom}
        </label>
      ))}
    </div>
  );
}

export function DialogueLancerSort({ pjs, pjId, agir, onFermer }: { pjs: PjAventure[]; pjId: number; agir: Agir; onFermer: () => void }) {
  const lanceur = pjs.find((p) => p.id === pjId)!;
  const [sort, setSort] = useState(lanceur.sortsConnus[0]?.nomEn ?? "");
  const [cibles, setCibles] = useState<number[]>([pjId]);
  const [d100, setD100] = useState<number | null>(null);
  const [resultat, setResultat] = useState<number | null>(null);
  return (
    <Dialogue
      titre={`${lanceur.nom} lance un sort`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={sort === ""}
            onClick={async () => {
              await agir(
                api.aventureLancerSort({
                  pj: pjId,
                  sort,
                  cibles,
                  echec_arcanique_d100: d100,
                  resultat_de: resultat,
                }),
              );
              onFermer();
            }}
          >
            Lancer
          </button>
        </>
      }
    >
      <div className="champs">
        <label className="champ-g">
          <span>Sort</span>
          <select className="champ-jeu" value={sort} onChange={(e) => setSort(e.target.value)}>
            {lanceur.sortsConnus.map((s) => (
              <option key={s.nomEn} value={s.nomEn}>
                {s.nom} (niveau {s.niveau})
              </option>
            ))}
          </select>
        </label>
      </div>
      <Cibles pjs={pjs} valeur={cibles} onChange={setCibles} />
      <div className="champs">
        <SaisieNombre etiquette="d100 d'échec des sorts profanes (si besoin)" min={1} max={100} valeur={d100} onChange={setD100} />
        <SaisieNombre etiquette="Total des dés du sort (soins ou dégâts)" min={1} max={100} valeur={resultat} onChange={setResultat} />
      </div>
    </Dialogue>
  );
}

export function DialogueUtiliserObjet({
  pjs,
  pjId,
  objet,
  agir,
  onFermer,
}: {
  pjs: PjAventure[];
  pjId: number;
  objet: ObjetVue;
  agir: Agir;
  onFermer: () => void;
}) {
  const [cibles, setCibles] = useState<number[]>([pjId]);
  const [cl20, setCl20] = useState<number | null>(null);
  const [accident20, setAccident20] = useState<number | null>(null);
  const [d100, setD100] = useState<number | null>(null);
  const [charges, setCharges] = useState<number | null>(1);
  const [resultat, setResultat] = useState<number | null>(null);
  const estBaton = objet.genreCode === "baton";
  return (
    <Dialogue
      titre={`Utiliser : ${objet.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            onClick={async () => {
              await agir(
                api.aventureUtiliserObjet({
                  pj: pjId,
                  objet: objet.id,
                  cibles: objet.genreCode === "potion" ? [pjId] : cibles,
                  charges: estBaton ? charges ?? 1 : 1,
                  niveau_de_lanceur_d20: cl20,
                  accident_d20: accident20,
                  echec_arcanique_d100: d100,
                  resultat_de: resultat,
                }),
              );
              onFermer();
            }}
          >
            Utiliser
          </button>
        </>
      }
    >
      {objet.genreCode !== "potion" && <Cibles pjs={pjs} valeur={cibles} onChange={setCibles} />}
      <div className="champs">
        {objet.genreCode === "parchemin" && (
          <>
            <SaisieNombre
              etiquette="d20 du test de niveau de lanceur (si le parchemin est au-dessus de votre niveau)"
              min={1}
              max={20}
              valeur={cl20}
              onChange={setCl20}
            />
            <SaisieNombre etiquette="d20 de l'accident (Sagesse DD 5, si le test est raté)" min={1} max={20} valeur={accident20} onChange={setAccident20} />
          </>
        )}
        {objet.sort && <SaisieNombre etiquette="d100 d'échec des sorts profanes (si besoin)" min={1} max={100} valeur={d100} onChange={setD100} />}
        {estBaton && <SaisieNombre etiquette="Charges dépensées" min={1} max={objet.charges?.[0] ?? 1} valeur={charges} onChange={setCharges} />}
        <SaisieNombre etiquette="Total des dés du sort (soins ou dégâts)" min={1} max={100} valeur={resultat} onChange={setResultat} />
      </div>
    </Dialogue>
  );
}

export function DialogueModifierObjet({
  pj,
  objet,
  envoyer,
  onFermer,
}: {
  pj: number;
  objet: ObjetVue;
  envoyer: (e: EvenementAventure) => Promise<unknown>;
  onFermer: () => void;
}) {
  const [nom, setNom] = useState(objet.nom);
  const [sort, setSort] = useState(objet.sortEn ?? "");
  const [niveauSort, setNiveauSort] = useState<number | null>(objet.niveauDuSort);
  const [cl, setCl] = useState<number | null>(objet.niveauDeLanceur);
  const [quantite, setQuantite] = useState<number | null>(objet.quantite);
  const [restantes, setRestantes] = useState<number | null>(objet.charges?.[0] ?? null);
  const [max, setMax] = useState<number | null>(objet.charges?.[1] ?? null);
  const [arcanique, setArcanique] = useState(objet.arcanique);
  const aCharges = objet.charges !== null;
  const valide = nom.trim() !== "" && (!aCharges || (restantes !== null && max !== null && restantes <= max));
  return (
    <Dialogue
      titre={`Modifier : ${objet.nom}`}
      onFermer={onFermer}
      actions={
        <>
          <button className="bouton" onClick={onFermer}>Annuler</button>
          <button
            className="bouton principal"
            disabled={!valide}
            onClick={async () => {
              const modifie: Consommable = {
                nom: nom.trim(),
                genre: objet.genreCode,
                sort: sort.trim() === "" ? null : sort.trim(),
                niveau_du_sort: niveauSort ?? 0,
                niveau_de_lanceur: cl ?? 0,
                arcanique,
                quantite: aCharges ? 1 : quantite ?? 1,
                charges: aCharges && restantes !== null && max !== null ? { restantes, max } : null,
                par_jour: objet.parJour ? { restantes: objet.parJour[0], max: objet.parJour[1] } : null,
                notes: objet.notes,
              };
              await envoyer(ev.objetModifie(pj, objet.id, modifie));
              onFermer();
            }}
          >
            Enregistrer
          </button>
        </>
      }
    >
      <div className="champs">
        <label className="champ-g">
          <span>Nom</span>
          <input className="champ-jeu" value={nom} onChange={(e) => setNom(e.target.value)} />
        </label>
        <label className="champ-g">
          <span>Sort contenu (nom anglais du catalogue)</span>
          <input className="champ-jeu" value={sort} onChange={(e) => setSort(e.target.value)} />
        </label>
      </div>
      <div className="champs">
        <SaisieNombre etiquette="Niveau du sort" min={0} max={9} valeur={niveauSort} onChange={setNiveauSort} />
        <SaisieNombre etiquette="Niveau de lanceur" min={1} max={20} valeur={cl} onChange={setCl} />
        {objet.genreCode === "parchemin" && (
          <label className="interrupteur">
            <input type="checkbox" checked={arcanique} onChange={(e) => setArcanique(e.target.checked)} />
            Parchemin profane (sinon divin)
          </label>
        )}
        {!aCharges && <SaisieNombre etiquette="Quantité" min={1} max={999} valeur={quantite} onChange={setQuantite} />}
        {aCharges && (
          <>
            <SaisieNombre etiquette="Charges restantes" min={0} max={max ?? 999} valeur={restantes} onChange={setRestantes} />
            <SaisieNombre etiquette="Charges maximum" min={1} max={999} valeur={max} onChange={setMax} />
          </>
        )}
      </div>
    </Dialogue>
  );
}
