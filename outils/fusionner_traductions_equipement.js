// Fusionne les traductions françaises (data/core/fr) dans les données d'équipement et d'états.
// Usage : node outils/fusionner_traductions_equipement.js data/core
const fs = require('fs');
const path = require('path');
const d = process.argv[2];
const lire = (f) => JSON.parse(fs.readFileSync(path.join(d, f), 'utf8'));
const ecrire = (f, o) => fs.writeFileSync(path.join(d, f), JSON.stringify(o, null, 2) + '\n');
const noms = lire('fr/equipement_noms.json');
let manques = [];
for (const [fichier, cle, liste] of [['armures.json', 'armures', 'armures'], ['armes.json', 'armes', 'armes'], ['materiel.json', 'materiel', 'materiel']]) {
  const o = lire(fichier);
  for (const x of o[liste]) {
    x.nom = noms[cle][x.nom_en];
    if (!x.nom) manques.push(x.nom_en);
  }
  ecrire(fichier, o);
}
const fr = lire('fr/etats.json').etats;
const e = lire('etats.json');
for (const x of e.etats) {
  const t = fr[x.id];
  if (!t) { manques.push('état ' + x.id); continue; }
  x.nom = t.nom; x.texte = t.texte;
}
ecrire('etats.json', e);
if (manques.length) { console.error('SANS TRADUCTION : ' + manques.join(', ')); process.exit(1); }
console.error('ok');
