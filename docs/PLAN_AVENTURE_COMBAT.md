# Plan — mode aventure et mode combat (proposition du 24/10/2026, à valider)

Phase 1 de ton plan : exploration + plan. **Aucun code n'est écrit** tant que ce plan et les
« points à confirmer » du bas ne sont pas validés. Branche locale `claude/affectionate-greider-c973bb`,
bâtie sur la branche du moteur de personnage (`fervent-tereshkova`, phase 6) ; rien n'est envoyé sur GitHub.

## 1. Ce que j'ai trouvé dans le dépôt

- **Cœur** : `domaine` → `regles` → `campagne` (journal d'événements rejoué, annuler/rétablir) →
  `vues` ; plus `fiches` (moteur de personnage : effets typés, empilement, explications,
  journal propre à chaque personnage, injecté dans la campagne par `Evenement::Fiche`).
- **Le temps de la campagne est en jours** (`JoursAvances`, `DateGolarion` = année/mois/jour).
  Il n'existe ni heure ni round de campagne. Les effets temporaires des fiches ne bougent qu'avec
  `TempsEcoule { rounds }`, déclenché au changement de jour (14 400 rounds/jour).
- **Effets temporaires** (`EffetTemporaire`, `fiches/etats.rs`) : nom + effets typés + `rounds_restants`
  (compte à rebours, `None` = illimité). Insuffisant pour ta spec : pas d'instant absolu, pas de
  durée « minutes/heures/concentration/jusqu'à décharge », pas de source ni de propriétaire, pas
  d'alerte d'expiration. États (34 du Core) : liste d'identifiants sur la fiche, sans durée.
- **Aucun suivi de santé** : la fiche calcule les PV maximum mais ne garde ni PV actuels, ni PV
  temporaires, ni dégâts non létaux, ni niveaux négatifs. Les dégâts/drain de caractéristique
  existent dans la fiche. Emplacements de sorts dépensés et `Repos` existent (journal du personnage).
- **Interface** : Tauri 2 + React/TypeScript dans `desktop/ui` (pages Tableau de bord, Campagnes,
  Personnages, Paramètres). Pas encore de bibliothèque d'animation ni de glisser-déposer.
- **Fichier de campagne** : JSON versionné (`FORMAT = 1`), lu par l'ancienne et la nouvelle application.

## 2. Critique du modèle actuel vis-à-vis de ta spec

1. Le compte à rebours `rounds_restants` doit devenir un **instant d'expiration absolu** + une
   énumération de durées (§1.3). Cela touche `fiches` : changement compatible (lecture de
   l'ancien format conservée) mais à coordonner avec le fil « Gestion de personnage ».
2. Il faut un **état vital** par personnage (PV, temporaires, non létaux, niveaux négatifs, état de
   santé). Il n'existe pas : à créer.
3. « Un seul modèle d'effet temporisé » : les états, poisons, maladies, sorts, entités passent tous
   par le même objet. Je propose de **remplacer** `EffetTemporaire` par ce modèle (migration à la
   lecture) plutôt que d'en ajouter un second.
4. L'intermède (jours) et l'aventure (rounds) doivent partager **une seule horloge** : l'horloge absolue en
   rounds depuis le jour 1 à 00:00 ; le jour d'intermède en est dérivé.

## 3. Architecture proposée

```
crates/temps      NOUVEAU  Instant (rounds), Durée (énumération extensible), règles de conversion §1.2,
                           effets temporisés + expirations. Sans interface. Tests de propriétés (proptest).
crates/aventure   NOUVEAU  état vital, repos (règle unique), régénération, faim/soif, sommeil/fatigue,
                           consommables, journal d'aventure, alertes.
crates/combat     NOUVEAU  état de combat transactionnel, initiative, tours, économie d'actions,
                           santé en combat, sorts en combat, entités, journal de combat, rejeu.
fiches            adapté   consomme `temps` pour ses effets temporaires (migration).
campagne          adapté   un événement `Aventure(..)` et un événement `Combat(résumé)` dans le journal
                           (annuler/rétablir, historique, simulation fonctionnent sans autre changement).
vues              adapté   modèles d'affichage (jauges, barre d'initiative, décompositions).
```

Principes : les règles de calcul restent dans `fiches` (CA, JS, DD, bonus) ; `aventure`/`combat`
l'appellent et ne recalculent rien. Les données (états, poisons, règles optionnelles) vont dans
`data/` en JSON, pas dans le code.

**Persistance** : l'aventure vit dans le journal de campagne (nouveaux événements). Le combat en cours
est un **fichier à part** (`combat_en_cours.json` à côté de la campagne), réécrit de façon atomique
(fichier temporaire puis renommage) après chaque action — reprise après plantage ; à la validation, un seul
événement résumé entre dans la campagne. Passer le fichier de campagne en `FORMAT = 2` : l'ancienne
application egui ne pourra plus l'ouvrir (elle est de toute façon destinée à disparaître).

## 4. Le temps

- Unité : le round (6 s) ; instant = nombre de rounds depuis le jour 1 à 00:00.
- Hors combat : pas de 5 min / 10 min / 30 min / 1 h, plus saisie directe de la date/heure.
- En combat : l'horloge avance par round ; les boutons horaires sont suspendus. À la validation, l'horloge
  de la campagne = début du combat + rounds × 6 s ; si le combat est abandonné, elle reste au début.
- **Passage de minuit** (décidé) : franchir minuit avance d'un jour d'intermède (`JoursAvances`).
- Conversion §1.2 : voir points à confirmer 1 et 2. Comme chaque effet garde un instant d'expiration
  absolu, les effets en minutes ou en heures n'ont **rien à convertir** : ils suivent l'horloge qui
  avance. Seuls les effets en rounds (et « jusqu'à… ») sont dissipés à la fin, avec l'alerte listant
  nom, personnage, source et temps perdu, et un bouton « maintenir » avec durée saisie.
- Alertes d'expiration : une file d'alertes dans le moteur (confirmer / repousser), l'interface ne fait qu'afficher.

## 5. Phases (une à la fois, validation à chaque fin)

| # | Contenu | Ce que tu peux tester |
|---|---|---|
| 1 | Ce document, validation des points à confirmer | lecture |
| 2 | `temps` : horloge, durées, conversion, effets temporisés, alertes, tests de propriétés | tests automatiques |
| 3 | `aventure` : avance du temps, repos, régénération, faim/soif, sommeil/fatigue, effets longs, journal. Les règles du Core sont relevées une à une avec leur page, lectures à valider | scénario « une journée de voyage » en tests |
| 4 | `combat` (machine à états) : transaction, initiative, tours, annuler/rétablir, journal, sauvegarde continue | scénarios rejoués |
| 5 | Santé et états : dégâts, soins, mourant/stable/mort, 34 états, poisons/maladies | idem |
| 6 | Sorts en combat | idem |
| 7 | Consommables et ressources | idem |
| 8 | Interface : prototype de la barre d'initiative + jauge animée dans 2–3 technologies, comparaison, ta décision, puis tableau de bord complet | fenêtre réelle |
| 9 | Fin de combat, statistiques, lien avec l'intermède, export | fenêtre réelle |
| 10 | Règles optionnelles de combat, finitions, accessibilité | fenêtre réelle |

Proposition : **faire le prototype d'interface dès après la phase 2** (une demi-journée, rien migré) pour
choisir le front avant d'écrire les phases 3–7, au cas où il influencerait les modèles d'affichage.
Hypothèse de départ à tester : rester en React (déjà en place, riche écosystème d'animation et de
glisser-déposer), contre Svelte et Solid (plus légers, animations natives) ; Leptos/Dioxus gardent tout en Rust
mais ont un écosystème d'animation/glisser-déposer bien plus mince. Je ne migre rien sans ton accord.

## 6. Hors périmètre v1 (à reporter dans `ROADMAP.md` dès ton accord)

Convocations détaillées (Bestiaire), ennemis modélisés, grille/mesure, sons (bus d'événements prévu),
raccourcis/actions épinglées, mode daltonisme et thèmes supplémentaires, personnalisation poussée, aventure
étendue (carte de voyage, rencontres).

## 7. Points à confirmer (réponse attendue avant tout code)

1. **Conversion §1.2** : je comprends « reliquat conservé à la minute près, arrondi à la minute supérieure,
   affiché en heures et minutes ». Exemple : il reste 7 min 24 s à un effet en minutes à la fin du combat →
   il reste 8 min. C'est une simplification « maison », pas une règle du Core. Correct ?
2. **Recul d'horloge** : ma proposition par défaut — un effet expiré **reste expiré** après un recul ;
   le recul affiche la liste des effets expirés dans l'intervalle avec une case « réactiver » (décochée
   par défaut). Les effets créés dans l'intervalle reculé sont, eux, retirés après confirmation. Reculer au-delà
   de minuit est refusé (utiliser Annuler, qui défait aussi le jour d'intermède). OK ?
3. **Structure** : trois crates (`temps`, `aventure`, `combat`) comme ci-dessus, et remplacement de
   `EffetTemporaire` dans `fiches` (je coordonnerai avec le fil « Gestion de personnage »). OK ?
4. **Persistance** : aventure dans le journal de campagne, combat en fichier à part, `FORMAT = 2`
   (l'ancienne application egui ne lit plus ces campagnes). OK ?
5. **Prototype d'interface** tôt (après la phase 2) plutôt qu'en phase 8 ?
6. **Durées des sorts** : tu as choisi « je saisis la durée » ; ta spec dit aussi « calcul de la durée ». Je
   propose : le champ est **prérempli** quand le livre donne une formule simple (« 1 min/niveau »), sinon
   vide, et tu valides ou corriges. OK ?

Déjà décidés avec toi : minuit avance l'intermède ; survie = nourriture/eau, sommeil/fatigue, guérison
naturelle (pas de torches) ; ordre d'initiative = uniquement tes personnages.

## 8. Décisions du 24/10/2026 (réponses d'Antoine)

1. **Arrondi §1.2** : le reliquat d'un effet en minutes est arrondi à la **minute entière la plus proche, minimum
   1 minute** (7 min 24 s → 7 min ; 7 min 30 s → 8 min ; 42 s → 1 min). Simplification « maison », pas une règle du Core.
2. Points 2 à 6 acceptés tels que proposés (recul sans résurrection, trois crates, `FORMAT = 2`, prototype
   d'interface tôt, durée de sort préremplie).

## 9. Avancement

- **Phase 2 faite** : crate `crates/temps` (instants en rounds, durées extensibles, horloge aventure/combat,
  effets temporisés génériques, alertes, conversion de fin de combat, recul d'horloge). 26 tests dont 7 tests de
  propriétés. Reste, pour le branchement : remplacer `EffetTemporaire` dans `fiches` par ce modèle (migration de
  l'ancien format à la lecture), à coordonner avec le fil « Gestion de personnage ».
- **Phase 3 (moteur) faite** : crate `crates/aventure` (état vital, faim et soif, repos, effets temporisés,
  journal d'aventure, horloge) + événement `DommagesCaracteristique` dans `fiches`. 22 tests. Lectures de
  règles à valider : `docs/POINTS_A_CONFIRMER_AVENTURE.md`. Le branchement dans la campagne
  (`Evenement::Aventure`, `FORMAT = 2`) et les commandes Tauri viennent avec l'interface.
  Écart avec le plan : au lieu de supprimer `EffetTemporaire`, il sert de simple porteur des effets actifs
  dans la fiche (sans compte à rebours) ; le temps est suivi par `crates/temps`, seule source de vérité.
- **Phase 4 (machine à états du combat) faite** : crate `crates/combat` (copie de travail transactionnelle,
  initiative avec départage, round de surprise, tours, retarder, modifier l'initiative, action préparée,
  annuler/rétablir action par action, aperçu de fin avec conversion §1.2 et contestation, validation totale,
  partielle ou abandon, sauvegarde atomique et reprise). 15 tests. Reste pour les phases suivantes :
  régénération et guérison accélérée par round (phase 5), lancement de sorts (phase 6), consommables (phase 7),
  économie d'actions (phase 5), statistiques de campagne (phase 9).
- **Phase 5 (santé, états, actions) faite** : états de santé (conscient, chancelant, hors de combat, inconscient,
  mourant, stable, mort), jets de stabilisation, états du Core donnés par des effets à durée (partagés, retirés à
  l'expiration), dégâts typés avec immunités, résistances et RD (détail dans le journal), soins par round,
  économie d'actions avec catalogue de 58 actions (`data/combat/actions.json`). 228 tests dans le workspace.
  Lectures à valider : `docs/POINTS_A_CONFIRMER_COMBAT.md`. Non modélisés : dégâts massifs, dégâts continus.
- **Phase 6 (sorts en combat) faite** : aperçu et lancement assistés (emplacements, métamagie, DD, niveau de
  lanceur, lancement défensif, échec des sorts profanes, durée proposée par le livre), effets de 14 sorts sur les
  personnages joués, tests de concentration (Core p. 206), journal rejouable. 253 tests dans le workspace.
  Lectures à valider : `docs/POINTS_A_CONFIRMER_SORTS.md`. Reste : sorts préparés, convocations, contresort,
  pouvoirs de lignage (phase 7).
- **Phase 7 (consommables et ressources) faite** : inventaire d'objets (potions, parchemins, baguettes, bâtons, objets
  à charges, munitions), utilisation en combat avec effet du sort, ressources quotidiennes des 10 lignages rendues
  par le repos. Lectures à valider : `docs/POINTS_A_CONFIRMER_OBJETS.md`.
- **Phase 8 (interface) faite pour l'aventure et le combat** : la campagne contient l'état d'aventure
  (événements `AventureDemarree`, `Aventure`, `CombatValide`, `AventureTerminee` ; format de fichier 2 écrit
  seulement si l'aventure est utilisée), l'application Tauri expose les commandes (`desktop/src-tauri/src/aventure.rs`),
  le combat est sauvegardé après chaque action (`<campagne>.combat.json`) et repris après un plantage, et l'interface
  React (+ `motion`) a deux écrans : « Aventure » (horloge, repos, personnages, effets, ressources, objets, journal)
  et « Combat » (initiative et départage, round de surprise, barre d'initiative, actions, sorts, objets, santé,
  concentration, fin de combat avec alertes). Essai automatique dans la vraie fenêtre : 62 vérifications.
  Palettes A et B choisissables dans Paramètres. Backlog : `ROADMAP.md`.
