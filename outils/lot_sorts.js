// Affiche un lot de sorts à traduire : node outils/lot_sorts.js <sorts.json> <numero> [taille]
// Ordre : sorts de l'ensorceleur par niveau puis par nom, puis les sorts de lignage hors liste.
const fs = require('fs');
const path = require('path');
const [fichier, numero, taille = '10'] = process.argv.slice(2);
const sorts = JSON.parse(fs.readFileSync(fichier, 'utf8')).sorts;
const lignages = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'data', 'core', 'ensorceleur.json'), 'utf8')).classe.lignages;
const canon = (n) => n.split(' (')[0].replace(/[*,]/g, '').toLowerCase().split(/\s+/).filter(Boolean).sort().join(' ');
const surListe = sorts.filter((s) => s.niveaux.sorcerer !== undefined);
const noms = new Set(surListe.map((s) => s.nom_en));
const extras = [];
for (const l of lignages) {
  for (const b of l.sorts_bonus) {
    const s = sorts.find((x) => canon(x.nom_en) === canon(b.nom_en));
    if (s && !noms.has(s.nom_en)) { noms.add(s.nom_en); extras.push(s); }
  }
}
const dossierFr = path.join(__dirname, "..", "data", "core", "fr");
const deja = new Set();
for (const f of fs.readdirSync(dossierFr).filter((x) => /^sorts_d+.json$/.test(x))) for (const k of Object.keys(JSON.parse(fs.readFileSync(path.join(dossierFr, f), "utf8")))) deja.add(k);
const ordre = [...surListe, ...extras].filter((s) => !deja.has(s.nom_en));
const n = Number(taille);
const lot = ordre.slice((Number(numero) - 1) * n, Number(numero) * n);
console.log(`# lot ${numero} : ${lot.length} sorts affichés, ${ordre.length} restants à traduire`);
for (const s of lot) {
  console.log(`\n## ${s.nom_en} [${s.niveaux.sorcerer ?? 'hors liste'}]`);
  console.log(`composantes: ${s.composantes_en}`);
  if (s.cible_en) console.log(`cible (${s.cible_type}): ${s.cible_en}`);
  console.log(s.description_en);
}
