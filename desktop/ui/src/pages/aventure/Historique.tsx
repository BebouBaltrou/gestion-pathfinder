// Combats passés, statistiques par personnage et export d'un combat.

import { useCallback, useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import type { HistoriqueVue } from "../../aventure-types";

export function Historique({ outils }: { outils: Outils }) {
  const [h, setH] = useState<HistoriqueVue | null>(null);
  const [exports, setExports] = useState<Record<number, string[]>>({});
  const [erreur, setErreur] = useState<string | null>(null);
  const charger = useCallback(() => {
    api.combatHistorique().then(setH);
  }, []);
  useEffect(charger, [charger, outils.version]);

  if (!h || h.combats.length === 0) return null;

  const exporter = async (numero: number) => {
    setErreur(null);
    try {
      const chemins = await api.combatExporter(numero);
      setExports({ ...exports, [numero]: chemins });
    } catch (e) {
      setErreur(String(e));
    }
  };

  return (
    <section className="carte" style={{ display: "flex", flexDirection: "column", gap: 16 }}>
      <div className="section-tete">
        <h2>Combats passés et statistiques</h2>
        <span className="aide">
          {h.combats.length} combat(s), {h.totalRounds} round(s) au total. Les dégâts infligés sont ceux que vous avez notés à la main.
        </span>
      </div>
      <table className="tableau-bilan">
        <thead>
          <tr>
            <th>Personnage</th>
            <th>Combats</th>
            <th>Dégâts subis</th>
            <th>Soins reçus</th>
            <th>Dégâts infligés</th>
            <th>Sorts lancés</th>
            <th>Objets utilisés</th>
          </tr>
        </thead>
        <tbody>
          {h.statistiques.map((s) => (
            <tr key={s.id}>
              <td>{s.nom}</td>
              <td>{s.combats}</td>
              <td>{s.degatsSubis}</td>
              <td>{s.soinsRecus}</td>
              <td>{s.degatsInfliges}</td>
              <td>{s.sortsLances}</td>
              <td>{s.objetsUtilises}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="liste-compacte">
        {[...h.combats].reverse().map((c) => (
          <div key={c.numero} className="ligne-objet">
            <span className="nom">
              <strong>Combat n° {c.numero}</strong> — jour {c.jour}, fin à {c.heureDeFin} — {c.resume}
              {exports[c.numero] && (
                <span className="doux" style={{ display: "block" }}>
                  Exporté : {exports[c.numero].join("  ·  ")}
                </span>
              )}
            </span>
            <button className="bouton mini" onClick={() => exporter(c.numero)} title="Écrit un fichier lisible (.md) et un fichier des actions avec les dés (.json)">
              Exporter
            </button>
          </div>
        ))}
      </div>
      {erreur && <div className="avertissement">{erreur}</div>}
    </section>
  );
}
