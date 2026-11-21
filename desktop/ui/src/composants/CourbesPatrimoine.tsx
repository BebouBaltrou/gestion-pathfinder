import { useState } from "react";
import type { CourbesPatrimoine } from "../api";

const PALETTE = [
  "var(--po)", "var(--influence)", "var(--travail)", "var(--magie)", "var(--marchandises)",
  "#e06c75", "#98c379", "#c678dd", "#56b6c2", "#d19a66", "#61afef", "#be5046",
];
const GROUPES_MASQUES_PAR_DEFAUT = ["Localités", "Autres"];

const enPo = (pc: number) => `${Math.round(pc / 100).toLocaleString("fr-FR")} po`;

/** Courbes d'évolution du patrimoine ; chaque courbe peut être affichée ou masquée. */
export function GraphiqueCourbes({ courbes }: { courbes: CourbesPatrimoine }) {
  const [masquees, setMasquees] = useState<Set<string>>(
    () => new Set(courbes.series.filter((s) => GROUPES_MASQUES_PAR_DEFAUT.includes(s.groupe)).map((s) => s.cle)),
  );
  const couleur = (i: number) => PALETTE[i % PALETTE.length];
  const bascule = (cle: string) =>
    setMasquees((m) => {
      const n = new Set(m);
      if (n.has(cle)) n.delete(cle);
      else n.add(cle);
      return n;
    });

  const visibles = courbes.series.map((s, i) => ({ s, i })).filter(({ s }) => !masquees.has(s.cle));
  const W = 680, H = 300;
  const M = { haut: 16, droite: 20, bas: 44, gauche: 72 };
  const largeur = W - M.gauche - M.droite;
  const hauteur = H - M.haut - M.bas;
  const jmin = Math.min(...courbes.jours);
  const jmax = Math.max(...courbes.jours);
  const x = (j: number) => M.gauche + (jmax === jmin ? largeur / 2 : ((j - jmin) / (jmax - jmin)) * largeur);
  const max = Math.max(1, ...visibles.flatMap(({ s }) => s.valeurs));
  const min = Math.min(0, ...visibles.flatMap(({ s }) => s.valeurs));
  const y = (v: number) => M.haut + hauteur - ((v - min) / (max - min)) * hauteur;
  const graduations = [0, 0.25, 0.5, 0.75, 1].map((f) => min + f * (max - min));
  const n = courbes.jours.length;
  const reperes = Array.from(new Set([0, ...[1, 2, 3, 4].map((k) => Math.round(((n - 1) * k) / 4)), n - 1])).filter((i) => i >= 0 && i < n);
  const groupes = Array.from(new Set(courbes.series.map((s) => s.groupe)));

  return (
    <section className="carte">
      <h2>Évolution des patrimoines (po)</h2>
      <svg viewBox={`0 0 ${W} ${H}`} width="100%" role="img" aria-label="Évolution des patrimoines">
        {graduations.map((g) => (
          <g key={g}>
            <line x1={M.gauche} x2={W - M.droite} y1={y(g)} y2={y(g)} stroke="var(--bord-fort)" strokeDasharray="3 3" />
            <text x={M.gauche - 8} y={y(g) + 4} textAnchor="end" fontSize="11" fill="currentColor">{enPo(g)}</text>
          </g>
        ))}
        <line x1={M.gauche} x2={M.gauche} y1={M.haut} y2={M.haut + hauteur} stroke="currentColor" />
        <line x1={M.gauche} x2={W - M.droite} y1={M.haut + hauteur} y2={M.haut + hauteur} stroke="currentColor" />
        {reperes.map((i) => (
          <text key={i} x={x(courbes.jours[i])} y={M.haut + hauteur + 18} textAnchor="middle" fontSize="10" fill="currentColor">
            {courbes.dates[i]}
          </text>
        ))}
        {visibles.map(({ s, i }) => (
          <g key={s.cle}>
            {n > 1 && (
              <polygon
                fill={couleur(i)}
                fillOpacity="0.15"
                stroke="none"
                points={`${x(courbes.jours[0])},${y(0)} ${s.valeurs.map((v, k) => `${x(courbes.jours[k])},${y(v)}`).join(" ")} ${x(courbes.jours[n - 1])},${y(0)}`}
              />
            )}
            <polyline
              fill="none"
              stroke={couleur(i)}
              strokeWidth="2"
              points={s.valeurs.map((v, k) => `${x(courbes.jours[k])},${y(v)}`).join(" ")}
            />
            {s.valeurs.map((v, k) => (
              <circle key={k} cx={x(courbes.jours[k])} cy={y(v)} r="3" fill={couleur(i)}>
                <title>{`${s.nom} — ${courbes.dates[k]} : ${enPo(v)}`}</title>
              </circle>
            ))}
          </g>
        ))}
      </svg>
      {visibles.length === 0 && <p className="detail">Aucune courbe affichée : coche au moins une donnée ci-dessous.</p>}
      <div style={{ display: "flex", gap: 24, flexWrap: "wrap", marginTop: 8 }}>
        {groupes.map((g) => (
          <fieldset key={g} style={{ border: "none", padding: 0, margin: 0 }}>
            <legend style={{ fontWeight: 600, marginBottom: 4 }}>{g}</legend>
            {courbes.series.map((s, i) =>
              s.groupe !== g ? null : (
                <label key={s.cle} style={{ display: "flex", alignItems: "center", gap: 6, fontSize: 13 }}>
                  <input type="checkbox" checked={!masquees.has(s.cle)} onChange={() => bascule(s.cle)} />
                  <span style={{ display: "inline-block", width: 12, height: 12, background: couleur(i), borderRadius: 2 }} />
                  {s.nom}
                </label>
              ),
            )}
          </fieldset>
        ))}
      </div>
    </section>
  );
}

