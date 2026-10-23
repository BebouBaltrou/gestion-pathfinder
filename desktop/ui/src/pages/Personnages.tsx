import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, FichePersonnage, formaterPo, PagePersonnages } from "../api";
import { icones } from "../icones";
import { ChampMontant } from "./Localites";

const AIDE_PRESTIGE =
  "Valeur de Prestige, ou niveau + modificateur de Charisme sans le don (Guide de campagne p. 93)";

export function Personnages({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PagePersonnages | null>(null);
  const [selection, setSelection] = useState<number | null>(null);
  const [creation, setCreation] = useState(false);

  useEffect(() => {
    api.pagePersonnages().then((x) => {
      setP(x);
      if (x && x.personnages.length === 0) setCreation(true);
    });
  }, [outils.version]);

  if (!p) return null;
  const choisi = p.personnages.find((x) => x.id === selection) ?? (creation ? undefined : p.personnages[0]);

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Personnages</h1>
        <p>Chaque PJ a un compte en po séparé des trésoreries des localités</p>
      </div>

      <div className="maitre-detail">
        <aside className="liste-maitre" aria-label="Liste des personnages">
          <button
            className={"bouton nouvelle" + (creation ? " actif" : "")}
            onClick={() => {
              setCreation(true);
              setSelection(null);
            }}
          >
            {icones.plus({})} Nouveau personnage
          </button>
          {p.personnages.map((x) => {
            const actif = !creation && choisi?.id === x.id;
            return (
              <button
                key={x.id}
                className={"element-liste" + (actif ? " actif" : "")}
                aria-current={actif ? "true" : undefined}
                onClick={() => {
                  setSelection(x.id);
                  setCreation(false);
                }}
              >
                <span className="nom">{x.nom}</span>
                <span className={"montant" + (x.compte.pa < 0 ? " negatif" : "")}>{formaterPo(x.compte.pa)}</span>
                <span className="doux">{x.lieu}</span>
              </button>
            );
          })}
        </aside>

        <div className="detail">
          {creation || !choisi ? (
            <Creation
              outils={outils}
              apres={async (avant) => {
                const apres = await api.pagePersonnages();
                setCreation(false);
                setSelection(apres?.personnages.find((x) => !avant.has(x.id))?.id ?? null);
              }}
              existants={new Set(p.personnages.map((x) => x.id))}
            />
          ) : (
            <Detail key={choisi.id} pj={choisi} p={p} outils={outils} />
          )}
        </div>
      </div>
    </div>
  );
}

function Creation(props: { outils: Outils; existants: Set<number>; apres: (avant: Set<number>) => void }) {
  const [nom, setNom] = useState("");
  const [compte, setCompte] = useState("");
  const [prestige, setPrestige] = useState("");
  return (
    <section className="carte section">
      <h2>Nouveau personnage</h2>
      <form
        className="formulaire"
        onSubmit={async (e) => {
          e.preventDefault();
          const r = await props.outils.agir(api.creerPersonnage(nom, compte, prestige));
          if (r?.ok) props.apres(props.existants);
        }}
      >
        <label htmlFor="pj-nom">Nom</label>
        <input id="pj-nom" className="champ" value={nom} onChange={(e) => setNom(e.target.value)} />
        <label htmlFor="pj-compte">Compte initial</label>
        <ChampMontant id="pj-compte" valeur={compte} changer={setCompte} />
        <label htmlFor="pj-prestige">Prestige</label>
        <div className="pile">
          <input id="pj-prestige" className="champ mini" inputMode="numeric" placeholder="—" value={prestige} onChange={(e) => setPrestige(e.target.value)} />
          <span className="doux">Facultatif. {AIDE_PRESTIGE}.</span>
        </div>
        <span />
        <div>
          <button type="submit" className="bouton principal" disabled={!nom.trim()}>Créer le personnage</button>
        </div>
      </form>
    </section>
  );
}

function Detail({ pj, p, outils }: { pj: FichePersonnage; p: PagePersonnages; outils: Outils }) {
  const [delta, setDelta] = useState("");
  const [motif, setMotif] = useState("");
  const [nom, setNom] = useState(pj.nom);
  const [prestige, setPrestige] = useState(pj.prestige === null ? "" : String(pj.prestige));

  return (
    <>
      <section className="carte fiche-localite">
        <div className="tete">
          <h2 className="nom">{pj.nom}</h2>
          {pj.prestige !== null && <span className="doux">Prestige {pj.prestige}</span>}
        </div>
        <div className="ligne">
          <div className="tresor">
            <span className="etiquette">Compte personnel</span>
            <span className={"montant" + (pj.compte.pa < 0 ? " negatif" : "")}>{formaterPo(pj.compte.pa)}</span>
          </div>
          <div>
            <label className="etiquette-champ" htmlFor="pj-lieu">Se trouve à</label>
            <select
              id="pj-lieu"
              className="champ moyen"
              value={pj.localite ?? ""}
              onChange={(e) => outils.agir(api.deplacerPersonnage(pj.id, e.target.value === "" ? null : Number(e.target.value)))}
            >
              <option value="">hors localité</option>
              {p.localites.map((l) => (
                <option key={l.id} value={l.id}>{l.libelle}</option>
              ))}
            </select>
          </div>
        </div>
        <p className="doux">La présence des PJ compte pour l'usure des capitaux et des entreprises (Guide de campagne p. 82).</p>
      </section>

      <section className="carte section">
        <h2>Gain ou dépense</h2>
        <form
          className="ligne operation"
          onSubmit={async (e) => {
            e.preventDefault();
            const r = await outils.agir(api.ajusterCompte(pj.id, delta, motif));
            if (r?.ok) {
              setDelta("");
              setMotif("");
            }
          }}
        >
          <div>
            <label className="etiquette-champ" htmlFor="pj-delta">Montant (négatif pour une dépense)</label>
            <ChampMontant id="pj-delta" valeur={delta} changer={setDelta} signe />
          </div>
          <div style={{ flex: 1 }}>
            <label className="etiquette-champ" htmlFor="pj-motif">Motif</label>
            <input id="pj-motif" className="champ large" placeholder="butin, achat…" value={motif} onChange={(e) => setMotif(e.target.value)} />
          </div>
          <button type="submit" className="bouton principal" disabled={!delta.trim() || !motif.trim()}>Appliquer</button>
        </form>
      </section>

      <details className="carte repli">
        <summary>Modifier le personnage</summary>
        <form
          className="formulaire"
          onSubmit={(e) => {
            e.preventDefault();
            outils.agir(api.modifierPersonnage(pj.id, nom, prestige));
          }}
        >
          <label htmlFor="pj-edit-nom">Nom</label>
          <input id="pj-edit-nom" className="champ" value={nom} onChange={(e) => setNom(e.target.value)} />
          <label htmlFor="pj-edit-prestige">Prestige</label>
          <div className="pile">
            <input id="pj-edit-prestige" className="champ mini" inputMode="numeric" placeholder="—" value={prestige} onChange={(e) => setPrestige(e.target.value)} />
            <span className="doux">{AIDE_PRESTIGE}.</span>
          </div>
          <span />
          <div>
            <button type="submit" className="bouton" disabled={!nom.trim()}>Enregistrer les modifications</button>
          </div>
        </form>
      </details>
    </>
  );
}
