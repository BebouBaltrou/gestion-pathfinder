import { CSSProperties, useEffect, useId, useState } from "react";
import type { CourbesPatrimoine } from "../api";

/** Couleurs des courbes : celles des capitaux, puis quelques teintes de plus. */
export const PALETTE = [
  "var(--po)", "var(--influence)", "var(--travail)", "var(--magie)", "var(--marchandises)",
  "var(--rouge)", "var(--vert)", "#c678dd", "#56b6c2", "#d19a66", "#61afef", "#be5046",
];
export const COULEUR_PART: Record<string, string> = {
  comptes: "var(--po)", objets: "var(--influence)", batiments: "var(--marchandises)",
  tresoreries: "var(--travail)", capitaux: "var(--magie)",
};

export const fmt = (n: number) => Math.round(n).toLocaleString("fr-FR");
const enPo = (po: number) => `${fmt(po)} po`;
const sansDeuxPoints = (s: string) => s.replace(/:/g, "_");
const css = (o: Record<string, string>) => o as unknown as CSSProperties;

/** Courbe lisse qui ne dépasse jamais les valeurs (même méthode que d3.curveMonotoneX). */
function monotone(pts: [number, number][]): string {
  const n = pts.length;
  if (n === 0) return "";
  if (n < 3) return "M" + pts.map((p) => p.join(",")).join("L");
  const dx: number[] = [], dy: number[] = [], m: number[] = [], t: number[] = new Array(n).fill(0);
  for (let i = 0; i < n - 1; i++) {
    dx[i] = pts[i + 1][0] - pts[i][0];
    dy[i] = pts[i + 1][1] - pts[i][1];
    m[i] = dx[i] === 0 ? 0 : dy[i] / dx[i];
  }
  t[0] = m[0];
  t[n - 1] = m[n - 2];
  for (let i = 1; i < n - 1; i++) t[i] = m[i - 1] * m[i] <= 0 ? 0 : (m[i - 1] + m[i]) / 2;
  for (let i = 0; i < n - 1; i++) {
    if (m[i] === 0) { t[i] = 0; t[i + 1] = 0; continue; }
    const a = t[i] / m[i], b = t[i + 1] / m[i], h = a * a + b * b;
    if (h > 9) { const k = 3 / Math.sqrt(h); t[i] = k * a * m[i]; t[i + 1] = k * b * m[i]; }
  }
  let d = `M${pts[0][0]},${pts[0][1]}`;
  for (let i = 0; i < n - 1; i++) {
    const h = dx[i] / 3;
    d += `C${pts[i][0] + h},${pts[i][1] + t[i] * h} ${pts[i + 1][0] - h},${pts[i + 1][1] - t[i + 1] * h} ${pts[i + 1][0]},${pts[i + 1][1]}`;
  }
  return d;
}

/** Graduations rondes (environ 4 intervalles) : { sommet, graduations }. */
function echelle(max: number): { top: number; graduations: number[] } {
  if (max <= 0) return { top: 1, graduations: [0, 1] };
  const brut = max / 4;
  const mag = Math.pow(10, Math.floor(Math.log10(brut)));
  const f = brut / mag;
  const pas = (f <= 1 ? 1 : f <= 2 ? 2 : f <= 2.5 ? 2.5 : f <= 5 ? 5 : 10) * mag;
  const nb = Math.ceil(max / pas - 1e-9);
  return { top: nb * pas, graduations: Array.from({ length: nb + 1 }, (_, k) => k * pas) };
}

export function useMonte(delai = 80): boolean {
  const [m, setM] = useState(false);
  useEffect(() => {
    const t = setTimeout(() => setM(true), delai);
    return () => clearTimeout(t);
  }, [delai]);
  return m;
}

