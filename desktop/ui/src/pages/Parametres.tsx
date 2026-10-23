import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, PageParametres } from "../api";

export function Parametres({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageParametres | null>(null);

  useEffect(() => {
    api.pageParametres().then(setP);
  }, [outils.version]);

  if (!p) return null;
  const cal = p.calendrier;
  const changerCalendrier = (c: Partial<{ jourSemaine: number; jour: number; mois: number; annee: number; bissextile: "quatre" | "huit" }>) => {
    const n = { jourSemaine: cal.jourSemaine, jour: cal.jour, mois: cal.mois, annee: cal.annee, bissextile: cal.bissextile, ...c };
    outils.agir(api.reglerCalendrier(n.jourSemaine, n.jour, n.mois, n.annee, n.bissextile));
  };

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Paramètres</h1>
        <p>Campagne « {p.campagne} »</p>
      </div>

      <section className="section">
        <h2>Règles</h2>
        {p.reglages.map((r) => (
          <div className="carte pile" key={r.id}>
            <div className="ligne" style={{ gap: 10 }}>
              <label className="choix-radio" style={{ fontWeight: 600 }}>
                <input type="checkbox" checked={r.actif} onChange={(e) => outils.agir(api.regler(r.id, e.target.checked))} /> {r.titre}
              </label>
              {r.statut && (
                <span
                  className={"badge " + (r.statut === "personnalise" ? "perso" : "")}
                  style={r.statut === "personnalise" ? undefined : { border: "1px solid var(--bord-fort)", marginLeft: 0 }}
                >
                  {r.statut === "personnalise" ? "Personnalisé" : "Officiel"}
                </span>
              )}
              {r.point && <span className="badge" style={{ color: "var(--vert)", border: "1px solid var(--vert)" }}>{r.point}</span>}
            </div>
            <span className="doux">{r.aide}</span>
          </div>
        ))}
      </section>

      <section className="section">
        <h2>Calendrier</h2>
        <div className="carte pile">
          <div className="ligne operation" style={{ gap: 12 }}>
            <span>Le jour 1 de la campagne est le</span>
            <select className="champ petit" aria-label="Jour de la semaine du jour 1" value={cal.jourSemaine} onChange={(e) => changerCalendrier({ jourSemaine: Number(e.target.value) })}>
              {cal.joursSemaine.map((j, i) => (
                <option key={j} value={i}>{j}</option>
              ))}
            </select>
            <input
              className="champ mini"
              type="number"
              min={1}
              max={31}
              aria-label="Jour du mois"
              defaultValue={cal.jour}
              key={`j${cal.jour}`}
              onBlur={(e) => {
                const v = Math.max(1, Math.min(31, Number(e.target.value) || 1));
                if (v !== cal.jour) changerCalendrier({ jour: v });
              }}
            />
            <select className="champ petit" aria-label="Mois" value={cal.mois} onChange={(e) => changerCalendrier({ mois: Number(e.target.value) })}>
              {cal.nomsMois.map((m, i) => (
                <option key={m} value={i + 1}>{m}</option>
              ))}
            </select>
            <input
              className="champ mini"
              type="number"
              min={1}
              max={9999}
              aria-label="Année"
              defaultValue={cal.annee}
              key={`a${cal.annee}`}
              onBlur={(e) => {
                const v = Math.max(1, Math.min(9999, Number(e.target.value) || cal.annee));
                if (v !== cal.annee) changerCalendrier({ annee: v });
              }}
            />
            <span>AR</span>
          </div>
          <div className="ligne" style={{ gap: 12 }}>
            <label className="doux" htmlFor="cal-bissextile">Années bissextiles</label>
            <select id="cal-bissextile" className="champ moyen" value={cal.bissextile} onChange={(e) => changerCalendrier({ bissextile: e.target.value as "quatre" | "huit" })}>
              <option value="quatre">tous les 4 ans (PF2)</option>
              <option value="huit">tous les 8 ans (PF1)</option>
            </select>
            <span className="badge" style={{ color: "var(--vert)", border: "1px solid var(--vert)" }}>PC-13</span>
          </div>
          <span className="doux">
            Mois et jours de la semaine : calendrier de Golarion (VF). Aucune source ne fixe le jour de la semaine d'une date donnée : choisissez celui du jour 1.
          </span>
        </div>
        <p className="doux">{cal.apercu}</p>
      </section>
    </div>
  );
}
