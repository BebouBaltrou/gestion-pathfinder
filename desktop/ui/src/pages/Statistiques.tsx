import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, CourbesPatrimoine, PageStatistiques, Somme } from "../api";
import { GraphiqueCourbes } from "../composants/CourbesPatrimoine";

const texte = (s: Somme) => s?.texte ?? "—";

type Serie = { nom: string; couleur: string; valeur: number };
type Barre = { label: string; series: Serie[] };

/** Barres verticales empilées, avec axes : une colonne par élément, les séries empilées (sans chevauchement). */
function GraphiqueEmpile({ titre, barres }: { titre: string; barres: Barre[] }) {
  const W = 680, H = 280;
  const M = { haut: 16, droite: 16, bas: 64, gauche: 72 };
  const largeur = W - M.gauche - M.droite;
  const hauteur = H - M.haut - M.bas;
  const totaux = barres.map((b) => b.series.reduce((s, x) => s + Math.max(0, x.valeur), 0));
  const max = Math.max(1, ...totaux);
  const y = (v: number) => M.haut + hauteur - (v / max) * hauteur;
  const graduations = [0, 0.25, 0.5, 0.75, 1].map((f) => f * max);
  const slot = barres.length ? largeur / barres.length : largeur;
  const largeurBarre = Math.min(56, slot * 0.6);
  const legende = barres[0]?.series ?? [];

  return (
    <section className="carte">
      <h2>{titre}</h2>
      {barres.length === 0 ? (
        <p className="detail">Aucune donnée.</p>
      ) : (
        <>
          <svg viewBox={`0 0 ${W} ${H}`} width="100%" role="img" aria-label={titre}>
            {graduations.map((g) => (
              <g key={g}>
                <line x1={M.gauche} x2={W - M.droite} y1={y(g)} y2={y(g)} stroke="var(--bord-fort)" strokeDasharray="3 3" />
                <text x={M.gauche - 8} y={y(g) + 4} textAnchor="end" fontSize="11" fill="currentColor">
                  {Math.round(g / 100).toLocaleString("fr-FR")} po
                </text>
              </g>
            ))}
            <line x1={M.gauche} x2={M.gauche} y1={M.haut} y2={M.haut + hauteur} stroke="currentColor" />
            <line x1={M.gauche} x2={W - M.droite} y1={M.haut + hauteur} y2={M.haut + hauteur} stroke="currentColor" />
            {barres.map((b, i) => {
              const cx = M.gauche + slot * i + slot / 2;
              let cumul = 0;
              return (
                <g key={b.label + i}>
                  {b.series.map((s) => {
                    const v = Math.max(0, s.valeur);
                    const haut = y(cumul + v);
                    const bas = y(cumul);
                    cumul += v;
                    return (
                      <rect key={s.nom} x={cx - largeurBarre / 2} y={haut} width={largeurBarre} height={Math.max(0, bas - haut)} fill={s.couleur}>
                        <title>{`${b.label} — ${s.nom} : ${Math.round(v / 100).toLocaleString("fr-FR")} po`}</title>
                      </rect>
                    );
                  })}
                  <text x={cx} y={M.haut + hauteur + 16} textAnchor="end" fontSize="11" fill="currentColor" transform={`rotate(-30 ${cx} ${M.haut + hauteur + 16})`}>
                    {b.label.length > 14 ? `${b.label.slice(0, 13)}…` : b.label}
                  </text>
                </g>
              );
            })}
          </svg>
          <div className="legende-graphique" style={{ display: "flex", gap: 16, flexWrap: "wrap", fontSize: 13 }}>
            {legende.map((s) => (
              <span key={s.nom}>
                <span style={{ display: "inline-block", width: 12, height: 12, background: s.couleur, marginRight: 6, borderRadius: 2 }} />
                {s.nom}
              </span>
            ))}
          </div>
        </>
      )}
    </section>
  );
}

