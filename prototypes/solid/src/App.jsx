import { createSignal, createMemo, For, Show } from "solid-js";
import { createStore } from "solid-js/store";

const MAX = 31;
const C = 2 * Math.PI * 15;
const titres = { initiative: "Initiative", sante: "Santé", effets: "Effets en cours" };

/** Animation FLIP maison : Solid n'a pas de transition de réordonnancement intégrée.
 *  On mesure les positions avant, on change l'état (le DOM est mis à jour tout de suite),
 *  puis on joue le déplacement inverse avec l'API d'animation du navigateur. */
function avecFlip(conteneur, selecteur, changement) {
  const avant = new Map();
  conteneur.querySelectorAll(selecteur).forEach((el) => avant.set(el.dataset.cle, el.getBoundingClientRect()));
  changement();
  conteneur.querySelectorAll(selecteur).forEach((el) => {
    const a = avant.get(el.dataset.cle);
    if (!a) return;
    const b = el.getBoundingClientRect();
    const dx = a.left - b.left;
    const dy = a.top - b.top;
    if (dx || dy) {
      el.animate([{ transform: `translate(${dx}px, ${dy}px)` }, { transform: "none" }], {
        duration: 350,
        easing: "cubic-bezier(.2,.7,.2,1)",
      });
    }
  });
}