/** Nombre qui s'incrémente jusqu'à sa valeur (sans animation si l'utilisateur la désactive). */
export function Compteur({ valeur, suffixe = "" }: { valeur: number; suffixe?: string }) {
  const [v, setV] = useState(0);
  useEffect(() => {
    if (window.matchMedia?.("(prefers-reduced-motion: reduce)").matches) { setV(valeur); return; }
    let raf = 0;
    const t0 = performance.now();
    const f = (t: number) => {
      const k = Math.min(1, (t - t0) / 1300);
      setV(valeur * (1 - Math.pow(1 - k, 3)));
      if (k < 1) raf = requestAnimationFrame(f);
    };
    raf = requestAnimationFrame(f);
    return () => cancelAnimationFrame(raf);
  }, [valeur]);
  return <>{fmt(v)}{suffixe}</>;
}

/** Mini courbe avec son aire, sans axes. */
export function Spark({ valeurs, couleur }: { valeurs: number[]; couleur: string }) {
  const id = sansDeuxPoints(useId());
  if (valeurs.length < 2) return null;
  const W = 300, H = 50;
  const mx = Math.max(...valeurs), mn = Math.min(...valeurs);
  const pts: [number, number][] = valeurs.map((v, i) => [(i / (valeurs.length - 1)) * W, H - 3 - ((v - mn) / (mx - mn || 1)) * (H - 8)]);
  const d = monotone(pts);
  return (
    <svg viewBox={`0 0 ${W} ${H}`} preserveAspectRatio="none" style={{ width: "100%", height: "100%", overflow: "visible" }} aria-hidden="true">
      <defs>
        <linearGradient id={id} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" style={{ stopColor: couleur, stopOpacity: 0.35 }} />
          <stop offset="100%" style={{ stopColor: couleur, stopOpacity: 0 }} />
        </linearGradient>
      </defs>
      <path d={`${d}L${W},${H}L0,${H}Z`} fill={`url(#${id})`} />
      <path d={d} fill="none" style={{ stroke: couleur }} strokeWidth="2" vectorEffect="non-scaling-stroke" strokeLinecap="round" />
    </svg>
  );
}

/** Anneau de répartition du patrimoine ; survol d'une part ou de sa ligne de légende. */
export function Anneau({ parts, centre }: { parts: { cle: string; nom: string; pc: number }[]; centre: number }) {
  const monte = useMonte(150);
  const [actif, setActif] = useState<number | null>(null);
  const positives = parts.map((p) => ({ ...p, v: Math.max(0, p.pc) }));
  const total = positives.reduce((a, p) => a + p.v, 0);
  const R = 70, C = 2 * Math.PI * R;
  let decalage = 0;
  const arcs = positives.map((p) => {
    const part = total > 0 ? (p.v / total) * C : 0;
    const a = { ...p, part, decalage };
    decalage += part;
    return a;
  });
  const choisi = actif === null ? null : arcs[actif];
  return (
    <div className="st-anneau">
      <svg viewBox="0 0 200 200" role="img" aria-label="Répartition du patrimoine">
        <circle cx="100" cy="100" r={R} fill="none" style={{ stroke: "var(--carte-2)" }} strokeWidth="22" />
        {arcs.map((a, i) => (
          <circle
            key={a.cle}
            className="st-tranche"
            cx="100" cy="100" r={R} fill="none" strokeWidth="22"
            style={{ stroke: COULEUR_PART[a.cle] ?? PALETTE[i], color: COULEUR_PART[a.cle] ?? PALETTE[i], opacity: actif === null || actif === i ? 1 : 0.25 }}
            strokeDasharray={monte ? `${Math.max(0, a.part - 2)} ${C}` : `0 ${C}`}
            strokeDashoffset={-a.decalage}
            transform="rotate(-90 100 100)"
            onMouseEnter={() => setActif(i)}
            onMouseLeave={() => setActif(null)}
          />
        ))}
        <text x="100" y="104" textAnchor="middle" fontSize="26" fontWeight="700">{fmt(choisi ? choisi.pc / 100 : centre / 100)}</text>
        <text x="100" y="124" textAnchor="middle" fontSize="11" style={{ fill: "var(--texte-doux)", fontFamily: "var(--police)" }}>
          {choisi ? choisi.nom : "po au total"}
        </text>
      </svg>
      <div className="st-legende">
        {arcs.map((a, i) => (
          <div key={a.cle} className={actif === i ? "act" : ""} onMouseEnter={() => setActif(i)} onMouseLeave={() => setActif(null)}>
            <i style={{ background: COULEUR_PART[a.cle] ?? PALETTE[i] }} />
            {a.nom}
            <span>{total > 0 ? Math.round((a.v / total) * 100) : 0} %</span>
          </div>
        ))}
      </div>
    </div>
  );
}

