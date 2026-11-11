# Tests à faire : classes de prestige

Branche `claude/v1.2` (commit 6a237d6). À faire sur le PC, dans PowerShell.

## 1. Tests automatiques

```
cd C:\Users\antoine\Documents\gestion-pathfinder
cargo test -p fiches classes_prestige
```

Résultat attendu : `2 passed`.

## 2. Test dans l'application

```
cd C:\Users\antoine\Documents\gestion-pathfinder
cd desktop
npx.cmd tauri dev
```

1. Ouvrir un personnage existant (ou en créer un).
2. Cliquer sur « Monter de niveau ».
3. Ouvrir le menu « Classe ».
4. Vérifier que les dix classes de prestige apparaissent sous « Classes de prestige ».
5. Choisir **Archer-mage** pour un personnage qui n'a pas BBA +6 :
   - le panneau « Conditions d'entrée » s'affiche ;
   - les conditions manquantes sont marquées ✗ ;
   - le bouton « Valider ce niveau » est grisé.
6. Choisir une classe dont les conditions sont remplies (ou à remplir pour le test) : « Valider » doit être possible.
7. Revenir à une classe de base (Guerrier, Magicien…) : rien ne doit avoir changé.

## 3. Sorts par jour (à vérifier)

Personnage ensorceleur de niveau 5, qui prend un niveau d'archer-mage (si ses conditions sont remplies) :
- le niveau de lanceur de sorts de l'ensorceleur doit passer de 5 à 6 ;
- les sorts par jour doivent suivre ce niveau.

Point à confirmer : si le personnage a plusieurs classes lanceuses, le niveau ajouté va à la première prise dans l'ordre de la fiche.

## 4. Ce qui n'est pas encore fait

- Les conditions « spéciales » (avoir tué quelqu'un, avoir fait payer un texte…) ne sont que rappelées, jamais vérifiées.
- Les noms anglais de certaines classes sont des choix (ex. « Shadowdancer », « Loremaster ») : à valider.
- Les textes des capacités sont en français dans un champ dont le nom dit « en ».
