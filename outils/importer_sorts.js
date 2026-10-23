// Import ponctuel des sorts du Core Rulebook (liste de l'ensorceleur) depuis Archives of Nethys.
// Usage : node outils/importer_sorts.js <dossier_de_cache> <fichier_de_sortie>
// Une requête par sort, avec pause (reprise possible : les pages déjà reçues sont gardées).
const fs = require('fs');
const path = require('path');
const https = require('https');

const [cache, sortie] = process.argv.slice(2);
if (!cache || !sortie) { console.error('usage : node importer_sorts.js <cache> <sortie.json>'); process.exit(1); }
fs.mkdirSync(path.join(cache, 'sorts'), { recursive: true });
const pause = (ms) => new Promise((r) => setTimeout(r, ms));

function telecharger(url, fichier) {
  const f = path.join(cache, fichier);
  if (fs.existsSync(f) && fs.statSync(f).size > 2000) return Promise.resolve(fs.readFileSync(f, 'utf8'));
  return new Promise((ok, ko) => {
    https.get(url, { headers: { 'User-Agent': 'gestion-pathfinder (usage personnel, hors ligne)' } }, (res) => {
      let s = ''; res.setEncoding('utf8');
      res.on('data', (d) => (s += d));
      res.on('end', async () => { fs.writeFileSync(f, s); await pause(700); ok(s); });
    }).on('error', ko);
  });
}
function decoder(s) {
  return s.replace(/&nbsp;/g, ' ').replace(/&amp;/g, '&').replace(/&mdash;/g, '—').replace(/&#39;/g, "'")
    .replace(/&quot;/g, '"').replace(/&lt;/g, '<').replace(/&gt;/g, '>');
}
function lignes(html) {
  return decoder(html.replace(/<script[\s\S]*?<\/script>|<style[\s\S]*?<\/style>/g, '')
    .replace(/<(br|\/p|\/h\d|\/tr|\/li|\/div)[^>]*>/g, '\n').replace(/<[^>]+>/g, ' '))
    .replace(/[ \t]+/g, ' ').split('\n').map((l) => l.trim()).filter(Boolean);
}

function analyser(nom, html) {
  const l = lignes(html);
  const iNom = l.findIndex((x) => x === nom);
  const debut = iNom >= 0 ? iNom : 0;
  const src = l.slice(debut).find((x) => x.startsWith('Source '));
  if (!src || !src.includes('PRPG Core Rulebook')) return null;
  const premiereSource = src.split(/ ?, ?/)[0];
  if (!premiereSource.includes('PRPG Core Rulebook')) return null;
  const page = Number((src.match(/PRPG Core Rulebook pg\. (\d+)/) || [])[1]);
  const iEcole = l.findIndex((x, i) => i > debut && x.startsWith('School '));
  const ecoleLigne = l[iEcole];
  const m = ecoleLigne.match(/^School (\S+)(?: \(([^)]*)\))?\s*(?:\[([^\]]*)\])?\s*; Level (.*?)(?: Casting)?$/);
  if (!m) throw new Error('ligne École illisible : ' + ecoleLigne);
  const niveaux = {};
  for (const part of m[4].split(/, (?=[A-Za-z\/ ]+ \d+)/)) {
    const mm = part.trim().match(/^(.+?) (\d+)$/);
    if (mm) niveaux[mm[1]] = Number(mm[2]);
  }
  // Champs : on lit les lignes entre « School » et « Description ».
  let i = iEcole + 1;
  const champs = {};
  const etiquettes = ['Casting Time', 'Components', 'Range', 'Duration', 'Saving Throw'];
  const reCible = /^((?:Targets?|Area|Effect)(?:(?:, or |, | or | and )(?:Targets?|Area|Effect))*) (.*)$/;
  let derniere = null;
  const desc = [];
  let enDescription = false;
  for (; i < l.length; i++) {
    let x = l[i];
    if (/^Site Owner:/.test(x)) break;
    if (!enDescription) {
      if (x === 'Description') { enDescription = true; continue; }
      let trouve = false;
      if (x.startsWith('Spell Resistance ')) {
        let v = x.slice(17);
        if (v.endsWith(' Description')) { v = v.slice(0, -12); enDescription = true; }
        champs.RM = v; derniere = 'RM'; continue;
      }
      const mc = x.match(reCible);
      if (mc) {
        champs.Cible = mc[2]; champs.CibleType = mc[1]; derniere = 'Cible'; trouve = true;
      }
      for (const e of trouve ? [] : etiquettes) {
        if (x.startsWith(e + ' ')) {
          let v = x.slice(e.length + 1);
          if (v.endsWith(' Description')) { v = v.slice(0, -12); enDescription = true; }
          v = v.replace(/ (Effect|Description)$/, '');
          const mm = v.match(/^(.*?) Spell Resistance (.*?) Description (.*)$/);
          if (mm) { v = mm[1]; champs.RM = mm[2]; desc.push(mm[3]); enDescription = true; }
          champs[e] = v; derniere = e; trouve = true; break;
        }
      }
      if (!trouve && derniere && !enDescription) champs[derniere] += ' ' + x;
    } else {
      if (/^Mythic /.test(x) && l[i + 1] && l[i + 1].startsWith('Source ')) break;
      desc.push(x);
    }
  }
  const sauv = champs['Saving Throw'] || '';
  const sm = sauv.match(/^(.*?); Spell Resistance (.*?)(?: Description)?$/);
  return {
    nom_en: nom,
    source: { livre: 'PRPG Core Rulebook', page, url: 'https://aonprd.com/SpellDisplay.aspx?ItemName=' + encodeURIComponent(nom) },
    ecole: m[1], sous_ecole: m[2] || null, descripteurs: m[3] ? m[3].split(',').map((s) => s.trim()) : [],
    niveaux, temps_d_incantation_en: champs['Casting Time'] || null, composantes_en: champs['Components'] || null,
    portee_en: champs['Range'] || null,
    cible_en: champs.Cible || null,
    cible_type: champs.CibleType || null,
    duree_en: champs['Duration'] || null,
    jet_de_sauvegarde_en: sm ? sm[1] : sauv, resistance_a_la_magie_en: sm ? sm[2] : champs.RM || null,
    description_en: desc.join('\n'),
  };
}

