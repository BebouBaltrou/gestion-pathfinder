import { sansAccents } from "../../texte";
import { useMemo, useState } from "react";
import { estMagique, nomDuType, type ObjetVue } from "../../inventaire-types";
import { ListeVirtuelle } from "./ListeVirtuelle";

type Ligne = { entete: string } | { objet: ObjetVue };
type Tri = "nom" | "poids" | "valeur";
type Groupe = "aucun" | "type" | "conteneur";


/** Liste filtrable et triable des objets d'un inventaire, en liste ou en grille, à fenêtrage. */
export function ListeObjets(props: {
  objets: ObjetVue[];
  /** Conteneur dont on montre le contenu direct ; `null` : tout l'inventaire. */
  conteneur: number | null;
  selection: number | null;
  onSelection: (id: number) => void;
  onGlisser: (o: ObjetVue | null) => void;
}) {
  const { objets, conteneur, selection, onSelection, onGlisser } = props;
  const [recherche, setRecherche] = useState("");
  const [type, setType] = useState("");
  const [equipe, setEquipe] = useState<"" | "oui" | "non">("");
  const [magique, setMagique] = useState<"" | "oui" | "non">("");
  const [tri, setTri] = useState<Tri>("nom");
  const [groupe, setGroupe] = useState<Groupe>("aucun");
  const [vue, setVue] = useState<"liste" | "grille">("liste");

  const types = useMemo(() => [...new Set(objets.map((o) => o.typeObjet))].sort(), [objets]);
  const nomsConteneurs = useMemo(() => new Map(objets.filter((o) => o.estConteneur).map((o) => [o.id, o.nom])), [objets]);

  const lignes = useMemo<Ligne[]>(() => {
    const mots = sansAccents(recherche).split(/\s+/).filter(Boolean);
    const filtres = objets.filter(
      (o) =>
        (conteneur === null || o.parent === conteneur) &&
        (!type || o.typeObjet === type) &&
        (equipe === "" || (equipe === "oui") === (o.equipe.length > 0)) &&
        (magique === "" || (magique === "oui") === estMagique(o.typeObjet)) &&
        mots.every((m) => sansAccents(o.nom).includes(m)),
    );
    const cle = (o: ObjetVue) =>
      tri === "poids" ? -o.poids.grammes : tri === "valeur" ? -(o.valeurUnite?.pc ?? 0) * o.quantite : 0;
    filtres.sort((a, b) => cle(a) - cle(b) || a.nom.localeCompare(b.nom, "fr"));
    if (groupe === "aucun" || vue === "grille") return filtres.map((objet) => ({ objet }));
    const etiquette = (o: ObjetVue) =>
      groupe === "type" ? nomDuType(o.typeObjet) : (o.parent !== null && nomsConteneurs.get(o.parent)) || "Sur soi";
    const groupes = new Map<string, ObjetVue[]>();
    for (const o of filtres) groupes.set(etiquette(o), [...(groupes.get(etiquette(o)) ?? []), o]);
    return [...groupes.entries()]
      .sort(([a], [b]) => a.localeCompare(b, "fr"))
      .flatMap(([nom, liste]) => [{ entete: `${nom} (${liste.length})` } as Ligne, ...liste.map((objet) => ({ objet }))]);
  }, [objets, conteneur, recherche, type, equipe, magique, tri, groupe, vue, nomsConteneurs]);

  const nombre = lignes.filter((l) => "objet" in l).length;

  const carte = (o: ObjetVue) => (
    <div
      className={"objet" + (o.id === selection ? " choisi" : "") + (vue === "grille" ? " tuile" : "")}
      draggable
      tabIndex={0}
      role="button"
      aria-pressed={o.id === selection}
      onClick={() => onSelection(o.id)}
      onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && onSelection(o.id)}
      onDragStart={(e) => {
        e.dataTransfer.setData("text/plain", String(o.id));
        e.dataTransfer.effectAllowed = "move";
        onGlisser(o);
      }}
      onDragEnd={() => onGlisser(null)}
    >
      <span className="type-objet">{nomDuType(o.typeObjet)}</span>
      <span className="nom-objet">
        {o.nom}
        {o.quantite > 1 && <em> ×{o.quantite}</em>}
      </span>
      <span className="pastilles-objet">
        {o.estConteneur && <span className="mini">Conteneur</span>}
        {o.equipe.length > 0 && <span className="mini equipe">✓ Équipé</span>}
        {o.colle && <span className="mini maudit">⚠ Collé</span>}
      </span>
      <span className="poids-objet">{o.poids.texte}</span>
      <span className="valeur-objet">{o.valeurUnite ? o.valeurUnite.texte : "—"}</span>
    </div>
  );

  return (
    <div className="liste-objets">
      <div className="barre-filtres">
        <input
          className="champ"
          type="search"
          placeholder="Rechercher un objet…"
          aria-label="Rechercher un objet"
          value={recherche}
          onChange={(e) => setRecherche(e.target.value)}
        />
        <select className="champ" aria-label="Type" value={type} onChange={(e) => setType(e.target.value)}>
          <option value="">Tous les types</option>
          {types.map((t) => (
            <option key={t} value={t}>{nomDuType(t)}</option>
          ))}
        </select>
        <select className="champ" aria-label="Équipé" value={equipe} onChange={(e) => setEquipe(e.target.value as "" | "oui" | "non")}>
          <option value="">Équipé ou non</option>
          <option value="oui">Équipés</option>
          <option value="non">Non équipés</option>
        </select>
        <select className="champ" aria-label="Magique" value={magique} onChange={(e) => setMagique(e.target.value as "" | "oui" | "non")}>
          <option value="">Magique ou non</option>
          <option value="oui">Magiques</option>
          <option value="non">Ordinaires</option>
        </select>
        <select className="champ" aria-label="Trier par" value={tri} onChange={(e) => setTri(e.target.value as Tri)}>
          <option value="nom">Tri : nom</option>
          <option value="poids">Tri : poids</option>
          <option value="valeur">Tri : valeur</option>
        </select>
        <select className="champ" aria-label="Regrouper par" value={groupe} onChange={(e) => setGroupe(e.target.value as Groupe)}>
          <option value="aucun">Sans regroupement</option>
          <option value="type">Par type</option>
          <option value="conteneur">Par conteneur</option>
        </select>
        <div className="groupe-boutons" role="group" aria-label="Affichage">
          <button className={"bouton" + (vue === "liste" ? " principal" : "")} onClick={() => setVue("liste")} aria-pressed={vue === "liste"}>Liste</button>
          <button className={"bouton" + (vue === "grille" ? " principal" : "")} onClick={() => setVue("grille")} aria-pressed={vue === "grille"}>Grille</button>
        </div>
      </div>
      <div className="compte-objets">{nombre} objet(s)</div>
      <ListeVirtuelle
        elements={lignes}
        hauteurLigne={vue === "grille" ? 96 : 44}
        largeurCase={vue === "grille" ? 190 : 0}
        cle={(l) => ("objet" in l ? l.objet.id : l.entete)}
        rendu={(l) => ("objet" in l ? carte(l.objet) : <div className="entete-groupe">{l.entete}</div>)}
        vide="Aucun objet ne correspond."
      />
    </div>
  );
}
