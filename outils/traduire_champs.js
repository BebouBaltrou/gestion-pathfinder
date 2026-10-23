// Traduction mécanique (non officielle) des champs normalisés des sorts : temps d'incantation,
// portée, durée, jet de sauvegarde, résistance à la magie, composantes, école, descripteurs.
// Les cibles, les composantes matérielles et les descriptions sont traduites à la main (lots).
// Usage : node outils/traduire_champs.js <sorts.json> [--residus]
const fs = require('fs');
const sorts = JSON.parse(fs.readFileSync(process.argv[2], 'utf8')).sorts;
const montrerResidus = process.argv.includes('--residus');
const manuel = JSON.parse(fs.readFileSync(require('path').join(__dirname, '..', 'data', 'core', 'fr', 'champs_manuels.json'), 'utf8'));
const deja = (champ, en) => (manuel[champ] && en in manuel[champ] ? manuel[champ][en] : null);

const metres = (ft) => {
  const m = Number(ft.replace(/,/g, '')) * 0.3;
  const t = Math.round(m * 100) / 100;
  return String(t).replace('.', ',') + ' m';
};
function pieds(s) {
  return s.replace(/(\d[\d,]*)(?:-| )(?:ft\.|feet|foot)/g, (_, n) => metres(n))
    .replace(/(\d[\d,]*)(?:-)?ft\./g, (_, n) => metres(n));
}
const ordonne = (s, regles) => regles.reduce((acc, [re, par]) => acc.replace(re, par), s);

const TEMPS = [
  [/standard action/g, 'action simple'], [/full-round action/g, 'action complexe'], [/immediate action/g, 'action immédiate'],
  [/swift action/g, 'action rapide'], [/minutes/g, 'minutes'], [/minute per page/g, 'minute par page'],
  [/\bminute\b/g, 'minute'], [/\brounds\b/g, 'rounds'], [/\bround\b/g, 'round'], [/\bhour\b/g, 'heure'],
  [/see text/g, 'voir texte'], [/at least/g, 'au moins'], [/\bor\b/g, 'ou'],
];
const PORTEE = [
  [/^personal$/, 'personnelle'], [/\bpersonal\b/g, 'personnelle'], [/\btouch\b/g, 'contact'], [/\bclose\b/g, 'courte'],
  [/\bmedium\b/g, 'moyenne'], [/\blong\b/g, 'longue'], [/\bunlimited\b/g, 'illimitée'], [/see text/g, 'voir texte'],
  [/\/level/g, '/niveau'], [/\/2 levels/g, '/2 niveaux'], [/\bor\b/g, 'ou'], [/\bsee below\b/g, 'voir ci-dessous'],
];
const DUREE = [
  [/instantaneous/g, 'instantanée'], [/permanent/g, 'permanente'], [/\(D\)/g, '(R)'], [/see text/g, 'voir texte'],
  [/\bconcentration\b/g, 'concentration'], [/rounds?\/level/g, (m) => m.replace('level', 'niveau')],
  [/min\.\/level/g, 'min./niveau'], [/hr\.\/level/g, 'h/niveau'], [/hours?\/level/g, (m) => m.replace('hour', 'heure').replace('level', 'niveau')],
  [/days?\/level/g, (m) => m.replace('day', 'jour').replace('level', 'niveau')], [/minutes?\/level/g, (m) => m.replace('level', 'niveau')],
  [/\bhours\b/g, 'heures'], [/\bhour\b/g, 'heure'], [/\bminutes\b/g, 'minutes'], [/\bminute\b/g, 'minute'],
  [/\brounds\b/g, 'rounds'], [/\bround\b/g, 'round'], [/\bdays\b/g, 'jours'], [/\bday\b/g, 'jour'],
  [/\bor\b/g, 'ou'], [/\band\b/g, 'et'], [/\bplus\b/g, 'plus'], [/\buntil\b/g, "jusqu'à"], [/\bdischarged\b/g, 'déchargé'],
  [/\bup to\b/g, "jusqu'à"], [/\bor until\b/g, 'ou jusqu’à'],
];
const SAUVEGARDE = [
  [/Fortitude/g, 'Vigueur'], [/Reflex/g, 'Réflexes'], [/Will/g, 'Volonté'], [/\bnone\b/g, 'aucun'],
  [/negates/g, 'annule'], [/partial/g, 'partiel'], [/\bhalf\b/g, 'moitié'], [/\(harmless\)/g, '(inoffensif)'],
  [/\(object\)/g, '(objet)'], [/\bharmless\b/g, 'inoffensif'], [/\bobject\b/g, 'objet'], [/see text/g, 'voir texte'],
  [/\bor\b/g, 'ou'], [/\band\b/g, 'et'], [/\bdisbelief\b/g, 'dévoilement'], [/\(if interacted with\)/g, "(en cas d'interaction)"],
  [/\bif\b/g, 'si'],
];
const RM = [
  [/\byes\b/g, 'oui'], [/\bno\b/g, 'non'], [/\(harmless\)/g, '(inoffensif)'], [/\(object\)/g, '(objet)'],
  [/\bharmless\b/g, 'inoffensif'], [/\bobject\b/g, 'objet'], [/see text/g, 'voir texte'], [/\bor\b/g, 'ou'], [/\band\b/g, 'et'],
];
const ECOLES = { abjuration: 'abjuration', conjuration: 'invocation', divination: 'divination', enchantment: 'enchantement',
  evocation: 'évocation', illusion: 'illusion', necromancy: 'nécromancie', transmutation: 'transmutation', universal: 'universelle' };
