import { useState } from "react";
import type {
  CatalogueCreation,
  ClasseVue,
  DivinVue,
  FicheDonnees,
  FicheVue,
  PreparationVue,
  SortCatalogue,
  SortPrepare,
} from "../../fiche-types";
import { Choix } from "./commun";

/** Alignements, dans l'ordre de la grille Loi / Neutre / Chaos × Bien / Neutre / Mal. */
export const ALIGNEMENTS: [string, string][] = [
  ["LB", "Loyal Bon"],
  ["NB", "Neutre Bon"],
  ["CB", "Chaotique Bon"],
  ["LN", "Loyal Neutre"],
  ["N", "Neutre"],
  ["CN", "Chaotique Neutre"],
  ["LM", "Loyal Mauvais"],
  ["NM", "Neutre Mauvais"],
  ["CM", "Chaotique Mauvais"],
];

export const nomAlignement = (code: string | null) => ALIGNEMENTS.find(([c]) => c === code)?.[1] ?? code ?? "—";

const PB = ["divinité", "alignement", "domaine", "énergie"];
/** Messages du cœur qui concernent la foi (divinité, alignement, domaines, énergie). */
export const problemesDeFoi = (problemes: string[]) =>
  problemes.filter((p) => PB.some((m) => p.toLowerCase().includes(m)));

// ------------------------------------------------------------------ création : dieu, alignement, domaines

