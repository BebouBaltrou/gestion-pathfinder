# Gestion d'intermède — Pathfinder 1 (Guide de campagne / Ultimate Campaign)

Application Windows locale de gestion des localités, bâtiments, salles, organisations,
équipes et gestionnaires selon les règles d'intermède du **Guide de campagne**
(VF de *Ultimate Campaign*).

## État : étape 2 — référentiel et moteur de règles

| Étape | Contenu | État |
| --- | --- | --- |
| 1 | Document des règles et points à confirmer | fait |
| 2 | Référentiel officiel + moteur de calcul (ce dépôt) | fait |
| 3 | Localités, trésorerie, compte des PJ, historique (persistance) | à venir |
| 4 | Bâtiments, étages, salles, construction | à venir |
| 5 | Organisations, équipes, gestionnaires | à venir |
| 6 | Tests de capital et génération de capitaux | à venir |
| 7 | Simulation, export, tableau de bord | à venir |

## Structure

```
crates/domaine   types purs : capitaux, monnaie (en pa), sources, statuts, fiches
crates/regles    référentiel officiel, personnalisations, calculs, contrôles
data/            referentiel_officiel.json : source de vérité des données officielles
outils/extraction  scripts de relevé depuis le PDF et de génération du code Rust
```

Le cœur n'a **aucune dépendance externe** : il compile hors ligne. L'interface
(egui/eframe) et la persistance (serde, SQLite) seront ajoutées dans un crate `app`.

## Commandes

```
cargo test                                            # 17 tests, dont les exemples chiffrés du livre
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