(async () => {
  // Liste officielle des sorts du Core Rulebook : page « source » d'Archives of Nethys.
  const liste = await telecharger('https://aonprd.com/SourceDisplay.aspx?FixedSource=PRPG%20Core%20Rulebook', 'source_core.html');
  const debut = liste.indexOf('Spells [');
  const fin = liste.indexOf('<h2', debut + 10);
  const noms = new Set();
  for (const m of liste.slice(debut, fin).matchAll(/SpellDisplay\.aspx\?ItemName=([^"]+)"/g)) noms.add(decoder(decodeURIComponent(m[1])));
  console.error(noms.size + ' sorts dans le Core Rulebook');
  const sorts = [];
  let n = 0;
  for (const nom of noms) {
    const f = 'sorts/' + nom.replace(/[^A-Za-z0-9]+/g, '_') + '.html';
    const html = await telecharger('https://aonprd.com/SpellDisplay.aspx?ItemName=' + encodeURIComponent(nom), f);
    try {
      const s = analyser(nom, html);
      if (s) sorts.push(s);
    } catch (e) { console.error('ERREUR ' + nom + ' : ' + e.message); }
    if (++n % 100 === 0) console.error(n + ' pages traitées, ' + sorts.length + ' sorts du Core');
  }
  // Tous les sorts du Core (les autres classes en auront besoin) ; tri par niveau d'ensorceleur puis par nom.
  const clef = (x) => (x.niveaux.sorcerer === undefined ? 99 : x.niveaux.sorcerer);
  sorts.sort((a, b) => clef(a) - clef(b) || a.nom_en.localeCompare(b.nom_en));
  fs.writeFileSync(sortie, JSON.stringify({ source: 'Archives of Nethys (aonprd.com), Core Rulebook, licence OGL', sorts }, null, 2) + '\n');
  console.error('terminé : ' + sorts.length + ' sorts du Core Rulebook');
})().catch((e) => { console.error(e); process.exit(1); });