export type SerieGraphe = { cle: string; nom: string; couleur: string; v: number[] };

/** Graphique dans le temps : courbes avec aires, ou aires empilées. Valeurs en po. */
export function GrapheTemporel({
  jours, dates, series, empile = false, W = 900, H = 300, titre,
}: {
  jours: number[]; dates: string[]; series: SerieGraphe[]; empile?: boolean; W?: number; H?: number; titre: string;
}) {
  const uid = sansDeuxPoints(useId());
  const [survol, setSurvol] = useState<{ i: number; px: number; largeur: number } | null>(null);
  const n = jours.length;
  const M = { h: 14, d: 14, b: 30, g: 60 };
  const w = W - M.g - M.d, h = H - M.h - M.b;
  const jmin = Math.min(...jours), jmax = Math.max(...jours);
  const x = (j: number) => M.g + (jmax === jmin ? w / 2 : ((j - jmin) / (jmax - jmin)) * w);

  let max = 0;
  if (empile) {
    for (let k = 0; k < n; k++) max = Math.max(max, series.reduce((a, s) => a + Math.max(0, s.v[k]), 0));
  } else {
    max = Math.max(0, ...series.flatMap((s) => s.v));
  }
  const { top, graduations } = echelle(max);
  const y = (v: number) => M.h + h - (Math.max(0, v) / top) * h;

  let base: number[] = jours.map(() => 0);
  const couches = series.map((s, i) => {
    const bas = empile ? base.slice() : jours.map(() => 0);
    const haut = empile ? s.v.map((v, k) => Math.max(0, v) + base[k]) : s.v;
    if (empile) base = haut;
    const pts: [number, number][] = haut.map((v, k) => [x(jours[k]), y(v)]);
    const dessous: [number, number][] = bas.map((v, k): [number, number] => [x(jours[k]), y(v)]).reverse();
    const ligne = monotone(pts);
    const aire = n > 1 ? `${ligne}L${dessous.map((p) => p.join(",")).join("L")}Z` : "";
    return { s, i, haut, ligne, aire };
  });

  const reperes = Array.from(new Set([0, 0.25, 0.5, 0.75, 1].map((f) => {
    const cible = jmin + f * (jmax - jmin);
    let mi = 0;
    jours.forEach((j, k) => { if (Math.abs(j - cible) < Math.abs(jours[mi] - cible)) mi = k; });
    return mi;
  })));

  function bouge(e: React.MouseEvent<SVGRectElement>) {
    const svg = e.currentTarget.ownerSVGElement;
    if (!svg) return;
    const r = svg.getBoundingClientRect();
    const px = ((e.clientX - r.left) / r.width) * W;
    let mi = 0;
    jours.forEach((j, k) => { if (Math.abs(x(j) - px) < Math.abs(x(jours[mi]) - px)) mi = k; });
    setSurvol({ i: mi, px: (x(jours[mi]) / W) * r.width, largeur: r.width });
  }

  if (series.length === 0 || n === 0) return <p className="st-vide">Aucune courbe affichée : choisis au moins une donnée.</p>;
  const i = survol?.i ?? 0;
  const totalJour = series.reduce((a, s) => a + Math.max(0, s.v[i] ?? 0), 0);

  return (
    <div className="st-zone">
      <svg viewBox={`0 0 ${W} ${H}`} role="img" aria-label={titre}>
        <defs>
          {couches.map(({ s, i: k }) => (
            <g key={s.cle}>
              <linearGradient id={`${uid}-g${k}`} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" style={{ stopColor: s.couleur, stopOpacity: empile ? 0.7 : 0.34 }} />
                <stop offset="100%" style={{ stopColor: s.couleur, stopOpacity: empile ? 0.35 : 0 }} />
              </linearGradient>
              <pattern id={`${uid}-h${k}`} width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
                <rect width="6" height="6" style={{ fill: s.couleur, fillOpacity: empile ? 0.16 : 0.06 }} />
                <line x1="0" y1="0" x2="0" y2="6" style={{ stroke: s.couleur, strokeOpacity: empile ? 0.55 : 0.38 }} strokeWidth="1.2" />
              </pattern>
            </g>
          ))}
        </defs>
        {graduations.map((g) => (
          <g key={g}>
            <line className="st-axe" x1={M.g} x2={W - M.d} y1={y(g)} y2={y(g)} strokeDasharray={g === 0 ? undefined : "3 3"} />
            <text x={M.g - 9} y={y(g) + 4} textAnchor="end">{enPo(g)}</text>
          </g>
        ))}
        {reperes.map((k) => (
          <text key={k} x={x(jours[k])} y={H - 8} textAnchor="middle">{dates[k] ?? `jour ${jours[k]}`}</text>
        ))}
        {couches.map(({ s, i: k, ligne, aire, haut }) => (
          <g key={s.cle} style={css({ "--c": s.couleur })}>
            {n > 1 && <path className="st-aire" d={aire} style={css({ "--g": `url(#${uid}-g${k})`, "--h": `url(#${uid}-h${k})` })} />}
            {n > 1 && <path className="st-ligne" d={ligne} pathLength={1} style={{ stroke: s.couleur }} />}
            {n === 1 && <circle cx={x(jours[0])} cy={y(haut[0])} r="5" style={{ fill: s.couleur }} />}
          </g>
        ))}
        {survol && (
          <g>
            <line className="st-visee" x1={x(jours[i])} x2={x(jours[i])} y1={M.h} y2={M.h + h} />
            {couches.map(({ s, haut }) => (
              <circle key={s.cle} cx={x(jours[i])} cy={y(haut[i])} r="4.5" style={{ fill: s.couleur, stroke: "var(--carte)" }} strokeWidth="2" />
            ))}
          </g>
        )}
        <rect x={M.g} y={M.h} width={w} height={h} fill="transparent" onMouseMove={bouge} onMouseLeave={() => setSurvol(null)} />
      </svg>
      {survol && (
        <div className="st-bulle" style={{ left: Math.max(0, Math.min(survol.px + 14, survol.largeur - 190)) }}>
          <b>{dates[i] ?? `jour ${jours[i]}`}</b>
          {series.map((s) => (
            <div key={s.cle}>
              <span><i style={{ background: s.couleur }} />{s.nom}</span>
              <em>{enPo(s.v[i] ?? 0)}</em>
            </div>
          ))}
          {empile && series.length > 1 && (
            <div><span>Total</span><em>{enPo(totalJour)}</em></div>
          )}
        </div>
      )}
    </div>
  );
}

