import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, ModeleVue, PageReferentiel, UniteVue } from "../api";
import { COULEUR_RESSOURCE } from "./Batiments";

type Onglet = "salles" | "equipes" | "extensions" | "gestionnaires" | "batiments" | "organisations" | "communautes" | "capitaux";

const ONGLETS: [Onglet, string][] = [
  ["salles", "Salles"],
  ["equipes", "Équipes"],
  ["extensions", "Extensions"],
  ["gestionnaires", "Gestionnaires"],
  ["batiments", "Modèles de bâtiments"],
  ["organisations", "Modèles d'organisations"],
  ["communautes", "Communautés"],
  ["capitaux", "Capitaux"],
];

const correspond = (nom: string, recherche: string) => !recherche.trim() || nom.toLowerCase().includes(recherche.trim().toLowerCase());

function Statut({ statut, point }: { statut: "officiel" | "a_confirmer"; point: string | null }) {
  return (
    <>
      <span
        className="badge"
        style={{
          marginLeft: 0,
          color: statut === "officiel" ? "var(--vert)" : "var(--orange)",
          border: `1px solid ${statut === "officiel" ? "var(--vert)" : "var(--orange)"}`,
        }}
      >
        {statut === "officiel" ? "Officiel" : "À confirmer"}
      </span>
      {point && <span className="badge confirmer">{point}</span>}
    </>
  );
}

