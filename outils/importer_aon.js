// Import ponctuel des données du Core Rulebook depuis Archives of Nethys (aonprd.com).
// Usage : node outils/importer_aon.js <dossier_de_cache_html> <dossier_de_sortie>
// Le dossier de cache reçoit les pages téléchargées (une requête par page, avec pause).
// Les valeurs structurées (ajustements, vitesses…) sont écrites ici et vérifiées contre le
// texte des pages ; les textes des traits viennent des pages telles quelles (anglais, OGL).
const fs = require('fs');
const path = require('path');
const https = require('https');

const [cache, sortie] = process.argv.slice(2);
if (!cache || !sortie) { console.error('usage : node importer_aon.js <cache> <sortie>'); process.exit(1); }
fs.mkdirSync(cache, { recursive: true });
fs.mkdirSync(sortie, { recursive: true });

const pause = (ms) => new Promise((r) => setTimeout(r, ms));
function telecharger(url, fichier) {
  const f = path.join(cache, fichier);
  if (fs.existsSync(f)) return Promise.resolve(fs.readFileSync(f, 'utf8'));
  return new Promise((ok, ko) => {
    https.get(url, { headers: { 'User-Agent': 'gestion-pathfinder (usage personnel, hors ligne)' } }, (res) => {
      let s = ''; res.setEncoding('utf8');
      res.on('data', (d) => (s += d));
      res.on('end', async () => { fs.writeFileSync(f, s); await pause(800); ok(s); });
    }).on('error', ko);
  });
}
function texte(html) {
  return html.replace(/<script[\s\S]*?<\/script>|<style[\s\S]*?<\/style>/g, '')
    .replace(/<(br|\/p|\/h\d|\/tr|\/li)[^>]*>/g, '\n').replace(/<[^>]+>/g, ' ')
    .replace(/&nbsp;/g, ' ').replace(/&amp;/g, '&').replace(/&mdash;/g, '—')
    .replace(/[ \t]+/g, ' ').split('\n').map((l) => l.trim()).filter(Boolean);
}
const ecrire = (nom, obj) => fs.writeFileSync(path.join(sortie, nom), JSON.stringify(obj, null, 2) + '\n');
const SOURCE_AON = 'Archives of Nethys (aonprd.com), contenu du Core Rulebook sous licence OGL';

// ---------------- Races ----------------
const C = { FOR: 'for', DEX: 'dex', CON: 'con', INT: 'int', SAG: 'sag', CHA: 'cha' };
const RACES = [
  { id: 'humain', aon: 'Human', nom: 'Humain', taille: 'moyenne', vitesse: 30, vision: 'normale', ajustements: [], choix_libre: 2,
    langues: ['commun'], langues_choix: 'toutes (sauf langues secrètes)', bonus_don: 1, rang_supplementaire: 1 },
  { id: 'nain', aon: 'Dwarf', nom: 'Nain', taille: 'moyenne', vitesse: 20, vitesse_jamais_reduite: true, vision: 'vision dans le noir 18 m (60 pi)',
    ajustements: [[C.CON, 2], [C.SAG, 2], [C.CHA, -2]], langues: ['commun', 'nain'],
    langues_choix: 'géant, gnome, gobelin, orque, terran, commun des profondeurs' },
  { id: 'elfe', aon: 'Elf', nom: 'Elfe', taille: 'moyenne', vitesse: 30, vision: 'vision nocturne',
    ajustements: [[C.DEX, 2], [C.INT, 2], [C.CON, -2]], langues: ['commun', 'elfique'],
    langues_choix: 'céleste, draconique, gnoll, gnome, gobelin, orque, sylvestre' },
  { id: 'gnome', aon: 'Gnome', nom: 'Gnome', taille: 'petite', vitesse: 20, vision: 'vision nocturne',
    ajustements: [[C.CON, 2], [C.CHA, 2], [C.FOR, -2]], langues: ['commun', 'gnome', 'sylvestre'],
    langues_choix: 'draconique, nain, elfique, géant, gobelin, orque' },
  { id: 'demi-elfe', aon: 'Half-Elf', nom: 'Demi-elfe', taille: 'moyenne', vitesse: 30, vision: 'vision nocturne',
    ajustements: [], choix_libre: 2, langues: ['commun', 'elfique'], langues_choix: 'toutes (sauf langues secrètes)',
    deux_classes_de_predilection: true, bonus_don: 1, don_impose: 'Skill Focus' },
  { id: 'demi-orque', aon: 'Half-Orc', nom: 'Demi-orque', taille: 'moyenne', vitesse: 30, vision: 'vision dans le noir 18 m (60 pi)',
    ajustements: [], choix_libre: 2, langues: ['commun', 'orque'], langues_choix: 'abyssal, draconique, géant, gnoll, gobelin' },
  { id: 'halfelin', aon: 'Halfling', nom: 'Halfelin', taille: 'petite', vitesse: 20, vision: 'normale',
    ajustements: [[C.DEX, 2], [C.CHA, 2], [C.FOR, -2]], langues: ['commun', 'halfelin'], langues_choix: 'nain, elfique, gnome, gobelin' },
];


