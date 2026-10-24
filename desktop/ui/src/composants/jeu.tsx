// Composants du mode aventure et du mode combat : jauge de PV animée, chronomètre circulaire des
// effets, gemmes d'emplacements, saisie des dés (composant de premier rang), fenêtre de dialogue.
// Aucune règle ici : tout ce qui est affiché est calculé par le cœur (crates/vues).

import { AnimatePresence, motion, useAnimate } from "motion/react";
import { ReactNode, useEffect, useId, useRef, useState } from "react";
import type { AlerteVue, EffetVue, EtatVue, PjAventure, SanteCode } from "../aventure-types";
import "../jeu.css";

// ------------------------------------------------------------------ santé

export const NOMS_SANTE: Record<SanteCode, string> = {
  conscient: "Conscient",
  chancelant: "Chancelant",
  hors_de_combat: "Hors de combat",
  inconscient: "Inconscient",
  mourant: "Mourant",
  stable: "Stable",
  mort: "Mort",
};

/** Pastille de santé : jamais la couleur seule, toujours une forme et un texte. */
export function PastilleSante({ code }: { code: SanteCode }) {
  const symbole: Record<SanteCode, string> = {
    conscient: "●",
    chancelant: "◐",
    hors_de_combat: "◍",
    inconscient: "◌",
    mourant: "✚",
    stable: "◆",
    mort: "✖",
  };
  return (
    <span className={"pastille-sante " + code} title={NOMS_SANTE[code]}>
      <span aria-hidden="true">{symbole[code]}</span> {NOMS_SANTE[code]}
    </span>
  );
}

type Flottant = { id: number; texte: string; genre: "degats" | "soin" | "autre" };

/** Jauge de PV : PV, PV temporaires, dégâts non létaux, traînée rouge, secousse et nombre flottant. */
export function JaugePv({ pj, grande = false }: { pj: Pick<PjAventure, "pv" | "pvMax" | "temporaires" | "nonLetaux" | "nom">; grande?: boolean }) {
  const { pv, pvMax, temporaires, nonLetaux } = pj;
  const total = Math.max(1, pvMax + temporaires);
  const pct = (n: number) => `${Math.max(0, Math.min(100, (n / total) * 100))}%`;
  const [zone, animer] = useAnimate();
  const [flottants, setFlottants] = useState<Flottant[]>([]);
  const precedent = useRef({ pv, temporaires, nonLetaux });
  const prochain = useRef(1);

  useEffect(() => {
    const p = precedent.current;
    precedent.current = { pv, temporaires, nonLetaux };
    const dPv = pv - p.pv;
    const dTemp = temporaires - p.temporaires;
    const dNl = nonLetaux - p.nonLetaux;
    const nouveaux: Flottant[] = [];
    if (dPv < 0) nouveaux.push({ id: prochain.current++, texte: `−${-dPv}`, genre: "degats" });
    else if (dPv > 0) nouveaux.push({ id: prochain.current++, texte: `+${dPv}`, genre: "soin" });
    if (dTemp > 0) nouveaux.push({ id: prochain.current++, texte: `+${dTemp} temp.`, genre: "autre" });
    else if (dTemp < 0 && dPv === 0) nouveaux.push({ id: prochain.current++, texte: `−${-dTemp} temp.`, genre: "degats" });
    if (dNl > 0) nouveaux.push({ id: prochain.current++, texte: `${dNl} non létaux`, genre: "autre" });
    if (nouveaux.length === 0) return;
    setFlottants((f) => [...f, ...nouveaux]);
    const ids = nouveaux.map((n) => n.id);
    const t = window.setTimeout(() => setFlottants((f) => f.filter((x) => !ids.includes(x.id))), 1300);
    if (zone.current) {
      if (dPv < 0 || dTemp < 0) {
        animer(zone.current, { x: [0, -7, 7, -4, 3, 0] }, { duration: 0.38 });
        animer(zone.current, { boxShadow: ["0 0 0 0 rgba(240,122,110,.7)", "0 0 0 16px rgba(240,122,110,0)"] }, { duration: 0.55 });
      } else if (dPv > 0) {
        animer(zone.current, { boxShadow: ["0 0 0 0 rgba(127,209,168,.7)", "0 0 0 16px rgba(127,209,168,0)"] }, { duration: 0.55 });
      }
    }
    return () => window.clearTimeout(t);
  }, [pv, temporaires, nonLetaux, zone, animer]);

  return (
    <div className={"jauge-pv" + (grande ? " grande" : "")}>
      <div
        className="jauge-pv-barre"
        ref={zone}
        role="meter"
        aria-label={`Points de vie de ${pj.nom}`}
        aria-valuemin={0}
        aria-valuemax={pvMax}
        aria-valuenow={pv}
      >
        <div className="jauge-retard" style={{ width: pct(pv) }} />
        <div className="jauge-vie" style={{ width: pct(pv) }} />
        <div className="jauge-temp" style={{ left: pct(Math.max(0, pv)), width: pct(temporaires) }} />
        <div className="jauge-nl" style={{ width: pct(nonLetaux) }} title={`${nonLetaux} dégâts non létaux`} />
      </div>
      <div className="jauge-pv-chiffres">
        <strong className={pv < 0 ? "negatif" : ""}>{pv}</strong>
        <span className="doux"> / {pvMax} PV</span>
        {temporaires > 0 && <span className="etiquette-temp">+{temporaires} temp.</span>}
        {nonLetaux > 0 && <span className="etiquette-nl">{nonLetaux} non létaux</span>}
      </div>
      <div className="flottants" aria-hidden="true">
        <AnimatePresence>
          {flottants.map((f) => (
            <motion.span
              key={f.id}
              className={"flottant " + f.genre}
              initial={{ opacity: 1, y: 0 }}
              animate={{ opacity: 0, y: -42 }}
              transition={{ duration: 1.2, ease: "easeOut" }}
            >
              {f.texte}
            </motion.span>
          ))}
        </AnimatePresence>
      </div>
    </div>
  );
}