export function Referentiel({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageReferentiel | null>(null);
  const [onglet, setOnglet] = useState<Onglet>("salles");
  const [recherche, setRecherche] = useState("");
  const [selection, setSelection] = useState<string | null>(null);

  useEffect(() => {
    api.pageReferentiel().then(setP);
  }, [outils.version]);

  if (!p) return null;
  const unites: UniteVue[] = onglet === "salles" ? p.salles : onglet === "equipes" ? p.equipes : p.extensions;
  const choisie = unites.find((u) => u.id === selection);

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Référentiel</h1>
        <p>Données officielles du Guide de campagne (VF d'Ultimate Campaign), avec leur page</p>
      </div>

      <div className="segments" role="tablist" aria-label="Catégorie">
        {ONGLETS.map(([o, nom]) => (
          <button
            key={o}
            role="tab"
            aria-selected={onglet === o}
            className={onglet === o ? "actif" : ""}
            onClick={() => {
              setOnglet(o);
              setSelection(null);
            }}
          >
            {nom}
          </button>
        ))}
      </div>
      <input className="champ moyen" aria-label="Rechercher par nom" placeholder="Rechercher par nom" value={recherche} onChange={(e) => setRecherche(e.target.value)} />

      {(onglet === "salles" || onglet === "equipes" || onglet === "extensions") && (
        <>
          {choisie && (
            <section className="carte pile">
              <div className="ligne" style={{ gap: 10 }}>
                <h2 style={{ color: "var(--or-clair)" }}>{choisie.nom}</h2>
                <Statut statut={choisie.statut} point={choisie.point} />
              </div>
              <span className="doux">Source : {choisie.source}</span>
              <div className="fiche-referentiel">
                <span className="doux">Recettes</span><span>{choisie.recettes}</span>
                {choisie.avantages && (<><span className="doux">Avantages</span><span>{choisie.avantages}</span></>)}
                <span className="doux">Création</span><span>{choisie.cout}</span>
                <span className="doux">Achat terminé</span><span>{choisie.prix}</span>
                <span className="doux">Durée</span><span>{choisie.duree}</span>
                <span className="doux">Taille</span><span>{choisie.taille}</span>
                {choisie.amelioreDe.length > 0 && (<><span className="doux">Amélioration de</span><span>{choisie.amelioreDe.join(", ")}</span></>)}
                {choisie.amelioreEn.length > 0 && (<><span className="doux">Amélioration en</span><span>{choisie.amelioreEn.join(", ")}</span></>)}
              </div>
            </section>
          )}
          <div className="carte tableau-ref">
            <div className="ligne-ref entete">
              <span>Nom</span><span>Recettes</span><span>Création</span><span>Prix</span><span>Durée</span><span>Page</span>
            </div>
            {unites.filter((u) => correspond(u.nom, recherche)).map((u) => (
              <button
                key={u.id}
                className={"ligne-ref" + (selection === u.id ? " actif" : "")}
                onClick={() => setSelection(selection === u.id ? null : u.id)}
              >
                <span style={{ color: "var(--or-clair)", fontWeight: 500 }}>{u.nom}</span>
                <span>{u.recettes}</span>
                <span>{u.cout}</span>
                <span>{u.prix}</span>
                <span>{u.duree}</span>
                <span className="doux">{u.page}</span>
              </button>
            ))}
          </div>
        </>
      )}

      {onglet === "gestionnaires" && (
        <>
          <p className="doux">Salaire payé chaque jour en po (Guide de campagne p. 89-92).</p>
          <div className="carte tableau-ref">
            <div className="ligne-ref entete g4"><span>Nom</span><span>Salaire</span><span>Compétences</span><span>Page</span></div>
            {p.gestionnaires.filter((g) => correspond(g.nom, recherche)).map((g) => (
              <div className="ligne-ref g4" key={g.nom}>
                <span style={{ color: "var(--or-clair)", fontWeight: 500 }}>{g.nom}</span>
                <span>{g.salaire}</span>
                <span>{g.competences}</span>
                <span className="doux">{g.page}</span>
              </div>
            ))}
          </div>
        </>
      )}

      {(onglet === "batiments" || onglet === "organisations") &&
        (onglet === "batiments" ? p.modelesBatiments : p.modelesOrganisations)
          .filter((m: ModeleVue) => correspond(m.nom, recherche))
          .map((m: ModeleVue) => (
            <article className="carte pile" key={m.nom}>
              <div className="ligne" style={{ gap: 10 }}>
                <strong style={{ fontSize: 17, color: "var(--or-clair)" }}>{m.nom}</strong>
                <Statut statut={m.statut} point={m.point} />
                <span className="doux">{m.page}</span>
              </div>
              <span>{m.cout} ({m.prix})</span>
              <span>{m.composition}</span>
              {m.imprime && <span className="doux">Texte imprimé : {m.imprime}</span>}
            </article>
          ))}

      {onglet === "communautes" && (
        <>
          <div className="carte tableau-ref">
            <div className="ligne-ref entete g4"><span>Catégorie</span><span>Population</span><span>Limite des dépenses / jour</span><span>Statut</span></div>
            {p.communautes.map((c) => (
              <div className="ligne-ref g4" key={c.nom}>
                <span style={{ color: "var(--or-clair)", fontWeight: 500 }}>{c.nom}</span>
                <span>{c.population}</span>
                <span>{c.limite}</span>
                <span className="ligne" style={{ gap: 8 }}><Statut statut={c.statut} point={c.point} /></span>
              </div>
            ))}
          </div>
          <p className="doux">Population : Guide du maître (profil d'une communauté). Limite : Guide de campagne p. 81.</p>
        </>
      )}

      {onglet === "capitaux" && (
        <>
          <div className="carte tableau-ref">
            <div className="ligne-ref entete g3"><span>Capital</span><span>Coût d'achat</span><span>Coût produit</span></div>
            {p.capitaux.map((v) => (
              <div className="ligne-ref g3" key={v.ressource}>
                <span style={{ color: COULEUR_RESSOURCE[v.ressource], fontWeight: 500 }}>{v.nom}</span>
                <span>{v.coutAchat}</span>
                <span>{v.coutProduit}</span>
              </div>
            ))}
          </div>
          <p className="doux">Table 2-1 : Valeurs des capitaux (Guide de campagne p. 78). Produire coûte la moitié de l'achat.</p>
        </>
      )}
    </div>
  );
}
