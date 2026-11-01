# Plan — gestion de personnage (proposition du 23/10/2026, à valider)

Aucun code n'est écrit tant que ce plan et les questions du bas ne sont pas validés.

## Ce que j'ai constaté dans l'application actuelle

- Un « personnage » de campagne est minuscule : nom, compte en po, localité, valeur de Prestige
  saisie à la main (`crates/campagne/src/modele.rs`, struct `Personnage`).
- L'intermède n'est pas un objet à part : c'est un **journal d'événements** (`Evenement`,
  `crates/campagne/src/evenements.rs`) rejoué pour calculer l'état (jour, trésoreries,
  chantiers…). Annuler/rétablir et la simulation reposent là-dessus. Chaque jour avancé est un
  événement `JoursAvances`.
- Les tests de capitaux (recettes) ne connaissent pas la fiche : on y saisit à la main les d20
  et un bonus (`crates/campagne/src/recettes.rs`, type `Jet`).
- Le fichier de campagne est un JSON versionné (`FORMAT = 1`) lu par les deux applications.
- Les crates sont indépendantes les unes des autres (domaine → regles → campagne → vues).

## Point d'intégration le moins intrusif

1. **Nouvelle crate `fiches`** (moteur de personnage), sans aucune dépendance vers `campagne`
   ni vers l'interface. Elle contient : le système d'effets, l'empilement des bonus, le calcul
   des valeurs dérivées avec leur explication, les prérequis de dons, la montée de niveau.
2. **Nouvelle crate `donnees_core`** (ou dossier `data/core/`) : les référentiels en JSON
   (races, classes, lignages, dons, compétences, sorts, équipement, états), comme
   `data/referentiel_officiel.json`. Le moteur ne contient aucune règle propre à une classe.
3. **Raccord dans `campagne`** : un seul nouvel événement `Fiche { personnage, evenement }`
   qui enveloppe le journal propre du personnage (création, montée de niveau, objets, XP…).
   Ainsi annuler/rétablir, historique et simulation marchent sans rien changer d'autre.
   Le `Personnage` existant reçoit un lien optionnel vers sa fiche ; sans fiche, tout se
   comporte comme aujourd'hui.
4. **Branchements intermède** (petits, un par un) : le bonus de compétence de la fiche
   alimente les jets de recettes ; le Prestige se calcule (niveau + mod. CHA ou don) au lieu
   d'être saisi ; or, XP, objets et jours dépensés sont répercutés sur la fiche par des
   événements du journal. Cela réalise `docs/FICHES_PERSONNAGE.md`.
5. **Interface** : uniquement dans la nouvelle application (`desktop/`), (l'ancienne application egui a été retirée).

## Phases (validation par vous à chaque fin de phase, tests verts à chaque étape)

| Phase | Contenu | Ce que vous pouvez tester |
|---|---|---|
| 0 | Décisions ci-dessous, format des données, registre des « points à confirmer » | — |
| 1 | Moteur : caractéristiques (floor), types de bonus et empilement, explicabilité, tests dédiés | tests automatiques |
| 2 | Création : méthodes de génération, 7 races, classe générique + Ensorceleur 1-20 (tables, 10 lignages), compétences, PV | fiche d'ensorceleur niveau 1 à 20 en lecture |
| 3 | Montée de niveau : XP, « montée en attente », assistant par étapes, annulation, historique | créer un perso, monter 1→5 |
| 4 | Dons et prérequis, sorts du Core (import), emplacements, DD, repos | choisir dons/sorts, dépenser des emplacements |
| 5 | Équipement, encombrement, CA/malus d'armure/échec des sorts, états et effets temporaires | équiper, voir la CA expliquée |
| 6 | Règles optionnelles de campagne (registre) + intégration intermède (jets, Prestige, or/XP/objets) | lier à un PJ, faire un test de recettes avec la fiche |

Les phases 1 et 2 posent tout le modèle ; ajouter une classe ensuite = ajouter un fichier de
données.

## Données : sourcing et droits

- J'ai vérifié que l'accès à Archives of Nethys fonctionne depuis cette machine. Le texte
  est **en anglais** et la page ne montre aucune mention de licence.
- Le contenu du Core Rulebook est publié sous licence OGL (hors « Product Identity »), ce qui
  autorise normalement la réutilisation avec les mentions de la licence. **À vérifier par
  vous** avant toute diffusion de l'application (je ne suis pas juriste) ; pour un usage
  personnel hors ligne le risque est faible.
- Le **texte français** du Manuel des joueurs (Black Book Éditions) reste protégé : je ne le
  recopierai pas. Je n'irai pas non plus chercher les 700 pages une à une en continu : un
  import ponctuel, poli (pauses entre requêtes), enregistré dans `data/`, puis l'application
  reste 100 % hors ligne.
- Chaque donnée garde sa source (livre, page, adresse AoN), comme le référentiel actuel.

## Questions à trancher avant de commencer (voir la réponse dans le fil)

1. Étendre la règle de travail « uniquement le Guide de campagne » au Core Rulebook pour les
   fiches (à inscrire dans le cahier des charges).
2. Langue des données importées.
3. Où stocker les fiches (dans le fichier de campagne, ou fichiers séparés).
4. Interface seulement dans la nouvelle application.
