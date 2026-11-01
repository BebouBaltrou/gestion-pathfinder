import { useEffect, useState } from "react";
import { api, NoeudOrga, Organigramme as OrganigrammeDonnees } from "../api";

const CLE = "gestion-pathfinder.personnage-joueur";

const LIBELLES: Record<string, string> = {
  joueur: "Joueur",
  localite: "Localité",
  batiment: "Bâtiment",
  organisation: "Organisation",
  gestionnaire: "Gestionnaire",
};

/** Un élément de l'arbre, avec ce qui lui est rattaché en dessous. */
function Ligne({ n }: { n: NoeudOrga }) {
  return (
    <li className={"orga-" + n.genre}>
      <span className="doux">{LIBELLES[n.genre] ?? n.genre}</span>
      <strong>{n.nom}</strong>
      {n.enfants.length > 0 && (
        <ul>
          {n.enfants.map((e) => (
            <Ligne key={e.genre + e.id} n={e} />
          ))}
        </ul>
      )}
    </li>
  );
}

function personnageEnregistre(): number | null {
  try {
    const v = window.localStorage.getItem(CLE);
    return v === null ? null : Number(v);
  } catch {
    return null;
  }
}

export function Organigramme() {
  const [donnees, setDonnees] = useState<OrganigrammeDonnees | null>(null);
  const [personnages, setPersonnages] = useState<{ id: number; nom: string }[]>([]);
  const [joueur, setJoueur] = useState<number | null>(personnageEnregistre);

  useEffect(() => {
    api.organigramme().then(setDonnees);
    api.pagePersonnages().then((p) => setPersonnages(p?.personnages ?? []));
  }, []);

  const choisir = (id: number | null) => {
    setJoueur(id);
    try {
      if (id === null) window.localStorage.removeItem(CLE);
      else window.localStorage.setItem(CLE, String(id));
    } catch {
      /* sans stockage, le choix ne dure que jusqu'à la fermeture */
    }
  };

  if (!donnees?.global) return null;
  const nomJoueur = personnages.find((p) => p.id === joueur)?.nom ?? "Choisissez votre personnage";

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Organigramme</h1>
        <p>Qui gère quoi, localité par localité. Votre personnage est au sommet.</p>
      </div>

      <div className="carte racine-orga">
        <span className="doux">Racine</span>
        <strong>{nomJoueur}</strong>
        <select
          className="champ"
          aria-label="Votre personnage"
          value={joueur ?? ""}
          onChange={(e) => choisir(e.target.value === "" ? null : Number(e.target.value))}
        >
          <option value="">— aucun —</option>
          {personnages.map((p) => (
            <option key={p.id} value={p.id}>
              {p.nom}
            </option>
          ))}
        </select>
      </div>

      <div className="grille-orga">
        {donnees.localites.map((l) => (
          <article className="carte colonne-orga" key={l.id}>
            <h2>{l.nom}</h2>
            {l.enfants.length === 0 ? (
              <p className="doux">Aucun bâtiment, organisation ni gestionnaire.</p>
            ) : (
              <ul className="arbre">
                {l.enfants.map((e) => (
                  <Ligne key={e.genre + e.id} n={e} />
                ))}
              </ul>
            )}
          </article>
        ))}
      </div>
    </div>
  );
}
