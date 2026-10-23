import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, PageHistorique } from "../api";

export function Historique({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageHistorique | null>(null);

  useEffect(() => {
    api.pageHistorique().then(setP);
  }, [outils.version]);

  if (!p) return null;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Historique</h1>
        <p>Toutes les opérations, dans l'ordre. Annuler (Ctrl+Z) retire la dernière ; Rétablir (Ctrl+Y) la remet.</p>
      </div>

      {p.annulees.length > 0 && (
        <section className="section">
          <h2>Annulées (peuvent être rétablies)</h2>
          <div className="carte pile">
            {p.annulees.map((e) => (
              <span className="doux" style={{ textDecoration: "line-through" }} key={e.numero}>
                #{e.numero} · jour {e.jour} · {e.texte}
              </span>
            ))}
          </div>
        </section>
      )}

      <section className="section">
        <h2>Journal ({p.journal.length} opérations)</h2>
        {p.journal.length === 0 ? (
          <p className="doux">Rien pour l'instant.</p>
        ) : (
          <div className="carte tableau-repartition">
            {p.journal.map((e) => (
              <div className="ligne-journal" key={e.numero}>
                <span className="doux">#{e.numero}</span>
                <span className="doux">Jour {e.jour} — {e.date}</span>
                <span>{e.texte}</span>
              </div>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
