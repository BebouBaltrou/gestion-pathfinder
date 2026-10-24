// Affiche un lot de dons à traduire : node outils/lot_dons.js <taille>  (les dons déjà traduits sont sautés)
const fs = require('fs');
const path = require('path');
const dossier = path.join(__dirname, '..', 'data', 'core', 'fr');
const dons = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'data', 'core', 'dons.json'), 'utf8')).dons;
const deja = new Set();
for (const f of fs.readdirSync(dossier).filter((x) => /^dons_\d+\.json$/.test(x))) for (const k of Object.keys(JSON.parse(fs.readFileSync(path.join(dossier, f), 'utf8')))) deja.add(k);
const reste = dons.filter((d) => !deja.has(d.nom_en));
const lot = reste.slice(0, Number(process.argv[2] || 20));
console.log(`# ${lot.length} dons affichés, ${reste.length} restants`);
for (const d of lot) {
  console.log(`\n## ${d.nom_en} (${d.types_en.join(', ') || 'général'})`);
  if (d.resume_en) console.log('Résumé: ' + d.resume_en);
  if (d.prerequis_en) console.log('Prérequis: ' + d.prerequis_en);
  if (d.avantage_en) console.log('Avantage: ' + d.avantage_en);
  if (d.normal_en) console.log('Normal: ' + d.normal_en);
  if (d.special_en) console.log('Spécial: ' + d.special_en);
}
