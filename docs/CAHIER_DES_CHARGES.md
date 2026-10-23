# Cahier des charges

Exigences fixées par l'utilisateur. Toute évolution de l'application doit les respecter ;
une exigence ne change que sur décision explicite de l'utilisateur (datée ici).

## Plateformes et lancement

- **Windows d'abord** : l'application est développée et livrée pour Windows. **Linux ensuite**,
  une fois l'application stable et terminée (décision du 23/10/2026) : le code reste portable
  (Tauri, chemins de fichiers multi-plateformes) et la compilation Linux, déjà au point, est
  seulement mise en pause dans GitHub Actions.
- **Application locale** : fonctionne hors ligne ; campagnes et modèles personnels enregistrés
  sur le disque de l'utilisateur.
- **Fenêtre propre** : l'application s'ouvre dans sa propre fenêtre, jamais dans un navigateur
  (pas d'onglet ni de barre d'adresse).
- Interface en **français**.

## Règles du jeu

- Uniquement les règles du **Guide de campagne** (VF d'*Ultimate Campaign*).
- Ne jamais inventer de règle ni de valeur ; donner la source (livre, page) de chaque donnée
  officielle.
- Séparer les données officielles des données personnalisées.
- En cas de doute : un « point à confirmer » plutôt qu'une supposition.

## Méthode de travail

- Inspecter et proposer avant de coder.
- Ne pas écraser de fichier sans le signaler.

## Interface (décisions du 23/10/2026)

- Style validé : **mode sombre, lisible, fluide** — maquette « Gestion Pathfinder — maquette »
  (tableau de bord, fiche de bâtiment, recettes) : police Inter, fort contraste, une couleur
  fixe par type de capital, cartes arrondies, une seule action principale dorée par écran.
- **Effets 3D à venir** : la technologie d'interface doit les permettre.
- Technologie retenue le 23/10/2026 : **Tauri** (fenêtre native Windows, Linux plus tard, interface
  web React, three.js pour la 3D), le moteur de règles Rust restant inchangé. Migration page
  par page dans `desktop/`, l'application egui actuelle (`app/`) restant disponible jusqu'à
  parité ; les deux travaillent sur les mêmes fichiers de campagne.

## Fiches de personnage (décisions du 23/10/2026)

- Périmètre étendu aux **règles officielles de Pathfinder 1 (Core Rulebook)** pour les fiches
  de personnage, en plus du Guide de campagne pour l'intermède. Aucune règle non officielle.
- Première classe : Ensorceleur et ses 10 lignages ; les autres classes s'ajoutent par données.
- Règles = données (JSON dans `data/`), tout produit des **effets typés**, un moteur unique
  (`crates/fiches`) les agrège et explique chaque valeur. Plan : `docs/PLAN_PERSONNAGES.md`.
- Données importées d'Archives of Nethys (textes en anglais), interface et noms en français.
- Fiches stockées **dans le fichier de campagne** (journal d'événements) ; l'ancienne
  application egui ne les gérera pas. Interface uniquement dans `desktop/`.
- (Mise à jour 23/10/2026, soir) **Application finale entièrement en français**, y compris les
  textes de sorts, dons et capacités : remplace le choix « descriptions en anglais ». Le mode
  de traduction reste à fixer avec l'utilisateur.
- Pas d'*Advanced Race Guide* : aucun trait racial alternatif. Méthode de calcul des PV
  (jet / moyenne / moyenne arrondie / maximum) : **réglage de campagne** au choix du joueur.
- L'ensorceleur n'a pas de composants matériels bon marché à fournir (Eschew Materials) ; il ne
  reçoit pas l'Écriture de parchemin gratuitement.
