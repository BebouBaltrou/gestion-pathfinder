# Feuille de route — ce qui reste après la première version du mode aventure et combat

Éléments **hors périmètre de la v1**, notés pour ne pas être perdus (décision d'Antoine, 24/10/2026).
Le plan et l'état d'avancement sont dans `docs/PLAN_AVENTURE_COMBAT.md`.

## Hors périmètre v1

- **Convocations détaillées** : sorts de convocation avec les créatures officielles du Bestiaire (fiches importées
  depuis Archives of Nethys). En v1 : un gabarit minimal saisi à la main (nom, PV, CA, attaques, notes, durée liée au
  sort) — pas encore dans l'interface.
- **Ennemis modélisés** : fiches, PV, CA, actions des adversaires (aujourd'hui : théâtre de l'esprit).
- **Grille et carte**, mesure des distances.
- **Sons** : un bus d'événements applicatifs est prévu pour les brancher plus tard.
- **Actions rapides et raccourcis clavier personnalisables** : les actions sont modélisées (catalogue
  `data/combat/actions.json`) pour pouvoir être épinglées plus tard.
- **Mode daltonisme et thèmes supplémentaires** : l'architecture de thèmes par jetons de couleur est en place
  (trois thèmes aujourd'hui) ; l'icône et le texte accompagnent déjà chaque état, jamais la couleur seule.
- **Personnalisation poussée de l'affichage** au-delà de la disposition des panneaux (dispositions enregistrées,
  glisser-déposer des panneaux : prototypés dans `prototypes/`, à intégrer).
- **Mode aventure étendu** : carte de voyage, rencontres aléatoires.

## Reporté par décision d'Antoine

- **Faim et soif** : le moteur existe (désactivé par défaut) ; à reprendre plus tard.
- **Métamagie** : les données et le calcul existent (désactivés) ; à intégrer plus tard, avec la règle de l'action
  complexe du lanceur spontané à confirmer.
- **Bâtons** (recharge, charges par sort) et **échec des sorts profanes des baguettes et bâtons**.

## Non modélisé pour l'instant (à demander ou à décider)

- Sorts à préparer (mage…), contresort, dissipation, résistance à la magie.
- Dégâts continus par round hors combat, régénération et guérison accélérée typées, tableau des accidents de parchemin.
- Calcul automatique des dés de dégâts et de soins des sorts (le résultat du dé est saisi).
- Statistiques de campagne (dégâts, sorts, rounds) conservées d'un combat à l'autre, XP et butin de fin de combat.
- Export du combat (journal rejouable), règles optionnelles de combat (initiative variable, critiques optionnels…).
- Disposition des panneaux enregistrable et réorganisable, accessibilité complète (navigation clavier à parcourir
  partout, tailles ajustables).

## Hors périmètre v1 de l'inventaire

À faire plus tard, sans être implémenté en v1 (décision d'Antoine, 25/10/2026) :

1. Inventaire des montures (capacité de charge, sacoches, barde, effet du poids sur la vitesse).
2. Véhicules (soutes, passagers, équipage, vitesse, entretien).
3. Durabilité et réparation d'objets.
4. Marchands, boutiques, taux de vente régionaux.
5. Fabrication d'objets magiques détaillée (liée à l'intermède).
6. Silhouettes personnalisées par race ou taille.
7. Icônes et images d'objets personnalisées.
8. Gestion de l'identification d'objets magiques.
9. Jet de récupération des munitions (bris à 50 %).
Présents dans le catalogue d'équipement (`data/core/`) mais sans traitement particulier en v1 (simples entrées du catalogue) :
Absents du catalogue d'équipement (`data/core/`) parce que relevant de ces points :
services, animaux, barde, moyens de transport, lancement de sorts.
