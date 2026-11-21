# Véhicules et montures : sources, lectures à valider, points à confirmer

Page « Biens » : véhicules (bateaux, chariots…) et montures, achetés et attribués comme des biens de
campagne, avec leur inventaire. Code : `crates/campagne/src/biens.rs`, `crates/vues/src/biens.rs`,
`crates/regles/src/montures.rs` ; données : `data/core/vehicules_manuel.json`,
`data/core/ameliorations_navires.json`.

## D'où viennent les données

| Donnée | Source |
|---|---|
| Moyens de transport : passagers, équipage, cargaison, vitesse | Manuel des joueurs (VF), p. 163, table 7-9 |
| Vitesse et charge des montures (cheval léger et lourd, poney, chien de selle) | Manuel, table 7-9 |
| Barde : prix, poids, vitesse | Manuel, p. 162 et tables 6-6 et 6-8 |
| Table de charge des personnages | Manuel, table 7-4 (p. 171) |
| Prix des véhicules et des montures | Catalogue d'équipement (livre anglais). Les chiffres lisibles du Manuel concordent ; la table 6-9 du PDF est décalée. |
| Améliorations de bateau | **Hors Manuel** : Ultimate Equipment, d'après le wiki fourni par Antoine (21/11/2026) |
| Taille, cases, rames et cases de voile des bateaux | **Hors Manuel** : même wiki, pour le coût de certaines améliorations |
| Attributions (propriétaire, confiée à, équipes, gestionnaires, équipage, suite du don Prestige) | **Règles personnalisées** : le Manuel n'en dit rien |

Quand le Manuel et le wiki se contredisent (ex. cargaison du drakkar : 50 t au Manuel, 500 t au wiki),
le Manuel l'emporte.

## Lectures à valider

1. **Charge d'une monture.** Les parenthèses de la table 7-9 sont les bornes des charges intermédiaire
   et lourde des quadrupèdes (confirmé par Antoine). Légère : jusqu'à la borne basse. La limite entre
   intermédiaire et lourde est posée aux deux tiers de la borne haute, comme la table 7-4 (un tiers,
   deux tiers, tout). Cheval léger : 87,5 / 175 / 262,5 kg.
2. **Vitesse de base d'une monture** (pour la barde) : les trois colonnes 12, 15 et 18 m de la table 6-8
   sont lues comme poney et chien de selle (12), cheval lourd (15), cheval léger (18).
3. **Au-delà de la charge lourde** : la table ne dit rien. Le dépassement est permis et signalé ; la
   vitesse n'est pas calculée.
4. **Poids du cavalier** : non compté dans la charge de la monture (pas de fiche pour lui). Les objets
   de l'inventaire de la monture (sacoches, fontes…) et sa barde comptent.
5. **Barde** : prix ×2 (taille M) ou ×4 (taille G), poids ×1 (M) ou ×2 (G) par rapport à l'armure.
   Chevaux = taille G, poney et chien de selle = taille M. Le poids de la barde compte dans la charge.
   La vitesse baisse avec une barde intermédiaire ou lourde (table 6-8). Les règles « cinq fois plus de
   temps pour la mettre » et « une monture en armure ne porte que son cavalier et ses fontes » sont
   affichées, pas imposées.
6. **Cargaison d'un bateau** : borne haute de la fourchette du Manuel (« 40 à 50 tonnes » : 50 t).
   Passagers : le nombre de passagers ou de soldats du Manuel.
7. **Améliorations de bateau** : les pourcentages (cargaison, passagers, vitesse) s'additionnent ; le
   prix de base est celui du catalogue ; les prérequis (Artisanat, DD) sont affichés mais pas vérifiés ;
   un échec au jet (moitié du coût perdue) n'est pas suivi ; « à installer à la construction » (coque
   effilée, quille allongée) est affiché mais pas imposé.
8. **Vitesse ajustée** par une amélioration : appliquée à la vitesse de déplacement du Manuel. La
   vitesse tactique en combat bord à bord n'est pas modélisée.

## Suite du don Prestige (partisans)

Le don ne dit rien de la production des partisans. Choix d'Antoine (21/11/2026) :
- un partisan peut avoir une **équipe équivalente** du livre ; il rejoint l'organisation « Suite de
  [maître] », se place comme une équipe et reste étiqueté « suite de [maître] » ;
- il **produit** une personne sur la taille de l'équipe équivalente (ex. archers, 5 personnes : 1/5 du
  bonus, arrondi à l'inférieur, comme les autres divisions) ;
- il compte pour **une seule personne** dans les effectifs, comptés à part des autres ;
- sans équipe équivalente : un effectif avec une note ;
- seuls les partisans sont concernés, pas l'acolyte.

**À confirmer** : valeur du partisan dans le patrimoine (zéro pour l'instant, le don ne lui donne pas de
prix) ; arrondi de la production (à l'inférieur : 1/5 de +6 donne +1).

## Reporté ou retiré

- **Entretien** des véhicules : reporté (pas dans le Manuel).
- **Marchands, boutiques, taux de vente régionaux** : retirés du périmètre à la demande d'Antoine.
- **Prix** : chien de selle sans prix au catalogue (à saisir à l'achat) ; Carriole et Carrosse : le
  catalogue nomme « Carriole » l'objet que le texte du Manuel décrit comme « Carrosse » (à vérifier).
- **Cargaison du chariot, de la charrette et du traîneau** : le Manuel n'en donne pas.

## Inventaires

Chaque véhicule et chaque monture achetés ont un inventaire (cargaison, sacoches), créé à l'achat et
supprimé avec le bien. Pour un bien acheté avant cette fonction, le bouton « Créer l'inventaire » le
crée. L'inventaire n'a pas de limite de poids : la charge est suivie par la jauge de la page Biens.
