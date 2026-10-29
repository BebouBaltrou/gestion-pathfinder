# Points à confirmer — fiches de personnage (phase 2)

Chaque point : ce que le moteur fait aujourd'hui, et ce qu'il faut te demander.

1. **Traits raciaux alternatifs** : absents du Core Rulebook (ils viennent de l'*Advanced Race
   Guide*). Non importés. Veux-tu étendre le périmètre à l'*Advanced Race Guide* plus tard ?
2. **Familier et Écriture de parchemin de l'ensorceleur** : la page Core d'Archives of Nethys
   ne liste, au niveau 1, que « lignage, tours de magie, Eschew Materials ». Rien n'est ajouté
   sans source ; à vérifier dans ton exemplaire du livre (p. 70-71).
3. **« Tableau standard »** : le Core ne contient pas de tableau fixe. Il décrit cinq méthodes :
   Standard (4d6 sans le plus bas), Classic (3d6), Heroic (2d6+6), Dice Pool (24 dés) et achat
   de points. Elles sont toutes implémentées, plus la saisie libre. Il n'y a pas de tableau fixe.
4. **Points de vie « moyenne »** : la règle de la moyenne n'est pas dans les pages relevées.
   Elle sera proposée comme règle de table (moyenne / moyenne arrondie au-dessus / maximum /
   jet), au niveau 1 toujours le maximum.
5. **Minimum 1 point de vie et 1 rang de compétence par niveau** : appliqués tels que demandés
   dans ton cahier, sans confirmation dans les pages relevées du Core.
6. **Don supplémentaire et sorts de lignage** : la table de classe est importée et exacte ;
   les effets chiffrés des pouvoirs de lignage sont conservés en texte (anglais) pour l'instant.
