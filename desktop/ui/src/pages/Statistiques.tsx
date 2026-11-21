import { CSSProperties, useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, CourbesPatrimoine, PageStatistiques, Somme } from "../api";
import { Anneau, Compteur, GraphiqueComposition, GraphiqueCourbes, PALETTE, Spark, fmt, useMonte } from "../composants/GraphiquesStats";
import "../statistiques.css";

const texte = (s: Somme) => s?.texte ?? "—";
const css = (o: Record<string, string>) => o as unknown as CSSProperties;
const po = (s: Somme) => (s?.pc ?? 0) / 100;

/** Variation d'une courbe sur les 30 derniers jours (ou depuis le début si la campagne est plus courte). */
function variation(c: CourbesPatrimoine | null, cle: string): { pct: number; depuis: string } | null {
  const s = c?.series.find((x) => x.cle === cle);
  if (!c || !s || c.jours.length < 2) return null;
  const fin = c.jours.length - 1;
  const cible = c.jours[fin] - 30;
  let base = 0;
  c.jours.forEach((j, k) => { if (j <= cible) base = k; });
  const avant = s.valeurs[base];
  if (avant <= 0) return null;
  return { pct: (s.valeurs[fin] / avant - 1) * 100, depuis: c.jours[0] > cible ? "depuis le début" : "sur 30 jours" };
}

function sommeSeries(c: CourbesPatrimoine | null, groupe: string): number[] {
  if (!c) return [];
  const ss = c.series.filter((s) => s.groupe === groupe);
  return c.jours.map((_, k) => ss.reduce((a, s) => a + s.valeurs[k], 0));
}

