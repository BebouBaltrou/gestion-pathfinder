import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, PagePoints } from "../api";

export function Points({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PagePoints | null>(null);

  useEffect(() => {
    api.pagePoints().then(setP);
  }, [outils.version]);

  if (!p) return null;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Points à confirmer</h1>
        <p>Règles ambiguës ou incohérentes du livre, et la lecture appliquée par l'application</p>
      </div>

      {p.avertissementsMasques && (
        <p className="doux">Les avertissements sont masqués dans les calculs (Paramètres), mais restent listés ici.</p>
      )}

      {p.controles.length > 0 && (
        <section className="section">
          <h2>Contrôles du référentiel</h2>
          {p.controles.map((c) => (
            <div className="alerte" key={c}>
              <span className="point" />
              <span className="texte">{c}</span>
            </div>
          ))}
        </section>
      )}

      <section className="section">
        <h2>{p.total} points, dont {p.ouverts} ouvert(s)</h2>
        {p.points.map((x) => (
          <article className="carte pile" key={x.code}>
            <div className="ligne" style={{ gap: 12 }}>
              <strong style={{ color: "var(--or)" }}>{x.code}</strong>
              <span
                className="badge"
                style={{
                  marginLeft: 0,
                  color: x.ouvert ? "var(--orange)" : "var(--vert)",
                  border: `1px solid ${x.ouvert ? "var(--orange)" : "var(--vert)"}`,
                }}
              >
                {x.ouvert ? "ouvert" : "validé"}
              </span>
              <span className="doux">{x.source}</span>
            </div>
            <span>{x.question}</span>
            <span style={{ color: "var(--or-clair)" }}>Lecture appliquée : {x.lectureAppliquee}</span>
            {x.decision && <span className="doux">{x.decision}</span>}
          </article>
        ))}
      </section>
    </div>
  );
}
