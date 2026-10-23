# Gestion d'intermède — Pathfinder 1 (Guide de campagne / Ultimate Campaign)

Application locale (Windows et Linux) de gestion des localités, bâtiments, salles, organisations,
équipes et gestionnaires selon les règles d'intermède du **Guide de campagne**
(VF de *Ultimate Campaign*).

Exigences : voir [docs/CAHIER_DES_CHARGES.md](docs/CAHIER_DES_CHARGES.md).

## État : étape 7 — simulation, export, tableau de bord, vente et réparation

| Étape | Contenu | État |
| --- | --- | --- |
| 1 | Document des règles et points à confirmer | fait |
| 2 | Référentiel officiel + moteur de calcul | fait |
| 3 | Campagnes, localités, trésoreries, comptes des PJ, transferts, historique, interface | fait |
| 4 | Bâtiments, étages, salles, construction (limite de dépense, accélération, achat, améliorations, modèles) | fait, extensions de salle comprises (GC p. 108) |
| 5 | Organisations, équipes (appartenance, lieu de travail, recrutement, délocalisation), gestionnaires et salaires | fait |
| 6 | Tests de capitaux (recettes), absence, usure des capitaux et des entreprises | fait |
| 7 | Simulation, export, tableau de bord, vente, abandon et réparation | fait |

## Structure

```
crates/domaine   types purs : capitaux, monnaie (en pa), sources, statuts, fiches
crates/regles    référentiel officiel, personnalisations, calculs, contrôles
crates/campagne  journal d'événements, état, annuler/rétablir, calendrier, sauvegarde JSON
crates/vues      modèles d'affichage : ce que l'interface montre, calculé par le cœur
desktop/         nouvelle application (Tauri) : src-tauri/ (Rust), ui/ (React), Windows et Linux
app/             application egui (ancienne interface), conservée jusqu'à la fin de la migration
data/            referentiel_officiel.json : source de vérité des données officielles
outils/extraction  scripts de relevé depuis le PDF et de génération du code Rust
```

Le cœur ne dépend que de serde (optionnel). L'application `app/` dépend d'eframe ;
GitHub Actions compile l'exécutable Windows à chaque envoi (artefact `gestion-pathfinder-windows`).
Les campagnes sont enregistrées dans `%APPDATA%\GestionPathfinder\campagnes` (JSON, copie `.bak`).

## Commandes

```
cargo test                                            # 54 tests, dont les exemples chiffrés du livre
cargo run -p regles --example verifier_referentiel    # contenu + avertissements
```

## Données

- 78 salles, 3 extensions, 22 équipes, 17 gestionnaires, 59 modèles de bâtiments,
  14 modèles d'organisations, 8 catégories de communauté, valeurs des 4 capitaux.
- Chaque fiche porte sa source (livre, page imprimée, section) et un éventuel point à confirmer.
- Les descriptions narratives du livre ne sont pas recopiées : la page citée permet de les consulter.
- Contrôles au chargement : prix imprimé = somme des coûts d'achat, total d'un modèle = somme
  de ses éléments, liens d'amélioration réciproques. Seul écart restant : Labyrinthe (PC-07, validé).

### Régénérer les données

```
cd outils/extraction
python3 -I extract.py <guide.pdf> lines.json 91 120
python3 -I parse.py lines.json entries.json
python3 -I normalize.py entries.json ref.json anomalies.json
python3 -I build_data.py ref.json ../../data/referentiel_officiel.json
python3 -I gen_rust.py ../../data/referentiel_officiel.json ../../crates/regles/src/donnees_officielles.rs
```

(`build_data.py` contient les corrections PC-08/PC-09 et la liste des points à confirmer.)

## Statuts

- **Officiel** : écrit dans la source citée.
- **Personnalisé** : défini par l'utilisateur ; surcharge sans jamais écraser l'officiel.
- **À confirmer** : lié à un point ouvert ; appliqué avec un avertissement désactivable.

Règles personnalisées actives par défaut (`regles::options::OptionsRegles`) : répartition
stricte des bonus (un type par unité et par jour), construction parallèle autorisée,
salaire d'un gestionnaire compté une fois par jour.

Pathfinder est une marque de Paizo Inc. ; la VF est publiée par Black Book Editions.
Ce projet est un outil personnel non officiel.
