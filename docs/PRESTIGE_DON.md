# Le don Prestige, la limite de recrutement et le compagnon

Sources : don « Prestige » (VF du don *Leadership* du Core, p. 129) sur
https://www.pathfinder-fr.org/Wiki/Pathfinder-RPG.Prestige.ashx ; Guide de campagne p. 93 (limite de
recrutement) et p. 108-110 (niveau des employés). Les données du tableau sont déjà dans
`data/core/dons.json` (anglais) et `data/core/fr/dons_04.json` (français, sous « Leadership »).

## 1. Fait : la limite de recrutement (GC p. 93)

« Votre personnage peut uniquement recruter des équipes composées d'employés dont le niveau de
personnage est inférieur ou égal à sa valeur de Prestige. » (Prestige = niveau + modificateur de
Charisme, ou la valeur saisie sur le personnage.)

- Le niveau des employés de chaque équipe est dans `crates/regles/src/niveaux_equipes.rs`.
- Le contrôle est fait par le cœur (`Etat::verifier_prestige`) au **lancement du recrutement**, à
  l'**achat** d'une équipe et à son **amélioration** (l'équipe obtenue doit respecter la limite).
- L'écran des équipes affiche « employés de niveau N (Prestige requis) ».

### Lectures (validées par le joueur le 24/10/2026)

1. **Quel personnage recrute ?** Le livre dit « votre personnage ». L'application ne demande pas
   encore qui recrute : on prend le **meilleur Prestige parmi les PJ présents** dans la localité.
2. **Niveau des employés** : le livre dit « généralement » ou « typiquement ». On prend ce niveau.
   Trois cas sont douteux : *artisans* (4, d'après « un charpentier typique »), *bureaucrates*
   (« expert de niveau 1, 2 ou 3 » : on retient 3), *marins* (2, « un marin typique »).
   *Fraudeurs* : le texte ne donne aucun niveau, donc aucune limite.
3. **Sans Prestige connu** (ni fiche ni valeur saisie), rien n'est bloqué.
4. **Amélioration d'une équipe** : la limite s'applique à l'équipe obtenue (le livre ne le dit pas).

## 2. Proposition : le don Prestige (à valider avant de coder)

### Ce que dit le don
- Condition : personnage de **niveau 7**.
- **Score de Prestige du don** = niveau du personnage + modificateur de Charisme, modifié par la
  réputation et des facteurs listés dans le don (grande renommée +2, juste et généreux +1, pouvoir
  spécial +1, échecs −1, arrogance −1, cruauté −2 ; pour l'acolyte : familier/monture/compagnon
  animal −2, alignement différent −1, mort d'un acolyte −2 par acolyte ; pour les partisans :
  forteresse ou base +2, déplacements fréquents −1, mort de partisans −1).
- Le **tableau** donne, selon le score, le **niveau maximal de l'acolyte** (compagnon d'armes) et le
  **nombre de partisans par niveau** (1er à 6e). L'acolyte doit avoir au moins **deux niveaux de
  moins** que le personnage.
- Un acolyte ne compte pas dans le groupe pour l'XP : il gagne `XP du personnage × (niveau de
  l'acolyte ÷ niveau du personnage)`, et ne peut jamais atteindre un niveau égal ou supérieur à
  « niveau du personnage − 1 » (son total d'XP s'arrête à 1 de moins que le niveau suivant).
- Les partisans ne gagnent ni XP ni niveaux ; on relit le tableau à chaque montée de niveau du
  personnage.

### Comment l'intégrer
1. **Le don** : quand une fiche a le don, une section « Prestige (don) » apparaît sur sa fiche. On y
   coche les modificateurs de réputation (et les facteurs acolyte/partisans). L'application calcule
   le score, lit le tableau, et affiche : niveau maximal de l'acolyte, partisans possibles par niveau,
   ce qui est déjà recruté, ce qu'il reste.
2. **L'acolyte** : un bouton « Recruter un acolyte » crée un **vrai personnage avec sa propre fiche**,
   rattaché à son maître. Création par le même assistant (niveau choisi de 1 à 20, borné par le
   tableau et par « niveau du maître − 2 »). L'acolyte apparaît dans la liste des personnages (avec
   une étiquette « acolyte de … ») et peut être mis dans l'aventure et le combat.
3. **L'XP de l'acolyte** : automatique. Chaque fois que le maître reçoit de l'XP (récompenses de fin
   de combat ou ajout manuel), l'acolyte reçoit la part prévue par le don, plafonnée comme le don le
   demande. L'événement reste annulable d'un bloc avec le reste. Le joueur peut toujours corriger.
4. **Les partisans** : de simples entrées (nom, niveau, notes), **sans fiche**, bornées par le
   tableau, avec une alerte quand une montée de niveau du maître ouvre de nouvelles places.
5. **Si le score baisse** (réputation, mort d'un acolyte…) : l'application avertit mais ne retire rien
   toute seule.

### Points à confirmer avec toi
- A. Un acolyte compte-t-il dans l'aventure/le combat comme un PJ (initiative, tours) ? Je propose oui,
  en tant que personnage joué par le joueur du maître.
- B. L'acolyte peut-il prendre son propre don Prestige ? Le livre ne dit rien : je propose non.
- C. Partisans sans fiche : suffit-il d'un nombre par niveau (par exemple « 5 partisans de niveau 1 »),
  ou veux-tu les nommer un par un ? Je propose les deux : compteur par niveau, noms facultatifs.
- D. Quand le maître gagne de l'XP, l'acolyte doit-il être présent/actif ? Le livre ne le demande pas :
  je propose de toujours le donner.

## 3. Fait (décisions du joueur : A oui, B non, C oui, D oui)

- Code : `crates/campagne/src/prestige.rs` (règles, tableau, modificateurs, XP partagé), événements
  `PrestigeDonRegle`, `AcolyteLie`, `AcolyteDelie`, `PartisanAjoute`, `PartisanRetire` ; vue
  `crates/vues/src/prestige_don.rs` ; onglet « Prestige » de la fiche (visible avec le don).
- Recruter un acolyte crée le personnage et le lie à son maître en un seul bloc annulable ; sa fiche
  se crée ensuite avec le même assistant (niveau limité par le tableau et par « maître − 2 »).
- L'acolyte est un personnage comme les autres : il entre dans l'aventure et le combat (A).
- Un acolyte ne peut pas avoir le don Prestige (B). Un seul acolyte par maître.
- Partisans : effectifs par niveau, bornés par le tableau, noms facultatifs (C).
- L'acolyte reçoit sa part d'XP à chaque gain du maître, présent ou non (D).

### Lectures (validées par le joueur le 24/10/2026)
1. Part d'XP de l'acolyte arrondie vers le bas ; sans effet en mode jalons (pas de points d'XP).
2. Les facteurs de réputation se cumulent (le texte les liste sans dire s'ils s'excluent).
3. Seule la « mort d'un acolyte » est comptée par acolyte ; les autres modificateurs valent une fois.
4. Si le score baisse, rien n'est retiré automatiquement : seul l'ajout est refusé au-delà du tableau.
5. Le niveau de l'acolyte ne dépasse jamais « niveau du maître − 2 » (texte du don) ; son XP s'arrête
   à 1 de moins que le niveau « maître − 1 ».
