/** Texte en minuscules et sans accents, pour comparer une recherche. */
export const sansAccents = (s: string) => s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");

/** Capitaux non nuls, « 3 Influence, 2 Travail », ou « rien ». */
export const texteCapitaux = (c: { marchandises: number; influence: number; travail: number; magie: number }) =>
  [
    ["Marchandises", c.marchandises],
    ["Influence", c.influence],
    ["Travail", c.travail],
    ["Magie", c.magie],
  ]
    .filter(([, n]) => (n as number) > 0)
    .map(([nom, n]) => `${n} ${nom}`)
    .join(", ") || "rien";
