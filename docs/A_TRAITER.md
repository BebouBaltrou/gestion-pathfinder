# Sujets à traiter

Questions mises de côté et décisions prises pour les étapes suivantes.
Ce fichier est tenu à jour à chaque étape.

## Mis en attente

- Fonctionnalités liées aux fiches de personnage (limite par le Prestige, travail du PJ,
  promotion d'entreprise) : voir `docs/FICHES_PERSONNAGE.md`.

## Migration vers la nouvelle application (Tauri, `desktop/`)

Fait : coquille (navigation, date, annuler/rétablir, simulation, notifications), tableau de
bord, campagnes, localités, personnages, bâtiments ; compilation Windows (.exe) avec essai de bout en bout (Linux en pause
jusqu'à ce que l'application soit stable et terminée).
À migrer, dans l'ordre :
Recettes, Organisations, Gestionnaires, Historique, Paramètres, Référentiel, Points à
confirmer, import/export, modèles personnels d'organisations (ceux des bâtiments sont faits). Puis retrait de l'ancienne application (`app/`).

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