const SOUS = { calling: 'appel', creation: 'création', healing: 'guérison', summoning: 'convocation', teleportation: 'téléportation',
  charm: 'charme', compulsion: 'coercition', figment: 'hallucination', glamer: 'chimère', pattern: 'motif', phantasm: 'fantasme',
  shadow: 'ombre', polymorph: 'métamorphose', scrying: 'scrutation' };
const DESC = { acid: 'acide', air: 'air', chaotic: 'chaotique', cold: 'froid', curse: 'malédiction', darkness: 'ténèbres',
  death: 'mort', disease: 'maladie', earth: 'terre', electricity: 'électricité', emotion: 'émotion', evil: 'mauvais', fear: 'peur',
  fire: 'feu', force: 'force', good: 'bon', 'language-dependent': 'dépend du langage', lawful: 'loyal', light: 'lumière',
  'mind-affecting': 'effet mental', pain: 'douleur', poison: 'poison', shadow: 'ombre', sonic: 'son', water: 'eau',
  'see text': 'voir texte', 'fire or cold': 'feu ou froid' };

function composantes(c) {
  if (!c) return null;
  // Lettres : V, S, M, F, DF ; les détails entre parenthèses (matériaux) restent à traduire à la main.
  return c.replace(/\bDF\b/g, 'FD').replace(/\bS\b/g, 'G');
}
function ecole(s) { return ECOLES[s] || s; }
function sousEcoles(s) {
  if (!s) return null;
  return s.split(/\s*(?:,| or )\s*/).map((x) => SOUS[x.trim()] || x.trim()).join(', ');
}
function descripteurs(d) {
  return d.map((x) => DESC[x] || (x.startsWith('see text') ? 'voir texte' : x));
}

const sortie = {};
const residus = {};
function noter(champ, en, fr) {
  residus[champ] = residus[champ] || new Map();
  if (/[A-Za-z]{4,}/.test(fr.replace(/\b(niveau|niveaux|round|rounds|minute|minutes|action|simple|complexe|voir|texte|instantanée|permanente|concentration|heure|heures|jour|jours|Vigueur|Volonté|Réflexes|annule|partiel|aucun|moitié|objet|inoffensif|oui|non|ou|et|contact|courte|moyenne|longue|personnelle|rapide|immédiate|illimitée|jusqu|déchargé|au moins|par page|plus|dévoilement|si|interaction|cas|dans|min|jour)\b/g, ''))) {
    residus[champ].set(en, fr);
  }
}
for (const s of sorts) {
  const e = {
    ecole: ecole(s.ecole),
    sous_ecole: sousEcoles((s.sous_ecole || '').trim() || null),
    descripteurs: descripteurs(s.descripteurs),
    temps_d_incantation: s.temps_d_incantation_en && (deja('temps_d_incantation', s.temps_d_incantation_en) || ordonne(s.temps_d_incantation_en, TEMPS)),
    composantes: composantes(s.composantes_en),
    portee: s.portee_en && (deja('portee', s.portee_en) || pieds(ordonne(s.portee_en, PORTEE))),
    duree: s.duree_en && (deja('duree', s.duree_en) || pieds(ordonne(s.duree_en, DUREE))),
    jet_de_sauvegarde: s.jet_de_sauvegarde_en && (deja('jet_de_sauvegarde', s.jet_de_sauvegarde_en) || ordonne(s.jet_de_sauvegarde_en, SAUVEGARDE)),
    resistance_a_la_magie: s.resistance_a_la_magie_en && ordonne(s.resistance_a_la_magie_en, RM),
  };
  for (const [champ, en] of [['temps_d_incantation', s.temps_d_incantation_en], ['portee', s.portee_en], ['duree', s.duree_en],
    ['jet_de_sauvegarde', s.jet_de_sauvegarde_en], ['resistance_a_la_magie', s.resistance_a_la_magie_en]]) {
    if (en) noter(champ, en, e[champ]);
  }
  sortie[s.nom_en] = e;
}
if (montrerResidus) {
  for (const [champ, m] of Object.entries(residus)) {
    console.log('== ' + champ + ' (' + m.size + ')');
    for (const [en, fr] of m) console.log('  ' + en + '  =>  ' + fr);
  }
} else {
  process.stdout.write(JSON.stringify(sortie, null, 1));
}
