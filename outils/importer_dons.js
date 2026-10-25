// Import ponctuel des dons du Core Rulebook depuis Archives of Nethys.
// Usage : node outils/importer_dons.js <dossier_de_cache> <fichier_de_sortie>
// Les prérequis reconnus sont structurés ; les autres restent en texte (« à vérifier »).
const fs = require('fs');
const path = require('path');
const https = require('https');

const [cache, sortie] = process.argv.slice(2);
if (!cache || !sortie) { console.error('usage : node importer_dons.js <cache> <sortie.json>'); process.exit(1); }
fs.mkdirSync(path.join(cache, 'dons'), { recursive: true });
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

const CARAC = { str: 'for', dex: 'dex', con: 'con', int: 'int', wis: 'sag', cha: 'cha' };
const COMP = ['acrobatics', 'appraise', 'bluff', 'climb', 'craft', 'diplomacy', 'disable device', 'disguise', 'escape artist', 'fly',
  'handle animal', 'heal', 'intimidate', 'knowledge', 'linguistics', 'perception', 'perform', 'profession', 'ride', 'sense motive',
  'sleight of hand', 'spellcraft', 'stealth', 'survival', 'swim', 'use magic device'];

// Un terme de prérequis, structuré si on le reconnaît.
function terme(t, nomsDons) {
  t = t.trim().replace(/\.$/, '');
  let m;
  if ((m = t.match(/^(Str|Dex|Con|Int|Wis|Cha) (\d+)$/))) return { genre: 'caracteristique', carac: CARAC[m[1].toLowerCase()], min: Number(m[2]) };
  if ((m = t.match(/^base attack bonus \+(\d+)$/i))) return { genre: 'bba', min: Number(m[1]) };
  if ((m = t.match(/^character level (\d+)(?:st|nd|rd|th)$/i))) return { genre: 'niveau_personnage', min: Number(m[1]) };
  if ((m = t.match(/^caster level (\d+)(?:st|nd|rd|th)$/i))) return { genre: 'niveau_lanceur', min: Number(m[1]) };
  if ((m = t.match(/^(.+?) (\d+) ranks?(?: \((.+)\))?$/i)) && COMP.includes(m[1].toLowerCase())) {
    return { genre: 'rangs', competence: m[1].toLowerCase().replace(/ /g, '_'), sous_type: m[3] ? m[3].toLowerCase() : null, min: Number(m[2]) };
  }
  if ((m = t.match(/^(?:ability to cast|able to cast) (\d+)(?:st|nd|rd|th)-level (arcane |divine )?spells?$/i))) return { genre: 'sorts_de_niveau', niveau_de_sort: Number(m[1]), type: m[2] ? m[2].trim() : null };
  if ((m = t.match(/^(\w[\w ]*?) level (\d+)(?:st|nd|rd|th)$/i)) && !/caster|character/i.test(m[1])) return { genre: 'niveau_de_classe', classe: m[1].toLowerCase(), min: Number(m[2]) };
  if ((m = t.match(/^(\d+)(?:st|nd|rd|th)-level (\w+)$/i))) return { genre: 'niveau_de_classe', classe: m[2].toLowerCase(), min: Number(m[1]) };
  const alias = { 'Light Armor Proficiency': 'Armor Proficiency, Light', 'Medium Armor Proficiency': 'Armor Proficiency, Medium', 'Heavy Armor Proficiency': 'Armor Proficiency, Heavy' };
  if (alias[t]) return { genre: 'don', nom_en: alias[t] };
  if (nomsDons.has(t)) return { genre: 'don', nom_en: t };
  if ((m = t.match(/^(.+?) \((.+)\)$/)) && nomsDons.has(m[1])) return { genre: 'don', nom_en: t };
  return { genre: 'texte', texte_en: t };
}
function decouper(s) {
  // virgules hors parenthèses
  const out = []; let niv = 0; let cur = '';
  for (const ch of s) {
    if (ch === '(') niv++;
    if (ch === ')') niv--;
    if (ch === ',' && niv === 0) { out.push(cur); cur = ''; } else cur += ch;
  }
  if (cur.trim()) out.push(cur);
  return out;
}
function prerequis(texteBrut, nomsDons) {
  if (!texteBrut) return [];
  return decouper(texteBrut).map((t) => {
    const parts = t.split(/ or /);
    if (parts.length > 1 && parts.every((p) => terme(p, nomsDons).genre !== 'texte')) return { genre: 'ou', termes: parts.map((p) => terme(p, nomsDons)) };
    return terme(t, nomsDons);
  });
}

