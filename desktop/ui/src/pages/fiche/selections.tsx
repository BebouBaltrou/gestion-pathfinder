import { Fragment, useState } from "react";
import type { CatalogueCreation, FicheVue, SortCatalogue } from "../../fiche-types";
import { Choix, Pas } from "./commun";

/** Choix de dons : un menu par don à prendre, avec le choix éventuel (arme, école…). */
export function SelectionDons({
  cat,
  nombre,
  valeur,
  onChange,
  imposes = [],
  lignage,
}: {
  cat: CatalogueCreation;
  nombre: number;
  valeur: string[];
  onChange: (v: string[]) => void;
  imposes?: string[];
  /** Les premiers dons sont des dons de lignage : on ne peut les prendre que dans la liste du lignage. */
  lignage?: { nombre: number; possibles: { nomEn: string; nom: string }[]; dejaPris: string[] };
}) {
  const lignes = Array.from({ length: nombre }, (_, i) => valeur[i] ?? "");
  const maj = (i: number, v: string) => {
    const n = [...lignes];
    n[i] = v;
    onChange(n);
  };
  return (
    <div className="section">
      {imposes.length > 0 && <p className="doux">Don imposé : {imposes.join(", ")} (à choisir parmi ses variantes).</p>}
      {lignage && (
        <p className="doux">
          {lignage.nombre > 1 ? `Les ${lignage.nombre} premiers dons sont` : "Le premier don est"} un don de lignage : seuls les dons
          de ton lignage sont proposés.
        </p>
      )}
      {lignes.map((d, i) => {
        if (lignage && i < lignage.nombre) {
          const pris = lignes.filter((_, j) => j !== i);
          const options = lignage.possibles
            .filter((p) => !lignage.dejaPris.includes(p.nomEn) && !pris.includes(p.nomEn))
            .map((p) => ({ v: p.nomEn, l: p.nom }));
          const def = cat.dons.find((x) => x.nomEn === d.split(" (")[0]);
          return (
            <div className="carte ligne-don" key={i}>
              <Choix
                etiquette={`Don de lignage ${i + 1}`}
                vide="Choisir un don de lignage…"
                valeur={options.some((o) => o.v === d) ? d : ""}
                options={options}
                onChange={(v) => maj(i, v)}
              />
              {def && (
                <p className="doux">
                  {def.resume}
                  {def.prerequis ? ` — Prérequis : ${def.prerequis}` : ""}
                </p>
              )}
            </div>
          );
        }
        const base = d.split(" (")[0];
        const choix = d.includes(" (") ? d.slice(d.indexOf(" (") + 2, -1) : "";
        const def = cat.dons.find((x) => x.nomEn === base);
        return (
          <div className="carte ligne-don" key={i}>
            <Choix
              etiquette={`Don ${i + 1}`}
              vide="Choisir un don…"
              valeur={def ? def.nomEn : ""}
              options={cat.dons.map((x) => ({ v: x.nomEn, l: x.nom }))}
              onChange={(v) => maj(i, v)}
            />
            {def?.aUnChoix && (
              <input
                className="champ"
                aria-label="Choix du don (en anglais, ex. longsword)"
                placeholder="Précision (nom anglais : arme, compétence, école…)"
                value={choix}
                onChange={(e) => maj(i, e.target.value ? `${base} (${e.target.value})` : base)}
              />
            )}
            {def && (
              <p className="doux">
                {def.resume}
                {def.prerequis ? ` — Prérequis : ${def.prerequis}` : ""}
              </p>
            )}
          </div>
        );
      })}
    </div>
  );
}

