import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api } from "../api";
import { Apparence } from "../composants/Apparence";
import type { RegleInfo } from "../fiche-types";

/** Registre des règles optionnelles de la campagne : description, impact, valeur par défaut. */
export function Parametres({ outils }: { outils: Outils }) {
  const [regles, setRegles] = useState<RegleInfo[]>([]);

  useEffect(() => {
    api.reglesCampagne().then(setRegles);
  }, [outils.version]);

  const groupes: [string, RegleInfo[]][] = [
    ["Personnages", regles.filter((r) => r.id.startsWith("pj."))],
    ["Intermède (bâtiments, organisations, recettes)", regles.filter((r) => r.id.startsWith("inter."))],
  ];

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Paramètres de la campagne</h1>
        <p>
          Chaque règle optionnelle de cette campagne, ce qu'elle change et sa valeur d'origine. Les changements des
          personnages sont enregistrés dans l'historique et peuvent être annulés.
        </p>
      </div>
      <Apparence />
      {groupes.map(([titre, liste]) => (
        <section className="section" key={titre}>
          <h2>{titre}</h2>
          <div className="liste-regles">
            {liste.map((r) => (
              <div className="carte regle" key={r.id}>
                <div className="regle-texte">
                  <div className="regle-nom">
                    <strong>{r.nom}</strong>
                    <span className={"puce" + (r.officielle ? "" : " or")}>
                      {r.officielle ? "règle du livre" : "règle personnalisée"}
                    </span>
                  </div>
                  <p className="doux">{r.description}</p>
                  <p className="doux">
                    <em>Ce que ça change :</em> {r.impact}
                  </p>
                  {r.valeur !== r.defaut && (
                    <p className="doux">
                      Valeur d'origine : {r.choix.find(([v]) => v === r.defaut)?.[1] ?? r.defaut}{" "}
                      <button className="bouton discret" onClick={() => outils.agir(api.changerRegle(r.id, r.defaut))}>
                        Rétablir
                      </button>
                    </p>
                  )}
                </div>
                <select
                  className="champ choix"
                  aria-label={r.nom}
                  value={r.valeur}
                  onChange={(e) => outils.agir(api.changerRegle(r.id, e.target.value))}
                >
                  {r.choix.map(([v, l]) => (
                    <option key={v} value={v}>
                      {l}
                    </option>
                  ))}
                </select>
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
