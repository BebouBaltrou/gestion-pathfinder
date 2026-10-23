// Icônes au trait (24 × 24), dessinées pour l'application.

import type { ReactElement, ReactNode } from "react";

type P = { taille?: number };

function Icone({ taille = 18, children }: P & { children: ReactNode }) {
  return (
    <svg
      width={taille}
      height={taille}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export const icones: Record<string, (p: P) => ReactElement> = {
  tableau: (p) => (
    <Icone {...p}>
      <rect x="3" y="3" width="7" height="9" rx="1.5" />
      <rect x="14" y="3" width="7" height="5" rx="1.5" />
      <rect x="14" y="12" width="7" height="9" rx="1.5" />
      <rect x="3" y="16" width="7" height="5" rx="1.5" />
    </Icone>
  ),
  recettes: (p) => (
    <Icone {...p}>
      <path d="M12 2.5 21 7.5v9L12 21.5 3 16.5v-9z" />
      <path d="M12 2.5v19M3 7.5l9 5 9-5" />
    </Icone>
  ),
  historique: (p) => (
    <Icone {...p}>
      <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
      <path d="M3 3v5h5M12 7v5l3 2" />
    </Icone>
  ),
  localites: (p) => (
    <Icone {...p}>
      <path d="M12 21s-7-6.2-7-11.5a7 7 0 0 1 14 0C19 14.8 12 21 12 21z" />
      <circle cx="12" cy="9.5" r="2.5" />
    </Icone>
  ),
  batiments: (p) => (
    <Icone {...p}>
      <path d="M3 21h18M5 21V9l7-5 7 5v12" />
      <path d="M9 21v-6h6v6" />
    </Icone>
  ),
  organisations: (p) => (
    <Icone {...p}>
      <circle cx="9" cy="8" r="3" />
      <path d="M3 20a6 6 0 0 1 12 0" />
      <circle cx="17" cy="9" r="2.5" />
      <path d="M16 14.5a5 5 0 0 1 5 5.5" />
    </Icone>
  ),
  gestionnaires: (p) => (
    <Icone {...p}>
      <circle cx="8" cy="15" r="4" />
      <path d="M11 12 20 3M16 7l3 3M14 9l2 2" />
    </Icone>
  ),
  personnages: (p) => (
    <Icone {...p}>
      <circle cx="12" cy="8" r="4" />
      <path d="M4 21a8 8 0 0 1 16 0" />
    </Icone>
  ),
  referentiel: (p) => (
    <Icone {...p}>
      <path d="M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2z" />
      <path d="M4 19V5M9 7h6" />
    </Icone>
  ),
  points: (p) => (
    <Icone {...p}>
      <circle cx="12" cy="12" r="9" />
      <path d="M9.5 9.5a2.5 2.5 0 0 1 4.8 1c0 1.7-2.3 2-2.3 3.5M12 17.5h.01" />
    </Icone>
  ),
  parametres: (p) => (
    <Icone {...p}>
      <path d="M4 7h10M18 7h2M4 17h4M12 17h8" />
      <circle cx="16" cy="7" r="2" />
      <circle cx="10" cy="17" r="2" />
    </Icone>
  ),
  campagnes: (p) => (
    <Icone {...p}>
      <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
    </Icone>
  ),
  calendrier: (p) => (
    <Icone {...p}>
      <rect x="3" y="5" width="18" height="16" rx="2" />
      <path d="M3 10h18M8 3v4M16 3v4" />
    </Icone>
  ),
  annuler: (p) => (
    <Icone {...p}>
      <path d="M9 14 4 9l5-5" />
      <path d="M4 9h11a5 5 0 0 1 0 10h-3" />
    </Icone>
  ),
  retablir: (p) => (
    <Icone {...p}>
      <path d="m15 14 5-5-5-5" />
      <path d="M20 9H9a5 5 0 0 0 0 10h3" />
    </Icone>
  ),
  suivant: (p) => (
    <Icone {...p}>
      <path d="M5 12h14M13 6l6 6-6 6" />
    </Icone>
  ),
  fiole: (p) => (
    <Icone {...p}>
      <path d="M9 3h6M10 3v6L4.5 18.5A2 2 0 0 0 6.2 21.5h11.6a2 2 0 0 0 1.7-3L14 9V3" />
      <path d="M7 15h10" />
    </Icone>
  ),
  fermer: (p) => (
    <Icone {...p}>
      <path d="M6 6l12 12M18 6 6 18" />
    </Icone>
  ),
  plus: (p) => (
    <Icone {...p}>
      <path d="M12 5v14M5 12h14" />
    </Icone>
  ),
};
