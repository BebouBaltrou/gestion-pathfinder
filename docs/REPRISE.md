# Reprise du projet — état au 23/10/2026

Document de passation : ce qui a été fait jusqu'ici dans une conversation Claude (environnement
cloud Linux, sans accès à crates.io ni npm, compilation vérifiée uniquement par GitHub Actions),
et ce qui reste. À lire après `CLAUDE.md`. Peut être supprimé une fois la reprise faite.

## Où on en est

**Cœur de règles (`crates/`)** — complet pour le périmètre actuel, 54 tests verts :
référentiel officiel du Guide de campagne (78 salles, 3 extensions, 22 équipes,
17 gestionnaires, 59 modèles de bâtiments, 14 d'organisations), localités et trésoreries,
personnages, transferts, bâtiments / étages / salles (construction avec limite de dépense,
accélération, achat, amélioration), extensions de salle, organisations / équipes /
délocalisation, gestionnaires et salaires, recettes (tests de capitaux), absence, usure,
test de prestige, vente / abandon / réparation, simulation, export HTML/CSV, modèles
personnels. Les décisions et lectures de règles sont dans `docs/A_TRAITER.md`.

**Ancienne application (`app/`, egui)** — retirée le 25/10/2026 (validée par l'utilisateur).

**Nouvelle application (`desktop/`, Tauri 2 + React)** — commencée le 23/10/2026 :
- `crates/vues` : modèles d'affichage (en-tête, tableau de bord, résumé des conséquences).
- `desktop/src-tauri` : `session.rs` (ouvrir, créer, exécuter + enregistrer, annuler,
  rétablir, simulation) et commandes Tauri dans `main.rs`.
- `desktop/ui` : coquille (navigation groupée, date, annuler/rétablir avec Ctrl+Z/Ctrl+Y,
  +7 j, +30 j, jour suivant, simulation, notifications), pages Tableau de bord et Campagnes ;
  les autres pages affichent « arrive bientôt ».
- Compilée et testée de bout en bout par GitHub Actions (job « Nouvelle application —
  Windows » : l'interface crée une campagne, avance, annule, simule, puis écrit un rapport).
- **Jamais vue par un humain** : la fenêtre réelle n'a pas encore été ouverte sur le PC de
  l'utilisateur. Les captures ont été faites avec des données d'exemple (`MOCK=1`).

## Première chose à faire en local

1. `cargo test` à la racine (54 tests attendus).
2. `cd desktop`, `npm install`, `npm run check`, `npx tauri icon src-tauri/icone-source.png`,
   puis `npx tauri dev` : première ouverture réelle de la nouvelle application. Vérifier avec
   l'utilisateur le rendu (police Inter, couleurs, mise en page à sa taille d'écran) et que sa
   campagne existante s'ouvre et s'affiche correctement dans le tableau de bord.
3. Corriger ce qui ne va pas avant de migrer d'autres pages.

## Ensuite : migration page par page

Ordre convenu : Localités, Personnages, Bâtiments (salles, extensions, vente, réparation),
Recettes, Organisations, Gestionnaires, Historique, Paramètres, Référentiel, Points à
confirmer, import/export, modèles personnels. Ancienne application (`app/`) retirée.

Méthode pour chaque page : lire l'historique git de `app/src/pages/<page>.rs` (comportement de référence), mettre
les calculs dans `crates/vues` (avec tests), ajouter les commandes Tauri, faire l'interface
React au style de la maquette, vérifier dans la vraie fenêtre, puis faire tester l'utilisateur.
Les maquettes validées (tableau de bord, fiche de bâtiment, recettes) sont dans l'artefact
« Gestion Pathfinder — maquette » sur claude.ai ; la page Recettes y suit l'ordre : période →
qui produit → jets de d20 → bilan → « Encaisser N jours ».

## Plus tard (décidé, pas encore commencé)

- Fiches de personnage et ce qui en dépend : `docs/FICHES_PERSONNAGE.md` (limite par le
  Prestige, travail du PJ avec Coût produit payant par défaut, promotion d'entreprise).
- Effets 3D (three.js dans l'interface Tauri) ; l'utilisateur n'a pas encore précisé lesquels.
- Linux : une fois l'application stable et terminée (job CI « Nouvelle application — Linux »
  déjà au point, désactivé par `if: ${{ false }}`).
- Pistes évoquées : adaptation pour le MJ, événements d'intermède, publication des exécutables
  en GitHub Releases.

## Points d'attention

- Les deux applications partagent les fichiers de campagne : ne pas changer le format du
  journal (`campagne::fichier::FORMAT`) sans migration.
- Recettes : depuis le 23/10, un seul type de revenu par salle/équipe et les jours en attente
  sont encaissés du plus ancien au plus récent ; d'anciens encaissements « en trop » des
  campagnes de test sont ignorés au rechargement (comportement voulu).
- Sécurité : un jeton GitHub à large portée avait été fourni à la conversation précédente ;
  l'utilisateur doit le révoquer. Utiliser désormais ses propres identifiants Git.
- L'utilisateur ne code pas : expliquer simplement, livrer quelque chose à tester, poser les
  questions de règles sous forme de « point à confirmer ».