7. **Attaques itératives** : -5 par attaque supplémentaire (jusqu'à 4 attaques) ; à confirmer
   pour les BBA qui ne sont pas des multiples de 5.

## Phase 4 (dons et sorts) — ajouts du 24/10/2026

8. **Noms français non officiels** : tous les noms de sorts, de dons, de compétences, de traits
   raciaux et de lignages sont de ma traduction (fichiers `data/core/fr/*_noms.json`). À corriger
   d'après le Manuel des joueurs quand tu veux ; le moteur ne dépend pas des noms français.
9. **Abréviations** : durée « (D) » traduite « (R) » ; composantes V/S/M/F/DF traduites
   V/G/M/F/FD ; conversions 30 pieds = 9 m, 1 mile = 1,5 km (même règle que le livre français,
   à vérifier).
10. **Dons à prérequis non évaluables** (36 prérequis, ex. « capacité de canalisation d'énergie »,
    « arme choisie ») : signalés « à vérifier », ils ne bloquent pas le choix.
11. **Familiers** : le Core liste 21 familiers (le lignage arcanique donne un lien arcanique,
    familier ou objet lié) et le don Familier amélioré renvoie à une liste : ces listes ne sont
    pas encore importées. À faire quand on traitera les compagnons.
12. **Sorts sans ligne « jet de sauvegarde »** dans AoN : traités comme « aucun ».
13. **Sorts à plusieurs versions** (Mass, Greater, Lesser) : les niveaux d'ensorceleur de ces
    variantes ont été corrigés (l'import initial les confondait avec le sort de base).

## Phase 5 (équipement et états) — ajouts du 24/10/2026

14. *(Décidé le 23/10 : la charge est indiquée par le joueur, le poids reste indicatif.)* **Tableau de charge, ligne Force 15** : le site indique « 67-134 » (intermédiaire) puis
    « 134-200 » (lourde), ce qui se chevauche. Je prends 133 comme limite intermédiaire (coquille
    probable). Le reste du tableau 7-4 est repris tel quel (Core p. 169).
15. **Malus d'armure du bouclier** : armure et bouclier additionnent leurs malus ; la charge ne
    s'y ajoute pas (on garde le pire, texte du Core p. 169). Sans maîtrise d'une armure ou d'un
    bouclier, son malus s'applique aussi aux jets d'attaque (texte des dons de maîtrise). Le livre
    ajoute « toutes les compétences impliquant un mouvement » : je n'applique le malus qu'aux 9
    compétences à malus d'armure.
16. **Surcharge** (au-delà de la charge lourde) : je reprends la règle « titube » du Core (bonus de
    DEX perdu, 5 pieds par round) et le malus de la charge lourde, faute de valeur propre.
17. *(Décidé le 23/10 : règle stricte, division entière, 25 → 12 ; arrondi à éclaircir.)* **Vitesse divisée par deux** (épuisé, empêtré, hors de combat) : division entière.
    (30 → 15, 25 → 12). Les nains ne sont pas ralentis par l'armure ni la charge.
18. **États : seuls les effets chiffrés sans ambiguïté sont calculés** (secoué, effrayé, malade,
    fatigué, épuisé, éblouie, etc.). Ce qui est conditionnel ou vague reste en note à l'écran :
    aveuglé (« la plupart » des jets de FOR/DEX : lesquels ?), à terre, sans défense, paralysé,
    agrippé, assourdi (échec des sorts), brisé (dépend de l'objet).
19. *(À implémenter plus tard, décision du 23/10.)* **Niveaux négatifs (drain d'énergie)** : non calculés. Je n'ai pas trouvé dans les pages du
    Core relevées le détail des malus par niveau négatif ; à confirmer d'après le livre avant de
    les chiffrer.
20. *(Charge non suivie pour l'instant, décision du 23/10.)* **Pièces de monnaie** : non comptées dans la charge (la règle des 50 pièces par livre n'est
    pas dans les pages relevées). Les objets dont le catalogue ne donne pas de poids (44 objets)
    comptent pour 0 et sont signalés.
21. **Armes de jet et dégâts de FOR** : la Force s'ajoute aux dégâts des armes de corps à corps,
    des armes de jet et de la fronde, pas des arcs ni arbalètes (arcs composites à bonus de Force
    : non gérés). Arme à deux mains : 1,5 × le bonus de FOR (malus non multiplié).
22. **Noms français de l'équipement et des états** : ma traduction, non officielle (comme pour les
    sorts). Prix en pièces conservés tels quels (po, pa, pc).

## Phase 6 (campagne, intermède, pages) — ajouts du 24/10/2026

23. **Bonus de compétence dans les jets de recettes** : dans le Guide de campagne, le test de
    recettes se fait avec le bonus des salles et des équipes (d20 + bonus), pas avec une
    compétence de PJ. Je n'ai donc rien branché : dis-moi quelle compétence ou quel test tu avais
    en tête (par exemple le travail du PJ lui-même, ou le test de Prestige).
24. **Prestige calculé** : sans valeur saisie, le Prestige affiché est « niveau + modificateur de
    Charisme » (lecture du commentaire existant sur la limite de recrutement, GC p. 93). Une
    valeur saisie l'emporte toujours.
25. **Le temps de campagne et les fiches** : un jour = 14 400 rounds (6 secondes par round) ; les
    effets temporaires à durée avancent avec les jours et disparaissent à zéro. Les sorts du jour
    ne sont PAS rendus automatiquement : bouton « Repos » sur la fiche.
26. **Sauvegardes** : une campagne avec fiches reste lisible par l'ancienne application, qui
    ignore les fiches. Les anciennes sauvegardes (sans fiches) s'ouvrent sans changement.

## Objets magiques portés — ajouts du 25/10/2026

27. **Bonus magique d'une armure, d'un bouclier ou d'une arme** (lecture à valider) : de +1 à +5 au
    plus (Core, p. 468-469, à vérifier). C'est un bonus d'altération : il s'ajoute au bonus
    d'armure ou de bouclier de base, et à l'attaque et aux dégâts pour une arme. Il disparaît
    quand on retire l'objet.
28. **Pas encore géré** : l'effet d'une armure magique sur son malus d'armure, l'arme « chef-d'œuvre »
    (+1 à l'attaque), les capacités spéciales (flamboyante, tranchante…), le prix et le poids. À
    confirmer avant d'y toucher.
29. **Objets magiques portés** (anneau, cape, ceinture…) : aucune liste fournie. Tu saisis le nom,
    ce qui change, le type de bonus et la valeur ; le moteur applique le cumul du Core (deux
    bonus du même type ne s'additionnent pas, sauf esquive, circonstance et sans type). Aucune
    limite d'emplacements corporels (un anneau par doigt, une cape…) n'est contrôlée : à confirmer.

## Barbare (classe ajoutée)

30. **Table 3-2 (p. 32)** : lue dans le texte du PDF (extraction brute, page 34 du fichier). Valeurs
    vérifiées : BBA, jets et aptitudes aux niveaux 1, 6, 11 et 20. À relire contre le livre.
31. **Rage** : nombre de rounds par jour = 4 + Constitution, +2 par niveau après le premier
    (formule dans `data/core/barbare.json`). Le moteur compte les rounds comme des utilisations
    du jour ; le repos les renouvelle.
32. *(Rage faite le 29/10 : +4/+4/+2 et −2 CA, +6/+3 au niveau 11, +8/+4 au niveau 20, fatigué en sortant sauf au niveau 17 ; durée de la fatigue non suivie.)* **Non calculé pour l'instant** : bonus de rage (+4 Force et Constitution, +2 Volonté, −2 CA,
    état temporaire), sens des pièges, réduction de dégâts, déplacement accéléré (conditionnel à
    l'armure), esquive instinctive. Le texte des aptitudes est dans `special_en` par niveau.
33. *(Fait le 29/10.)* **Choix des pouvoirs de rage** : un au niveau 2, puis tous les deux niveaux ; niveau, prérequis, doublons et groupe unique (totémique, sang) vérifiés. Pouvoirs de base et APG (icône MJRA) seuls affichés ; UC, ACG et Animal Archive masqués. À vérifier : Percussion (APG d’après le wiki) et Précision étonnante (sans source).

## Barde (classe ajoutée)

34. **Tables 3-3 (p. 38) et 3-4 (p. 39)** : lues dans le texte brut du PDF. Valeurs vérifiées aux niveaux 1, 11 et 20. À relire contre le livre.
35. **Représentation bardique** : rounds par jour = 4 + Charisme, +2 par niveau après le premier (même formule que la rage du barbare).
36. **Non calculé pour l'instant** : savoir bardique (moitié du niveau aux Connaissances), inspiration (vaillante, talentueuse, héroïque, glorieuse), maître du savoir, versatilité artistique, et les effets de représentation (contre-chant, distraction, fascination, etc.). Le texte est dans `special_en` par niveau.
37. **Compétence « Connaissances (toutes) »** : la compétence de classe couvre toutes les Connaissances (règle ajoutée au moteur, valable pour toute classe qui la liste).

## Guerrier (classe ajoutée)

38. **Table 3-9 (p. 55)** : valeurs vérifiées aux niveaux 1, 11 et 20. À relire contre le livre.
39. **Dons supplémentaires** (niveaux 1, 4, 6, 8…, 20) : comptés comme un don de plus au niveau de classe concerné. **À confirmer** : la liste de dons parmi laquelle le guerrier les choisit n'est pas restreinte (le livre renvoie aux dons de combat et de guerrier) ; le moteur accepte tout don du catalogue.
40. **Non calculé pour l'instant** : courage (+1 à +5 contre la terreur, bonus conditionnel), entraînement aux armures et aux armes (malus d'armure réduit, Dex maximale, groupes d'armes à choisir), maîtrise des armures et des armes. Le texte est dans `special_en` par niveau.

## Moine (classe ajoutée)

41. **Table 3-11 (p. 65)** : BBA et jets vérifiés aux niveaux 1, 8 et 20. Dégâts à mains nues, bonus à la CA, déplacement accéléré et colonnes de ki non saisis (texte seulement). À relire contre le livre.
42. **Dons supplémentaires** (niveaux 1, 2, 6, 10, 14, 18) : comptés en plus. **À confirmer** : la liste de dons du moine (p. 64, complétée au niveau 6) n'est pas appliquée ; le moteur accepte tout don du catalogue.
43. **Non calculé pour l'instant** : réserve de ki (points selon le niveau et la Sagesse : à confirmer, la moitié du niveau arrondie), déluge de coups, attaques à mains nues, bonus de Sagesse à la CA sans armure, chute ralentie, pureté physique, et autres pouvoirs. Le texte est dans `special_en` par niveau.

## Paladin (classe ajoutée)

44. **Table 3-12 (p. 68)** : valeurs vérifiées aux niveaux 1, 4, 11 et 20 (sorts par jour : la colonne du 1er niveau est 0 au niveau 4, sorts de sagesse seuls). À relire contre le livre.
45. **Châtiment du mal** : 1 par jour, +1 tous les 3 niveaux (formule du moteur, valeurs de la table). **Imposition des mains** : moitié du niveau (arrondi inférieur) + Charisme, à partir du niveau 2 ; à confirmer avec le texte (« un nombre de fois par jour égal à la moitié de son niveau + Charisme »).
46. **Non calculé pour l'instant** : grâces (choix à faire), aura du bien, canalisation d'énergie positive (niveau 4), santé divine, pacte divin, champion sacré, et les effets de soins. Le texte est dans `special_en` par niveau.

## Rôdeur (classe ajoutée)

47. **Table 3-15 (p. 82)** : valeurs vérifiées aux niveaux 1, 11 et 20. À relire contre le livre.
48. **Dons de style de combat** (niveaux 2, 6, 10, 14, 18) : comptés en plus, sans restriction de liste (le moteur accepte tout don du catalogue). À confirmer : la liste des dons de style de combat.
49. **Non calculé pour l'instant** : ennemis jurés et environnements de prédilection (bonus contre le type choisi), pistage, empathie sauvage, pacte du chasseur, proie du chasseur, camouflage et discrétion totale, maître chasseur. Le texte est dans `special_en` par niveau.

## Roublard (classe ajoutée)

50. **Table 3-16 (p. 85)** : valeurs vérifiées aux niveaux 1, 11 et 20. À relire contre le livre.
51. **Non calculé pour l'instant** : attaque sournoise (+1d6 au niveau 1, +1d6 tous les deux niveaux, le nombre de dés suit le niveau), recherche de pièges, sens des pièges, esquive instinctive et totale, talents de roublard et de maître roublard (choix à faire), coup de maître. Le texte est dans `special_en` par niveau.

## Magicien (classe ajoutée)

52. **Table 3-10 (p. 60)** : lue dans le texte brut du PDF, colonnes lues par programme ; valeurs vérifiées aux niveaux 1, 11 et 20. À relire contre le livre.
53. **Grimoire** : le nombre de sorts du grimoire (et les sorts de départ) n'est pas encore saisi ; la liste de sorts connus du magicien est vide. Le magicien prépare chaque jour selon les sorts par jour de la table.
54. **Non calculé pour l'instant** : école de magie (choix et interdictions), écriture de parchemins, pacte magique (familier), et les effets des sorts. Le texte est dans `special_en`.
55. **Dons supplémentaires** (niveaux 5, 10, 15, 20) : comptés en plus, sans restriction de liste.
