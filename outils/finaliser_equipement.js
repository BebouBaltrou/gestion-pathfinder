// Nettoie les relevés bruts de l'équipement (armures.json, armes.json, materiel.json) :
// ajoute le genre des protections (table 6-? du Core : légère / intermédiaire / lourde / bouclier),
// retire les descriptions parasites du site et les doublons, écrit data/core/charge.json.
// Usage : node outils/finaliser_equipement.js data/core
const fs = require('fs');
const path = require('path');
const dossier = process.argv[2];
const lire = (f) => JSON.parse(fs.readFileSync(path.join(dossier, f), 'utf8'));
const ecrire = (f, o) => fs.writeFileSync(path.join(dossier, f), JSON.stringify(o, null, 2) + '\n');

const GENRES = {
  Padded: 'legere', Leather: 'legere', 'Studded leather': 'legere', 'Chain shirt': 'legere',
  Hide: 'intermediaire', 'Scale mail': 'intermediaire', Chainmail: 'intermediaire', Breastplate: 'intermediaire',
  'Splint mail': 'lourde', 'Banded mail': 'lourde', 'Half-plate': 'lourde', 'Full plate': 'lourde',
  Buckler: 'bouclier', 'Light wooden': 'bouclier', 'Light steel': 'bouclier', 'Heavy wooden': 'bouclier', 'Heavy steel': 'bouclier', Tower: 'bouclier',
  'Armor spikes': 'accessoire', 'Locked gauntlet': 'accessoire', 'Shield spikes': 'accessoire',
};
const nettoyer = (s) => (s || '').replace(/\s*Site Owner:.*$/s, '').trim();

const a = lire('armures.json');
for (const x of a.armures) {
  x.genre = GENRES[x.nom_en];
  if (!x.genre) throw new Error('genre inconnu : ' + x.nom_en);
  x.description_en = nettoyer(x.description_en);
}
ecrire('armures.json', a);

const w = lire('armes.json');
const vus = new Set();
w.armes = w.armes.filter((x) => (vus.has(x.nom_en) ? false : (vus.add(x.nom_en), true)));
for (const x of w.armes) {
  x.description_en = nettoyer(x.description_en);
  x.maniement_en = ((x.maniement_en || '').match(/^(Simple|Martial|Exotic)/) || [])[1] || null;
}
ecrire('armes.json', w);

const m = lire('materiel.json');
for (const x of m.materiel) {
  const cat = ((x.description_en || '').match(/^Category (.*?) Description/) || [])[1];
  x.categorie_en = cat || x.categorie_en || null;
  delete x.description_en;
}
ecrire('materiel.json', m);

// Table 7-4 (Core, « Carrying Capacity ») : charge légère, intermédiaire, lourde par FOR 1 à 29.
const brut = fs.readFileSync(path.join(process.env.TEMP, 'aon_cache', 'carry.html'), 'utf8')
  .replace(/<script[\s\S]*?<\/script>|<style[\s\S]*?<\/style>/g, '').replace(/<\/tr>/g, '\n')
  .replace(/<[^>]+>/g, ' ').replace(/&nbsp;/g, ' ').replace(/[ \t]+/g, ' ');
const lignes = [];
for (const l of brut.split('\n').map((x) => x.trim())) {
  const g = l.match(/^(\d+) ([\d,]+) lbs\. or less ([\d,]+)-([\d,]+) lbs\. ([\d,]+)-([\d,]+) lbs\.$/);
  if (!g) continue;
  const n = (s) => Number(s.replace(/,/g, ''));
  lignes.push({ force: n(g[1]), legere: n(g[2]), intermediaire_max: n(g[4]), lourde: n(g[6]), brut: l });
}
if (lignes.length !== 29) throw new Error('lignes de charge : ' + lignes.length);
// La ligne FOR 15 du site se chevauche (« 67-134 » puis « 134-200 ») : on garde la charge
// légère et la charge maximale du site et on place la limite intermédiaire à 133 ; point à confirmer.
const ligne15 = lignes.find((x) => x.force === 15);
ligne15.intermediaire_max = 133;
ligne15.a_confirmer = 'le site indique 67-134 puis 134-200 (chevauchement) ; limite intermédiaire prise à 133';
// Vitesse réduite (Core, « Armor and Encumbrance for Other Base Speeds ») : [vitesse min, max, réduite], en pieds.
const vitesse_reduite = [[5, 5, 5], [10, 15, 10], [20, 20, 15], [25, 30, 20], [35, 35, 25], [40, 45, 30], [50, 50, 35],
  [55, 60, 40], [65, 65, 45], [70, 75, 50], [80, 80, 55], [85, 90, 60], [95, 95, 65], [100, 105, 70], [110, 110, 75], [115, 120, 80]];
ecrire('charge.json', {
  source: { livre: 'PRPG Core Rulebook', page: 169, note: 'Table 7-4 : Carrying Capacity (Archives of Nethys)' },
  vitesse_reduite,
  au_dela_de_29: 'chaque tranche de 10 points de FOR au-dessus multiplie les valeurs par 4 (Core)',
  lignes: lignes.map(({ brut, ...r }) => r),
});
console.error('ok');
