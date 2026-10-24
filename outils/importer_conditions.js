// Import des états (conditions) du Core Rulebook (p. 565) depuis AoN, avec leurs effets chiffrés.
// Usage : node outils/importer_conditions.js <cache> <fichier_de_sortie>
// Les effets chiffrés sont écrits ici d'après le texte de chaque état (vérifié à la lecture) ;
// ce qui dépend d'une situation (prone vs distance, etc.) reste dans le texte, marqué « conditionnel ».
const fs = require('fs');
const https = require('https');
const path = require('path');
const [cache, sortie] = process.argv.slice(2);
function telecharger(url, fichier) {
  const f = path.join(cache, fichier);
  if (fs.existsSync(f)) return Promise.resolve(fs.readFileSync(f, 'utf8'));
  return new Promise((ok, ko) => {
    https.get(url, { headers: { 'User-Agent': 'gestion-pathfinder (usage personnel, hors ligne)' } }, (res) => {
      let s = ''; res.setEncoding('utf8'); res.on('data', (d) => (s += d));
      res.on('end', () => { fs.writeFileSync(f, s); ok(s); });
    }).on('error', ko);
  });
}
const dec = (s) => s.replace(/&nbsp;/g, ' ').replace(/&amp;/g, '&').replace(/&mdash;/g, '—');
const lignes = (h) => dec(h.replace(/<script[\s\S]*?<\/script>|<style[\s\S]*?<\/style>/g, '').replace(/<(br|\/p|\/h\d|\/tr|\/li|\/div)[^>]*>/g, '\n').replace(/<[^>]+>/g, ' '))
  .replace(/[ \t]+/g, ' ').split('\n').map((l) => l.trim()).filter(Boolean);

// effets : cible, type (toujours « sans_type » : les malus se cumulent), valeur. Cibles génériques :
// attaque, degats, jds (tous les jets de sauvegarde), comp (toutes les compétences), ca, init,
// carac.xxx, comp_carac.xxx (compétences basées sur la caractéristique).
const E = (cible, valeur) => ({ cible, type: 'sans_type', valeur });
const EFFETS = {
  'Blinded': { effets: [E('ca', -2)], perd_bonus_dex_ca: true },
  'Cowering': { effets: [E('ca', -2)], perd_bonus_dex_ca: true },
  'Dazzled': { effets: [E('attaque', -1)] },
  'Deafened': { effets: [E('init', -4)] },
  'Disabled': { effets: [], vitesse_moitie: true },
  'Entangled': { effets: [E('attaque', -2), E('carac.dex', -4)], vitesse_moitie: true },
  'Exhausted': { effets: [E('carac.for', -6), E('carac.dex', -6)], vitesse_moitie: true },
  'Fatigued': { effets: [E('carac.for', -2), E('carac.dex', -2)] },
  'Flat-Footed': { effets: [], perd_bonus_dex_ca: true },
  'Frightened': { effets: [E('attaque', -2), E('jds', -2), E('comp', -2)] },
  'Grappled': { effets: [E('carac.dex', -4), E('attaque', -2)] },
  'Invisible': { effets: [E('attaque', 2)] },
  'Panicked': { effets: [E('jds', -2), E('comp', -2)] },
  'Pinned': { effets: [E('ca', -4)], perd_bonus_dex_ca: true },
  'Shaken': { effets: [E('attaque', -2), E('jds', -2), E('comp', -2)] },
  'Sickened': { effets: [E('attaque', -2), E('degats', -2), E('jds', -2), E('comp', -2)] },
  'Stunned': { effets: [E('ca', -2)], perd_bonus_dex_ca: true },
};
// Ce que le texte du livre prévoit mais que le moteur ne calcule pas (en français, à lire à l'écran).
const NOTES = {
  'Blinded': 'Non calculé : –4 sur « la plupart » des jets de FOR et de DEX et sur les jets opposés de Perception (le livre ne dit pas lesquels) ; les adversaires ont 50 % de chances de rater (camouflage total).',
  'Deafened': 'Non calculé : 20 % d’échec des sorts à composante verbale ; –4 aux jets opposés de Perception.',
  'Entangled': 'Non calculé : test de concentration (DD 15 + niveau du sort) pour lancer un sort.',
  'Grappled': 'Non calculé : le –2 ne touche pas les jets pour lutter ou se libérer ; test de concentration pour lancer un sort.',
  'Fascinated': 'Non calculé : –4 aux jets de compétence faits en réaction (Perception…).',
  'Prone': 'Non calculé (dépend de l’attaque) : –4 aux attaques de corps à corps ; +4 à la CA contre les attaques à distance, –4 contre celles de corps à corps.',
  'Helpless': 'Non calculé : Dextérité traitée comme 0 (–5) ; +4 aux attaques de corps à corps contre elle.',
  'Paralyzed': 'Non calculé : Force et Dextérité effectives de 0 ; sans défense.',
  'Broken': 'Dépend de l’objet : arme –2 à l’attaque et aux dégâts ; armure ou bouclier : bonus de CA divisé par deux et malus d’armure doublé.',
  'Energy Drained': 'Niveaux négatifs : non calculés (valeurs à confirmer, voir « Points à confirmer »).',
};

(async () => {
  const html = await telecharger('https://aonprd.com/Rules.aspx?Name=Conditions&Category=Basics', 'etats.html');
  const l = lignes(html);
  const deb = l.findIndex((x) => x === 'Conditions') + 3;
  const fin = l.findIndex((x) => x.startsWith('Site Owner'));
  const etats = [];
  let courant = null;
  for (const x of l.slice(deb, fin)) {
    const m = x.match(/^([A-Z][A-Za-z\- ]+) : (.*)$/);
    if (m && m[1].length < 20) {
      courant = { nom_en: m[1], texte_en: m[2] };
      etats.push(courant);
    } else if (courant) {
      courant.texte_en += '\n' + x;
    }
  }
  for (const e of etats) {
    const d = EFFETS[e.nom_en] || { effets: [] };
    e.id = e.nom_en.toLowerCase().replace(/[^a-z]+/g, '_');
    e.effets = d.effets;
    e.perd_bonus_dex_ca = !!d.perd_bonus_dex_ca;
    e.vitesse_moitie = !!d.vitesse_moitie;
    e.note = NOTES[e.nom_en] || null;
    e.source = { livre: 'PRPG Core Rulebook', page: 565 };
  }
  const attendus = Object.keys(EFFETS).filter((k) => !etats.find((e) => e.nom_en === k));
  if (attendus.length) throw new Error('états absents de la page : ' + attendus.join(', '));
  fs.writeFileSync(sortie, JSON.stringify({ source: 'Archives of Nethys (aonprd.com), Core Rulebook, licence OGL', etats }, null, 2) + '\n');
  console.error(etats.length + ' états');
})().catch((e) => { console.error(e); process.exit(1); });