const COMPOSANTES: { cle: "compte" | "objets" | "batiments"; nom: string; couleur: string }[] = [
  { cle: "compte", nom: "Compte", couleur: "var(--po)" },
  { cle: "objets", nom: "Objets", couleur: "var(--influence)" },
  { cle: "batiments", nom: "Bâtiments", couleur: "var(--marchandises)" },
];

/** Aires empilées : évolution et composition du patrimoine du personnage choisi. */
export function GraphiqueComposition({ courbes }: { courbes: CourbesPatrimoine }) {
  const [choix, setChoix] = useState<number | null>(null);
  const perso = courbes.compositions.find((p) => p.id === choix) ?? courbes.compositions[0];
  if (!perso) return null;

  const n = courbes.jours.length;
  const W = 680, H = 300;
  const M = { haut: 16, droite: 20, bas: 44, gauche: 72 };
  const largeur = W - M.gauche - M.droite;
  const hauteur = H - M.haut - M.bas;
  const jmin = Math.min(...courbes.jours);
  const jmax = Math.max(...courbes.jours);
  const x = (j: number) => M.gauche + (jmax === jmin ? largeur / 2 : ((j - jmin) / (jmax - jmin)) * largeur);
  // valeurs négatives ignorées : les aires empilées ne montrent que les avoirs
  const val = (cle: "compte" | "objets" | "batiments", k: number) => Math.max(0, perso[cle][k]);
  const totaux = courbes.jours.map((_, k) => COMPOSANTES.reduce((s, c) => s + val(c.cle, k), 0));
  const max = Math.max(1, ...totaux);
  const y = (v: number) => M.haut + hauteur - (v / max) * hauteur;
  const graduations = [0, 0.25, 0.5, 0.75, 1].map((f) => f * max);
  const reperes = Array.from(new Set([0, ...[1, 2, 3, 4].map((q) => Math.round(((n - 1) * q) / 4)), n - 1])).filter((i) => i >= 0 && i < n);
  const largeurBarre = 48;

  // limites basse et haute de chaque couche, jour par jour
  const couches = COMPOSANTES.map((c, ci) => {
    const bas = courbes.jours.map((_, k) => COMPOSANTES.slice(0, ci).reduce((s, d) => s + val(d.cle, k), 0));
    const haut = bas.map((b, k) => b + val(c.cle, k));
    return { ...c, bas, haut };
  });

  return (
    <section className="carte">
      <h2>Composition du patrimoine d'un personnage (po)</h2>
      <label style={{ display: "inline-flex", alignItems: "center", gap: 8, marginBottom: 8 }}>
        Personnage
        <select value={perso.id} onChange={(e) => setChoix(Number(e.target.value))}>
          {courbes.compositions.map((p) => (
            <option key={p.id} value={p.id}>{p.nom}</option>
          ))}
        </select>
      </label>
      <svg viewBox={`0 0 ${W} ${H}`} width="100%" role="img" aria-label={`Composition du patrimoine de ${perso.nom}`}>
        {graduations.map((g) => (
          <g key={g}>
            <line x1={M.gauche} x2={W - M.droite} y1={y(g)} y2={y(g)} stroke="var(--bord-fort)" strokeDasharray="3 3" />
            <text x={M.gauche - 8} y={y(g) + 4} textAnchor="end" fontSize="11" fill="currentColor">{enPo(g)}</text>
          </g>
        ))}
        <line x1={M.gauche} x2={M.gauche} y1={M.haut} y2={M.haut + hauteur} stroke="currentColor" />
        <line x1={M.gauche} x2={W - M.droite} y1={M.haut + hauteur} y2={M.haut + hauteur} stroke="currentColor" />
        {reperes.map((i) => (
          <text key={i} x={x(courbes.jours[i])} y={M.haut + hauteur + 18} textAnchor="middle" fontSize="10" fill="currentColor">
            {courbes.dates[i]}
          </text>
        ))}
        {couches.map((c) =>
          n > 1 ? (
            <polygon
              key={c.cle}
              fill={c.couleur}
              fillOpacity="0.8"
              stroke={c.couleur}
              points={[
                ...c.haut.map((v, k) => `${x(courbes.jours[k])},${y(v)}`),
                ...c.bas.map((v, k) => `${x(courbes.jours[k])},${y(v)}`).reverse(),
              ].join(" ")}
            >
              <title>{c.nom}</title>
            </polygon>
          ) : (
            <rect key={c.cle} x={x(courbes.jours[0]) - largeurBarre / 2} y={y(c.haut[0])} width={largeurBarre} height={Math.max(0, y(c.bas[0]) - y(c.haut[0]))} fill={c.couleur}>
              <title>{`${c.nom} : ${enPo(c.haut[0] - c.bas[0])}`}</title>
            </rect>
          ),
        )}
        {courbes.jours.map((j, k) => (
          <circle key={j} cx={x(j)} cy={y(totaux[k])} r="3" fill="currentColor">
            <title>{`${courbes.dates[k]} — total ${enPo(totaux[k])} (compte ${enPo(val("compte", k))}, objets ${enPo(val("objets", k))}, bâtiments ${enPo(val("batiments", k))})`}</title>
          </circle>
        ))}
      </svg>
      <div style={{ display: "flex", gap: 16, flexWrap: "wrap", fontSize: 13 }}>
        {COMPOSANTES.map((c) => (
          <span key={c.cle}>
            <span style={{ display: "inline-block", width: 12, height: 12, background: c.couleur, marginRight: 6, borderRadius: 2 }} />
            {c.nom}
          </span>
        ))}
      </div>
    </section>
  );
}
