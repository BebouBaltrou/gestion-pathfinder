import { useEffect, useState } from "react";
import { nomDuType, type CommandeInventaire, type EmplacementVue, type ObjetVue } from "../../inventaire-types";

export type Contenant = { id: number; nom: string };

/** Fiche d'un objet : détail, propriétés, historique de provenance et boutons d'action. */
export function FicheObjet(props: {
  objet: ObjetVue;
  objets: ObjetVue[];
  racine: number;
  emplacements: EmplacementVue[];
  cmd: (c: CommandeInventaire, message: string) => void;
  onNote: (id: number, note: string) => void;
}) {
  const { objet: o, objets, racine, emplacements, cmd, onNote } = props;
  const [note, setNote] = useState(o.note);
  const [vers, setVers] = useState(String(racine));
  const [part, setPart] = useState("1");
  const [confirmer, setConfirmer] = useState(false);

  useEffect(() => {
    setVers(String(o.parent ?? racine));
    setPart("1");
    setNote(o.note);
    setConfirmer(false);
  }, [o.id, o.parent, o.note, racine]);

  // Où on peut le ranger : l'inventaire et les conteneurs, sauf lui-même et ce qu'il contient.
  const interdits = new Set<number>([o.id]);
  if (o.estConteneur) {
    let ajout = true;
    while (ajout) {
      ajout = false;
      for (const x of objets) {
        if (x.parent !== null && interdits.has(x.parent) && !interdits.has(x.id)) {
          interdits.add(x.id);
          ajout = true;
        }
      }
    }
  }
  const contenants: Contenant[] = [
    { id: racine, nom: "Sur soi" },
    ...objets.filter((x) => x.estConteneur && !interdits.has(x.id)).map((x) => ({ id: x.id, nom: x.nom })),
  ];
  const nomSlot = (id: string) => emplacements.find((e) => e.id === id)?.nom ?? id;
  const jumeau = objets.find((x) => x.id !== o.id && !x.estConteneur && x.nom === o.nom && x.parent === o.parent && x.equipe.length === 0);
  const valeurTotale = o.valeurUnite ? `${o.valeurUnite.texte} l'unité` : "valeur non précisée";

  return (
    <aside className="fiche-objet" aria-label={`Fiche de ${o.nom}`}>
      <h3>{o.nom}</h3>
      <p className="doux">
        {nomDuType(o.typeObjet)}
        {o.quantite > 1 && ` · quantité ${o.quantite}`}
        {o.source && ` · ${o.source}`}
      </p>
      <dl className="details-objet">
        <dt>Poids</dt>
        <dd>{o.poids.texte}{o.estConteneur && o.contenu && ` (dont contenu ${o.contenu.texte}${o.contenuCompte ? "" : ", non compté"})`}</dd>
        {o.capacite && (<><dt>Capacité</dt><dd>{o.capacite.texte}</dd></>)}
        <dt>Valeur</dt>
        <dd>{valeurTotale}</dd>
        {o.equipe.length > 0 && (<><dt>Équipé</dt><dd>{o.equipe.map(nomSlot).join(", ")}</dd></>)}
        {o.proprietes.filter(([k]) => k !== "nom_en").map(([k, v]) => (
          <div key={k} className="propriete">
            <dt>{k.replace(/_/g, " ")}</dt>
            <dd>{v}</dd>
          </div>
        ))}
      </dl>
      <label className="note-objet-saisie">
        <span className="doux">Note de l'objet (provenance, histoire)</span>
        <textarea className="champ zone large" rows={3} value={note} onChange={(e) => setNote(e.target.value)} placeholder="Trouvée dans la crypte de Sandpoint…" aria-label="Note de l'objet" />
        <button className="bouton" disabled={note.trim() === o.note.trim()} onClick={() => onNote(o.id, note)}>Enregistrer la note</button>
      </label>

      <div className="actions-objet">
        {o.equipe.length > 0 ? (
          <button className="bouton" disabled={o.colle} title={o.colle ? "Objet collé : impossible à retirer" : ""} onClick={() => cmd({ Deequiper: { id: o.id } }, `${o.nom} déséquipé`)}>
            Déséquiper
          </button>
        ) : (
          o.emplacementsPossibles.map((e) => (
            <button key={e} className="bouton" onClick={() => cmd({ Equiper: { id: o.id, emplacement: e } }, `${o.nom} équipé`)}>
              Équiper : {nomSlot(e)}
            </button>
          ))
        )}
        <label className="ligne-action">
          <span>Ranger dans</span>
          <select className="champ" value={vers} onChange={(e) => setVers(e.target.value)}>
            {contenants.map((c) => (
              <option key={c.id} value={c.id}>{c.nom}</option>
            ))}
          </select>
          <button className="bouton" disabled={Number(vers) === o.parent} onClick={() => cmd({ Deplacer: { id: o.id, vers: Number(vers) } }, `${o.nom} déplacé`)}>
            Déplacer
          </button>
        </label>
        {o.quantite > 1 && !o.estConteneur && (
          <label className="ligne-action">
            <span>Séparer</span>
            <input className="champ mini" type="number" min={1} max={o.quantite - 1} value={part} onChange={(e) => setPart(e.target.value)} />
            <button className="bouton" onClick={() => cmd({ Scinder: { id: o.id, quantite: Math.max(1, Number(part) || 1) } }, "Pile séparée")}>
              Séparer la pile
            </button>
          </label>
        )}
        {jumeau && (
          <button className="bouton" onClick={() => cmd({ Fusionner: { garde: o.id, absorbe: jumeau.id } }, "Piles réunies")}>
            Réunir avec l'autre pile ({jumeau.quantite})
          </button>
        )}
        <label className="case-a-cocher">
          <input type="checkbox" checked={o.colle} onChange={(e) => cmd({ DefinirColle: { id: o.id, colle: e.target.checked } }, e.target.checked ? "Objet maudit : collé" : "Objet décollé")} />
          Objet maudit (collé : on ne peut plus le retirer)
        </label>
        {confirmer ? (
          <div className="ligne-action">
            <span>Jeter définitivement ?</span>
            <button className="bouton danger" onClick={() => cmd({ Retirer: { id: o.id } }, `${o.nom} jeté`)}>Oui, jeter</button>
            <button className="bouton" onClick={() => setConfirmer(false)}>Annuler</button>
          </div>
        ) : (
          <button className="bouton danger" onClick={() => setConfirmer(true)}>Jeter</button>
        )}
      </div>
    </aside>
  );
}
