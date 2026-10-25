<script>
  import { flip } from "svelte/animate";
  import { fly, slide } from "svelte/transition";

  const MAX = 31;
  let theme = $state("sombre");
  let round = $state(1);
  let actif = $state(0);
  let persos = $state([
    { nom: "Mialee", init: 18 },
    { nom: "Valéros", init: 15 },
    { nom: "Seoni", init: 12 },
    { nom: "Kyra", init: 9 },
  ]);
  let pv = $state({ cur: 23, temp: 0, nl: 0 });
  let effets = $state([
    { id: 1, nom: "Bouclier", source: "sort, Mialee", restant: 3, total: 6 },
    { id: 2, nom: "Hâte", source: "sort, Seoni", restant: 4, total: 6 },
    { id: 3, nom: "Prière", source: "sort, Kyra", restant: 8, total: 10 },
  ]);
  let alertes = $state([]);
  let flottants = $state([]);
  let fx = $state(null);
  let ordre = $state(["initiative", "sante", "effets"]);
  let glisse = $state(null);
  let survol = $state(null);
  let compteur = 0;

  const tries = $derived([...persos].sort((a, b) => b.init - a.init));
  const joueur = $derived(tries[actif % tries.length].nom);
  const total = $derived(MAX + pv.temp);
  const pct = (n) => `${(n / total) * 100}%`;
  const titres = { initiative: "Initiative", sante: "Santé", effets: "Effets en cours" };
  const C = 2 * Math.PI * 15;

  function effet(type, texte) {
    const n = ++compteur;
    flottants.push({ n, type, texte });
    setTimeout(() => (flottants = flottants.filter((x) => x.n !== n)), 1100);
    fx = { type, n };
  }
  function degats(n) {
    let reste = n;
    const absorbe = Math.min(pv.temp, reste);
    pv.temp -= absorbe;
    reste -= absorbe;
    pv.cur = Math.max(0, pv.cur - reste);
    effet("degats", `−${n}`);
  }
  function soin(n) {
    pv.cur = Math.min(MAX, pv.cur + n);
    effet("soin", `+${n}`);
  }
  function temporaires(n) {
    pv.temp = Math.max(pv.temp, n);
    effet("autre", `+${n} temp.`);
  }
  function nonLetal(n) {
    pv.nl = Math.min(MAX, pv.nl + n);
    effet("autre", `${n} non létaux`);
  }
  function suivant() {
    if (actif + 1 >= tries.length) {
      actif = 0;
      round += 1;
      effets.forEach((e) => (e.restant -= 1));
      alertes.push(...effets.filter((e) => e.restant <= 0).map((e) => ({ ...e })));
      effets = effets.filter((e) => e.restant > 0);
    } else actif += 1;
  }
  function deposer(cible) {
    if (!glisse || glisse === cible) return;
    const o = ordre.filter((x) => x !== glisse);
    o.splice(o.indexOf(cible), 0, glisse);
    ordre = o;
  }
  function basculerTheme() {
    theme = theme === "sombre" ? "clair" : "sombre";
    document.documentElement.dataset.theme = theme;
  }
  const animationJauge = $derived(
    !fx ? "" : fx.type === "degats" ? "secousse 380ms, flash-d 520ms" : fx.type === "soin" ? "flash-s 520ms" : "",
  );
</script>

{#snippet initiative()}
  <div class="barre">
    {#each tries as p, i (p.nom)}
      <div class="jeton" class:actif={i === actif} animate:flip={{ duration: 350 }}>
        <div class="nom">{p.nom}</div>
        <div class="init">
          <button aria-label="Initiative de {p.nom} moins 1" onclick={() => p.init--}>−</button>
          {p.init}
          <button aria-label="Initiative de {p.nom} plus 1" onclick={() => p.init++}>+</button>
        </div>
        <div class="marque">{i === actif ? "▶ son tour" : ""}</div>
      </div>
    {/each}
  </div>
{/snippet}

{#snippet sante()}
  <div class="jauge-zone">
    {#key fx?.n}
      <div class="jauge" style:animation={animationJauge}>
        <div class="jauge-retard" style:width={pct(pv.cur)}></div>
        <div class="jauge-pv" style:width={pct(pv.cur)}></div>
        <div class="jauge-temp" style:left={pct(pv.cur)} style:width={pct(pv.temp)}></div>
        <div class="jauge-nl" style:width={pct(pv.nl)} style:left="auto" style:right="0"></div>
      </div>
    {/key}
    {#each flottants as f (f.n)}
      <span class="flottant {f.type}">{f.texte}</span>
    {/each}
  </div>
  <div class="jauge-texte">
    <span>Mialee <b>{pv.cur}</b> / {MAX} PV</span>
    <span>temporaires : <b>{pv.temp}</b></span>
    <span>non létaux : <b>{pv.nl}</b></span>
    {#if pv.cur === 0}<span style="color: var(--degats)">⚠ hors de combat</span>{/if}
  </div>
  <div class="actions">
    <button onclick={() => degats(5)}>Prendre 5 dégâts</button>
    <button onclick={() => soin(5)}>Se soigner de 5</button>
    <button onclick={() => temporaires(4)}>4 PV temporaires</button>
    <button onclick={() => nonLetal(4)}>4 non létaux</button>
  </div>
{/snippet}

{#snippet liste_effets()}
  <div class="alertes">
    {#each alertes as e (e.id)}
      <div class="alerte" in:fly={{ y: -8, duration: 200 }} out:slide={{ duration: 200 }}>
        <span>⏱ « {e.nom} » ({e.source}) a pris fin.</span>
        <button onclick={() => (alertes = alertes.filter((a) => a.id !== e.id))}>OK</button>
      </div>
    {/each}
  </div>
  <div class="effets">
    {#each effets as e (e.id)}
      <div class="effet" animate:flip={{ duration: 250 }} out:fly={{ x: 30, duration: 250 }}>
        <svg class="chrono" viewBox="0 0 38 38" aria-hidden="true">
          <circle class="fond" cx="19" cy="19" r="15" />
          <circle
            class="reste"
            class:bas={e.restant / e.total < 0.3}
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
    {/each}
  </div>
{/snippet}

<div class="app">
  <div class="entete">
    <h1>Prototype — Svelte 5</h1>
    <span class="round">Round <b>{round}</b> · tour de {joueur}</span>
    <button class="principal" onclick={suivant}>Tour suivant</button>
    <button onclick={basculerTheme}>Thème</button>
  </div>
  <div class="panneaux">
    {#each ordre as k (k)}
      <section
        class="panneau"
        class:glisse={glisse === k}
        class:survol={survol === k}
        animate:flip={{ duration: 300 }}
        ondragover={(e) => { e.preventDefault(); survol = k; }}
        ondragleave={() => (survol = null)}
        ondrop={() => { deposer(k); survol = null; }}
      >
        <h2
          draggable="true"
          ondragstart={() => (glisse = k)}
          ondragend={() => { glisse = null; survol = null; }}
        >
          <span class="poignee">⠿</span>{titres[k]}
        </h2>
        {#if k === "initiative"}{@render initiative()}{:else if k === "sante"}{@render sante()}{:else}{@render liste_effets()}{/if}
      </section>
    {/each}
  </div>
</div>
