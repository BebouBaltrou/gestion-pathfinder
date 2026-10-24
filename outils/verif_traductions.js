// Vérifie les lots de traduction des sorts : node outils/verif_traductions.js
const fs = require('fs');
const path = require('path');
const dossier = path.join(__dirname, '..', 'data', 'core', 'fr');
const sorts = JSON.parse(fs.readFileSync(path.join(__dirname, '..', 'data', 'core', 'sorts.json'), 'utf8')).sorts;
const lots = fs.readdirSync(dossier).filter((f) => /^sorts_\d+\.json$/.test(f)).sort();
const traduits = {};
for (const f of lots) Object.assign(traduits, JSON.parse(fs.readFileSync(path.join(dossier, f), 'utf8')));
const attendus = sorts.filter((s) => s.niveaux.sorcerer !== undefined);
const manquants = attendus.filter((s) => !traduits[s.nom_en]);
const inconnus = Object.keys(traduits).filter((n) => !sorts.find((s) => s.nom_en === n));
const sansDesc = Object.entries(traduits).filter(([, v]) => !v.description || v.description.length < 20).map(([n]) => n);
const sansCible = attendus.filter((s) => s.cible_en && traduits[s.nom_en] && !traduits[s.nom_en].cible).map((s) => s.nom_en);
const matDeCote = attendus.filter((s) => /\(/.test(s.composantes_en || '') && traduits[s.nom_en] && !traduits[s.nom_en].composantes).map((s) => s.nom_en);
console.log(`${Object.keys(traduits).length} sorts traduits sur ${attendus.length} (liste de l'ensorceleur) ; manquants : ${manquants.length}`);
if (inconnus.length) console.log('noms inconnus :', inconnus.join(', '));
if (sansDesc.length) console.log('sans description :', sansDesc.join(', '));
if (sansCible.length) console.log('cible à traduire :', sansCible.join(', '));
if (matDeCote.length) console.log('composantes matérielles à traduire :', matDeCote.join(', '));
