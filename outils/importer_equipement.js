// Import ponctuel de l'équipement du Core Rulebook (armures, boucliers, armes, matériel) depuis AoN.
// Usage : node outils/importer_equipement.js <cache> <dossier_de_sortie>
const fs = require('fs');
const path = require('path');
const https = require('https');

const [cache, sortie] = process.argv.slice(2);
if (!cache || !sortie) { console.error('usage : node importer_equipement.js <cache> <sortie>'); process.exit(1); }
fs.mkdirSync(path.join(cache, 'equip'), { recursive: true });
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
function telecharger(url, fichier) {
  const f = path.join(cache, fichier);
  if (fs.existsSync(f) && fs.statSync(f).size > 1500) return Promise.resolve(fs.readFileSync(f, 'utf8'));
  return new Promise((ok, ko) => {
    https.get(url, { headers: { 'User-Agent': 'gestion-pathfinder (usage personnel, hors ligne)' } }, (res) => {
      let s = ''; res.setEncoding('utf8');
      res.on('data', (d) => (s += d));
      res.on('end', async () => { fs.writeFileSync(f, s); await pause(600); ok(s); });
    }).on('error', ko);
  });
}
const dec = (s) => s.replace(/&nbsp;/g, ' ').replace(/&amp;/g, '&').replace(/&mdash;/g, '—').replace(/&#39;/g, "'").replace(/&quot;/g, '"');
function lignes(html) {
  return dec(html.replace(/<script[\s\S]*?<\/script>|<style[\s\S]*?<\/style>/g, '').replace(/<(br|\/p|\/h\d|\/tr|\/li|\/div)[^>]*>/g, '\n').replace(/<[^>]+>/g, ' '))
    .replace(/[ \t]+/g, ' ').split('\n').map((l) => l.trim()).filter(Boolean);
}
const nombre = (s) => (s === undefined || s === null || s === '—' ? null : Number(String(s).replace(/,/g, '').replace('+', '')));
const pagePrincipale = (l) => { const m = l.join(' ').match(/PRPG Core Rulebook pg\. (\d+)/); return m ? Number(m[1]) : null; };
const prix = (s) => { if (!s) return null; const m = s.match(/([\d,]+(?:\.\d+)?) ?(gp|sp|cp|pp)/); if (!m) return null; const v = Number(m[1].replace(/,/g, '')); return { valeur: v, unite: m[2] }; };
const poids = (s) => { if (!s) return null; const m = s.match(/([\d.,\/ ]+) ?lbs?\./); if (!m) return null; const t = m[1].trim(); if (t.includes('/')) { const [a, b] = t.split('/').map(Number); return a / b; } return Number(t.replace(/,/g, '')); };

function liste(source, titre, motif) {
  const d = source.indexOf(titre);
  const f = source.indexOf('<h2', d + 10);
  return [...source.slice(d, f).matchAll(motif)].map((m) => dec(decodeURIComponent(m[1])));
}

(async () => {
  const src = await telecharger('https://aonprd.com/SourceDisplay.aspx?FixedSource=PRPG%20Core%20Rulebook', 'source_core.html');
  const armures = [], armes = [], divers = [];
  for (const nom of liste(src, 'Equipment (Armor)', /EquipmentArmorDisplay\.aspx\?ItemName=([^"]+)"/g)) {
    const l = lignes(await telecharger('https://aonprd.com/EquipmentArmorDisplay.aspx?ItemName=' + encodeURIComponent(nom), 'equip/a_' + nom.replace(/\W+/g, '_') + '.html'));
    const i = l.findIndex((x) => x === nom || x.startsWith(nom + ' Source'));
    const bloc = l.slice(i).join('\n');
    const get = (re) => (bloc.match(re) || [])[1];
    armures.push({
      nom_en: nom, source: { livre: 'PRPG Core Rulebook', page: pagePrincipale(l.slice(i)) },
      cout: prix(get(/Cost ([^\n]*?)(?: Weight|\n)/)), poids_lb: poids(get(/Weight ([^\n]*?)(?: Description|\n)/)),
      bonus_armure: nombre(get(/Armor Bonus ([+\-–\d—]+)/)), dex_max: nombre(get(/Max Dex Bonus ([+\-–\d—]+)/)),
      malus_armure: nombre((get(/Armor Check Penalty ([+\-–\d—]+)/) || '').replace('–', '-')),
      echec_sorts_pct: nombre(get(/Arcane Spell Failure Chance (\d+)%/)),
      vitesse_9m_pi: nombre(get(/Speed ([\d—]+) ft\./)), vitesse_6m_pi: nombre(get(/Speed [\d—]+ ft\.\/([\d—]+) ft\./)),
      description_en: (bloc.match(/Description\n([\s\S]*?)(?:\n Site Owner|$)/) || [])[1] || '',
    });
  }
  for (const nom of liste(src, 'Equipment (Weapons)', /EquipmentWeaponsDisplay\.aspx\?ItemName=([^"]+)"/g)) {
    const l = lignes(await telecharger('https://aonprd.com/EquipmentWeaponsDisplay.aspx?ItemName=' + encodeURIComponent(nom), 'equip/w_' + nom.replace(/\W+/g, '_') + '.html'));
    const i = l.findIndex((x) => x === nom || x.startsWith(nom + ' Source'));
    const bloc = l.slice(i).join('\n');
    const get = (re) => (bloc.match(re) || [])[1];
    armes.push({
      nom_en: nom, source: { livre: 'PRPG Core Rulebook', page: pagePrincipale(l.slice(i)) },
      cout: prix(get(/Cost ([^\n]*?)(?: Weight|\n)/)), poids_lb: poids(get(/Weight ([^\n]*?)(?:\n|$)/)),
      degats_en: get(/Damage ([^;\n]*?)(?:;|\n)/) || null, critique_en: get(/Critical ([^;\n]*?)(?:;|\n)/) || null,
      portee_en: get(/Range ([^;\n]*?)(?:;|\n)/) || null, type_en: get(/Type ([^;\n]*?)(?:;|\n)/) || null,
      special_en: get(/Special ([^\n]*?)(?:\n|$)/) || null,
      categorie_en: get(/Category ([^;\n]*?)(?:;|\n)/) || null, maniement_en: get(/Proficiency ([^\n]*?)(?:\n| Weapon Groups|$)/) || null,
      groupes_en: get(/Weapon Groups ([^\n]*?)(?: Description|\n)/) || null,
      description_en: (bloc.match(/Description\n([\s\S]*?)(?:\n Site Owner|$)/) || [])[1] || '',
    });
  }
  for (const nom of liste(src, 'Equipment (Misc.)', /EquipmentMiscDisplay\.aspx\?ItemName=([^"]+)"/g)) {
    const l = lignes(await telecharger('https://aonprd.com/EquipmentMiscDisplay.aspx?ItemName=' + encodeURIComponent(nom), 'equip/m_' + nom.replace(/\W+/g, '_') + '.html'));
    const j = l.findIndex((x) => /^(Price|Cost) /.test(x));
    const ligne = j >= 0 ? l[j] : '';
    const p = (ligne.match(/(?:Price|Cost) (.*?)(?:; Weight|\bWeight|$)/) || [])[1] || null;
    const w = (ligne.match(/Weight (.*?)(?: Description|$)/) || [])[1] || null;
    divers.push({
      nom_en: nom, source: { livre: 'PRPG Core Rulebook', page: pagePrincipale(l) },
      prix_en: p, cout: prix(p), poids_en: w, poids_lb: poids(w),
      description_en: (l.slice(j + 1).find((x) => !/^Source|^Site Owner/.test(x)) || '').slice(0, 600),
    });
  }
  const cadre = { source: 'Archives of Nethys (aonprd.com), Core Rulebook, licence OGL' };
  fs.mkdirSync(sortie, { recursive: true });
  fs.writeFileSync(path.join(sortie, 'armures.json'), JSON.stringify({ ...cadre, armures }, null, 2) + '\n');
  fs.writeFileSync(path.join(sortie, 'armes.json'), JSON.stringify({ ...cadre, armes }, null, 2) + '\n');
  fs.writeFileSync(path.join(sortie, 'materiel.json'), JSON.stringify({ ...cadre, materiel: divers }, null, 2) + '\n');
  console.error(`armures ${armures.length}, armes ${armes.length}, matériel ${divers.length}`);
})().catch((e) => { console.error(e); process.exit(1); });
