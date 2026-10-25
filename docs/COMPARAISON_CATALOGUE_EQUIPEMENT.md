# Comparaison des deux catalogues d'équipement (25/10/2026)

Catalogue de la v1.0 (`data/core/*.json`, relevé sur Archives of Nethys, noms français non officiels)
contre le catalogue transcrit du Manuel des joueurs (`data/catalogue/core/*.txt`, noms officiels).

- Armes : 87 de chaque côté. 61 ont le même nom ; 15 de plus se rapprochent sans ambiguïté par le prix
  et le poids (liste ci-dessous, « nom du Manuel des joueurs ↔ nom de la v1.0 »). Tous les prix et poids
  rapprochés sont identiques, sauf « Billes de fronde (10) » des armes exotiques (1 po dans la transcription, 1 pa dans la v1.0 et pour « Billes (10) » des armes courantes) : à vérifier dans le livre.
- Armures et boucliers : 21 de chaque côté. 9 ont le même nom ; 12 se rapprochent par le prix et le poids.
  Tous les prix et poids sont identiques.
- Matériel : 124 objets du Manuel contre 161 dans la v1.0 (qui contient aussi des services, des animaux,
  des véhicules). 29 ont le même nom, les autres sont nommés différemment (« Acide » et « Acide (flasque) »).
- La v1.0 écrit les poids en livres (1 lb = 500 g) et garde, pour chaque objet, la description anglaise, la
  catégorie et la page du Core.

Rapprochements par le prix et le poids :

### armes (15)
- Attaque à mains nues ↔ Coup de poing
- Dague coup-de-poing ↔ Dague de poing
- Épieu ↔ Lance courte
- Morgenstern ↔ Étoile du matin
- Dard ↔ Fléchette
- Javeline ↔ Javelot
- Dard, sarbacane (10) ↔ Fléchettes de sarbacane (10)
- Lamétoile ↔ Couteau étoilé
- Fléau d’armes léger ↔ Fléau léger
- Fléau d’armes lourd ↔ Fléau lourd
- Halllebarde ↔ Hallebarde
- Double-lame ↔ Épée à deux lames
- Lame elfique incurvée ↔ Lame courbe elfique
- Marteau-piolet gnome ↔ Marteau à crochet gnome
- Bâton de jet halfelin ↔ Bâton-fronde halfelin
### armures (12)
- Armure de cuir cloutée ↔ Cuir clouté
- Armure de peau ↔ Armure de peaux
- Clibanion ↔ Armure de lamelles
- Crevice ↔ Cotte de bandes
- Armure de plaques ↔ Demi-plate
- Rondache en bois ↔ Petit bouclier en bois
- Rondache en acier ↔ Petit bouclier en acier
- Écu en bois ↔ Grand bouclier en bois
- Écu en acier ↔ Grand bouclier en acier
- Gantelet d’armes ↔ Gantelet verrouillé
- Pointes pour armure ↔ Pointes d'armure
- Pointes pour bouclier ↔ Pointes de bouclier

## Décision d'Antoine (25/10/2026) : fusion

La v1.0 (`data/core/`) est la source unique. Ses noms ont été remplacés par ceux de la transcription du
Manuel des joueurs en cas de conflit (17 noms dans `data/core/fr/equipement_noms.json`, puis
`node outils/fusionner_traductions_equipement.js data/core` pour réécrire les noms dans les données).
Deux fautes d'impression du livre suivent la règle déjà en place (`docs/COMPARAISON_MANUEL_DES_JOUEURS.md`) :
nom corrigé (« Hallebarde », « Clepsydre ») et note « erreur du livre » dans `data/core/fr/notes_livre.json`,
qui accompagne l'objet dans l'inventaire (propriété `note_livre`). La note de l'objet « Outils de cambrioleur
de qualité supérieure » a été retirée : son nom suit maintenant le libellé du livre. La transcription en texte
a été supprimée ; l'inventaire construit ses modèles depuis `data/core/` (`campagne::catalogue_inventaire`).
