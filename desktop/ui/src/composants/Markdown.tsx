import type { ReactNode } from "react";

/**
 * Rendu minimal et sûr du Markdown des notes (aucun HTML n'est interprété) : titres (#), listes (- ),
 * gras (**), italique (*), code (`) et liens internes `[[genre:cible|libellé]]`.
 */
export function Markdown(props: { texte: string; onLien?: (genre: string, cible: string) => void }) {
  const { texte, onLien } = props;

  const enLigne = (s: string, cle: string): ReactNode[] =>
    s.split(/(\[\[[^\]]+\]\]|\*\*[^*]+\*\*|\*[^*\s][^*]*\*|`[^`]+`)/g).map((p, i) => {
      const k = `${cle}-${i}`;
      if (p.startsWith("[[") && p.endsWith("]]")) {
        const interieur = p.slice(2, -2);
        const deuxPoints = interieur.indexOf(":");
        if (deuxPoints < 0) return p;
        const genre = interieur.slice(0, deuxPoints).trim();
        const reste = interieur.slice(deuxPoints + 1);
        const barre = reste.indexOf("|");
        const cible = (barre < 0 ? reste : reste.slice(0, barre)).trim();
        const libelle = (barre < 0 ? reste : reste.slice(barre + 1)).trim();
        if (!genre || !cible) return p;
        return (
          <button key={k} type="button" className="lien-interne" title={`${genre} : ${cible}`} onClick={() => onLien?.(genre, cible)}>
            {libelle}
          </button>
        );
      }
      if (p.startsWith("**") && p.endsWith("**") && p.length > 4) return <strong key={k}>{p.slice(2, -2)}</strong>;
      if (p.startsWith("*") && p.endsWith("*") && p.length > 2) return <em key={k}>{p.slice(1, -1)}</em>;
      if (p.startsWith("`") && p.endsWith("`") && p.length > 2) return <code key={k}>{p.slice(1, -1)}</code>;
      return p;
    });

  const blocs: ReactNode[] = [];
  let paragraphe: string[] = [];
  let liste: string[] = [];
  const vider = () => {
    if (paragraphe.length) {
      const k = `p${blocs.length}`;
      blocs.push(
        <p key={k}>
          {paragraphe.flatMap((l, i) => [...(i ? [<br key={`${k}-b${i}`} />] : []), ...enLigne(l, `${k}-${i}`)])}
        </p>,
      );
      paragraphe = [];
    }
    if (liste.length) {
      const k = `l${blocs.length}`;
      blocs.push(
        <ul key={k}>
          {liste.map((l, i) => (
            <li key={i}>{enLigne(l, `${k}-${i}`)}</li>
          ))}
        </ul>,
      );
      liste = [];
    }
  };

  for (const ligne of texte.split(/\r?\n/)) {
    const t = ligne.trimEnd();
    const titre = /^(#{1,3})\s+(.*)$/.exec(t);
    const puce = /^\s*[-*]\s+(.*)$/.exec(t);
    if (!t.trim()) {
      vider();
    } else if (titre) {
      vider();
      const k = `h${blocs.length}`;
      blocs.push(titre[1].length === 1 ? <h4 key={k}>{enLigne(titre[2], k)}</h4> : <h5 key={k}>{enLigne(titre[2], k)}</h5>);
    } else if (puce) {
      if (paragraphe.length) vider();
      liste.push(puce[1]);
    } else {
      if (liste.length) vider();
      paragraphe.push(t);
    }
  }
  vider();
  return <div className="markdown">{blocs}</div>;
}