const PRESETS: [string, (groupe: string) => boolean][] = [
  ["Personnages", (g) => g === "Personnages"],
  ["Villes", (g) => g === "Localités"],
  ["Total", (g) => g === "Total"],
  ["Tout", () => true],
];

/** Évolution des patrimoines : une courbe par donnée, que l'on affiche ou masque. */
export function GraphiqueCourbes({ courbes }: { courbes: CourbesPatrimoine }) {
  const parGroupe = (f: (g: string) => boolean) => new Set(courbes.series.filter((s) => f(s.groupe)).map((s) => s.cle));
  const [visibles, setVisibles] = useState<Set<string>>(() => {
    const p = parGroupe((g) => g === "Personnages");
    return p.size > 0 ? p : parGroupe((g) => g === "Total");
  });
  const bascule = (cle: string) =>
    setVisibles((v) => {
      const n = new Set(v);
      if (n.has(cle)) n.delete(cle); else n.add(cle);
      return n;
    });
  const actif = (f: (g: string) => boolean) => {
    const p = parGroupe(f);
    return p.size > 0 && p.size === visibles.size && [...p].every((c) => visibles.has(c));
  };
  const series: SerieGraphe[] = courbes.series
    .map((s, i) => ({ cle: s.cle, nom: s.nom, couleur: PALETTE[i % PALETTE.length], v: s.valeurs.map((pc) => pc / 100) }))
    .filter((s) => visibles.has(s.cle));

  return (
    <section className="carte st-carte">
      <div className="st-entete">
        <div>
          <h2>Évolution des patrimoines</h2>
          <p>Chaque courbe peut être affichée ou masquée</p>
        </div>
        <div className="st-puces">
          {PRESETS.map(([nom, f]) => (
            <button key={nom} className={`st-preset${actif(f) ? " on" : ""}`} onClick={() => setVisibles(parGroupe(f))}>{nom}</button>
          ))}
          {courbes.series.map((s, i) => (
            <button
              key={s.cle}
              className={`st-puce ${visibles.has(s.cle) ? "on" : "off"}`}
              style={css({ "--c": PALETTE[i % PALETTE.length] })}
              onClick={() => bascule(s.cle)}
              aria-pressed={visibles.has(s.cle)}
            >
              <i />{s.nom}
            </button>
          ))}
        </div>
      </div>
      <GrapheTemporel jours={courbes.jours} dates={courbes.dates} series={series} H={320} titre="Évolution des patrimoines" />
    </section>
  );
}