// Effets chiffrés inconditionnels des traits raciaux (les traits conditionnels restent du texte).
function effetsDeRace(id, taille) {
  const e = (cible, type, valeur, source) => ({ cible, type, valeur, source });
  const r = [];
  if (taille === 'petite') {
    r.push(e('ca', 'taille', 1, 'Petite taille'), e('attaque', 'taille', 1, 'Petite taille'),
      e('bmo', 'taille', -1, 'Petite taille'), e('dmd', 'taille', -1, 'Petite taille'),
      e('comp.stealth', 'taille', 4, 'Petite taille'));
  }
  const sens = (src) => e('comp.perception', 'racial', 2, src);
  if (id === 'elfe') r.push(sens('Sens aiguisés'));
  if (id === 'gnome') r.push(sens('Sens aiguisés'));
  if (id === 'halfelin') {
    r.push(sens('Sens aiguisés'), e('comp.acrobatics', 'racial', 2, 'Pied sûr'), e('comp.climb', 'racial', 2, 'Pied sûr'));
    for (const j of ['vig', 'ref', 'vol']) r.push(e('jds.' + j, 'racial', 1, 'Chance du halfelin'));
  }
  if (id === 'demi-elfe') r.push(sens('Sens aiguisés'));
  if (id === 'demi-orque') r.push(e('comp.intimidate', 'racial', 2, 'Intimidant'));
  return r;
}

async function races() {
  const sortieRaces = [];
  for (const r of RACES) {
    const html = await telecharger(`https://aonprd.com/RacesDisplay.aspx?ItemName=${r.aon}`, `race_${r.aon}.html`);
    const l = texte(html);
    const page = (l.join(' ').match(/PRPG Core Rulebook pg\. (\d+)/) || [])[1];
    const deb = l.findIndex((x) => /Racial Traits$/.test(x) || /Racial Traits\s*$/.test(x) || x.includes('Racial Traits'));
    // La ligne « … Racial Traits » est suivie des traits, jusqu'à « Subraces ».
    const traits = [];
    for (let i = deb + 1; i < l.length; i++) {
      if (/^Subraces|^Alternate Racial/.test(l[i])) break;
      const m = l[i].match(/^(.+?) : (.+)$/);
      if (m) { const sub = m[2].replace(/ Subraces$/, ''); traits.push({ nom_en: m[1].trim(), texte_en: sub }); }
      if (/Subraces$/.test(l[i])) break;
    }
    const { aon, ...reste } = r;
    sortieRaces.push({ ...reste, effets: effetsDeRace(r.id, r.taille), nom_en: aon, source: { livre: 'PRPG Core Rulebook', page: Number(page), url: `https://aonprd.com/RacesDisplay.aspx?ItemName=${aon}` }, traits });
  }
  ecrire('races.json', { source: SOURCE_AON, traits_raciaux_alternatifs: 'absents du Core Rulebook (Advanced Race Guide) : non importés', races: sortieRaces });
}