export function EtatsPuces({ etats }: { etats: EtatVue[] }) {
  if (etats.length === 0) return null;
  return (
    <div className="puces etats">
      {etats.map((e) => (
        <span key={e.id} className="puce etat" title={`État : ${e.nom}`}>
          <span aria-hidden="true">⚑</span> {e.nom}
        </span>
      ))}
    </div>
  );
}

// ------------------------------------------------------------------ effets

/** Chronomètre circulaire : la part de durée qui reste ; la valeur exacte est écrite à côté. */
export function Chrono({ fraction, taille = 38 }: { fraction: number | null; taille?: number }) {
  const r = taille / 2 - 4;
  const c = 2 * Math.PI * r;
  const reste = fraction ?? 1;
  return (
    <svg className="chrono" width={taille} height={taille} viewBox={`0 0 ${taille} ${taille}`} aria-hidden="true">
      <circle className="chrono-fond" cx={taille / 2} cy={taille / 2} r={r} />
      <circle
        className={"chrono-reste" + (fraction !== null && fraction < 0.3 ? " bas" : "") + (fraction === null ? " sans" : "")}
        cx={taille / 2}
        cy={taille / 2}
        r={r}
        strokeDasharray={c}
        strokeDashoffset={fraction === null ? 0 : c * (1 - reste)}
        transform={`rotate(-90 ${taille / 2} ${taille / 2})`}
      />
      <text x="50%" y="52%" textAnchor="middle" dominantBaseline="middle" className="chrono-texte">
        {fraction === null ? "∞" : `${Math.round(reste * 100)}`}
      </text>
    </svg>
  );
}

