# Points à confirmer — consommables et ressources (phase 7)

Règles relevées le 24/10/2026 sur Archives of Nethys (Core Rulebook, pages citées par ce site) et formules des
pouvoirs de lignage recopiées de `data/core/ensorceleur.json` (un test vérifie chaque formule contre le texte).
« lecture à valider » = comportement implémenté d'après ma lecture.

## Ce que fait le moteur

- **Inventaire de consommables** par personnage (potions, parchemins, baguettes, bâtons, objets à charges,
  objets à utilisations quotidiennes, munitions), suivi dans l'état d'aventure : annuler/rétablir et validation du
  combat fonctionnent comme pour le reste. Ce n'est pas le catalogue de matériel du Core, qui ne contient pas les
  objets magiques : tu saisis l'objet (nom, sort, niveaux, charges).
- **Utiliser un objet en combat** : dépense de l'objet, action du tour, et **effet du sort appliqué** (même
  mécanisme que le lancement de sorts, sauf pour un sort sur un ennemi ou sans effet connu).
- **Ressources à usages quotidiens** : les pouvoirs de lignage des 10 lignages (3 + Charisme, une fois par jour,
  paliers aux niveaux 9, 17, 20…), rendues par une nuit de repos complète.
- **Munitions** : décomptées à chaque tir, récupérables après le combat.

## Règles et lectures

| # | Règle | Source | Implémentation | Statut |
|---|---|---|---|---|
| 1 | Potion : un sort de niveau 3 ou moins ; boire est une action simple qui agit tout de suite ; le buveur est le lanceur et la cible | p. 477 | Création refusée au-delà du niveau 3 ; une seule cible (le buveur) ; effet au niveau de lanceur de la potion | sûr |
| 2 | Baguette : un sort de niveau 4 ou moins, 50 charges au plus, une charge par usage ; activation par déclenchement de sort, en général une action simple ; il faut pouvoir lancer le sort | p. 496, p. 458 | Création refusée au-delà ; « pouvoir lancer » = le sort est sur la liste de classe du porteur ; action simple ; niveau de lanceur de la baguette | lecture à valider |
| 3 | Bâton : 10 charges au plus, niveau de lanceur minimum 8 ; un sort peut coûter plusieurs charges ; le porteur utilise son niveau de lanceur s'il est plus haut ; DD selon la caractéristique du porteur | p. 491 | Charges saisies par usage ; niveau de lanceur = le plus haut des deux ; action simple ; le DD n'est pas calculé (le dé de sauvegarde est celui des adversaires, non modélisé) | lecture à valider |
| 4 | Parchemin : il faut pouvoir le lire, que le sort soit du bon type (profane ou divin), sur la liste de classe, et avoir le score minimum | p. 490 | Type, liste de classe et score (10 + niveau du sort) contrôlés | sûr |
| 5 | Parchemin : si ton niveau de lanceur est inférieur, test de niveau de lanceur DD = niveau de lanceur du parchemin + 1 ; échec : test de Sagesse DD 5 (un 1 naturel rate) pour éviter l'accident ; sans accident le parchemin n'est pas consommé | p. 490 | d20 demandés ; le modificateur de Sagesse vient de la fiche ; un accident consomme le parchemin, le sort est perdu et l'accident se résout à la main (tableau du livre non modélisé) | lecture à valider |
| 6 | Parchemin : action simple, ou le temps d'incantation du sort s'il est plus long ; provoque une attaque d'opportunité | p. 490 | Action simple ou complexe ; attaques d'opportunité non modélisées | lecture à valider |
| 7 | Parchemin profane et armure : échec des sorts profanes si le sort a une composante gestuelle | p. 83, 208 | d100 demandé comme pour un sort lancé | **point à confirmer** (le livre le dit pour le lancer de sorts ; non vérifié pour les parchemins) |
| 8 | Baguette ou bâton et échec des sorts profanes | — | **Non appliqué** | **point à confirmer** |
| 9 | Pouvoirs de lignage : « 3 + modificateur de Charisme » par jour (rayons, griffes en rounds), « une fois par jour », paliers 9 / 17 / 20 (souffle, explosion, enfer…), « autant que le niveau d'ensorceleur » (rounds ou minutes), Adepte de la métamagie (1 au niveau 3 puis +1 tous les 4 niveaux, 5 au plus) | p. 72-77 | `data/combat/ressources_lignage.json` ; un test compare chaque formule au texte du livre | sûr |
| 10 | Une nuit de repos complète rend les usages quotidiens | repos p. 191 (lecture de la phase 3) | Rendus avec les emplacements de sorts | lecture à valider |
| 11 | Charges des baguettes et bâtons | p. 491, 496 | Ne reviennent pas au repos (le livre décrit une recharge de bâton par un lanceur : non modélisée) | **point à confirmer** |
| 12 | Munitions : une unité par tir | décision d'Antoine | Décompte automatique à l'attaque, attaque refusée sans munition ; récupération saisie après le combat | validé |

## Pas encore fait

- Aspect **prix, poids, création d'objets** : hors périmètre ici.
- **Tableau des accidents de parchemin** : à résoudre à la main.
- **Objets de lignage de l'ensorceleur « familier »**, **Écriture de parchemin** : à faire plus tard.
- **Nom français des pouvoirs** : le catalogue ne donne pour l'instant que les noms anglais.

## Décisions d'Antoine du 24/10/2026

- Lectures validées : 2 (baguette), 5 et 6 (parchemin), 10 (repos), 11 (charges non rendues au repos).
- **Parchemin et échec des sorts profanes (7)** : comme pour un sort lancé (déjà le cas).
- **Bâtons (3)** et **échec des sorts profanes d'une baguette ou d'un bâton (8)** : on y reviendra plus tard.
- **Munitions (12)** : le décompte est **automatique** à chaque attaque à distance (action `Attaquer` : une
  munition par tir, plusieurs à l'attaque à outrance, rendues si on annule). L'attaque est **refusée** si le
  personnage n'a pas la munition dans son inventaire, ou pas assez pour les tirs demandés ; l'action du tour n'est
  alors pas dépensée. La récupération après combat reste une saisie (aucune règle de perte inventée).