// ---------------- Ensorceleur ----------------
const ORD = (s) => Number(s.match(/(\d+)/)[1]);
function parseTable(lignes, debut, n) {
  const out = [];
  for (let i = debut; i < debut + n; i++) out.push(lignes[i]);
  return out;
}
async function ensorceleur() {
  const html = await telecharger('https://aonprd.com/ClassDisplay.aspx?ItemName=Sorcerer', 'classe_Sorcerer.html');
  const l = texte(html);
  const iBBA = l.findIndex((x) => x.startsWith('Level Base Attack Bonus'));
  const progression = [];
  for (let n = 1; n <= 20; n++) {
    const ligne = l[iBBA + n];
    const m = ligne.match(/^(\d+)(?:st|nd|rd|th) (\S+(?:\/\S+)*) \+(\d+) \+(\d+) \+(\d+) (.+?) ((?:\d+|-) (?:(?:\d+|-) ){7}(?:\d+|-))$/);
    if (!m) throw new Error('ligne de table illisible : ' + ligne);
    const par_jour = m[7].split(' ').map((x) => (x === '-' ? null : Number(x)));
    progression.push({
      niveau: Number(m[1]), bba: m[2].split('/').map((x) => Number(x.replace('+', ''))),
      vigueur: Number(m[3]), reflexes: Number(m[4]), volonte: Number(m[5]),
      special_en: m[6] === '—' || m[6] === '&mdash;' ? '' : m[6], sorts_par_jour: par_jour,
    });
  }
  const iConnus = l.findIndex((x) => x === 'Spells Known');
  const connus = [];
  for (let n = 1; n <= 20; n++) {
    const m = l[iConnus + 1 + n].match(/^(\d+)(?:st|nd|rd|th) (.+)$/);
    connus.push(m[2].split(' ').map((x) => (x === '-' ? null : Number(x))));
  }
  const bba_chaine = progression.map((p) => p.bba[0]);
  // Vérifications croisées sur les tables lues.
  const attendu_bba = Array.from({ length: 20 }, (_, i) => Math.floor(((i + 1) * 1) / 2));
  if (JSON.stringify(bba_chaine) !== JSON.stringify(attendu_bba)) throw new Error('BBA inattendu');
  progression.forEach((p, i) => (p.sorts_connus = connus[i]));

  const bloodlines = [];
  for (const b of ['Aberrant', 'Abyssal', 'Arcane', 'Celestial', 'Destined', 'Draconic', 'Elemental', 'Fey', 'Infernal', 'Undead']) {
    const h = await telecharger(`https://aonprd.com/BloodlineDisplay.aspx?ItemName=${b}&Class=Sorcerer`, `lignage_${b}.html`);
    let bl = texte(h);
    const fin = bl.findIndex((x) => /Expanded Bloodlines|Site Owner/.test(x));
    // « … Expanded Bloodlines » peut être collé à la fin d'une ligne : on le coupe.
    bl = bl.slice(0, fin < 0 ? bl.length : fin + 1).map((x) => x.replace(/ Expanded Bloodlines.*$/, '').replace(/ Site Owner.*$/, ''));
    const page = (bl.join(' ').match(/PRPG Core Rulebook pg\. (\d+)/) || [])[1];
    const champ = (nom) => { const x = bl.find((y) => y.startsWith(nom + ' :')); return x ? x.slice(nom.length + 3) : null; };
    const sorts = [];
    const brut = champ('Bonus Spells').replace(/\.$/, '');
    for (const m of brut.matchAll(/(.+?) \((\d+)(?:st|nd|rd|th)\)(?:, |$)/g)) sorts.push({ nom_en: m[1].trim(), niveau_classe: Number(m[2]) });
    if (sorts.length !== 9) throw new Error(`${b} : ${sorts.length} sorts de lignage`);
    const dons = champ('Bonus Feats').replace(/\.$/, '').split(/, (?![^\[]*\])/).map((x) => x.trim());
    const iPow = bl.findIndex((x) => x.startsWith('Bloodline Powers :'));
    const pouvoirs = [];
    for (let i = iPow + 1; i < bl.length; i++) {
      const m = bl[i].match(/^(.+?) \((Su|Ex|Sp)\) : (.+)$/);
      if (m) {
        const niv = (m[3].match(/(\d+)(?:st|nd|rd|th) level/) || [])[1];
        pouvoirs.push({ nom_en: m[1], type: m[2], niveau_classe: niv ? Number(niv) : null, texte_en: m[3] });
      }
    }
    const tableau = bl.filter((x, i) => i > iPow && !x.match(/\((Su|Ex|Sp)\) :/) && !x.startsWith('*'));
    bloodlines.push({
      id: b.toLowerCase(), nom_en: b, source: { livre: 'PRPG Core Rulebook', page: Number(page), url: `https://aonprd.com/BloodlineDisplay.aspx?ItemName=${b}&Class=Sorcerer` },
      competence_de_classe_en: champ('Class Skill'), sorts_bonus: sorts, dons_bonus_en: dons,
      arcane_en: champ('Bloodline Arcana'), pouvoirs, choix_en: tableau,
    });
  }
  const classe = {
    id: 'ensorceleur', nom: 'Ensorceleur', nom_en: 'Sorcerer',
    source: { livre: 'PRPG Core Rulebook', page: 70, url: 'https://aonprd.com/ClassDisplay.aspx?ItemName=Sorcerer' },
    de_vie: 6, rangs_par_niveau: 2, caracteristique_rangs: 'int',
    competences_de_classe_en: ['Appraise', 'Bluff', 'Craft', 'Fly', 'Intimidate', 'Knowledge (arcana)', 'Profession', 'Spellcraft', 'Use Magic Device'],
    bba: 'moyen', vigueur: 'lente', reflexes: 'lente', volonte: 'rapide',
    maitrises_en: 'all simple weapons; no armor, no shield',
    incantation: { type: 'profane', mode: 'spontané', caracteristique: 'cha', echange_niveau_min: 4, echange_pas: 2, echange_de_sort: 'niveaux 4, 6, 8… : un sort connu contre un autre du même niveau' },
    or_de_depart_en: '2d6 x 10 gp (average 70 gp)',
    don_bonus_niveau_1: 'Eschew Materials',
    progression, lignages: bloodlines,
    capacites_en: {
      'bloodline': 'niveau 1 : un lignage au choix, définitif ; sorts de lignage aux niveaux 3, 5, 7… 19 ; don de lignage aux niveaux 7, 13, 19',
    },
  };
  ecrire('ensorceleur.json', { source: SOURCE_AON, classe });
}