export function ListeEffets({
  effets,
  onRetirer,
}: {
  effets: EffetVue[];
  onRetirer?: (e: EffetVue) => void;
}) {
  const [ouvert, setOuvert] = useState<number | null>(null);
  if (effets.length === 0) return <p className="doux vide">Aucun effet en cours.</p>;
  return (
    <ul className="effets-liste">
      <AnimatePresence initial={false}>
        {effets.map((e) => (
          <motion.li
            layout
            key={e.id}
            className="effet-ligne"
            initial={{ opacity: 0, y: -6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, x: 24 }}
            transition={{ duration: 0.22 }}
          >
            <button
              className="effet-bouton"
              onClick={() => setOuvert(ouvert === e.id ? null : e.id)}
              aria-expanded={ouvert === e.id}
              title="Voir la source, la durée et les bonus"
            >
              <Chrono fraction={e.fraction} />
              <span className="effet-texte">
                <strong>{e.nom}</strong>
                <small>{e.restant ? `reste ${e.restant}` : e.duree}</small>
              </span>
            </button>
            {ouvert === e.id && (
              <div className="effet-detail">
                <div>
                  <span className="doux">Source :</span> {e.source}
                </div>
                <div>
                  <span className="doux">Durée :</span> {e.duree}
                  {e.restant && <> — reste {e.restant}</>}
                </div>
                {e.details.length > 0 && (
                  <ul>
                    {e.details.map((d) => (
                      <li key={d}>{d}</li>
                    ))}
                  </ul>
                )}
                {e.etats.length > 0 && <div>États donnés : {e.etats.join(", ")}</div>}
                {onRetirer && (
                  <button className="bouton discret danger" onClick={() => onRetirer(e)}>
                    Retirer cet effet
                  </button>
                )}
              </div>
            )}
          </motion.li>
        ))}
      </AnimatePresence>
    </ul>
  );
}

