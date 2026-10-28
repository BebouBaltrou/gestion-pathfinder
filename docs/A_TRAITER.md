# Sujets à traiter

Questions mises de côté et décisions prises pour les étapes suivantes.
Ce fichier est tenu à jour à chaque étape.

## Mis en attente

- Fonctionnalités liées aux fiches de personnage (limite par le Prestige, travail du PJ,
  promotion d'entreprise) : voir `docs/FICHES_PERSONNAGE.md`.

## Migration vers la nouvelle application (Tauri, `desktop/`)

Fait : toutes les pages de l'ancienne application (coquille, tableau de bord, campagnes avec
export/import, localités, personnages, bâtiments, recettes, organisations, gestionnaires,
historique, paramètres, référentiel, points à confirmer, modèles personnels) ; compilation
Windows (.exe) avec essai de bout en bout (Linux en pause jusqu'à ce que l'application soit
stable et terminée).
Reste : essai à la main de chaque page par l'utilisateur, puis retrait de l'ancienne application
(`app/`) avec son accord.

## Décisions du 23/10/2026

- **Coût produit** des capitaux produits par les bâtiments et organisations : gratuits par
  défaut, option « faire payer le Coût produit » (GC p. 94). Travail du PJ : payant par défaut.
- **Effectifs** : tailles des équipes selon le livre (pas de réglage 50-150 %), totaux par
  organisation, bâtiment, localité et campagne.
- **Extensions** (GC p. 108) : chantiers sur les salles (fortification, ameublement +5 au test
  de la salle, piège au coût saisi), une de chaque type par salle, permanentes, conservées à
  l'amélioration ; créées avec les modèles du livre (salles repérées d'après le texte du
  modèle, ex. « deux chambres à coucher »). Validé le 23/10 : une salle sans recettes propres
  mais meublée produit un des types que le bâtiment génère déjà, avec +5.
- **Recettes** (retours du 23/10) : un seul type de revenu coché par salle ou équipe ; les
  jours en attente sont encaissés une seule fois, du plus ancien au plus récent, et un
  encaissement partiel laisse le reste en attente.
- **Vente** : la moitié du prix d'achat, en po. Sans acheteur (25 % des cas), le joueur garde
  le bien ou l'abandonne.

## Lectures appliquées à l'étape 6 (à valider à l'usage)

- **Usure des capitaux** (GC p. 82) appliquée jour par jour : −1 à chaque type en réserve au
  7e jour consécutif sans PJ (14e avec un gestionnaire payé ; aucune avec l'option
  personnalisée décidée le 22/10/2026).
- **Gestionnaire payé** : actif, et la trésorerie qui le paie n'est pas négative. Il protège
  la localité qui le paie et celles de ses bâtiments et organisations.
- **Usure des entreprises** : au retour après 30 jours ou plus sans gestionnaire payé, test de
  prestige DD = jours − 10, seulement s'il y a une salle construite ou une équipe recrutée.
  L'application note le résultat ; la perte de contrôle est laissée au MJ.
- **Réduction d'absence des recettes** (GC p. 84) : jours sans aucun PJ dans la localité
  pendant la période traitée, par tranches de 7.
- Les po des recettes sont créditées avant le paiement des capitaux gardés.

## Modèles personnels

- Enregistrés depuis un bâtiment (étages, modes, salles) ou une organisation (équipes), dans
  `%APPDATA%\GestionPathfinder\modeles_personnels.json`, communs à toutes les campagnes.
  Les salles améliorées sont enregistrées dans leur version actuelle.

## Lectures appliquées à l'étape 7 (vente, abandon, réparation)

- **Vente** : seules les parties construites ou recrutées comptent ; le bien reste au
  personnage (et continue de produire) jusqu'à la vente.
- **Réparation** (GC p. 92) : moitié du prix en po, ou moitié du coût en capitaux (arrondi
  inférieur) pris dans les réserves ; immédiate (aucune durée indiquée).

