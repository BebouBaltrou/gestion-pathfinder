import { ReactNode, useEffect, useLayoutEffect, useRef, useState } from "react";

/**
 * Liste (ou grille) à fenêtrage : seules les lignes visibles sont dessinées, pour rester fluide avec
 * plusieurs centaines d'objets. `largeurCase` > 0 : plusieurs cases par ligne selon la largeur.
 */
export function ListeVirtuelle<T>(props: {
  elements: T[];
  hauteurLigne: number;
  largeurCase?: number;
  cle: (e: T) => string | number;
  rendu: (e: T) => ReactNode;
  vide?: ReactNode;
}) {
  const { elements, hauteurLigne, largeurCase = 0, cle, rendu, vide } = props;
  const conteneur = useRef<HTMLDivElement>(null);
  const [haut, setHaut] = useState(0);
  const [taille, setTaille] = useState({ largeur: 600, hauteur: 600 });

  useLayoutEffect(() => {
    const c = conteneur.current;
    if (!c) return;
    const mesurer = () => setTaille({ largeur: c.clientWidth, hauteur: c.clientHeight });
    mesurer();
    const o = new ResizeObserver(mesurer);
    o.observe(c);
    return () => o.disconnect();
  }, []);

  useEffect(() => {
    if (conteneur.current) conteneur.current.scrollTop = 0;
    setHaut(0);
  }, [elements.length === 0]);

  const colonnes = largeurCase > 0 ? Math.max(1, Math.floor(taille.largeur / largeurCase)) : 1;
  const lignes = Math.ceil(elements.length / colonnes);
  const premiere = Math.max(0, Math.floor(haut / hauteurLigne) - 3);
  const derniere = Math.min(lignes, Math.ceil((haut + taille.hauteur) / hauteurLigne) + 3);
  const visibles: ReactNode[] = [];
  for (let l = premiere; l < derniere; l++) {
    const cases = elements.slice(l * colonnes, (l + 1) * colonnes);
    visibles.push(
      <div
        key={l}
        className={largeurCase > 0 ? "ligne-virtuelle grille" : "ligne-virtuelle"}
        style={{
          top: l * hauteurLigne,
          height: hauteurLigne,
          gridTemplateColumns: largeurCase > 0 ? `repeat(${colonnes}, 1fr)` : undefined,
        }}
      >
        {cases.map((e) => (
          <div key={cle(e)} className="case-virtuelle">
            {rendu(e)}
          </div>
        ))}
      </div>,
    );
  }

  return (
    <div className="liste-virtuelle" ref={conteneur} onScroll={(e) => setHaut(e.currentTarget.scrollTop)}>
      {elements.length === 0 && <div className="vide-virtuel">{vide ?? "Rien à afficher."}</div>}
      <div style={{ height: lignes * hauteurLigne, position: "relative" }}>{visibles}</div>
    </div>
  );
}