/** Choix de sorts connus par niveau de sort. */
export function SelectionSorts({
  sorts,
  attendus,
  valeur,
  onChange,
  exclus = [],
}: {
  sorts: SortCatalogue[];
  /** (niveau de sort, nombre à choisir) */
  attendus: [number, number][];
  valeur: string[];
  onChange: (v: string[]) => void;
  exclus?: string[];
}) {
  return (
    <div className="section">
      {attendus.map(([niveau, nombre]) => {
        const liste = sorts.filter((s) => s.niveau === niveau && !exclus.includes(s.nomEn));
        const choisis = valeur.filter((v) => liste.some((s) => s.nomEn === v));
        return (
          <details key={niveau} className="carte" open={niveau <= 1}>
            <summary>
              {niveau === 0 ? "Tours de magie" : `Sorts de niveau ${niveau}`} : {choisis.length}/{nombre} choisi(s)
            </summary>
            <div className="liste-sorts">
              {liste.map((s) => {
                const coche = valeur.includes(s.nomEn);
                return (
                  <label key={s.nomEn} className="case sort-choix" title={s.resume}>
                    <input
                      type="checkbox"
                      checked={coche}
                      disabled={!coche && choisis.length >= nombre}
                      onChange={() =>
                        onChange(coche ? valeur.filter((v) => v !== s.nomEn) : [...valeur, s.nomEn])
                      }
                    />
                    <span>{s.nom}</span>
                    <small className="doux">{s.ecole}</small>
                  </label>
                );
              })}
            </div>
          </details>
        );
      })}
    </div>
  );
}

/** Rangs de compétence à placer : un compteur par compétence. */
export function SelectionRangs({
  cat,
  apercu,
  rangs,
  onChange,
  disponibles,
  maximum,
  dejaPlaces = {},
}: {
  cat: CatalogueCreation;
  /** Fiche d'aperçu, pour montrer le total et les compétences de classe. */
  apercu: FicheVue | null;
  rangs: Record<string, number>;
  onChange: (r: Record<string, number>) => void;
  disponibles: number;
  /** Rangs maximum dans une compétence (= niveau du personnage). */
  maximum: number;
  /** Rangs déjà placés aux niveaux précédents. */
  dejaPlaces?: Record<string, number>;
}) {
  const [base, setBase] = useState("");
  const [precision, setPrecision] = useState("");
  const places = Object.values(rangs).reduce((a, b) => a + b, 0);
  const libres = cat.competences.filter((c) => !c.sousTypes);
  const sousTypes = cat.competences.filter((c) => c.sousTypes);
  const extras = Object.keys(rangs).filter((k) => !libres.some((c) => c.id === k));
  const regler = (cle: string, n: number) => {
    const r = { ...rangs };
    if (n <= 0) delete r[cle];
    else r[cle] = n;
    onChange(r);
  };
  const ligne = (cle: string, nom: string) => {
    const info = apercu?.competences.find((c) => c.cle === cle);
    const avant = dejaPlaces[cle] ?? 0;
    const ici = rangs[cle] ?? 0;
    return (
      <Fragment key={cle}>
        <tr>
          <td>
            {nom}
            {info?.deClasse && <span className="puce or"> de classe</span>}
          </td>
          <td className="num">{info ? (info.valeur.total >= 0 ? `+${info.valeur.total}` : info.valeur.total) : ""}</td>
          <td>
            <Pas
              etiquette={nom}
              valeur={ici}
              min={0}
              max={Math.max(0, Math.min(maximum - avant, ici + disponibles - places))}
              onChange={(n) => regler(cle, n)}
            />
          </td>
        </tr>
      </Fragment>
    );
  };
  return (
    <div className="section">
      <p>
        Rangs placés : <strong>{places}</strong> / {disponibles}
        {places > disponibles && <span className="alerte-texte"> (trop de rangs)</span>}
      </p>
      <table className="tableau">
        <thead>
          <tr><th>Compétence</th><th>Total</th><th>Rangs ajoutés</th></tr>
        </thead>
        <tbody>
          {libres.map((c) => ligne(c.id, c.nom))}
          {extras.map((k) => ligne(k, k.replace(":", " : ")))}
        </tbody>
      </table>
      <div className="ligne">
        <Choix
          etiquette="Compétence à préciser"
          vide="Artisanat, Représentation, Profession…"
          valeur={base}
          options={sousTypes.filter((c) => !c.id.startsWith("knowledge")).map((c) => ({ v: c.id, l: c.nom }))}
          onChange={setBase}
        />
        <input className="champ" aria-label="Précision" placeholder="Précision (ex. forgeron)" value={precision} onChange={(e) => setPrecision(e.target.value)} />
        <button
          className="bouton"
          disabled={!base || !precision.trim()}
          onClick={() => {
            regler(`${base}:${precision.trim().toLowerCase()}`, 1);
            setPrecision("");
          }}
        >
          Ajouter
        </button>
      </div>
    </div>
  );
}