/** Alertes d'expiration : on peut les confirmer, ou maintenir l'effet (erreur de saisie, simplification de fin de combat). */
export function Alertes({
  alertes,
  onConfirmer,
  onMaintenir,
}: {
  alertes: AlerteVue[];
  onConfirmer: (a: AlerteVue) => void;
  onMaintenir?: (a: AlerteVue) => void;
}) {
  return (
    <div className="alertes-effets" role="alert" aria-live="assertive">
      <AnimatePresence initial={false}>
        {alertes.map((a) => (
          <motion.div
            key={a.id}
            className="alerte-effet"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, height: 0, marginBottom: 0 }}
          >
            <span className="alerte-icone" aria-hidden="true">⏱</span>
            <span className="alerte-texte">
              « <strong>{a.nom}</strong> » ({a.personnage}) {a.raison}. <span className="doux">{a.source} — {a.quand}</span>
            </span>
            {onMaintenir && (
              <button className="bouton discret" onClick={() => onMaintenir(a)} title="Remettre l'effet avec une durée saisie">
                Maintenir…
              </button>
            )}
            <button className="bouton" onClick={() => onConfirmer(a)}>
              OK
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}

// ------------------------------------------------------------------ sorts

/** Emplacements de sorts en gemmes (pleine : disponible, vide : dépensée) avec le chiffre écrit. */
export function Gemmes({ niveau, restants, total, dd }: { niveau: number; restants: number; total: number; dd?: number }) {
  return (
    <div className="gemmes" title={`Niveau ${niveau} : ${restants} sur ${total}${dd ? ` — DD ${dd}` : ""}`}>
      <span className="gemmes-niveau">{niveau}</span>
      <span className="gemmes-rangee" aria-hidden="true">
        {Array.from({ length: total }, (_, i) => (
          <span key={i} className={"gemme" + (i < restants ? " pleine" : "")} />
        ))}
      </span>
      <span className="gemmes-compte">
        {restants}/{total}
      </span>
    </div>
  );
}

// ------------------------------------------------------------------ dés physiques

/**
 * Saisie du résultat d'un dé physique : champ numérique, validation de la fourchette, rappel de ce
 * qu'il faut lancer, pavé de touches pour aller vite.
 */
export function SaisieDe({
  etiquette,
  rappel,
  min,
  max,
  valeur,
  onChange,
  pave = true,
  autoFocus = false,
}: {
  etiquette: string;
  /** Ce qu'il faut lancer et contre quoi (ex. « d20 + Constitution +1 contre DD 10 »). */
  rappel?: string;
  min: number;
  max: number;
  valeur: number | null;
  onChange: (v: number | null) => void;
  pave?: boolean;
  autoFocus?: boolean;
}) {
  const id = useId();
  const [texte, setTexte] = useState(valeur === null ? "" : String(valeur));
  useEffect(() => setTexte(valeur === null ? "" : String(valeur)), [valeur]);
  const hors = texte !== "" && (Number.isNaN(Number(texte)) || Number(texte) < min || Number(texte) > max);
  const poser = (t: string) => {
    const propre = t.replace(/[^0-9-]/g, "");
    setTexte(propre);
    if (propre === "") return onChange(null);
    const n = Number(propre);
    onChange(Number.isInteger(n) && n >= min && n <= max ? n : null);
  };
  const boutons = max <= 20 ? Array.from({ length: max - min + 1 }, (_, i) => min + i) : null;
  return (
    <div className={"saisie-de" + (hors ? " invalide" : "")}>
      <label htmlFor={id}>
        <span className="de-etiquette">{etiquette}</span>
        {rappel && <span className="de-rappel">{rappel}</span>}
      </label>
      <input
        id={id}
        inputMode="numeric"
        autoComplete="off"
        autoFocus={autoFocus}
        value={texte}
        placeholder={`${min} à ${max}`}
        onChange={(e) => poser(e.target.value)}
        aria-invalid={hors}
      />
      {hors && <span className="de-erreur">De {min} à {max}.</span>}
      {pave && boutons && (
        <div className="de-pave" role="group" aria-label={`Touches ${etiquette}`}>
          {boutons.map((n) => (
            <button key={n} type="button" className={"de-touche" + (valeur === n ? " choisie" : "")} onClick={() => poser(String(n))}>
              {n}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/** Nombre entier (dégâts, soins, durée) avec rappel de la source, sans pavé. */
export function SaisieNombre({
  etiquette,
  valeur,
  onChange,
  min = 0,
  max = 9999,
  rappel,
  autoFocus,
}: {
  etiquette: string;
  valeur: number | null;
  onChange: (v: number | null) => void;
  min?: number;
  max?: number;
  rappel?: string;
  autoFocus?: boolean;
}) {
  return <SaisieDe etiquette={etiquette} rappel={rappel} min={min} max={max} valeur={valeur} onChange={onChange} pave={false} autoFocus={autoFocus} />;
}

// ------------------------------------------------------------------ dialogue

export function Dialogue({
  titre,
  onFermer,
  children,
  actions,
  large = false,
}: {
  titre: string;
  onFermer: () => void;
  children: ReactNode;
  actions?: ReactNode;
  large?: boolean;
}) {
  useEffect(() => {
    const clavier = (e: KeyboardEvent) => {
      if (e.key === "Escape") onFermer();
    };
    window.addEventListener("keydown", clavier);
    return () => window.removeEventListener("keydown", clavier);
  }, [onFermer]);
  return (
    <motion.div className="voile" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} onMouseDown={onFermer}>
      <motion.div
        className={"dialogue" + (large ? " large" : "")}
        role="dialog"
        aria-modal="true"
        aria-label={titre}
        initial={{ opacity: 0, y: 14, scale: 0.98 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        transition={{ duration: 0.18 }}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <header>
          <h2>{titre}</h2>
          <button className="bouton icone discret" aria-label="Fermer" onClick={onFermer}>
            ✕
          </button>
        </header>
        <div className="dialogue-corps">{children}</div>
        {actions && <footer>{actions}</footer>}
      </motion.div>
    </motion.div>
  );
}

/** Choix d'un personnage parmi plusieurs. */
export function ChoixPj({
  liste,
  valeur,
  onChange,
}: {
  liste: { id: number; nom: string }[];
  valeur: number | null;
  onChange: (id: number) => void;
}) {
  return (
    <div className="choix-pj" role="radiogroup">
      {liste.map((p) => (
        <button key={p.id} role="radio" aria-checked={valeur === p.id} className={"bouton" + (valeur === p.id ? " principal" : "")} onClick={() => onChange(p.id)}>
          {p.nom}
        </button>
      ))}
    </div>
  );
}
