# Gestion Pathfinder — consignes pour Claude

Application locale de gestion d'intermède pour **Pathfinder 1** : localités, bâtiments, salles,
organisations, équipes, gestionnaires, selon le **Guide de campagne** (VF d'*Ultimate
Campaign*). L'utilisateur est un joueur qui **ne code pas** : on lui parle en français, sans
jargon, et on lui livre des choses qu'il peut lancer et tester.

## Règles de travail (fixées par l'utilisateur, non négociables)

- Uniquement les règles du Guide de campagne. **Ne jamais inventer** de règle ni de valeur.
- Donner la **source** (livre, page) de chaque donnée officielle.
- Séparer les données **officielles** des données **personnalisées**.
- En cas de doute : un **point à confirmer**, posé à l'utilisateur, plutôt qu'une supposition.
- **Inspecter et proposer avant de coder** ; ne pas écraser de fichier sans le signaler.
- Toute lecture de règle non évidente est signalée comme « lecture à valider ».

Noms et textes français des dons, sorts, races, états, équipement… : ceux du **Manuel des joueurs**
(comparaison et points à confirmer : `docs/COMPARAISON_MANUEL_DES_JOUEURS.md`). Le livre fait foi.

Passation (état détaillé et prochaines étapes) : `docs/REPRISE.md`.
Exigences complètes : `docs/CAHIER_DES_CHARGES.md`. Décisions et lectures validées :
`docs/A_TRAITER.md`. À faire avec les fiches de personnage : `docs/FICHES_PERSONNAGE.md`.
Règles et points à confirmer du référentiel : page « Points à confirmer » de l'application.

## Architecture

```
crates/domaine   types purs : capitaux, monnaie (en pc), sources, statuts, fiches
crates/regles    référentiel officiel (données du livre), calculs, contrôles
crates/campagne  journal d'événements (event sourcing), état recalculé, annuler/rétablir, modèles personnels,
                 calendrier de Golarion, sauvegarde JSON (feature « serde »)
crates/vues      modèles d'affichage calculés par le cœur pour l'interface
crates/notes       notes de partie : feuille de session, journal de campagne par rubrique, notes de MJ (liens internes, recherche, export Markdown) ; rubriques dans data/notes/
crates/inventaire  objets, conteneurs, poids, transferts, équipement porté (emplacements : data/equipement/), import de catalogues texte ;
                 les modèles d'équipement viennent de data/core/ via campagne::catalogue_inventaire
crates/fiches      personnages (PF1, Core) : effets typés, fiche, progression, dons, sorts, équipement,
                 états ; données dans data/core/ ; plan : docs/PLAN_PERSONNAGES.md
desktop/         NOUVELLE application (Tauri 2) : src-tauri/ (Rust, commandes), ui/ (React + TS)
data/            referentiel_officiel.json (source de vérité des données du livre)
```

- Toute règle vit dans le cœur (`crates/`), jamais dans l'interface : l'interface affiche
  ce que `crates/vues` calcule et agit via les commandes de `desktop/src-tauri/src/main.rs`.
- Chaque action est un `Evenement` du journal ; un ensemble d'actions atomique est un
  `Evenement::Groupe`. Les identifiants nouveaux se calculent par `nouvel_id_*`.
- Les montants sont en pièces de cuivre (`Montant`, i64 : 1 pa = 10 pc, 1 po = 100 pc, 1 pp = 1 000 pc). Le format de fichier est 3 ; l'ancien format en pa est converti à la lecture.
- Les campagnes sont enregistrées dans `%APPDATA%\GestionPathfinder\campagnes`.

## Plateforme et interface

- **Windows d'abord** ; Linux plus tard, une fois l'application stable (code gardé portable).
- Fenêtre propre, jamais de navigateur ; hors ligne ; interface en français.
- Style validé : sombre, lisible, fluide (police Inter, une couleur fixe par capital :
  po or, Marchandises ambre, Influence violet, Travail bleu, Magie turquoise). Effets 3D
  prévus plus tard (three.js).

## Commandes (sous Windows)

```
cargo test                                   # tests du cœur
cargo clippy --all-targets -- -D warnings    # analyse
cargo fmt --all                              # formatage

cd desktop
npm install                                  # une seule fois
npm run build                                # interface -> desktop/dist
npm run check                                # vérification TypeScript
npx tauri icon src-tauri/icone-source.png    # une seule fois (icônes)
npx tauri dev                                # lance l'application (rechargement à chaud)
npx tauri build --no-bundle                  # exécutable : src-tauri/target/release/
```

Aperçu de l'interface sans le cœur, avec des données d'exemple : `MOCK=1 node build.mjs`
puis servir `dist/` (utile pour des captures d'écran).

GitHub Actions (`.github/workflows/windows.yml`) compile et teste à chaque envoi, dont un
essai de bout en bout de la nouvelle application (variable `GESTION_PATHFINDER_AUTOTEST`).

## État de la migration vers `desktop/`

Toutes les pages de l'ancienne application sont migrées dans `desktop/` (tableau de bord, campagnes avec
export/import, localités, personnages, bâtiments, recettes, organisations, gestionnaires,
historique, paramètres, référentiel, points à confirmer, modèles personnels). Reste : essai à
la main de chaque page par l'utilisateur. L'ancienne application egui (`app/`) a été retirée le 25/10/2026 à la demande de l'utilisateur. Le détail
est dans `docs/A_TRAITER.md`. Pour une page nouvelle : mettre les calculs dans `crates/vues`
(avec tests), ajouter les commandes Tauri, puis l'interface React.
