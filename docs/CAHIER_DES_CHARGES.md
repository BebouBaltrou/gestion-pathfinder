# Cahier des charges

Exigences fixées par l'utilisateur. Toute évolution de l'application doit les respecter ;
une exigence ne change que sur décision explicite de l'utilisateur (datée ici).

## Plateformes et lancement

- **Windows et Linux** : l'application doit se lancer sur les deux systèmes (exigence du
  23/10/2026). La compilation automatique (GitHub Actions) doit produire un exécutable pour
  chacun.
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
- Technologie retenue le 23/10/2026 : **Tauri** (fenêtre native Windows et Linux, interface
  web React, three.js pour la 3D), le moteur de règles Rust restant inchangé. Migration page
  par page dans `desktop/`, l'application egui actuelle (`app/`) restant disponible jusqu'à
  parité ; les deux travaillent sur les mêmes fichiers de campagne.