const COMPOSANTES: { cle: "compte" | "objets" | "batiments"; nom: string; couleur: string }[] = [
  { cle: "compte", nom: "Compte", couleur: "var(--po)" },
  { cle: "objets", nom: "Objets", couleur: "var(--influence)" },
  { cle: "batiments", nom: "Bâtiments", couleur: "var(--marchandises)" },
];

/** Aires empilées : composition du patrimoine du personnage choisi, dans le temps. */
export function GraphiqueComposition({ courbes }: { courbes: CourbesPatrimoine }) {
  const [choix, setChoix] = useState<number | null>(null);
  const monte = useMonte(120);
  const perso = courbes.compositions.find((p) => p.id === choix) ?? courbes.compositions[0];
  if (!perso) return null;
  const dernier = courbes.jours.length - 1;
  const valeurs = COMPOSANTES.map((c) => ({ ...c, pc: Math.max(0, perso[c.cle][dernier] ?? 0) }));
  const total = valeurs.reduce((a, c) => a + c.pc, 0);
  const series: SerieGraphe[] = COMPOSANTES.map((c) => ({ cle: c.cle, nom: c.nom, couleur: c.couleur, v: perso[c.cle].map((pc) => pc / 100) }));

  return (
    <section className="carte st-carte">
      <div className="st-entete">
        <div>
          <h2>Composition du patrimoine d'un personnage</h2>
          <p>Aires empilées : compte, objets, bâtiments</p>
        </div>
      </div>
      <div className="st-persos">
        {courbes.compositions.map((p) => (
          <button key={p.id} className={`st-perso${p.id === perso.id ? " on" : ""}`} onClick={() => setChoix(p.id)} aria-pressed={p.id === perso.id}>
            <span className="av">{p.nom.trim().charAt(0).toUpperCase() || "?"}</span>{p.nom}
          </button>
        ))}
      </div>
      <GrapheTemporel key={perso.id} jours={courbes.jours} dates={courbes.dates} series={series} empile W={760} H={250} titre={`Composition du patrimoine de ${perso.nom}`} />
      <div className="st-compo">
        {valeurs.map((c) => (
          <div className="st-ligne-c" key={c.cle} style={css({ "--c": c.couleur })}>
            <div className="t"><span>{c.nom}</span><b>{enPo(c.pc / 100)}</b></div>
            <div className="st-barre"><span style={{ width: monte && total > 0 ? `${(c.pc / total) * 100}%` : "0%" }} /></div>
          </div>
        ))}
        <div className="st-total-p">
          <span className="st-etq">Patrimoine</span>
          <b>{enPo(total / 100)}</b>
        </div>
      </div>
    </section>
  );
}
