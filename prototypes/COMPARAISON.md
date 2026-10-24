# Prototype d'interface — comparaison des technologies (24/10/2026)

Même vue dans trois technologies, même feuille de style (`commun/proto.css`, jetons de couleur,
deux thèmes sombre/clair) : barre d'initiative (tri animé, tour actif), jauge de PV animée (PV,
temporaires, non létaux, traînée rouge, secousse, flash, nombre flottant), effets avec chronomètre
circulaire et alerte d'expiration, panneaux réordonnables par glisser-déposer, bouton de thème.
Le comportement est identique (vérifié par un scénario scripté : mêmes PV, mêmes rounds, mêmes alertes).

| | React 19 + motion | Svelte 5 | Solid 1.9 |
|---|---|---|---|
| Taille du code (fichier principal) | 242 lignes | **190 lignes** | 223 lignes |
| Poids envoyé (gzip) | 113 ko | 19 ko | **8,5 ko** |
| Animation de réordonnancement | `layout` (bibliothèque motion) | **intégrée** (`animate:flip`) | à écrire soi-même (FLIP, ~20 lignes) |
| Entrées / sorties animées | `AnimatePresence` | **intégrées** (`in:`/`out:`) | à écrire (ou bibliothèque tierce) |
| Glisser-déposer | natif HTML5 (même code) ; bibliothèques riches (dnd-kit) | natif ; svelte-dnd-action | natif ; peu de bibliothèques |
| Écosystème (composants, aide, exemples) | **le plus vaste** | moyen | petit |
| Cohérence avec l'application actuelle (React) | **identique** | à réécrire | à réécrire |
| Intégration Tauri | oui | oui | oui |

## Ce que cela change pour toi

- Une application de bureau charge ses fichiers depuis le disque : **le poids en ko n'a aucun effet
  visible** ici. Les écarts de taille ne comptent donc pas.
- Les 5 pages existantes sont en React ; passer à Svelte ou Solid veut dire les réécrire (coût réel,
  sans gain visible pour toi sur ces pages).
- Svelte donne les animations les plus simples à écrire (réordonnancement, entrée/sortie intégrés),
  mais React + motion produit **le même rendu** ici.

## Leptos et Dioxus (non prototypés)

Ils permettraient d'écrire l'interface en Rust, mais : écosystème d'animation et de glisser-déposer
très mince (tout serait à écrire à la main), chaîne de compilation plus lourde (WebAssembly), et
réécriture complète des pages existantes. Je ne les prototype pas sauf demande de ta part.

## Recommandation

**Rester en React, ajouter la bibliothèque `motion`** pour les animations et garder le glisser-déposer
natif (ou dnd-kit si on veut plus de finesse). Même rendu qu'avec Svelte, aucune réécriture, et la
logique reste 100 % en Rust, l'interface ne faisant qu'afficher. Si tu préfères la simplicité d'écriture
de Svelte et acceptes de réécrire les pages existantes, c'est le deuxième choix. Décision à toi ;
aucune migration n'est faite.

## Lancer les prototypes

Double-clic sur `prototypes\lancer.bat` (installe puis ouvre les trois dans ton navigateur : React,
Svelte et Solid sur les ports 5171, 5172, 5173). Pour la vraie fenêtre, ce serait l'étape suivante.
