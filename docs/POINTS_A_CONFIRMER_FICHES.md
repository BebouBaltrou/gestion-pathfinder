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
17. *(Décidé le 23/10 : règle stricte, division entière, 25 → 12 ; arrondi à éclaircir.)* **Vitesse divisée par deux** (épuisé, empêtré, hors de combat) : arrondie à un multiple de 5
    vers le bas (30 → 15, 25 → 10). Les nains ne sont pas ralentis par l'armure ni la charge.
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
