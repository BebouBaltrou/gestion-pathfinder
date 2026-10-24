# Points à confirmer — mode aventure (phase 3)

Règles relevées le 24/10/2026 sur Archives of Nethys (texte anglais du Core Rulebook, citations de page de ce
site). **Aucune n'est inventée** ; chaque ligne dit ce que le moteur fait et ce qui reste à valider.
Statut : « lecture à valider » = comportement implémenté d'après ma lecture, à confirmer par toi.

## Sources utilisées

- Guérison, PV temporaires, dégâts non létaux, seuils : Core Rulebook p. 189-191 (« Injury and Death »).
- Faim et soif : Core Rulebook p. 444 (« Starvation and Thirst »).

## Ce que le moteur applique

| # | Règle | Source | Implémentation | Statut |
|---|---|---|---|---|
| 1 | Une nuit de repos complète (8 h ou plus) rend 1 PV par niveau de personnage ; toute interruption notable annule la guérison de la nuit | p. 191 | `Repos` : ≥ 8 h et non interrompu. Une période de repos = une nuit, même si elle dure plus de 8 h | lecture à valider |
| 2 | Alitement complet pendant une journée et une nuit : 2 × le niveau en PV | p. 191 | Il faut 24 h ; 48 h comptent 2 périodes | lecture à valider |
| 3 | Dommages de caractéristique temporaires : 1 point par nuit et par caractéristique atteinte ; 2 points par 24 h d'alitement complet | p. 191 | Idem règles 1 et 2 ; jamais en dessous de 0 | lecture à valider |
| 4 | On ne récupère jamais plus de PV qu'on n'en a perdu ; un soin magique ne dépasse pas le maximum | p. 191 | `Vital::soigner` | sûr |
| 5 | Les PV temporaires absorbent les dégâts avant les PV, disparaissent à la fin de l'effet, ne se récupèrent pas | p. 191 | Les temporaires passent en premier, aussi pour les dégâts non létaux | lecture à valider (dégâts non létaux) |
| 6 | Plusieurs sources de PV temporaires | non trouvé dans le texte relevé | On garde le plus grand lot, sans cumul | **point à confirmer** |
| 7 | Dégâts non létaux : 1 PV par heure et par niveau ; un sort qui soigne des PV retire autant de dégâts non létaux | p. 191 | `laisser_passer` à chaque avance du temps ; `soigner` retire autant de non létaux | sûr |
| 8 | Soif : 1 jour + (score de Constitution) heures sans eau, puis test de Constitution toutes les heures (DD 10, +1 par test précédent), sinon 1d6 dégâts non létaux | p. 444 | `Survie`, test demandé au joueur (d20 physique + mod. de Con. de la fiche) | lecture à valider |
| 9 | Faim : 3 jours sans nourriture, puis test de Constitution par jour (même DD), sinon 1d6 non létaux | p. 444 | idem | lecture à valider |
| 10 | Premier test : « après » le délai de grâce | p. 444 | Le premier test tombe **une période complète après** la fin du délai (1 h pour la soif, 1 jour pour la faim) | **point à confirmer** |
| 11 | Les dégâts non létaux dus à la faim ou à la soif rendent fatigué et ne se récupèrent pas avant d'avoir bu ou mangé | p. 444 | « Fatigué » ajouté à la fiche au premier échec, retiré quand plus rien n'est bloqué ; la part due à la soif se libère en buvant, celle de la faim en mangeant | lecture à valider |
| 12 | Besoins : 1 gallon d'eau et ~1 livre de nourriture par jour (Moyen), moitié pour Petit, 2 à 3 fois l'eau par climat très chaud | p. 444 | **Non modélisé** (taille et climat). Un bouton « Boire » / « Manger » remet les compteurs à zéro | à décider |

## Ce que la spec demande et que le Core ne prévoit pas

- **Repos court** : le Core PF1 n'a pas de « repos court ». Je ne l'implémente pas. Si tu veux une règle
  maison (ex. 1 heure de repos pour reprendre son souffle), dis laquelle.
- **Fatigue par manque de sommeil** : aucune règle officielle trouvée dans le Core (les forums Paizo confirment
  qu'il n'y en a pas ; seul un module d'aventure en propose une). **Le moteur n'en invente pas.** Par défaut :
  pas de fatigue de sommeil ; le temps depuis le dernier repos sera simplement affiché. Si tu veux une règle maison
  (ex. fatigué après 24 h sans repos), elle sera marquée comme personnalisée.
- **Torches et lumière** : écartés par toi.

## Sorts de l'ensorceleur au repos

Le repos appelle `EvenementFiche::Repos` du moteur de personnage, qui rend **tous** les emplacements de sorts
(lanceur spontané : aucune préparation). Si une règle plus fine existe (heure de la journée, interruption),
le moteur de personnage devra la porter ; je ne l'ai pas inventée.