export function Statistiques({ outils }: { outils: Outils }) {
  const [s, setS] = useState<PageStatistiques | null>(null);
  const [courbes, setCourbes] = useState<CourbesPatrimoine | null>(null);
  const monte = useMonte(150);

  useEffect(() => {
    api.statistiques().then((r) => setS(r && Array.isArray(r.personnages) && Array.isArray(r.repartition) ? r : null));
    api.courbesPatrimoine().then((r) => setCourbes(r && Array.isArray(r.series) && r.series.length > 0 ? r : null));
  }, [outils.version]);

  if (!s) return null;

  const evol = variation(courbes, "total");
  const maxBat = Math.max(1, ...s.batiments.map((b) => b.valeur.pc));
  const maxOrg = Math.max(1, ...s.organisations.map((o) => o.valeurEquipes.pc));
  const maxVille = Math.max(1, ...s.localites.map((l) => Math.max(0, l.tresorerie.pc) + Math.max(0, l.capitaux.pc) + Math.max(0, l.batiments.pc)));

  // plus fortes hausses : personnages et villes, depuis 30 jours
  const hausses = (() => {
    if (!courbes || courbes.jours.length < 2) return [];
    const fin = courbes.jours.length - 1;
    const cible = courbes.jours[fin] - 30;
    let base = 0;
    courbes.jours.forEach((j, k) => { if (j <= cible) base = k; });
    return courbes.series
      .map((x, i) => ({ nom: x.nom, couleur: PALETTE[i % PALETTE.length], groupe: x.groupe, d: x.valeurs[fin] - x.valeurs[base], p: x.valeurs[base] > 0 ? (x.valeurs[fin] / x.valeurs[base] - 1) * 100 : null }))
      .filter((x) => (x.groupe === "Personnages" || x.groupe === "Localités") && x.d > 0)
      .sort((a, b) => b.d - a.d)
      .slice(0, 4);
  })();
  const maxHausse = Math.max(1, ...hausses.map((h) => h.d));
  const serieBat = courbes?.series.find((x) => x.cle === "batiments")?.valeurs ?? [];

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Statistiques</h1>
        <p>Patrimoine, composition et évolution de la campagne</p>
      </div>

      <section className="carte st-carte st-hero" aria-label="Patrimoine total">
        <div>
          <div className="st-etq">Patrimoine total de la campagne</div>
          <div className="st-gros"><Compteur valeur={po(s.patrimoineTotal)} /><small>po</small></div>
          {evol && (
            <span className={`st-delta ${evol.pct >= 0 ? "hausse" : "baisse"}`}>
              {evol.pct >= 0 ? "▲ +" : "▼ "}{evol.pct.toFixed(1).replace(".", ",")} % {evol.depuis}
            </span>
          )}
          <div className="st-spark">
            <Spark valeurs={(courbes?.series.find((x) => x.cle === "total")?.valeurs ?? []).map((v) => v / 100)} couleur="var(--po)" />
          </div>
        </div>
        <Anneau parts={s.repartition.map((p) => ({ cle: p.cle, nom: p.nom, pc: p.valeur.pc }))} centre={s.patrimoineTotal.pc} />
      </section>

      <section className="st-grille st-g4" aria-label="Chiffres clés">
        <div className="carte st-kpi" style={css({ "--c": "var(--po)" })}>
          <div className="st-etq">Personnages</div>
          <div className="st-gros"><Compteur valeur={po(s.totalPersonnages)} suffixe=" po" /></div>
          <div className="st-det">Comptes, objets et bâtiments possédés</div>
          <div className="st-spark"><Spark valeurs={sommeSeries(courbes, "Personnages").map((v) => v / 100)} couleur="var(--po)" /></div>
        </div>
        <div className="carte st-kpi" style={css({ "--c": "var(--travail)" })}>
          <div className="st-etq">Localités</div>
          <div className="st-gros"><Compteur valeur={po(s.totalLocalites)} suffixe=" po" /></div>
          <div className="st-det">Trésoreries, capitaux et bâtiments</div>
          <div className="st-spark"><Spark valeurs={sommeSeries(courbes, "Localités").map((v) => v / 100)} couleur="var(--travail)" /></div>
        </div>
        <div className="carte st-kpi" style={css({ "--c": "var(--marchandises)" })}>
          <div className="st-etq">Bâtiments</div>
          <div className="st-gros"><Compteur valeur={po(s.totalBatiments)} suffixe=" po" /></div>
          <div className="st-det">{s.batiments.length} bâtiment{s.batiments.length > 1 ? "s" : ""} · {s.batiments.reduce((a, b) => a + b.salles, 0)} salle(s) construite(s)</div>
          <div className="st-spark"><Spark valeurs={serieBat.map((v) => v / 100)} couleur="var(--marchandises)" /></div>
        </div>
        <div className="carte st-kpi" style={css({ "--c": "var(--influence)" })}>
          <div className="st-etq">Équipes</div>
          <div className="st-gros"><Compteur valeur={s.effectifTotal} suffixe=" employés" /></div>
          <div className="st-det">+{s.effectifAVenirTotal} à venir · valeur {texte(s.valeurEquipes)}</div>
        </div>
      </section>

      {courbes && (
        <div style={{ marginTop: 18 }}>
          <GraphiqueCourbes key={courbes.series.length} courbes={courbes} />
        </div>
      )}

      <section className="st-grille st-g2">
        {courbes && courbes.compositions.length > 0 ? <GraphiqueComposition courbes={courbes} /> : <div />}
        <div className="carte st-carte">
          <div className="st-entete">
            <div>
              <h2>Patrimoine par ville</h2>
              <p>Trésorerie, capitaux, bâtiments</p>
            </div>
          </div>
          {s.localites.length === 0 && <p className="st-vide">Aucune localité.</p>}
          {s.localites.map((l) => {
            const parts: [string, number][] = [["var(--po)", l.tresorerie.pc], ["var(--magie)", l.capitaux.pc], ["var(--marchandises)", l.batiments.pc]];
            return (
              <div className="st-ville" key={l.id}>
                <div className="t"><b>{l.nom}</b><span>{texte(l.total)}</span></div>
                <div className="st-pile">
                  {parts.map(([c, v]) => (
                    <span key={c} style={{ background: c, color: c, width: monte ? `${(Math.max(0, v) / maxVille) * 100}%` : "0%" }} title={`${fmt(v / 100)} po`} />
                  ))}
                </div>
              </div>
            );
          })}
          <div className="st-mini">
            <span><i style={{ background: "var(--po)" }} />Trésorerie</span>
            <span><i style={{ background: "var(--magie)" }} />Capitaux</span>
            <span><i style={{ background: "var(--marchandises)" }} />Bâtiments</span>
          </div>
          {hausses.length > 0 && (
            <>
              <h2 className="st-sous-titre">Plus fortes hausses {variation(courbes, "total")?.depuis ?? "sur 30 jours"}</h2>
              <div style={{ display: "grid", gap: 14 }}>
                {hausses.map((h) => (
                  <div className="st-ligne-c" key={h.nom} style={css({ "--c": h.couleur })}>
                    <div className="t"><span>{h.nom}</span><b>+{fmt(h.d / 100)} po{h.p !== null ? ` · +${h.p.toFixed(0)} %` : ""}</b></div>
                    <div className="st-barre"><span style={{ width: monte ? `${(h.d / maxHausse) * 100}%` : "0%" }} /></div>
                  </div>
                ))}
              </div>
            </>
          )}
        </div>
      </section>

      <section className="st-grille st-g2e">
        <div className="carte st-carte">
          <div className="st-entete">
            <div>
              <h2>Bâtiments</h2>
              <p>Prix d'achat des salles et extensions construites</p>
            </div>
          </div>
          {s.batiments.length === 0 ? <p className="st-vide">Aucun bâtiment.</p> : (
            <table className="st-table">
              <thead><tr><th>Bâtiment</th><th>Ville</th><th>Propriétaire</th><th className="n">Valeur</th></tr></thead>
              <tbody>
                {s.batiments.map((b) => (
                  <tr key={b.id}>
                    <td>{b.nom}</td>
                    <td>{b.localite}</td>
                    <td>{b.proprietaire ? <span className="st-pastille">{b.proprietaire}</span> : <span className="st-pastille aucun">Aucun</span>}</td>
                    <td className="n">
                      <div className="st-val">
                        <span className="b"><i style={{ background: "var(--marchandises)", width: monte ? `${(b.valeur.pc / maxBat) * 100}%` : "0%" }} /></span>
                        {texte(b.valeur)}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
        <div className="carte st-carte">
          <div className="st-entete">
            <div>
              <h2>Organisations</h2>
              <p>Effectif et valeur des équipes</p>
            </div>
          </div>
          {s.organisations.length === 0 ? <p className="st-vide">Aucune organisation.</p> : (
            <table className="st-table">
              <thead><tr><th>Organisation</th><th className="n">Effectif</th><th className="n">À venir</th><th className="n">Valeur des équipes</th></tr></thead>
              <tbody>
                {s.organisations.map((o) => (
                  <tr key={o.id}>
                    <td>{o.nom}<div className="st-det" style={{ fontSize: 12, color: "var(--texte-doux)" }}>{o.localite}</div></td>
                    <td className="n">{o.effectif}</td>
                    <td className="n">{o.effectifAVenir}</td>
                    <td className="n">
                      <div className="st-val">
                        <span className="b"><i style={{ background: "var(--travail)", width: monte ? `${(o.valeurEquipes.pc / maxOrg) * 100}%` : "0%" }} /></span>
                        {texte(o.valeurEquipes)}
                      </div>
                    </td>
                  </tr>
                ))}
                <tr className="total"><td>Total</td><td className="n">{s.effectifTotal}</td><td className="n">{s.effectifAVenirTotal}</td><td className="n">{texte(s.valeurEquipes)}</td></tr>
              </tbody>
            </table>
          )}
        </div>
      </section>

      <details className="carte st-carte st-details" style={{ marginTop: 18 }}>
        <summary>Voir les chiffres détaillés</summary>
        <table className="st-table">
          <thead><tr><th>Personnage</th><th className="n">Compte</th><th className="n">Objets</th><th className="n">Bâtiments</th><th className="n">Total</th></tr></thead>
          <tbody>
            {s.personnages.map((p) => (
              <tr key={p.id}><td>{p.nom}</td><td className="n">{texte(p.compte)}</td><td className="n">{texte(p.objets)}</td><td className="n">{texte(p.batiments)}</td><td className="n">{texte(p.total)}</td></tr>
            ))}
          </tbody>
        </table>
        <table className="st-table" style={{ marginTop: 18 }}>
          <thead><tr><th>Localité</th><th className="n">Trésorerie</th><th className="n">Capitaux</th><th className="n">Bâtiments</th><th className="n">Total</th></tr></thead>
          <tbody>
            {s.localites.map((l) => (
              <tr key={l.id}><td>{l.nom}</td><td className="n">{texte(l.tresorerie)}</td><td className="n">{texte(l.capitaux)}</td><td className="n">{texte(l.batiments)}</td><td className="n">{texte(l.total)}</td></tr>
            ))}
          </tbody>
        </table>
        <p className="st-det" style={{ marginTop: 12, fontSize: 12.5, color: "var(--texte-doux)" }}>
          Objets hors personnages : {texte(s.objetsAutres)}. Un bâtiment possédé compte chez son propriétaire et dans sa ville, mais une seule fois dans le total.
        </p>
      </details>
    </div>
  );
}
