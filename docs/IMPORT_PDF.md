# Importation d'une fiche de personnage PDF — fonctionnalité en développement

**Statut (25/10/2026) : en développement, à reprendre plus tard.** Priorité actuelle : implémenter toutes les
classes dans le créateur de fiche (Prêtre, puis Druide).

## Ce qui existe

- `crates/importation` : lecture d'un PDF à champs remplissables (feuille française « Pathfinder 1 », 6 pages,
  755 champs), comparaison avec le moteur de règles (`crates/fiches`), rapport d'écarts.
- Écran « Corriger / Garder » : Personnages → « Importer une fiche PDF… » (en développement). Une valeur gardée
  devient un bonus « personnalisé » dans la fiche.
- Créé à partir de la fiche de test de Caladrel (ensorceleur humain niveau 11). Classes lues : ensorceleur,
  prêtre, druide (prêtre et druide sans fiche réelle de test).
- Objets magiques à bonus chiffré reconnus d'après le Manuel des joueurs (`data/core/objets_magiques.json`).

## Limites connues

- Les sorts de la feuille sont ignorés (décision de l'utilisateur).
- Les domaines, la divinité et l'alignement ne sont pas lus (les champs n'existaient pas dans la fiche) ;
  le pacte avec la nature du druide non plus.
- Une feuille avec des dons manquants donne une fiche inachevée (acceptée à la création).
- Hypothèses à valider : la colonne « Base » inclut la race ; le bonus libre de l'humain va à la caractéristique
  d'incantation ; les dés de vie sont répartis régulièrement pour retrouver le total de PV.
- Un modèle de PDF = un gabarit (`crates/importation/src/feuille.rs`).

## À faire plus tard

- Reprendre après l'implémentation des classes dans le créateur de fiche : lire domaines, divinité, alignement,
  sorts préparés.
- Tester sur de vraies fiches de prêtre et de druide.
