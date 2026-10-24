import { useState, useRef } from "react";
import { motion, AnimatePresence } from "motion/react";

const PERSOS = [
  { nom: "Mialee", init: 18 },
  { nom: "Valéros", init: 15 },
  { nom: "Seoni", init: 12 },
  { nom: "Kyra", init: 9 },
];
const EFFETS = [
  { id: 1, nom: "Bouclier", source: "sort, Mialee", restant: 3, total: 6 },
  { id: 2, nom: "Hâte", source: "sort, Seoni", restant: 4, total: 6 },
  { id: 3, nom: "Prière", source: "sort, Kyra", restant: 8, total: 10 },
];
const MAX = 31;

function Chrono({ restant, total }) {
  const r = 15;
  const c = 2 * Math.PI * r;
  return (
    <svg className="chrono" viewBox="0 0 38 38" aria-hidden="true">
      <circle className="fond" cx="19" cy="19" r={r} />
      <circle
        className={"reste" + (restant / total < 0.3 ? " bas" : "")}
        cx="19" cy="19" r={r}
        strokeDasharray={c}
        strokeDashoffset={c * (1 - restant / total)}
      />
    </svg>
  );
}

export default function App() {
  const [theme, setTheme] = useState("sombre");
  const [round, setRound] = useState(1);
  const [actif, setActif] = useState(0);
  const [persos, setPersos] = useState(PERSOS);
  const [pv, setPv] = useState({ cur: 23, temp: 0, nl: 0 });
  const [effets, setEffets] = useState(EFFETS);
  const [alertes, setAlertes] = useState([]);
  const [flottants, setFlottants] = useState([]);
  const [fx, setFx] = useState(null);
  const [ordre, setOrdre] = useState(["initiative", "sante", "effets"]);
  const [glisse, setGlisse] = useState(null);
  const [survol, setSurvol] = useState(null);
  const id = useRef(0);

  const tries = [...persos].sort((a, b) => b.init - a.init);
  const joueur = tries[actif % tries.length].nom;

  function effet(type, texte) {
    const n = ++id.current;
    setFlottants((f) => [...f, { n, type, texte }]);
    setTimeout(() => setFlottants((f) => f.filter((x) => x.n !== n)), 1100);
    setFx({ type, n });
  }
  function degats(n) {
    let reste = n;
    let temp = pv.temp;
    const absorbe = Math.min(temp, reste);
    temp -= absorbe;
    reste -= absorbe;
    setPv({ ...pv, temp, cur: Math.max(0, pv.cur - reste) });
    effet("degats", `−${n}`);
  }
  function soin(n) {
    setPv({ ...pv, cur: Math.min(MAX, pv.cur + n) });
    effet("soin", `+${n}`);
  }
  function temporaires(n) {
    setPv({ ...pv, temp: Math.max(pv.temp, n) });
    effet("autre", `+${n} temp.`);
  }
  function nonLetal(n) {
    setPv({ ...pv, nl: Math.min(MAX, pv.nl + n) });
    effet("autre", `${n} non létaux`);
  }
  function suivant() {
    if (actif + 1 >= tries.length) {
      setActif(0);
      setRound(round + 1);
      const restants = effets.map((e) => ({ ...e, restant: e.restant - 1 }));
      setEffets(restants.filter((e) => e.restant > 0));
      setAlertes((a) => [...a, ...restants.filter((e) => e.restant <= 0)]);
    } else setActif(actif + 1);
  }
  const bouge = (nom, d) =>
    setPersos(persos.map((p) => (p.nom === nom ? { ...p, init: p.init + d } : p)));
  const total = MAX + pv.temp;
  const pct = (n) => `${(n / total) * 100}%`;

  const panneaux = {
    initiative: (
      <div className="barre">
        {tries.map((p, i) => (
          <motion.div
            layout
            key={p.nom}
            className={"jeton" + (i === actif ? " actif" : "")}
            transition={{ type: "spring", stiffness: 420, damping: 34 }}
          >
            <div className="nom">{p.nom}</div>
            <div className="init">
              <button aria-label={`Initiative de ${p.nom} moins 1`} onClick={() => bouge(p.nom, -1)}>−</button>
              {p.init}
              <button aria-label={`Initiative de ${p.nom} plus 1`} onClick={() => bouge(p.nom, 1)}>+</button>
            </div>
            <div className="marque">{i === actif ? "▶ son tour" : ""}</div>
          </motion.div>
        ))}
      </div>
    ),
    sante: (
      <div>
        <div className="jauge-zone">
          <div
            className="jauge"
            key={fx ? fx.type + fx.n : "x"}
            style={
              fx
                ? fx.type === "degats"
                  ? { animation: "secousse 380ms, flash-d 520ms" }
                  : fx.type === "soin"
                    ? { animation: "flash-s 520ms" }
                    : undefined
                : undefined
            }
          >
            <div className="jauge-retard" style={{ width: pct(pv.cur) }} />
            <div className="jauge-pv" style={{ width: pct(pv.cur) }} />
            <div className="jauge-temp" style={{ left: pct(pv.cur), width: pct(pv.temp) }} />
            <div className="jauge-nl" style={{ width: pct(pv.nl), left: "auto", right: 0 }} />
          </div>
          {flottants.map((f) => (
            <span key={f.n} className={"flottant " + f.type}>{f.texte}</span>
          ))}
        </div>
        <div className="jauge-texte">
          <span>Mialee <b>{pv.cur}</b> / {MAX} PV</span>
          <span>temporaires : <b>{pv.temp}</b></span>
          <span>non létaux : <b>{pv.nl}</b></span>
          {pv.cur === 0 && <span style={{ color: "var(--degats)" }}>⚠ hors de combat</span>}
        </div>
        <div className="actions">
          <button onClick={() => degats(5)}>Prendre 5 dégâts</button>
          <button onClick={() => soin(5)}>Se soigner de 5</button>
          <button onClick={() => temporaires(4)}>4 PV temporaires</button>
          <button onClick={() => nonLetal(4)}>4 non létaux</button>
        </div>
      </div>
    ),
    effets: (
      <div>
        <div className="alertes">
          <AnimatePresence>
            {alertes.map((e) => (
              <motion.div
                key={e.id}
                className="alerte"
                initial={{ opacity: 0, y: -8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, height: 0 }}
              >
                <span>⏱ « {e.nom} » ({e.source}) a pris fin.</span>
                <button onClick={() => setAlertes(alertes.filter((a) => a.id !== e.id))}>OK</button>
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
        <div className="effets">
          <AnimatePresence>
            {effets.map((e) => (
              <motion.div
                layout
                key={e.id}
                className="effet"
                exit={{ opacity: 0, x: 30 }}
                transition={{ duration: 0.25 }}
              >
                <Chrono restant={e.restant} total={e.total} />
                <div>
                  <span className="nom">{e.nom}</span>
                  <small>{e.source} · {e.restant} round(s) restant(s)</small>
                </div>
              </motion.div>
            ))}
          </AnimatePresence>
        </div>
      </div>
    ),
  };
  const titres = { initiative: "Initiative", sante: "Santé", effets: "Effets en cours" };

  function deposer(cible) {
    if (!glisse || glisse === cible) return;
    const o = ordre.filter((x) => x !== glisse);
    o.splice(o.indexOf(cible), 0, glisse);
    setOrdre(o);
  }

  return (
    <div className="app">
      <div className="entete">
        <h1>Prototype — React + motion</h1>
        <span className="round">Round <b>{round}</b> · tour de {joueur}</span>
        <button className="principal" onClick={suivant}>Tour suivant</button>
        <button
          onClick={() => {
            const t = theme === "sombre" ? "clair" : "sombre";
            setTheme(t);
            document.documentElement.dataset.theme = t;
          }}
        >
          Thème
        </button>
      </div>
      <div className="panneaux">
        {ordre.map((k) => (
          <motion.section
            layout
            key={k}
            className={"panneau" + (glisse === k ? " glisse" : "") + (survol === k ? " survol" : "")}
            transition={{ type: "spring", stiffness: 400, damping: 36 }}
            onDragOver={(e) => { e.preventDefault(); setSurvol(k); }}
            onDragLeave={() => setSurvol(null)}
            onDrop={() => { deposer(k); setSurvol(null); }}
          >
            <h2
              draggable
              onDragStart={() => setGlisse(k)}
              onDragEnd={() => { setGlisse(null); setSurvol(null); }}
            >
              <span className="poignee">⠿</span>
              {titres[k]}
            </h2>
            {panneaux[k]}
          </motion.section>
        ))}
      </div>
    </div>
  );
}