## Décisions déjà appliquées (étape 5)

- Limite de dépense partagée entre chantiers et recrutements d'une localité : les salles sont
  servies avant les équipes.
- Délocalisation équipe par équipe (1d6 par groupe, règle officielle) ; option personnalisée
  « un seul jet pour toute l'organisation ».

## Fiches de personnage : à faire plus tard (décisions du 23/10/2026)

- **Charge portée** : conversion automatique poids -> charge (tableau 7-4 déjà chargé dans
  `data/core/charge.json`), pièces de monnaie (50 par livre) et poids manquants ; pour l'instant le
  joueur indique la charge.
- **Niveaux négatifs (drain d'énergie)** : malus par niveau négatif, perte de niveau effectif,
  jets de sauvegarde pour les rendre permanents.
- Vitesse divisée par deux : arrondi à clarifier (voir `docs/POINTS_A_CONFIRMER_FICHES.md`, n° 17).

## Pages des personnages (nouvelle application) : fait et reste à faire

Fait : liste des PJ, création de fiche pas à pas, fiche complète (chiffres détaillés au clic,
compétences, dons, sorts et repos, équipement, états et effets temporaires, XP, historique),
montée de niveau, page Paramètres (registre des règles optionnelles de la campagne).
Reste : équipement de départ dans l'assistant de création, multiclassage (une seule classe
existe pour l'instant), tirage de dés intégré pour les caractéristiques et les dés de vie,
familiers, lien de la fiche avec le déplacement du PJ dans une localité, impression de la fiche.

## À élucider plus tard (décision du 23/10/2026)

Les points 23 à 26 de `docs/POINTS_A_CONFIRMER_FICHES.md` sont gardés tels quels, à trancher plus
tard : bonus de compétence dans les jets de recettes, Prestige calculé, temps de campagne et
fiches, compatibilité des sauvegardes.

- **Importation de fiche PDF** : fonctionnalité en développement, à reprendre plus tard (voir `docs/IMPORT_PDF.md`). Priorité : toutes les classes dans le créateur de fiche (Prêtre, puis Druide).

## Prêtre et Druide : ce qui reste à faire plus tard (24/10/2026)

- **Combat** : les lanceurs qui préparent leurs sorts (Prêtre, Druide) sont encore refusés par le combat (garde-fou existant).
- **Forme animale du Druide** : vitesses, attaques naturelles et aptitudes de chaque forme restent en texte (suivre le sort forme bestiale, corps élémentaire ou forme végétale) ; seuls les bonus chiffrés sont calculés.
- **Compagnon animal** : compétences, dons et tours ne sont que des nombres, sans choix ; PV en moyenne arrondie (4,5 par d8, à confirmer).
- **Druide** : conversion spontanée des sorts en « convocation d'alliés naturels » (celle du Prêtre est faite).
- **Domaines** : résistances (Air, Eau, Feu, Terre) en texte seulement ; domaines de Calistria (table et pages du livre ne concordent pas), sorts de domaine non résolus (Obscurité 8e, Runes 6e) : points à confirmer.
- **Armes et objets** : « mains nues » et « masse » à confirmer ; Bottes elfiques = Acrobaties dans le livre français à confirmer ; type du bonus de forme bestiale (taille) à confirmer.
- **Importation PDF** : domaines, divinité, alignement et pacte de nature ne sont pas lus (fonctionnalité en développement).
- **Autres classes du créateur de fiche** : Barbare fait (voir POINTS_A_CONFIRMER_FICHES n° 30-33). Restent Barde, Guerrier, Moine, Paladin, Rôdeur, Roublard, Magicien.

## Monnaie (25/10/2026)

`Montant` compte en pièces de cuivre (1 pa = 10 pc, 1 po = 100 pc, 1 pp = 1 000 pc), pour que les prix du
Manuel des joueurs (une bougie : 1 pc) soient exacts, sans arrondi. Format de fichier 3 ; l'ancien format
(`{"pa": n}`) est converti à la lecture. La vente à moitié prix garde l'arrondi à la pièce d'argent d'avant
(`ventes.rs`) : à confirmer si Antoine préfère la moitié exacte en pc.

## Inventaire et équipement : empilement des bonus (décidé par Antoine, 25/10/2026)

Le moteur d'effets existant (`crates/fiches/src/effet.rs`) est conservé tel quel :

1. Deux bonus du même type ne s'additionnent pas : on retient le plus élevé.
2. Les bonus d'esquive, de circonstance et sans type s'additionnent.
3. Les malus s'additionnent toujours. Antoine avait proposé que deux pénalités de même type ne
   s'additionnent pas, puis a répondu « si tout fonctionne, inutile de changer » : le moteur actuel reste.
4. Distinguer « armure » (type armure), « armure magique » (altération) et « armure naturelle » (naturel).

## Objets magiques : tables du chapitre 15 (25/10/2026)

Données transcrites par Antoine dans `data/core/objets_magiques/` (anneaux, sceptres, bâtons, armures, boucliers
et armes spécifiques, objets merveilleux, propriétés spéciales, améliorations, tarifs des potions, parchemins et
baguettes, estimation du prix). Les 15 objets à bonus chiffré de `data/core/objets_magiques.json` gardent leurs
effets et leur emplacement ; un objet des tables qui y figure déjà n'est pas répété. Les d100 par puissance et
les notes de bas de page n'ont pas été transmis.

**À confirmer dans le livre** (écarts ou doutes, transcrits tels quels) :

- « Bandeau de supériorité mentale (+4) » : 16 000 po dans la table 15-19, 64 000 po dans la table 15-20 et
  dans `objets_magiques.json` (16 000 po vaut le +2).
- « Épée de bonne fortune (0 souhait) » apparaît deux fois (22 060 po et 142 960 po) ; la seconde est
  probablement « 3 souhaits ».
- Sceptre « Métamagie modérée, Incantation rapide » à 24 500 po et à 75 500 po.
- « Collier à boules de feu (5e modèle) » : 6 000 po dans la table, 5 850 po dans la description.
- « Traité de perspicacité (+5) » à 135 500 po, les autres traités (+5) à 137 500 po.
- « Amulette des poings invincibles » à 100 000 po, sans « (+5) ».
- « Timbales de panique » : le texte transmis donne « panique3 0 000 po », lu comme 30 000 po.
- Paladin et rôdeur : les tables 15-12, 15-15 et 15-17 et les « Prix de base » de la fin du chapitre donnent
  des prix différents pour les potions, parchemins et baguettes. Les deux versions sont gardées.
- Emplacement des objets merveilleux : les tables ne le donnent pas ; décision d'Antoine : ils restent « sans
  emplacement » (illimités). Seuls les objets de
  `objets_magiques.json` et les anneaux peuvent être équipés dans un emplacement du corps.
## Réattribution des dons et des sorts (à confirmer)

Ajouté sur le modèle des rangs de compétence : on refait les dons (par niveau) et la liste des sorts connus.
Le cœur refuse ce qui sort des règles (nombre de dons du niveau dépassé, doublon, don hors liste du lignage,
sort inconnu, sort hors liste de la classe, trop de sorts d'un niveau). Points à confirmer :

- **Changer un don déjà pris à un niveau passé** : permis hors montée de niveau ? Le livre ne le dit pas
  (lecture à valider). Actuellement permis, comme les rangs.
- **Prérequis d'un don** : non bloquant ; un prérequis manquant reste signalé sur la fiche (comme après une montée).
- **Sorts connus** : la liste complète est refaite, sans garder le niveau où chaque sort a été appris.
- **Lignage** : la liste proposée n'est pas encore filtrée sur le lignage ; le cœur refuse le mauvais choix.