export default function App() {
  const [theme, setTheme] = createSignal("sombre");
  const [round, setRound] = createSignal(1);
  const [actif, setActif] = createSignal(0);
  const [persos, setPersos] = createStore([
    { nom: "Mialee", init: 18 },
    { nom: "Valéros", init: 15 },
    { nom: "Seoni", init: 12 },
    { nom: "Kyra", init: 9 },
  ]);
  const [pv, setPv] = createStore({ cur: 23, temp: 0, nl: 0 });
  const [effets, setEffets] = createStore([
    { id: 1, nom: "Bouclier", source: "sort, Mialee", restant: 3, total: 6 },
    { id: 2, nom: "Hâte", source: "sort, Seoni", restant: 4, total: 6 },
    { id: 3, nom: "Prière", source: "sort, Kyra", restant: 8, total: 10 },
  ]);
  const [alertes, setAlertes] = createSignal([]);
  const [flottants, setFlottants] = createSignal([]);
  const [fx, setFx] = createSignal(null);
  const [ordre, setOrdre] = createSignal(["initiative", "sante", "effets"]);
  const [glisse, setGlisse] = createSignal(null);
  const [survol, setSurvol] = createSignal(null);
  let compteur = 0;
  let barre, panneaux;

  const tries = createMemo(() => [...persos].sort((a, b) => b.init - a.init));
  const joueur = () => tries()[actif() % tries().length].nom;
  const total = () => MAX + pv.temp;
  const pct = (n) => `${(n / total()) * 100}%`;

  function effet(type, texte) {
    const n = ++compteur;
    setFlottants((f) => [...f, { n, type, texte }]);
    setTimeout(() => setFlottants((f) => f.filter((x) => x.n !== n)), 1100);
    setFx({ type, n });
  }
  function degats(n) {
    let reste = n;
    const absorbe = Math.min(pv.temp, reste);
    reste -= absorbe;
    setPv({ temp: pv.temp - absorbe, cur: Math.max(0, pv.cur - reste) });
    effet("degats", `−${n}`);
  }
  const soin = (n) => { setPv("cur", Math.min(MAX, pv.cur + n)); effet("soin", `+${n}`); };
  const temporaires = (n) => { setPv("temp", Math.max(pv.temp, n)); effet("autre", `+${n} temp.`); };
  const nonLetal = (n) => { setPv("nl", Math.min(MAX, pv.nl + n)); effet("autre", `${n} non létaux`); };
  const bouge = (p, d) => avecFlip(barre, ".jeton", () => setPersos((x) => x.nom === p.nom, "init", (i) => i + d));

  function suivant() {
    if (actif() + 1 >= tries().length) {
      setActif(0);
      setRound(round() + 1);
      setEffets((e) => true, "restant", (r) => r - 1);
      const finis = effets.filter((e) => e.restant <= 0).map((e) => ({ ...e }));
      setAlertes((a) => [...a, ...finis]);
      setEffets((liste) => liste.filter((e) => e.restant > 0));
    } else setActif(actif() + 1);
  }
  function deposer(cible) {
    const g = glisse();
    if (!g || g === cible) return;
    avecFlip(panneaux, ".panneau", () => {
      const o = ordre().filter((x) => x !== g);
      o.splice(o.indexOf(cible), 0, g);
      setOrdre(o);
    });
  }
  function basculerTheme() {
    const t = theme() === "sombre" ? "clair" : "sombre";
    setTheme(t);
    document.documentElement.dataset.theme = t;
  }
  const animationJauge = () => {
    const f = fx();
    return !f ? "" : f.type === "degats" ? "secousse 380ms, flash-d 520ms" : f.type === "soin" ? "flash-s 520ms" : "";
  };

  const Initiative = () => (
    <div class="barre" ref={barre}>
      <For each={tries()}>
        {(p, i) => (
          <div class="jeton" classList={{ actif: i() === actif() }} data-cle={p.nom}>
            <div class="nom">{p.nom}</div>
            <div class="init">
              <button aria-label={`Initiative de ${p.nom} moins 1`} onClick={() => bouge(p, -1)}>−</button>
              {p.init}
              <button aria-label={`Initiative de ${p.nom} plus 1`} onClick={() => bouge(p, 1)}>+</button>
            </div>
            <div class="marque">{i() === actif() ? "▶ son tour" : ""}</div>
          </div>
        )}
      </For>
    </div>
  );

  const Sante = () => (
    <div>
      <div class="jauge-zone">
        <Show when={fx()?.n ?? 0} keyed>
          <div class="jauge" style={{ animation: animationJauge() }}>
            <div class="jauge-retard" style={{ width: pct(pv.cur) }} />
            <div class="jauge-pv" style={{ width: pct(pv.cur) }} />
            <div class="jauge-temp" style={{ left: pct(pv.cur), width: pct(pv.temp) }} />
            <div class="jauge-nl" style={{ width: pct(pv.nl), left: "auto", right: "0" }} />
          </div>
        </Show>
        <For each={flottants()}>{(f) => <span class={"flottant " + f.type}>{f.texte}</span>}</For>
      </div>
      <div class="jauge-texte">
        <span>Mialee <b>{pv.cur}</b> / {MAX} PV</span>
        <span>temporaires : <b>{pv.temp}</b></span>
        <span>non létaux : <b>{pv.nl}</b></span>
        <Show when={pv.cur === 0}><span style={{ color: "var(--degats)" }}>⚠ hors de combat</span></Show>
      </div>
      <div class="actions">
        <button onClick={() => degats(5)}>Prendre 5 dégâts</button>
        <button onClick={() => soin(5)}>Se soigner de 5</button>
        <button onClick={() => temporaires(4)}>4 PV temporaires</button>
        <button onClick={() => nonLetal(4)}>4 non létaux</button>
      </div>
    </div>
  );

  const Effets = () => (
    <div>
      <div class="alertes">
        <For each={alertes()}>
          {(e) => (
            <div class="alerte">
              <span>⏱ « {e.nom} » ({e.source}) a pris fin.</span>
              <button onClick={() => setAlertes(alertes().filter((a) => a.id !== e.id))}>OK</button>
            </div>
          )}
        </For>
      </div>
      <div class="effets">
        <For each={effets}>
          {(e) => (
            <div class="effet">
              <svg class="chrono" viewBox="0 0 38 38" aria-hidden="true">
                <circle class="fond" cx="19" cy="19" r="15" />
                <circle
                  class="reste"
                  classList={{ bas: e.restant / e.total < 0.3 }}
                  cx="19" cy="19" r="15"
                  stroke-dasharray={C}
                  stroke-dashoffset={C * (1 - e.restant / e.total)}
                />
              </svg>
              <div>
                <span class="nom">{e.nom}</span>
                <small>{e.source} · {e.restant} round(s) restant(s)</small>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  );
  const contenus = { initiative: Initiative, sante: Sante, effets: Effets };

  return (
    <div class="app">
      <div class="entete">
        <h1>Prototype — Solid</h1>
        <span class="round">Round <b>{round()}</b> · tour de {joueur()}</span>
        <button class="principal" onClick={suivant}>Tour suivant</button>
        <button onClick={basculerTheme}>Thème</button>
      </div>
      <div class="panneaux" ref={panneaux}>
        <For each={ordre()}>
          {(k) => {
            const Contenu = contenus[k];
            return (
              <section
                class="panneau"
                classList={{ glisse: glisse() === k, survol: survol() === k }}
                data-cle={k}
                onDragOver={(e) => { e.preventDefault(); setSurvol(k); }}
                onDragLeave={() => setSurvol(null)}
                onDrop={() => { deposer(k); setSurvol(null); }}
              >
                <h2 draggable={true} onDragStart={() => setGlisse(k)} onDragEnd={() => { setGlisse(null); setSurvol(null); }}>
                  <span class="poignee">⠿</span>{titres[k]}
                </h2>
                <Contenu />
              </section>
            );
          }}
        </For>
      </div>
    </div>
  );
}
