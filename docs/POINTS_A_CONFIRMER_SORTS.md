# Points à confirmer — sorts en combat (phase 6)

Sources relevées le 24/10/2026 : textes des dons et des sorts du Core déjà dans `data/core/` (page indiquée dans
chaque fichier de données) et règles d'Archives of Nethys (Core Rulebook p. 83, 206-208). Rien n'est inventé ;
« lecture à valider » = comportement implémenté d'après ma lecture.

## Ce que fait le lancement

1. **Aperçu** (avant de confirmer) : niveau du sort, niveau de l'emplacement dépensé, emplacements restants,
   DD (pris dans la fiche), niveau de lanceur, type d'action, composantes, % d'échec des sorts profanes, DD du
   lancement défensif, durée proposée par le livre, portée, jet de sauvegarde, description.
2. **Lancement** : le joueur saisit les résultats des dés physiques (d100 d'échec profane, d20 défensif, dés du sort) ;
   l'emplacement est décompté (même si le sort échoue), la fiche reçoit l'événement « sort lancé », le journal
   garde tout. Les tours de magie sont à volonté.
3. **Effets sur les personnages joués** : pour les sorts de `data/combat/sorts_effets.json` (14 sorts pour
   l'instant : Mage Armor, Shield, les six « force/dextérité/… » +4, Heroism, Haste, Resist Energy, Stoneskin,
   Invisibility), l'effet est créé comme effet à durée sur la ou les cibles choisies (le lanceur compris), avec
   sa source. Un sort ciblant un ennemi ou sans effet connu est seulement décompté et noté.
4. **Concentration** (p. 206) : tous les cas du tableau du Core (blessure, dégâts continus, sort sur soi, agrippé,
   mouvements, intempéries, entravé…). Un échec rompt l'effet à concentration visé.

## Règles et lectures

| # | Règle | Implémentation | Statut |
|---|---|---|---|
| 1 | Métamagie : un don augmente le niveau d'emplacement (Empower +2, Enlarge +1, Extend +1, Maximize +3, Quicken +4, Silent +1, Still +1, Widen +3 ; Heighten : le niveau visé) | `data/combat/metamagie.json`, vérifié par un test contre le texte des dons (page du Core) | sûr |
| 2 | Heighten Spell : le DD et tout ce qui dépend du niveau suivent le niveau visé | DD pris au niveau visé ; emplacement = niveau visé + autres métamagies | sûr |
| 3 | Extend Spell : durée doublée (sauf concentration, instantané, permanent) | La durée proposée est doublée ; une durée saisie n'est pas modifiée | sûr |
| 4 | Quicken Spell : action rapide, ne compte pas pour l'action simple (p. 132) | Action rapide ; texte du don : applicable à un sort spontané jusqu'à 1 action complexe, sans allonger le temps | sûr |
| 5 | **Lanceur spontané + métamagie : le sort prenant une action simple devient une action complexe** (règle du livre « Spontaneous Casting and Metamagic Feats », non retrouvée dans les textes de données) | Action complexe ; un sort qui prend déjà une action complexe ou plus avec métamagie spontanée est **refusé en combat** | **point à confirmer** |
| 6 | Silent Spell / Still Spell : sans composante verbale / gestuelle | La composante est retirée ; Still Spell supprime l'échec des sorts profanes (p. 83) | sûr |
| 7 | Échec des sorts profanes : seulement si le sort a une composante gestuelle (p. 83, 208) | d100 demandé si le % de la fiche est > 0 et le sort gestuel ; échec si le d100 ≤ % ; le sort et l'emplacement sont perdus | lecture à valider (« ≤ % » = échec) |
| 8 | Lancement défensif : test de concentration DD 15 + 2 × niveau du sort (p. 206) | d20 + niveau de lanceur + mod. d'incantation + bonus de la fiche ; échec : sort perdu, emplacement dépensé. « Niveau du sort » = niveau visé par Heighten, sans les autres métamagies | lecture à valider |
| 9 | Temps d'incantation : « 1 standard action » = action simple ; « 1 round » ou « 1 full-round action » = action complexe | Un sort « 1 round » est traité comme une action complexe terminée dans le tour (le livre le termine au début du tour suivant) ; les temps plus longs sont refusés en combat | lecture à valider |
| 10 | Durée proposée d'après le livre (« 1 min./level », « 10 min./level », « 1 hour/level », « 1 round/level »…) | Formule simple lue ; sinon (« see text », dés) le joueur saisit la durée | décidé avec toi |
| 11 | Plusieurs résistances à l'énergie / RD | La plus haute (voir `POINTS_A_CONFIRMER_COMBAT.md`) | validé |
| 12 | Résistance à l'énergie : 10, 20 au niveau 7, 30 au niveau 11 | Palier lu dans `sorts_effets.json` d'après le niveau de lanceur ; type d'énergie choisi | sûr |

## Pas encore fait

- **Lanceurs à sorts préparés** (mage…) : refusés pour l'instant (l'ensorceleur est spontané).
- **Sorts à décharge** (ex. peau de pierre : déchargée après 10 PV évités par niveau) : durée en minutes suivie ;
  la décharge est à noter à la main.
- **Pouvoirs de lignage à utilisations quotidiennes** et autres ressources : phase 7.
- **Dés de dégâts / soins du sort** : la formule du livre est affichée dans la description ; le résultat est saisi
  (noté dans le journal), pas calculé ni appliqué à la cible (aucun ennemi n'est modélisé).
- **Convocations** : gabarit minimal prévu (phase suivante) ; les sorts de convocation sont décomptés seulement.
- **Contresort et dissipation** : à faire ; **résistance à la magie** : le lanceur la jette physiquement.
- **Sort lancé sur un personnage joué sans effet connu** : « appliquer un effet » à la main.

## Décisions d'Antoine du 24/10/2026

- **Métamagie (5)** : à intégrer plus tard. Le code et les données existent mais la métamagie est **désactivée par
  défaut** (`metamagie_active` de l'incantation fournie par la fiche) : tant qu'elle ne l'est pas, le lancement la
  refuse. La lecture sur l'action complexe d'un sort spontané avec métamagie reste donc à confirmer à ce moment-là.
- Lectures validées : 7 (échec des sorts profanes si le d100 est ≤ au pourcentage), 8 (lancement défensif),
  9 (sort « 1 round » traité comme action complexe).
- **Durée (10)** : quand la formule du livre n'est pas claire, le joueur décide et la saisit (déjà le cas).