function analyser(nom, html, nomsDons) {
  const l = lignes(html);
  const iTitre = l.findIndex((x) => x.startsWith(nom + ' (') || x === nom);
  if (iTitre < 0) throw new Error('titre introuvable');
  const types = ((l[iTitre].match(/\(([^)]*)\)$/) || [])[1] || '').split(',').map((s) => s.trim()).filter(Boolean);
  const src = l[iTitre + 1];
  if (!src || !src.startsWith('Source ') || !src.includes('PRPG Core Rulebook')) throw new Error('source non Core : ' + src);
  const page = Number((src.match(/PRPG Core Rulebook pg\. (\d+)/) || [])[1]);
  const champs = {};
  let resume = [];
  let courant = null;
  for (let i = iTitre + 2; i < l.length; i++) {
    const x = l[i];
    if (/^Site Owner:/.test(x) || /^Source /.test(x)) break;
    const m = x.match(/^(Prerequisites?|Benefits?|Normal|Special|Goal|Completion Benefit|Note) ?: ?(.*)$/);
    if (m) { courant = m[1].replace(/s$/, ''); champs[courant] = m[2]; continue; }
    if (courant) champs[courant] += '\n' + x; else resume.push(x);
  }
  // le dernier champ peut contenir le titre de la section suivante (« Mythic X ») : on le coupe
  for (const k of Object.keys(champs)) champs[k] = champs[k].replace(/ (Mythic|Combat Trick|Stamina)[^\n]*$/, '').trim();
  const pre = (champs.Prerequisite || '').replace(/\.$/, '');
  return {
    nom_en: nom, types_en: types,
    source: { livre: 'PRPG Core Rulebook', page, url: 'https://aonprd.com/FeatDisplay.aspx?ItemName=' + encodeURIComponent(nom) },
    resume_en: resume.join(' '),
    prerequis_en: pre || null, prerequis: prerequis(pre, nomsDons),
    avantage_en: champs.Benefit || champs['Completion Benefit'] || null,
    normal_en: champs.Normal || null, special_en: champs.Special || null,
  };
}

(async () => {
  const liste = await telecharger('https://aonprd.com/SourceDisplay.aspx?FixedSource=PRPG%20Core%20Rulebook', 'source_core.html');
  const debut = liste.indexOf('Feats [');
  const fin = liste.indexOf('<h2', debut + 10);
  const noms = [];
  for (const m of liste.slice(debut, fin).matchAll(/FeatDisplay\.aspx\?ItemName=([^"]+)"/g)) noms.push(decoder(decodeURIComponent(m[1])));
  console.error(noms.length + ' dons dans le Core Rulebook');
  const set = new Set(noms);
  const dons = [];
  for (const nom of noms) {
    const f = 'dons/' + nom.replace(/[^A-Za-z0-9]+/g, '_') + '.html';
    const html = await telecharger('https://aonprd.com/FeatDisplay.aspx?ItemName=' + encodeURIComponent(nom), f);
    try { dons.push(analyser(nom, html, set)); } catch (e) { console.error('ERREUR ' + nom + ' : ' + e.message); }
  }
  fs.writeFileSync(sortie, JSON.stringify({ source: 'Archives of Nethys (aonprd.com), Core Rulebook, licence OGL', dons }, null, 2) + '\n');
  const nonStruct = dons.flatMap((d) => d.prerequis.filter((p) => p.genre === 'texte').map((p) => d.nom_en + ' : ' + p.texte_en));
  console.error('terminé : ' + dons.length + ' dons, ' + nonStruct.length + ' prérequis laissés en texte');
  fs.writeFileSync(sortie + '.texte.txt', nonStruct.join('\n'));
})().catch((e) => { console.error(e); process.exit(1); });
