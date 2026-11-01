// Portrait d'un personnage : une image importée, recadrée en carré puis affichée en rond.
// L'image reste sur cet ordinateur (réduite à 160 × 160) ; elle ne fait pas partie du fichier de campagne.

import { useRef, useState } from "react";

const TAILLE = 160;
const cle = (id: number) => `gestion-pathfinder.portrait.${id}`;

function lire(id: number): string | null {
  try {
    return window.localStorage.getItem(cle(id));
  } catch {
    return null;
  }
}

// Recadre le centre de l'image en carré de TAILLE × TAILLE, au format JPEG (données compactes).
function reduire(fichier: File): Promise<string> {
  return new Promise((resoudre, rejeter) => {
    const url = URL.createObjectURL(fichier);
    const image = new Image();
    image.onload = () => {
      const cote = Math.min(image.naturalWidth, image.naturalHeight);
      const x = (image.naturalWidth - cote) / 2;
      const y = (image.naturalHeight - cote) / 2;
      const canevas = document.createElement("canvas");
      canevas.width = TAILLE;
      canevas.height = TAILLE;
      canevas.getContext("2d")?.drawImage(image, x, y, cote, cote, 0, 0, TAILLE, TAILLE);
      URL.revokeObjectURL(url);
      resoudre(canevas.toDataURL("image/jpeg", 0.85));
    };
    image.onerror = () => {
      URL.revokeObjectURL(url);
      rejeter(new Error("image illisible"));
    };
    image.src = url;
  });
}

export function Portrait({ id, nom }: { id: number; nom: string }) {
  const [image, setImage] = useState<string | null>(() => lire(id));
  const [erreur, setErreur] = useState("");
  const entree = useRef<HTMLInputElement>(null);

  const importer = async (fichier: File | undefined) => {
    if (!fichier) return;
    try {
      const donnees = await reduire(fichier);
      window.localStorage.setItem(cle(id), donnees);
      setImage(donnees);
      setErreur("");
    } catch {
      setErreur("Image non prise en charge.");
    }
  };

  const retirer = () => {
    try {
      window.localStorage.removeItem(cle(id));
    } catch {
      /* sans stockage, rien à retirer */
    }
    setImage(null);
  };

  return (
    <div className="portrait-cadre">
      <button type="button" className="portrait" onClick={() => entree.current?.click()} title="Changer le portrait" aria-label={`Portrait de ${nom} : changer l'image`}>
        {image ? <img src={image} alt="" /> : nom.charAt(0).toUpperCase()}
      </button>
      <input ref={entree} type="file" accept="image/*" hidden onChange={(e) => importer(e.target.files?.[0])} />
      {image && (
        <button type="button" className="bouton discret portrait-retirer" onClick={retirer}>
          Retirer
        </button>
      )}
      {erreur && <span className="doux">{erreur}</span>}
    </div>
  );
}
