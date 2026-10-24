# Points à confirmer — santé, états et actions de combat (phase 5)

Règles relevées le 24/10/2026 sur Archives of Nethys (texte anglais du Core Rulebook, pages citées par ce
site). Rien n'est inventé ; « lecture à valider » = comportement implémenté d'après ma lecture.

## Santé (Core p. 189-191)

| # | Règle | Implémentation | Statut |
|---|---|---|---|
| 1 | À exactement 0 PV : hors de combat, chancelant ; une seule action de mouvement ou une seule action simple par tour, pas d'action complexe ; les actions de mouvement n'ont pas de coût ; une action simple ou autre effort fait subir 1 dégât après coup (p. 189) | État « disabled » + « staggered » sur la fiche. `Agir` : une seule action ; une action simple ou complexe inflige 1 dégât létal | lecture à valider |
| 2 | PV négatifs au-dessus de −Con : mourant, inconscient, ne peut pas agir (p. 189-190) | États « dying » + « unconscious ». `Agir` refusé | sûr |
| 3 | À −Con ou en dessous : mort (p. 190) | État « dead » (Con = score de Constitution actuel) | sûr |
| 4 | À son tour, un mourant fait un jet de Constitution DD 10 avec un malus égal à ses PV négatifs ; un 20 naturel réussit ; l'échec coûte 1 PV (p. 190) | `Stabilisation` : d20 saisi, Con. et malus calculés ; l'échec retire 1 PV | lecture à valider |
| 5 | Tout soin qui rend au moins 1 PV stabilise (p. 190) | Un soin d'au moins 1 PV sur un mourant le rend « stable » | lecture à valider |
| 6 | Un personnage stable qui subit de nouveaux dégâts redevient mourant | Le texte relevé ne le dit pas | **point à confirmer** |
| 7 | Dégâts non létaux égaux aux PV : chancelant ; supérieurs aux PV : inconscient (p. 191) | États « staggered » / « unconscious » | sûr |
| 8 | Dégâts massifs (règle optionnelle, p. 189) : jet de Vigueur DD 15 si une attaque inflige au moins la moitié des PV | **Non modélisé** pour l'instant | à décider |
| 9 | Perte de 1 PV par round d'un mourant | Le texte relevé donne la perte de 1 PV comme conséquence de l'échec du jet de stabilisation à chaque tour ; je n'ai pas ajouté de perte automatique en plus | lecture à valider |
| 10 | Saignement et dégâts continus (« les dégâts continus font échouer automatiquement la stabilisation et coûtent 1 PV de plus par round », p. 190) | **Non modélisé** pour l'instant | à décider |

## États préjudiciables

Les 34 états du Core (`data/core/etats.json`, déjà dans le moteur de personnage) se donnent par un **effet à
durée** (rounds, minutes, permanent…) : l'état reste sur la fiche tant qu'au moins un effet le donne, puis part
tout seul. Les valeurs chiffrées de chaque état sont celles du moteur de personnage.
Les états de santé ci-dessus passent par le même mécanisme.

## Dégâts typés, résistances, immunités, RD

- Un dégât a un type (« feu », « tranchant »…). Immunité : 0 dégât. Résistance à l'énergie : retire sa valeur
  (acide, froid, électricité, feu, son sont les types d'énergie). RD : retire sa valeur aux dégâts physiques,
  sauf si l'attaque la contourne (case saisie par le joueur).
- Le détail du calcul est écrit dans le journal (« 20 feu, −10 résistance au feu (…) = 10 »).
- **Points à confirmer** : (a) plusieurs résistances ou RD : je prends la plus haute ; (b) les pages du Core
  pour ces définitions (glossaire) ne sont pas relevées.

## Guérison accélérée, régénération

Le Core n'en définit pas les valeurs (ce sont des capacités de créatures). Le moteur offre un effet avec « PV rendus
au début de chaque tour » saisi par toi ; il ne gère ni les types de dégâts qui contournent la régénération ni
l'impossibilité de mourir.

## Économie d'actions (Core p. 181-182)

Source : tableau « Actions in Combat » relevé dans `data/combat/actions.json` (58 actions, types et pages).

| # | Règle | Implémentation | Statut |
|---|---|---|---|
| 11 | Tour normal : action simple + action de mouvement, ou une action complexe ; action rapide : une par tour ; actions libres : limite laissée au MJ | `EconomieTour` | sûr |
| 12 | On peut toujours remplacer une action simple par une action de mouvement | Deux mouvements possibles, noté dans le journal | sûr |
| 13 | Pas de 1,5 m : seulement si on ne se déplace pas réellement dans le round | Refus croisé entre pas et déplacement (action marquée « déplace ») | sûr |
| 14 | Round de surprise : une seule action de mouvement ou action simple, pas d'action complexe, plus actions libres et rapides | Une seule action simple ou de mouvement ; rapides autorisées | lecture à valider |
| 15 | Action immédiate : utilisable hors de son tour | Une seule avant le prochain tour ; hors tour, elle prend l'action rapide du prochain tour ; sur son tour, elle compte comme l'action rapide | **point à confirmer** (non relevé dans le texte) |
| 16 | Une action complexe peut aussi se faire avec un pas de 1,5 m avant, pendant ou après | Autorisé | sûr |
| 17 | « Retarder » : aucune action | Déjà géré (phase 4) | validé |

## Décisions d'Antoine du 24/10/2026

- Lectures validées : 1, 4, 5, 9, 14, 15 (et 6 : un stable qui reprend des dégâts redevient mourant).
- **Dégâts massifs (8)** : on suit la règle du Core (p. 189). Elle est **optionnelle** : activable par campagne
  (`degats_massifs`), éteinte par défaut. Quand une seule attaque inflige au moins 50 PV **et** au moins la moitié
  des PV maximum sans tuer, le moteur demande un jet de Vigueur DD 15 (d20 physique + bonus de la fiche ;
  20 naturel réussit, 1 naturel rate). Échec : mort, quels que soient les PV restants.
  Lecture à valider : « la moitié » arrondie vers le haut ; seuls les dégâts létaux comptent.
- **Dégâts continus (10)** : tant qu'ils durent, un mourant ne fait pas de jet de stabilisation. Il faut **d'abord un
  soin** (au moins 1 PV), qui les arrête, **puis** le jet de stabilisation (le soin stabilise déjà, lecture 5).
  Au début de chaque tour d'un mourant qui subit des dégâts continus : jet raté d'office (1 PV) et 1 PV de plus,
  soit 2 PV (lecture à valider d'après le texte « fail stabilization automatically and lose 1 extra hit point
  per round »).

## Expérience et récompenses de fin de combat (phase 9)
- **XP** : saisi par le joueur pour chaque personnage. Aide facultative : total d'après les FP (tableau 12-2, Core p. 397, `data/combat/experience.json`), divisé entre les personnages (Core p. 399), simple suggestion. Lecture à valider : arrondi vers le bas.
  être saisi à la main.
- Or et objets trouvés : saisis par le joueur, jamais calculés (aucun trésor n'est inventé).
- Les récompenses sont rattachées au combat validé : annuler le combat les annule aussi.