export function Statistiques({ outils }: { outils: Outils }) {
  const [s, setS] = useState<PageStatistiques | null>(null);
  const [courbes, setCourbes] = useState<CourbesPatrimoine | null>(null);

  useEffect(() => {
    api.statistiques().then((r) => setS(r && Array.isArray(r.personnages) ? r : null));
    api.courbesPatrimoine().then((r) => setCourbes(r && Array.isArray(r.series) && r.series.length > 0 ? r : null));
  }, [outils.version]);

  if (!s) return null;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Statistiques</h1>
        <p>Patrimoine, objets, localités et organisations</p>
      </div>

      <section className="grille-chiffres" aria-label="Chiffres clés">
        <div className="carte chiffre">
          <span className="etiquette">Patrimoine total</span>
          <span className="valeur" style={{ color: "var(--po)" }}>{texte(s.patrimoineTotal)}</span>
          <span className="detail">Comptes, objets, trésoreries et capitaux</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Personnages</span>
          <span className="valeur">{texte(s.totalPersonnages)}</span>
          <span className="detail">Comptes et objets des PJ</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Localités</span>
          <span className="valeur">{texte(s.totalLocalites)}</span>
          <span className="detail">Trésoreries, capitaux et bâtiments</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Bâtiments</span>
          <span className="valeur">{texte(s.totalBatiments)}</span>
          <span className="detail">Salles et extensions construites</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Objets hors PJ</span>
          <span className="valeur">{texte(s.objetsAutres)}</span>
          <span className="detail">Autres inventaires</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Effectif des organisations</span>
          <span className="valeur">{s.effectifTotal}</span>
          <span className="detail">{s.effectifAVenirTotal} à recruter ou en recrutement</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Valeur des équipes</span>
          <span className="valeur">{texte(s.valeurEquipes)}</span>
          <span className="detail">Prix imprimés d'achat des équipes</span>
        </div>
      </section>

      {courbes && <GraphiqueCourbes key={courbes.series.length} courbes={courbes} />}
      <GraphiqueEmpile
        titre="Patrimoine par personnage (po)"
        barres={s.personnages.map((p) => ({
          label: p.nom,
          series: [
            { nom: "Compte", couleur: "var(--po)", valeur: p.compte.pc },
            { nom: "Objets", couleur: "var(--influence)", valeur: p.objets.pc },
          ],
        }))}
      />
      <GraphiqueEmpile
        titre="Patrimoine par localité (po)"
        barres={s.localites.map((l) => ({
          label: l.nom,
          series: [
            { nom: "Trésorerie", couleur: "var(--po)", valeur: l.tresorerie.pc },
            { nom: "Capitaux", couleur: "var(--travail)", valeur: l.capitaux.pc },
            { nom: "Bâtiments", couleur: "var(--marchandises)", valeur: l.batiments.pc },
          ],
        }))}
      />
      <GraphiqueEmpile
        titre="Valeur des bâtiments (po)"
        barres={s.batiments.map((b) => ({
          label: b.nom,
          series: [{ nom: "Salles et extensions", couleur: "var(--marchandises)", valeur: b.valeur.pc }],
        }))}
      />
      <GraphiqueEmpile
        titre="Valeur des équipes par organisation (po)"
        barres={s.organisations.map((o) => ({
          label: o.nom,
          series: [{ nom: "Équipes", couleur: "var(--travail)", valeur: o.valeurEquipes.pc }],
        }))}
      />

      <section className="carte">
        <h2>Détail des personnages</h2>
        <table className="tableau">
          <thead>
            <tr><th>Nom</th><th>Compte</th><th>Objets</th><th>Total</th></tr>
          </thead>
          <tbody>
            {s.personnages.map((p) => (
              <tr key={p.id}><td>{p.nom}</td><td>{texte(p.compte)}</td><td>{texte(p.objets)}</td><td>{texte(p.total)}</td></tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="carte">
        <h2>Détail des localités</h2>
        <table className="tableau">
          <thead>
            <tr><th>Nom</th><th>Trésorerie</th><th>Capitaux</th><th>Bâtiments</th><th>Total</th></tr>
          </thead>
          <tbody>
            {s.localites.map((l) => (
              <tr key={l.id}><td>{l.nom}</td><td>{texte(l.tresorerie)}</td><td>{texte(l.capitaux)}</td><td>{texte(l.batiments)}</td><td>{texte(l.total)}</td></tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="carte">
        <h2>Organisations</h2>
        <table className="tableau">
          <thead>
            <tr><th>Organisation</th><th>Localité</th><th>Équipes</th><th>Effectif</th><th>À venir</th><th>Gestionnaires</th><th>Valeur des équipes</th></tr>
          </thead>
          <tbody>
            {s.organisations.map((o) => (
              <tr key={o.id}>
                <td>{o.nom}</td><td>{o.localite}</td><td>{o.equipes}</td><td>{o.effectif}</td>
                <td>{o.effectifAVenir}</td><td>{o.gestionnaires}</td><td>{texte(o.valeurEquipes)}</td>
              </tr>
            ))}
            <tr className="total">
              <td colSpan={3}>Total</td><td>{s.effectifTotal}</td><td>{s.effectifAVenirTotal}</td><td></td><td>{texte(s.valeurEquipes)}</td>
            </tr>
          </tbody>
        </table>
      </section>
    </div>
  );
}
