import { useEffect, useState } from "react";
import { api } from "../../api";
import type { CatalogueCreation, ValeurVue } from "../../fiche-types";

let cache: Promise<CatalogueCreation> | null = null;

/** Catalogue du Core (races, dons, sorts, équipement…) : chargé une seule fois. */
export function useCatalogue(): CatalogueCreation | null {
  const [c, setC] = useState<CatalogueCreation | null>(null);
  useEffect(() => {
    cache ??= api.catalogueCreation();
    cache.then(setC);
  }, []);
  return c;
}

export const signe = (n: number) => (n >= 0 ? `+${n}` : `−${Math.abs(n)}`);

/** Un chiffre de la fiche : le total, et au clic le détail de son calcul. */
export function Chiffre({ nom, v, avecSigne = false, sous }: { nom: string; v: ValeurVue; avecSigne?: boolean; sous?: string }) {
  const [ouvert, setOuvert] = useState(false);
  return (
    <div className="carte chiffre-fiche">
      <button className="chiffre-bouton" onClick={() => setOuvert(!ouvert)} aria-expanded={ouvert} title="Voir le détail du calcul">
        <span className="etiquette">{nom}</span>
        <span className="valeur">{avecSigne ? signe(v.total) : v.total}</span>
      </button>
      {sous && <span className="modificateur">{sous}</span>}
      {ouvert && <Detail v={v} />}
    </div>
  );
}

export function Detail({ v }: { v: ValeurVue }) {
  return (
    <ul className="detail-calcul">
      {v.termes.map((t, i) => (
        <li key={i} className={t.compte ? "" : "ignore"}>
          <span>{t.libelle}</span>
          <span>{signe(t.valeur)}</span>
        </li>
      ))}
      <li className="total">
        <span>Total</span>
        <span>{v.total}</span>
      </li>
    </ul>
  );
}

/** Saisie d'un entier avec boutons − et +. */
export function Pas({
  valeur,
  min,
  max,
  onChange,
  etiquette,
}: {
  valeur: number;
  min: number;
  max: number;
  onChange: (n: number) => void;
  etiquette: string;
}) {
  return (
    <span className="pas" role="group" aria-label={etiquette}>
      <button className="bouton icone petit" aria-label={`Moins ${etiquette}`} disabled={valeur <= min} onClick={() => onChange(valeur - 1)}>
        −
      </button>
      <span className="pas-valeur">{valeur}</span>
      <button className="bouton icone petit" aria-label={`Plus ${etiquette}`} disabled={valeur >= max} onClick={() => onChange(valeur + 1)}>
        +
      </button>
    </span>
  );
}

/** Menu déroulant filtrable sur une liste (nom affiché, valeur envoyée). */
export function Choix<T extends string>({
  options,
  valeur,
  onChange,
  vide,
  etiquette,
}: {
  options: { v: T; l: string }[];
  valeur: T | "";
  onChange: (v: T | "") => void;
  vide: string;
  etiquette: string;
}) {
  return (
    <select className="champ choix" aria-label={etiquette} value={valeur} onChange={(e) => onChange(e.target.value as T | "")}>
      <option value="">{vide}</option>
      {options.map((o) => (
        <option key={o.v} value={o.v}>
          {o.l}
        </option>
      ))}
    </select>
  );
}
