import { useEffect, useMemo, useRef, useState } from "react";
import type { Outils } from "../App";
import { api } from "../api";
import type { CommandeInventaire, ObjetVue, PageInventaire } from "../inventaire-types";
import "../inventaire.css";
import { AjoutRapide } from "./inventaire/AjoutRapide";
import { Equipement } from "./inventaire/Equipement";
import { FicheObjet, type Contenant } from "./inventaire/FicheObjet";
import { ListeObjets } from "./inventaire/ListeObjets";
import { prendreDemande } from "../lien-interne";

/** Inventaire et équipement des personnages : conteneurs, objets, poids, vue « RPG » de l'équipement. */
export function Inventaire({ outils }: { outils: Outils }) {
  const [page, setPage] = useState<PageInventaire | null>(null);
  const [racine, setRacine] = useState<number | null>(null);
  const [mode, setMode] = useState<"inventaire" | "equipement">("inventaire");
  const [conteneur, setConteneur] = useState<number | null>(null);
  const [selection, setSelection] = useState<number | null>(null);
  const [glisse, setGlisse] = useState<ObjetVue | null>(null);
  const [cible, setCible] = useState(0);
  const aSelectionner = useRef<number | null>(null);

  useEffect(() => {
    api.pageInventaire().then(setPage, () => setPage(null));
  }, [outils.version]);

  const inventaire = useMemo(
    () => page?.inventaires.find((i) => i.racine === racine) ?? page?.inventaires[0] ?? null,
    [page, racine],
  );

  useEffect(() => {
    setConteneur(null);
    setSelection(aSelectionner.current);
    aSelectionner.current = null;
    setCible(inventaire?.racine ?? 0);
  }, [inventaire?.racine]);

  // Un objet cité dans une note s'ouvre à l'arrivée sur la page.
  useEffect(() => {
    if (!page) return;
    const cibleLien = prendreDemande("objet");
    if (cibleLien === null) return;
    const id = Number(cibleLien);
    const inv = page.inventaires.find((i) => i.objets.some((o) => o.id === id));
    if (!inv) return;
    setMode("inventaire");
    if (inv.racine === inventaire?.racine) {
      setSelection(id);
    } else {
      aSelectionner.current = id;
      setRacine(inv.racine);
    }
  }, [page]);

  const cmd = (c: CommandeInventaire, message: string) => {
    outils.agir(api.inventaireCommande(c, message));
  };

  if (!page) {
    return (
      <div className="page">
        <div className="titre-page"><h1>Inventaire</h1></div>
        <p className="doux">Ouvrez une campagne pour gérer les objets de ses personnages.</p>
      </div>
    );
  }

  const objets = inventaire?.objets ?? [];
  const contenants: Contenant[] = inventaire
    ? [{ id: inventaire.racine, nom: "Sur soi" }, ...objets.filter((o) => o.estConteneur).map((o) => ({ id: o.id, nom: o.nom }))]
    : [];
  const choisi = objets.find((o) => o.id === selection) ?? null;

  // Les conteneurs en arbre : profondeur de chacun.
  const profondeur = (o: ObjetVue): number => {
    let d = 0;
    let p = o.parent;
    while (p !== null && p !== inventaire?.racine) {
      d++;
      p = objets.find((x) => x.id === p)?.parent ?? null;
    }
    return d;
  };
  const arbre = objets.filter((o) => o.estConteneur);

  const deposerSur = (e: React.DragEvent, vers: number) => {
    e.preventDefault();
    const id = Number(e.dataTransfer.getData("text/plain"));
    setGlisse(null);
    const o = objets.find((x) => x.id === id);
    if (o && o.parent !== vers) cmd({ Deplacer: { id, vers } }, `${o.nom} déplacé`);
  };

  return (
    <div className="page page-inventaire">
      <div className="titre-page">
        <h1>Inventaire</h1>
        <p>Les objets des personnages, leurs conteneurs, leur poids et ce qu'ils portent. Les poids, les capacités et les emplacements viennent du moteur de règles.</p>
      </div>

      <div className="onglets-proprietaires" role="tablist" aria-label="Inventaires">
        {page.inventaires.map((i) => (
          <button key={i.racine} role="tab" aria-selected={i.racine === inventaire?.racine} className={"bouton" + (i.racine === inventaire?.racine ? " principal" : "")} onClick={() => setRacine(i.racine)}>
            {i.proprietaire}
          </button>
        ))}
        {page.sansInventaire.map((p) => (
          <button key={p.id} className="bouton discret" onClick={() => outils.agir(api.creerInventairePersonnage(p.id))} title={`Créer l'inventaire de ${p.nom}`}>
            + Inventaire de {p.nom}
          </button>
        ))}
      </div>

      {!inventaire && <p className="doux">Aucun inventaire pour l'instant. Créez celui d'un personnage avec le bouton ci-dessus.</p>}

      {inventaire && (
        <>
          <div className="bilan-inventaire">
            <div><small>Poids porté</small><strong>{inventaire.poids.texte}</strong></div>
            <div><small>Valeur</small><strong>{inventaire.valeur.texte}</strong></div>
            <div><small>Objets</small><strong>{objets.length}</strong></div>
            {inventaire.charge && <div><small>Charge</small><strong>{inventaire.charge.nom}</strong></div>}
            <span className="espace" />
            <div className="groupe-boutons" role="tablist" aria-label="Vue">
              <button role="tab" aria-selected={mode === "inventaire"} className={"bouton" + (mode === "inventaire" ? " principal" : "")} onClick={() => setMode("inventaire")}>Inventaire</button>
              <button role="tab" aria-selected={mode === "equipement"} className={"bouton" + (mode === "equipement" ? " principal" : "")} onClick={() => setMode("equipement")}>Équipement</button>
            </div>
          </div>

          {mode === "equipement" ? (
            <Equipement outils={outils} inventaire={inventaire} emplacements={page.emplacements} glisse={glisse} onGlisser={setGlisse} cmd={cmd} />
          ) : (
            <div className="disposition-inventaire">
              <nav className="arbre-conteneurs" aria-label="Conteneurs">
                <button className={"noeud" + (conteneur === null ? " choisi" : "")} onClick={() => setConteneur(null)}>
                  Tout l'inventaire <span className="doux">{inventaire.poids.texte}</span>
                </button>
                <button
                  className={"noeud" + (conteneur === inventaire.racine ? " choisi" : "")}
                  onClick={() => setConteneur(inventaire.racine)}
                  onDragOver={(e) => glisse && e.preventDefault()}
                  onDrop={(e) => deposerSur(e, inventaire.racine)}
                >
                  Sur soi
                </button>
                {arbre.map((c) => (
                  <button
                    key={c.id}
                    className={"noeud" + (conteneur === c.id ? " choisi" : "")}
                    style={{ paddingLeft: 12 + 14 * (profondeur(c) + 1) }}
                    onClick={() => setConteneur(c.id)}
                    onDragOver={(e) => glisse && glisse.id !== c.id && e.preventDefault()}
                    onDrop={(e) => deposerSur(e, c.id)}
                    title={c.contenuCompte ? undefined : "Conteneur extradimensionnel : son contenu ne pèse pas"}
                  >
                    {c.nom}
                    <span className="doux">
                      {c.contenu?.texte ?? "0 g"}
                      {c.capacite && ` / ${c.capacite.texte}`}
                      {!c.contenuCompte && " ∞"}
                    </span>
                  </button>
                ))}
              </nav>

              <div className="centre-inventaire">
                <ListeObjets objets={objets} conteneur={conteneur} selection={selection} onSelection={setSelection} onGlisser={setGlisse} />
                <details className="carte encadre">
                  <summary>Ajouter un objet</summary>
                  <AjoutRapide outils={outils} contenants={contenants} cible={cible || inventaire.racine} onCible={setCible} />
                </details>
              </div>

              {choisi ? (
                <FicheObjet objet={choisi} objets={objets} racine={inventaire.racine} emplacements={page.emplacements} cmd={cmd} onNote={(id, note) => outils.agir(api.noterObjet(id, note))} />
              ) : (
                <aside className="fiche-objet vide">
                  <p className="doux">Choisissez un objet pour voir sa fiche, l'équiper, le ranger ou le jeter. Glissez un objet sur un conteneur pour l'y ranger.</p>
                </aside>
              )}
            </div>
          )}
        </>
      )}
    </div>
  );
}