export function ChoixFoi({
  cat,
  fiche,
  maj,
  apercu,
  classe,
}: {
  cat: CatalogueCreation;
  fiche: FicheDonnees;
  maj: (p: Partial<FicheDonnees>) => void;
  apercu: FicheVue | null;
  classe: ClasseVue;
}) {
  const dieu = cat.divinites.find((d) => d.id === fiche.divinite);
  const domainesPermis = dieu ? cat.domaines.filter((d) => dieu.domaines.includes(d.id)) : cat.domaines;
  const basculer = (id: string) => {
    const deja = fiche.domaines.includes(id);
    if (deja) maj({ domaines: fiche.domaines.filter((d) => d !== id) });
    else if (fiche.domaines.length < classe.domainesChoisis) maj({ domaines: [...fiche.domaines, id] });
  };
  const problemes = problemesDeFoi(apercu?.problemes ?? []);
  return (
    <div className="section">
      <p className="doux">
        {classe.nom} : on choisit un dieu (ou aucun), l'alignement, puis {classe.domainesChoisis} domaines parmi ceux de son dieu. Le prêtre peut
        différer d'au plus une catégorie de l'alignement de son dieu ; un domaine de Bien, Mal, Loi ou Chaos demande l'alignement correspondant.
      </p>
      <div>
        <label className="etiquette-champ">Divinité</label>
        <Choix
          etiquette="Divinité"
          vide="Aucune divinité (concept divin)"
          valeur={fiche.divinite ?? ""}
          options={cat.divinites.map((d) => ({ v: d.id, l: `${d.nom} (${d.alignement})` }))}
          onChange={(v) => maj({ divinite: v || null, domaines: [] })}
        />
        {dieu && (
          <p className="doux">
            {dieu.nom}, {dieu.theme}. Arme de prédilection : {dieu.armeDePredilection}.
          </p>
        )}
      </div>
      <div>
        <label className="etiquette-champ">Alignement du personnage</label>
        <div className="groupe-boutons">
          {ALIGNEMENTS.map(([code, nom]) => (
            <button key={code} className={"bouton" + (fiche.alignement === code ? " principal" : "")} onClick={() => maj({ alignement: code })}>
              {nom}
            </button>
          ))}
        </div>
      </div>
      <div>
        <label className="etiquette-champ">
          Domaines ({fiche.domaines.length}/{classe.domainesChoisis})
        </label>
        <div className="liste-domaines">
          {domainesPermis.map((d) => {
            const choisi = fiche.domaines.includes(d.id);
            return (
              <div className="carte" key={d.id}>
                <label className="ligne">
                  <input
                    type="checkbox"
                    checked={choisi}
                    disabled={!choisi && fiche.domaines.length >= classe.domainesChoisis}
                    onChange={() => basculer(d.id)}
                  />
                  <strong>{d.nom}</strong>
                  {d.alignement && <small className="doux"> (réservé à l'alignement {d.alignement})</small>}
                </label>
                <details>
                  <summary>Pouvoirs et sorts</summary>
                  <p className="doux">{d.description}</p>
                  {d.pouvoirs.map((p) => (
                    <p key={p.nom}>
                      <strong>{p.nom} ({p.genre}, niveau {p.niveau}).</strong> {p.texte}
                    </p>
                  ))}
                  <p className="doux">Sorts de domaine : {d.sorts.map((s) => `${s.niveau}e ${s.nom}`).join(", ")}</p>
                </details>
              </div>
            );
          })}
        </div>
      </div>
      {classe.canalisation && (
        <div>
          <label className="etiquette-champ">Énergie canalisée (prêtre Neutre sur l'axe Bien-Mal)</label>
          <div className="groupe-boutons">
            <button className={"bouton" + (fiche.energie === "positive" ? " principal" : "")} onClick={() => maj({ energie: "positive" })}>
              Énergie positive (soigne les vivants)
            </button>
            <button className={"bouton" + (fiche.energie === "negative" ? " principal" : "")} onClick={() => maj({ energie: "negative" })}>
              Énergie négative (soigne les morts-vivants)
            </button>
            <button className={"bouton" + (!fiche.energie ? " principal" : "")} onClick={() => maj({ energie: null })}>
              Selon l'alignement
            </button>
          </div>
          <p className="doux">
            Un prêtre Bon canalise l'énergie positive, un prêtre Mauvais l'énergie négative. Un prêtre Neutre choisit une fois pour toutes ; ce choix
            décide aussi s'il convertit ses sorts en soins ou en blessures.
          </p>
        </div>
      )}
      {problemes.map((p, i) => (
        <p key={i} className="alerte-texte">{p}</p>
      ))}
    </div>
  );
}

// ------------------------------------------------------------------ préparation des sorts du jour

const sorts_vides = (): SortPrepare[] => [];

/** Éditeur de la préparation : un menu par emplacement, selon ce que le cœur autorise. */
export function EditeurPreparation({
  prep,
  liste,
  valeur,
  onChange,
}: {
  prep: PreparationVue;
  liste: SortCatalogue[];
  valeur: SortPrepare[];
  onChange: (v: SortPrepare[]) => void;
}) {
  const interdit = (s: SortCatalogue) => s.descripteurs.some((d) => prep.interdits.includes(d));
  const du_niveau = (niveau: number, domaine: boolean) => valeur.filter((s) => s.niveau === niveau && s.domaine === domaine);
  const remplacer = (niveau: number, domaine: boolean, i: number, sort: string) => {
    const autres = valeur.filter((s) => !(s.niveau === niveau && s.domaine === domaine));
    const courants = du_niveau(niveau, domaine).map((s) => s.sort);
    courants[i] = sort;
    onChange([...autres, ...courants.filter(Boolean).map((s) => ({ niveau, sort: s, domaine }))]);
  };
  return (
    <div className="section">
      {prep.interdits.length > 0 && (
        <p className="doux">
          Sorts exclus par l'alignement du personnage ou de son dieu : {prep.interdits.map((d) => ({ good: "Bien", evil: "Mal", lawful: "Loi", chaotic: "Chaos" }[d] ?? d)).join(", ")}.
        </p>
      )}
      {prep.capacites.map((c) => {
        const options = liste.filter((s) => s.niveau === c.niveau && !interdit(s));
        const domaine = prep.sortsDeDomaine.filter((s) => s.niveau === c.niveau && s.nomEn);
        return (
          <div className="carte" key={c.niveau}>
            <h3>
              {c.niveau === 0 ? "Oraisons" : `Sorts de niveau ${c.niveau}`}
              <small className="doux"> — {c.normaux} préparé(s){c.domaine ? " + 1 de domaine" : ""}</small>
            </h3>
            <div className="groupe-boutons">
              {Array.from({ length: c.normaux }, (_, i) => (
                <Choix
                  key={i}
                  etiquette={`Sort ${i + 1} de niveau ${c.niveau}`}
                  vide="Choisir un sort…"
                  valeur={du_niveau(c.niveau, false)[i]?.sort ?? ""}
                  options={options.map((s) => ({ v: s.nomEn, l: s.nom }))}
                  onChange={(v) => remplacer(c.niveau, false, i, v)}
                />
              ))}
              {c.domaine > 0 && (
                <Choix
                  etiquette={`Sort de domaine de niveau ${c.niveau}`}
                  vide="Sort de domaine…"
                  valeur={du_niveau(c.niveau, true)[0]?.sort ?? ""}
                  options={domaine.map((s) => ({ v: s.nomEn as string, l: `${s.nom} (domaine)` }))}
                  onChange={(v) => remplacer(c.niveau, true, 0, v)}
                />
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
}

export { sorts_vides };

// ------------------------------------------------------------------ fiche : foi

export function PanneauFoi({ divin }: { divin: DivinVue }) {
  return (
    <div className="section">
      <section className="carte">
        <h3>{divin.divinite ?? "Aucune divinité"}</h3>
        <p className="doux">
          Alignement : {nomAlignement(divin.alignement)}
          {divin.armeDePredilection ? ` · arme de prédilection : ${divin.armeDePredilection}` : ""}
        </p>
      </section>
      {divin.canalisation && (
        <section className="carte">
          <h3>Canalisation d'énergie</h3>
          {divin.canalisation.positive === null ? (
            <p className="alerte-texte">Énergie à choisir (positive ou négative).</p>
          ) : (
            <p>
              Énergie {divin.canalisation.positive ? "positive : soigne les vivants, blesse les morts-vivants" : "négative : blesse les vivants, soigne les morts-vivants"}.
            </p>
          )}
          <p>
            <strong>{divin.canalisation.des}d6</strong> · DD {divin.canalisation.dd} (Volonté, demi-dégâts) · {divin.canalisation.utilisations} utilisation(s) par jour
            · rayon 9 m, action simple.
          </p>
        </section>
      )}
      {divin.domaines.map((d) => (
        <section className="carte" key={d.id}>
          <h3>Domaine : {d.nom}</h3>
          <p className="doux">{d.description}</p>
          {d.pouvoirs.map((p) => (
            <details key={p.nom} open={p.actif}>
              <summary>
                <strong>{p.nom}</strong> ({p.genre}) — {p.actif ? "disponible" : `niveau ${p.niveau}`}
                {p.actif && p.utilisations !== null ? ` · ${p.utilisations} fois par jour` : ""}
              </summary>
              <p>{p.texte}</p>
            </details>
          ))}
          <p className="doux">
            Sorts de domaine : {d.sorts.map((s) => `${s.niveau}e : ${s.nom}${s.note ? ` (${s.note})` : ""}${s.accessible ? "" : " (pas encore)"}`).join(" ; ")}
          </p>
        </section>
      ))}
    </div>
  );
}

// ------------------------------------------------------------------ fiche : sorts préparés

export function PanneauPreparation({
  f,
  cat,
  agir,
}: {
  f: FicheVue;
  cat: CatalogueCreation | null;
  agir: (e: import("../../fiche-types").EvenementFiche) => void;
}) {
  const prep = f.preparation;
  const [edition, setEdition] = useState(false);
  const [brouillon, setBrouillon] = useState<SortPrepare[]>([]);
  const [conversion, setConversion] = useState<{ niveau: number } | null>(null);
  const [soin, setSoin] = useState("");
  if (!prep) return null;
  const liste = cat?.sortsParClasse[prep.classeId] ?? [];
  const restants = (n: number) => prep.capacites.find((c) => c.niveau === n)?.restants ?? null;
  const debut = prep.conversion === false ? "Inflict " : "Cure ";
  const sortsDeConversion = (niveau: number) =>
    liste.filter((s) => s.nomEn.startsWith(debut) && s.niveau >= 1 && s.niveau <= niveau);
  const niveaux = [...new Set(prep.prepares.map((s) => s.niveau))].sort((a, b) => a - b);

  if (edition && cat) {
    return (
      <div className="section">
        <h3>Préparer les sorts du jour</h3>
        <EditeurPreparation prep={prep} liste={liste} valeur={brouillon} onChange={setBrouillon} />
        <div className="ligne">
          <button
            className="bouton principal"
            onClick={() => {
              agir({ Preparation: { classe: prep.classeId, sorts: brouillon } });
              setEdition(false);
            }}
          >
            Valider la préparation
          </button>
          <button className="bouton" onClick={() => setEdition(false)}>Annuler</button>
        </div>
      </div>
    );
  }

  return (
    <div className="section">
      <div className="section-tete">
        <h2>Sorts préparés ({prep.classe})</h2>
        <span className="espace" />
        <button
          className="bouton"
          onClick={() => {
            setBrouillon(prep.prepares.map((s) => ({ niveau: s.niveau, sort: s.nomEn, domaine: s.domaine })));
            setEdition(true);
          }}
        >
          {prep.prepares.length ? "Modifier la préparation" : "Préparer les sorts"}
        </button>
        <button className="bouton" onClick={() => agir("Repos")}>Repos : rendre les sorts</button>
      </div>
      <div className="grille-fiche">
        {prep.capacites.map((c) => (
          <div className="carte chiffre-fiche" key={c.niveau}>
            <div className="chiffre-bouton">
              <span className="etiquette">{c.niveau === 0 ? "Oraisons" : `Niveau ${c.niveau}`}</span>
              <span className="valeur">{c.niveau === 0 ? c.normaux : `${c.restants ?? c.normaux + c.domaine}/${c.normaux + c.domaine}`}</span>
            </div>
            <span className="doux">{c.domaine ? "dont 1 de domaine" : " "}</span>
          </div>
        ))}
      </div>
      {prep.prepares.length === 0 && <p className="doux">Aucun sort préparé : préparez vos sorts pour la journée.</p>}
      {niveaux.map((n) => (
        <table className="tableau" key={n}>
          <thead>
            <tr>
              <th>{n === 0 ? "Oraisons" : `Niveau ${n}`}</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {prep.prepares
              .filter((s) => s.niveau === n)
              .map((s, i) => (
                <tr key={i} className={s.utilise ? "ignore" : ""}>
                  <td>
                    {s.nom}
                    {s.domaine && <small className="doux"> (domaine)</small>}
                    {s.utilise && <small className="doux"> — lancé</small>}
                  </td>
                  <td>
                    {n > 0 && !s.utilise && (
                      <button
                        className="bouton discret"
                        onClick={() => agir({ SortLance: { classe: prep.classeId, niveau_de_sort: s.niveau, sort: s.nomEn } })}
                      >
                        Lancer
                      </button>
                    )}
                    {n > 0 && !s.utilise && !s.domaine && prep.conversion !== null && (
                      <button className="bouton discret" onClick={() => setConversion({ niveau: n })}>
                        {prep.conversion ? "Convertir en soins" : "Convertir en blessures"}
                      </button>
                    )}
                  </td>
                </tr>
              ))}
          </tbody>
        </table>
      ))}
      {conversion && (
        <div className="carte">
          <h3>
            Sacrifier un sort de niveau {conversion.niveau} pour lancer {prep.conversion ? "un sort de soins" : "un sort de blessure"}
          </h3>
          <div className="ligne">
            <Choix
              etiquette="Sort de conversion"
              vide="Choisir le sort…"
              valeur={soin}
              options={sortsDeConversion(conversion.niveau).map((s) => ({ v: s.nomEn, l: `${s.nom} (niveau ${s.niveau})` }))}
              onChange={setSoin}
            />
            <button
              className="bouton principal"
              disabled={!soin || restants(conversion.niveau) === 0}
              onClick={() => {
                agir({ SortConverti: { classe: prep.classeId, niveau_sacrifie: conversion.niveau, sort: soin } });
                setConversion(null);
                setSoin("");
              }}
            >
              Lancer
            </button>
            <button className="bouton" onClick={() => setConversion(null)}>Annuler</button>
          </div>
        </div>
      )}
    </div>
  );
}