// ---------------- Compétences ----------------
const CARAC_FR = { Str: 'for', Dex: 'dex', Con: 'con', Int: 'int', Wis: 'sag', Cha: 'cha' };
async function competences() {
  const NOMS = ['Acrobatics','Appraise','Bluff','Climb','Craft','Diplomacy','Disable Device','Disguise','Escape Artist','Fly','Handle Animal','Heal','Intimidate','Knowledge','Linguistics','Perception','Perform','Profession','Ride','Sense Motive','Sleight of Hand','Spellcraft','Stealth','Survival','Swim','Use Magic Device'];
  const liste = [];
  for (const nom of NOMS) {
  const html = await telecharger('https://aonprd.com/Skills.aspx?ItemName=' + encodeURIComponent(nom), 'comp_' + nom.replace(/ /g, '_') + '.html');
  const l = texte(html);
  for (let i = 0; i < l.length; i++) {
    const m = l[i].match(/^([A-Z][A-Za-z ]+) \((Str|Dex|Con|Int|Wis|Cha)((?:; [^)]+)?)\)$/);
    if (!m) continue;
    const p = (l[i + 1] || '').match(/PRPG Core Rulebook pg\. (\d+)/);
    const opts = m[3].split(';').map((x) => x.trim()).filter(Boolean);
    liste.push({
      id: m[1].toLowerCase().replace(/ /g, '_'), nom_en: m[1], caracteristique: CARAC_FR[m[2]],
      formation_requise: opts.includes('Trained Only'), malus_d_armure: opts.includes('Armor Check Penalty'),
      sous_types: ['Craft', 'Knowledge', 'Perform', 'Profession'].includes(m[1]),
      source: { livre: 'PRPG Core Rulebook', page: p ? Number(p[1]) : null },
    });
    break;
  }
  }
  if (liste.length !== 26) throw new Error('compétences : ' + liste.length);
  ecrire('competences.json', { source: SOURCE_AON, competences: liste });
}


// ---------------- Progression de niveau (tableau 3-1, p. 31) ----------------
async function progression() {
  const html = await telecharger('https://aonprd.com/Rules.aspx?Name=Character%20Advancement&Category=Basics', 'progression.html');
  const l = texte(html);
  const i0 = l.findIndex((x) => x.startsWith('Character Level Slow Medium Fast'));
  const niveaux = [];
  for (let n = 1; n <= 20; n++) {
    const m = l[i0 + n].match(/^(\d+)(?:st|nd|rd|th) (\S+) (\S+) (\S+) (\S+) (\S+)$/);
    if (!m || Number(m[1]) !== n) throw new Error('ligne XP illisible : ' + l[i0 + n]);
    const xp = (x) => (x === '—' ? 0 : Number(x.replace(/,/g, '')));
    niveaux.push({ niveau: n, xp_lente: xp(m[2]), xp_moyenne: xp(m[3]), xp_rapide: xp(m[4]), don: m[5] !== '—', augmentation: m[6] !== '—' });
  }
  ecrire('progression.json', { source: SOURCE_AON, livre: 'PRPG Core Rulebook', page: 31, niveaux });
}

(async () => { await progression(); await races(); await competences(); await ensorceleur(); console.log('ok'); })().catch((e) => { console.error(e); process.exit(1); });
