import { useEffect, useState } from "react";
import type { Outils } from "../../App";
import { api } from "../../api";
import { nomDuType, type ModeleObjetVue } from "../../inventaire-types";
import type { Contenant } from "./FicheObjet";

type Onglet = "catalogue" | "libre";

/** Ajout rapide : recherche dans le catalogue, ajout en un geste, ou saisie libre. */
export function AjoutRapide(props: { outils: Outils; contenants: Contenant[]; cible: number; onCible: (id: number) => void }) {
  const { outils, contenants, cible, onCible } = props;
  const [onglet, setOnglet] = useState<Onglet>("catalogue");
  const [requete, setRequete] = useState("");
  const [type, setType] = useState("");
  const [types, setTypes] = useState<string[]>([]);
  const [resultats, setResultats] = useState<ModeleObjetVue[]>([]);
  const [quantite, setQuantite] = useState("1");
  const [commeConteneur, setCommeConteneur] = useState(false);
  const [capacite, setCapacite] = useState("");
  const [extra, setExtra] = useState(false);
  const [libre, setLibre] = useState({ nom: "", type: "", valeur: "", poidsKg: "0", quantite: "1" });

  useEffect(() => {
    api.typesObjets().then(setTypes, () => setTypes([]));
  }, []);

  useEffect(() => {
    const t = window.setTimeout(() => {
      api.catalogueObjets(requete, type || null, 60).then(setResultats, () => setResultats([]));
    }, 150);
    return () => window.clearTimeout(t);
  }, [requete, type]);

  const q = Math.max(1, Math.floor(Number(quantite) || 1));

  const ajouter = (m: ModeleObjetVue) => {
    if (commeConteneur) {
      const grammes = capacite.trim() ? Math.round(Number(capacite.replace(",", ".")) * 1000) : null;
      outils.agir(api.ajouterConteneur(cible, m.id, grammes && grammes > 0 ? grammes : null, extra));
    } else {
      outils.agir(api.ajouterObjetCatalogue(cible, m.id, q));
    }
  };

  const ajouterLibre = async () => {
    const poids = Math.round((Number(libre.poidsKg.replace(",", ".")) || 0) * 1000);
    const r = await outils.agir(
      api.ajouterObjetLibre(cible, libre.nom, libre.type, libre.valeur, poids, Math.max(1, Math.floor(Number(libre.quantite) || 1))),
    );
    if (r?.ok) setLibre({ ...libre, nom: "", valeur: "" });
  };

  return (
    <section className="ajout-rapide" aria-label="Ajout rapide">
      <div className="groupe-boutons" role="tablist">
        <button role="tab" aria-selected={onglet === "catalogue"} className={"bouton" + (onglet === "catalogue" ? " principal" : "")} onClick={() => setOnglet("catalogue")}>Catalogue</button>
        <button role="tab" aria-selected={onglet === "libre"} className={"bouton" + (onglet === "libre" ? " principal" : "")} onClick={() => setOnglet("libre")}>Saisie libre</button>
      </div>
      <label className="ligne-action">
        <span>Ajouter dans</span>
        <select className="champ" value={cible} onChange={(e) => onCible(Number(e.target.value))}>
          {contenants.map((c) => (
            <option key={c.id} value={c.id}>{c.nom}</option>
          ))}
        </select>
      </label>

      {onglet === "catalogue" ? (
        <>
          <div className="barre-filtres">
            <input className="champ" type="search" placeholder="Chercher dans le catalogue…" aria-label="Chercher dans le catalogue" value={requete} onChange={(e) => setRequete(e.target.value)} />
            <select className="champ" aria-label="Type d'objet" value={type} onChange={(e) => setType(e.target.value)}>
              <option value="">Tous les types</option>
              {types.map((t) => (
                <option key={t} value={t}>{nomDuType(t)}</option>
              ))}
            </select>
            <input className="champ mini" type="number" min={1} aria-label="Quantité" value={quantite} onChange={(e) => setQuantite(e.target.value)} disabled={commeConteneur} />
          </div>
          <label className="case-a-cocher">
            <input type="checkbox" checked={commeConteneur} onChange={(e) => setCommeConteneur(e.target.checked)} />
            Ajouter comme conteneur (sac, coffre…)
          </label>
          {commeConteneur && (
            <div className="ligne-action">
              <input className="champ petit" placeholder="Capacité (kg), vide = illimitée" aria-label="Capacité en kilogrammes" value={capacite} onChange={(e) => setCapacite(e.target.value)} />
              <label className="case-a-cocher">
                <input type="checkbox" checked={extra} onChange={(e) => setExtra(e.target.checked)} />
                Extradimensionnel (le contenu ne pèse pas)
              </label>
            </div>
          )}
          <ul className="resultats-catalogue">
            {resultats.length === 0 && <li className="doux">Aucun objet trouvé.</li>}
            {resultats.map((m) => (
              <li key={m.id}>
                <span className="nom-objet">{m.nom}</span>
                <span className="type-objet">{nomDuType(m.typeObjet)}</span>
                <span className="poids-objet">{m.poids ? m.poids.texte : "—"}</span>
                <span className="valeur-objet">{m.valeur ? m.valeur.texte : "—"}</span>
                <button className="bouton" onClick={() => ajouter(m)} aria-label={`Ajouter ${m.nom}`}>Ajouter</button>
              </li>
            ))}
          </ul>
        </>
      ) : (
        <div className="formulaire-libre">
          <input className="champ large" placeholder="Nom de l'objet" aria-label="Nom de l'objet" value={libre.nom} onChange={(e) => setLibre({ ...libre, nom: e.target.value })} />
          <input className="champ moyen" placeholder="Type (divers, arme…)" aria-label="Type" value={libre.type} onChange={(e) => setLibre({ ...libre, type: e.target.value })} />
          <input className="champ petit" placeholder="Valeur (ex. 12 po, 5 pc)" aria-label="Valeur" value={libre.valeur} onChange={(e) => setLibre({ ...libre, valeur: e.target.value })} />
          <input className="champ mini" aria-label="Poids en kg" title="Poids en kg" value={libre.poidsKg} onChange={(e) => setLibre({ ...libre, poidsKg: e.target.value })} />
          <input className="champ mini" type="number" min={1} aria-label="Quantité" value={libre.quantite} onChange={(e) => setLibre({ ...libre, quantite: e.target.value })} />
          <button className="bouton principal" disabled={!libre.nom.trim()} onClick={ajouterLibre}>Ajouter</button>
        </div>
      )}
    </section>
  );
}
