import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, ActionBien, ChargeBien, CibleBien, ChoixCible, FicheMonture, FicheVehicule, formaterPo, JaugeEquipage, ModeleBien, PageBiens, Retour } from "../api";
import { ChampMontant } from "./Localites";

const cle = (c: CibleBien) => `${c.genre}:${c.id}`;
const trouver = (liste: ChoixCible[], k: string) => liste.find((x) => cle(x.cible) === k)?.cible;

/** Formulaire d'achat commun aux véhicules et aux montures. */
function FormulaireAchat(props: {
  id: string;
  modeles: ModeleBien[];
  libelleCible: string;
  cibles: ChoixCible[];
  payeurs: ChoixCible[];
  /** Pour les montures : à qui elle peut être confiée tout de suite (facultatif). */
  detenteurs?: ChoixCible[];
  acheter: (modele: string, nom: string, cible: CibleBien, payeur: CibleBien, prix: string | null, detenteur: CibleBien | null) => Promise<Retour | null | undefined>;
}) {
  const [modele, setModele] = useState("");
  const [nom, setNom] = useState("");
  const [cibleCle, setCibleCle] = useState("");
  const [payeurCle, setPayeurCle] = useState("");
  const [detenteurCle, setDetenteurCle] = useState("");
  const [prix, setPrix] = useState("");
  const m = props.modeles.find((x) => x.id === (modele || props.modeles[0]?.id));
  const cible = trouver(props.cibles, cibleCle) ?? props.cibles[0]?.cible;
  const payeur = trouver(props.payeurs, payeurCle) ?? props.payeurs[0]?.cible;
  const pret = !!m && !!cible && !!payeur && (m.prix !== null || prix.trim() !== "");
  return (
    <form
      className="formulaire"
      onSubmit={async (e) => {
        e.preventDefault();
        if (!m || !cible || !payeur) return;
        const r = await props.acheter(m.id, nom, cible, payeur, prix.trim() || null, (props.detenteurs && trouver(props.detenteurs, detenteurCle)) || null);
        if (r?.ok) {
          setNom("");
          setPrix("");
        }
      }}
    >
      <label htmlFor={`${props.id}-modele`}>Modèle</label>
      <select id={`${props.id}-modele`} className="champ" value={m?.id ?? ""} onChange={(e) => setModele(e.target.value)}>
        {props.modeles.map((x) => (
          <option key={x.id} value={x.id}>{x.nom}{x.prix ? ` — ${formaterPo(x.prix.pc)}` : " — prix à saisir"}</option>
        ))}
      </select>
      {m && m.resume && (
        <>
          <span className="etiquette-grille">Manuel</span>
          <span className="doux">{m.resume}</span>
        </>
      )}
      <label htmlFor={`${props.id}-nom`}>Nom</label>
      <input id={`${props.id}-nom`} className="champ" placeholder={m?.nom ?? ""} value={nom} onChange={(e) => setNom(e.target.value)} />
      <label htmlFor={`${props.id}-cible`}>{props.libelleCible}</label>
      <select id={`${props.id}-cible`} className="champ" value={cible ? cle(cible) : ""} onChange={(e) => setCibleCle(e.target.value)}>
        {props.cibles.map((x) => (
          <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
        ))}
      </select>
      {props.detenteurs && (
        <>
          <label htmlFor={`${props.id}-detenteur`}>Confiée à</label>
          <select id={`${props.id}-detenteur`} className="champ" value={detenteurCle} onChange={(e) => setDetenteurCle(e.target.value)}>
            <option value="">personne (à l'écurie du propriétaire)</option>
            {props.detenteurs.map((x) => (
              <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
            ))}
          </select>
        </>
      )}
      <label htmlFor={`${props.id}-payeur`}>Payé par</label>
      <select id={`${props.id}-payeur`} className="champ" value={payeur ? cle(payeur) : ""} onChange={(e) => setPayeurCle(e.target.value)}>
        {props.payeurs.map((x) => (
          <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
        ))}
      </select>
      <label htmlFor={`${props.id}-prix`}>Prix</label>
      <ChampMontant id={`${props.id}-prix`} valeur={prix} changer={setPrix} />
      <span className="doux" style={{ gridColumn: "2" }}>
        {m?.prix ? `Laissez vide pour le prix du catalogue (${formaterPo(m.prix.pc)}).` : "Ce modèle n'a pas de prix au catalogue : saisissez-le."}
      </span>
      <span />
      <div>
        <button type="submit" className="bouton principal" disabled={!pret}>Acheter</button>
      </div>
    </form>
  );
}

function Jauge({ j }: { j: JaugeEquipage }) {
  const cible = Math.max(j.requisMax, j.requisMin, 1);
  const pctRecrute = Math.min(100, (j.recrutes / cible) * 100);
  const pctCours = Math.min(100 - pctRecrute, (j.enCours / cible) * 100);
  const plage = j.requisMin === j.requisMax ? `${j.requisMin}` : `${j.requisMin} à ${j.requisMax}`;
  return (
    <div className="pile" style={{ gap: 4 }}>
      <span>
        Équipage : <strong>{j.recrutes}</strong> / {plage}
        {j.enCours > 0 && <span className="doux"> (+ {j.enCours} en cours de recrutement)</span>}
        {j.complete && <span className="badge" style={{ marginLeft: 8, border: "1px solid var(--bord-fort)" }}>équipage complet</span>}
      </span>
      <div className="barre" role="progressbar" aria-valuemin={0} aria-valuemax={j.requisMin} aria-valuenow={j.recrutes} aria-label="Équipage recruté">
        <div style={{ width: `${pctRecrute}%`, display: "inline-block" }} />
        <div style={{ width: `${pctCours}%`, display: "inline-block", opacity: 0.4 }} />
      </div>
    </div>
  );
}

const nombre = (x: number) => x.toLocaleString("fr-FR", { maximumFractionDigits: 1 });

/** Poids porté face aux limites : trois limites pour une monture, une cargaison pour un véhicule. */
function JaugeCharge({ c, titre }: { c: ChargeBien; titre: string }) {
  const max = Math.max(1, c.poidsKg, ...c.limitesKg);
  const pct = (x: number) => Math.min(100, (x / max) * 100);
  return (
    <div className="pile" style={{ gap: 4 }}>
      <span>
        {titre} : <strong style={{ color: c.depassement ? "var(--rouge)" : undefined }}>{nombre(c.poidsKg)} kg</strong>
        <span className="badge" style={{ marginLeft: 8, border: "1px solid var(--bord-fort)", color: c.depassement ? "var(--rouge)" : undefined }}>{c.categorie}</span>
        {c.limitesKg.length === 1 && <span className="doux"> sur {nombre(c.limitesKg[0])} kg</span>}
      </span>
      <div className="barre" role="progressbar" aria-valuemin={0} aria-valuemax={max} aria-valuenow={c.poidsKg} aria-label={titre} style={{ position: "relative" }}>
        <div style={{ width: `${pct(c.poidsKg)}%`, background: c.depassement ? "var(--rouge)" : undefined }} />
        {c.limitesKg.length === 3 && c.limitesKg.map((l) => (
          <span key={l} style={{ position: "absolute", left: `${pct(l)}%`, top: 0, bottom: 0, width: 1, background: "var(--bord-fort)" }} />
        ))}
      </div>
      {c.limitesKg.length === 3 && (
        <span className="doux">
          légère jusqu'à {nombre(c.limitesKg[0])} kg · intermédiaire jusqu'à {nombre(c.limitesKg[1])} kg · lourde jusqu'à {nombre(c.limitesKg[2])} kg
        </span>
      )}
      {c.depassement && <span style={{ color: "var(--rouge)" }}>Dépassement de la limite : permis, mais le Manuel ne donne pas la vitesse dans ce cas.</span>}
    </div>
  );
}

function BoutonsInventaire(props: { genre: "vehicule" | "monture"; id: number; inventaire: number | null; agir: (a: ActionBien) => Promise<Retour | null>; outils: Outils }) {
  return props.inventaire === null ? (
    <button className="bouton" onClick={() => props.agir({ type: "creerInventaire", genre: props.genre, id: props.id })}>Créer l'inventaire</button>
  ) : (
    <button className="bouton" onClick={() => props.outils.aller("inventaire")} title="La page Inventaire montre son contenu, comme celui d'un personnage">
      Ouvrir l'inventaire ({props.genre === "vehicule" ? "cargaison" : "sacoches"})
    </button>
  );
}

function SuiviVehicule({ v, payeurs, agir, outils }: { v: FicheVehicule; payeurs: ChoixCible[]; agir: (a: ActionBien) => Promise<Retour | null>; outils: Outils }) {
  const [passagers, setPassagers] = useState<string | null>(null);
  const [choix, setChoix] = useState("");
  const [quantite, setQuantite] = useState("1");
  const [prix, setPrix] = useState("");
  const [payeurCle, setPayeurCle] = useState("");
  const a = v.ameliorationsPossibles.find((x) => x.id === choix);
  const payeur = trouver(payeurs, payeurCle) ?? payeurs[0]?.cible;
  const nb = passagers ?? String(v.passagers);
  return (
    <div className="pile" style={{ gap: 10 }}>
      <div className="ligne" style={{ gap: 10 }}>
        <BoutonsInventaire genre="vehicule" id={v.id} inventaire={v.inventaire} agir={agir} outils={outils} />
        {v.vitesseAjustee && <span>Vitesse avec les améliorations : <strong>{v.vitesseAjustee}</strong></span>}
        {v.bonusNavigation !== 0 && <span>Navigation : <strong>{v.bonusNavigation > 0 ? "+" : ""}{v.bonusNavigation}</strong></span>}
      </div>
      {v.charge && <JaugeCharge c={v.charge} titre="Cargaison" />}
      <div className="ligne" style={{ gap: 8 }}>
        <label htmlFor={`pass-${v.id}`}>Passagers à bord</label>
        <input id={`pass-${v.id}`} className="champ" style={{ width: 80 }} type="number" min={0} value={nb} onChange={(e) => setPassagers(e.target.value)} />
        {v.passagersMax !== null && <span className="doux">sur {v.passagersMax}</span>}
        {v.passagersMax !== null && Number(nb) > v.passagersMax && <span style={{ color: "var(--rouge)" }}>au-delà du maximum</span>}
        <button className="bouton" disabled={passagers === null || passagers === "" || Number(passagers) === v.passagers} onClick={async () => {
          const r = await agir({ type: "passagers", id: v.id, nombre: Math.max(0, Math.floor(Number(passagers))) });
          if (r?.ok) setPassagers(null);
        }}>Enregistrer</button>
      </div>
      {(v.ameliorations.length > 0 || v.ameliorationsPossibles.length > 0) && (
        <div className="pile" style={{ gap: 6 }}>
          <strong>Améliorations du bateau</strong>
          <span className="doux">Données hors Manuel (Ultimate Equipment). Un échec au jet d'Artisanat n'est pas suivi.</span>
          {v.ameliorations.length === 0 && <span className="doux">aucune installée</span>}
          {v.ameliorations.map((i) => (
            <div className="ligne" style={{ gap: 8 }} key={i.index}>
              <span className="puce or">
                {i.nom}{i.quantite > 1 ? ` × ${i.quantite}` : ""} — {formaterPo(i.prix.pc)}{i.effets ? ` — ${i.effets}` : ""}{" "}
                <button className="bouton discret" style={{ height: 22, padding: "0 4px" }} aria-label={`Retirer ${i.nom}`} title="Retirer (sans remboursement)" onClick={() => agir({ type: "retirerAmelioration", id: v.id, index: i.index })}>✕</button>
              </span>
            </div>
          ))}
          {v.ameliorationsPossibles.length > 0 && (
            <div className="pile" style={{ gap: 6 }}>
              <select className="champ" style={{ maxWidth: 420 }} aria-label="Installer une amélioration" value={choix} onChange={(e) => { setChoix(e.target.value); setPrix(""); setQuantite("1"); }}>
                <option value="">Installer une amélioration…</option>
                {v.ameliorationsPossibles.map((x) => (
                  <option key={x.id} value={x.id}>{x.nom} — {x.prix ? formaterPo(x.prix.pc) : x.cout}</option>
                ))}
              </select>
              {a && (
                <div className="pile" style={{ gap: 6 }}>
                  <span className="doux">{a.texte}</span>
                  <span className="doux">Prérequis : {a.prerequis} · coût : {a.cout}{a.effets ? ` · effets : ${a.effets}` : ""}</span>
                  {a.aLaConstruction && <span style={{ color: "var(--ambre, var(--po))" }}>À installer à la construction du bateau, pas après : à vous de le respecter.</span>}
                  <div className="ligne" style={{ gap: 8 }}>
                    {a.unite && (
                      <>
                        <label htmlFor={`qte-${v.id}`}>Nombre ({a.unite})</label>
                        <input id={`qte-${v.id}`} className="champ" style={{ width: 80 }} type="number" min={1} value={quantite} onChange={(e) => setQuantite(e.target.value)} />
                      </>
                    )}
                    <label htmlFor={`prix-${v.id}`}>Prix</label>
                    <ChampMontant id={`prix-${v.id}`} valeur={prix} changer={setPrix} />
                    <span className="doux">{a.prix && !a.unite ? `vide = ${formaterPo(a.prix.pc)}` : a.unite ? "vide = prix par unité × nombre" : "à saisir"}</span>
                  </div>
                  <div className="ligne" style={{ gap: 8 }}>
                    <label htmlFor={`payeur-${v.id}`}>Payé par</label>
                    <select id={`payeur-${v.id}`} className="champ" style={{ maxWidth: 320 }} value={payeur ? cle(payeur) : ""} onChange={(e) => setPayeurCle(e.target.value)}>
                      {payeurs.map((x) => <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>)}
                    </select>
                    <button className="bouton principal" disabled={!payeur || (a.prix === null && !prix.trim() && !a.unite)} onClick={async () => {
                      if (!payeur) return;
                      const r = await agir({ type: "installerAmelioration", id: v.id, amelioration: a.id, quantite: a.unite ? Math.max(1, Math.floor(Number(quantite))) : null, payeur, prix: prix.trim() || null });
                      if (r?.ok) { setChoix(""); setPrix(""); }
                    }}>Installer</button>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function SuiviMonture({ m, payeurs, agir, outils }: { m: FicheMonture; payeurs: ChoixCible[]; agir: (a: ActionBien) => Promise<Retour | null>; outils: Outils }) {
  const [armure, setArmure] = useState("");
  const [prix, setPrix] = useState("");
  const [payeurCle, setPayeurCle] = useState("");
  const [vente, setVente] = useState(false);
  const [prixVente, setPrixVente] = useState("");
  const payeur = trouver(payeurs, payeurCle) ?? payeurs[0]?.cible;
  const choisie = m.bardesPossibles.find((b) => b.nomEn === armure);
  return (
    <div className="pile" style={{ gap: 10 }}>
      <div className="ligne" style={{ gap: 10 }}>
        <BoutonsInventaire genre="monture" id={m.id} inventaire={m.inventaire} agir={agir} outils={outils} />
      </div>
      {m.charge && <JaugeCharge c={m.charge} titre="Charge (sacoches, objets, barde)" />}
      <div className="pile" style={{ gap: 6 }}>
        <strong>Barde</strong>
        {m.barde ? (
          <>
            <span>
              {m.barde.nom} ({m.barde.genre}) — CA +{m.barde.bonusCa}
              {m.barde.dexMax !== null ? `, bonus de Dex max +${m.barde.dexMax}` : ""}, malus d'armure {m.barde.malusArmure} — {formaterPo(m.barde.prix.pc)}
              {m.barde.poidsKg !== null ? ` — ${nombre(m.barde.poidsKg)} kg` : ""}
            </span>
            {m.barde.vitesse && <span>Vitesse de la monture : <strong>{m.barde.vitesse}</strong></span>}
            <div className="ligne" style={{ gap: 8 }}>
              <button className="bouton" title="La barde va dans l'inventaire de la monture, où elle peut être déplacée ou rééquipée sans la repayer" onClick={() => agir({ type: "retirerBarde", id: m.id })}>Ranger dans l'inventaire</button>
              <button className="bouton discret" onClick={() => setVente(!vente)}>Vendre…</button>
            </div>
            {vente && (
              <div className="ligne" style={{ gap: 8 }}>
                <span className="doux">Revente à la moitié du prix payé ({formaterPo(m.barde.prixRevente.pc)}).</span>
                <label htmlFor={`vprix-${m.id}`}>Prix</label>
                <ChampMontant id={`vprix-${m.id}`} valeur={prixVente} changer={setPrixVente} />
                <label htmlFor={`vpay-${m.id}`}>Versé à</label>
                <select id={`vpay-${m.id}`} className="champ" style={{ maxWidth: 280 }} value={payeur ? cle(payeur) : ""} onChange={(e) => setPayeurCle(e.target.value)}>
                  {payeurs.map((x) => <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>)}
                </select>
                <button className="bouton principal" disabled={!payeur} onClick={async () => {
                  if (!payeur) return;
                  const r = await agir({ type: "vendreBarde", id: m.id, beneficiaire: payeur, prix: prixVente.trim() || null });
                  if (r?.ok) { setVente(false); setPrixVente(""); }
                }}>Vendre</button>
              </div>
            )}
          </>
        ) : (
          <>
            {m.bardesRangees.map((b) => (
              <div className="ligne" style={{ gap: 8 }} key={b.objet}>
                <span>{b.libelle} <span className="doux">(dans l'inventaire)</span></span>
                <button className="bouton" onClick={() => agir({ type: "equiperBardeRangee", id: m.id, objet: b.objet })}>Équiper sans payer</button>
              </div>
            ))}
          <div className="ligne" style={{ gap: 8 }}>
            <select className="champ" style={{ maxWidth: 420 }} aria-label="Équiper une barde" value={armure} onChange={(e) => { setArmure(e.target.value); setPrix(""); }}>
              <option value="">Équiper une barde…</option>
              {m.bardesPossibles.map((b) => <option key={b.nomEn} value={b.nomEn}>{b.libelle}</option>)}
            </select>
            {choisie && (
              <>
                <label htmlFor={`bprix-${m.id}`}>Prix</label>
                <ChampMontant id={`bprix-${m.id}`} valeur={prix} changer={setPrix} />
                <label htmlFor={`bpay-${m.id}`}>Payé par</label>
                <select id={`bpay-${m.id}`} className="champ" style={{ maxWidth: 280 }} value={payeur ? cle(payeur) : ""} onChange={(e) => setPayeurCle(e.target.value)}>
                  {payeurs.map((x) => <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>)}
                </select>
                <button className="bouton principal" disabled={!payeur} onClick={async () => {
                  if (!payeur) return;
                  const r = await agir({ type: "equiperBarde", id: m.id, armure, payeur, prix: prix.trim() || null });
                  if (r?.ok) { setArmure(""); setPrix(""); }
                }}>Équiper</button>
              </>
            )}
          </div>
          </>
        )}
      </div>
      {m.notes.length > 0 && (
        <ul className="doux" style={{ margin: 0, paddingLeft: 18 }}>
          {m.notes.map((n) => <li key={n}>{n}</li>)}
        </ul>
      )}
    </div>
  );
}

export function Biens({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageBiens | null>(null);
  const [equipeChoisie, setEquipeChoisie] = useState<Record<number, string>>({});
  const [gestionnaireChoisi, setGestionnaireChoisi] = useState<Record<number, string>>({});

  useEffect(() => {
    api.pageBiens().then(setP);
  }, [outils.version]);

  if (!p) return null;
  const agir = (a: ActionBien) => outils.agir(api.actionBien(a));

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Biens</h1>
        <p>Véhicules et montures de la campagne : propriétaire, équipes, gestionnaires, équipage. Caractéristiques du Manuel des joueurs ; les attributions sont des règles personnalisées.</p>
      </div>

      <section className="carte section">
        <h2>Acheter un véhicule</h2>
        <FormulaireAchat
          id="veh"
          modeles={p.modelesVehicules}
          libelleCible="Propriétaire"
          cibles={p.proprietaires}
          payeurs={p.payeurs}
          acheter={(modele, nom, cible, payeur, prix) => agir({ type: "acheterVehicule", modele, nom, proprietaire: cible, payeur, prix })}
        />
      </section>

      <section className="section">
        <h2>Véhicules</h2>
        {p.vehicules.length === 0 && <p className="doux">Aucun véhicule pour l'instant.</p>}
        {p.vehicules.map((v) => (
          <div className="carte pile" key={v.id}>
            <div className="ligne" style={{ gap: 12 }}>
              <strong style={{ fontSize: 17, color: "var(--texte-fort)" }}>{v.nom}</strong>
              <span className="doux">{v.modele}</span>
              <strong style={{ color: "var(--po)" }} title="Compté dans le patrimoine du propriétaire">{formaterPo(v.prix.pc)}</strong>
              <button className="bouton discret" onClick={() => agir({ type: "retirerVehicule", id: v.id })}>Retirer</button>
            </div>
            <div className="ligne" style={{ gap: 10 }}>
              <label className="doux" htmlFor={`veh-prop-${v.id}`}>Propriétaire</label>
              <select
                id={`veh-prop-${v.id}`}
                className="champ"
                style={{ maxWidth: 320 }}
                value={cle(v.proprietaire.cible)}
                onChange={(e) => {
                  const c = trouver(p.proprietaires, e.target.value);
                  if (c) agir({ type: "modifierVehicule", id: v.id, nom: v.nom, proprietaire: c });
                }}
              >
                {p.proprietaires.map((x) => (
                  <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
                ))}
              </select>
            </div>
            {v.details.length > 0 && (
              <ul className="doux" style={{ margin: 0, paddingLeft: 18 }}>
                {v.details.map((d) => <li key={d}>{d}</li>)}
              </ul>
            )}
            {v.jauge && <Jauge j={v.jauge} />}
            <SuiviVehicule v={v} payeurs={p.payeurs} agir={agir} outils={outils} />
            <div className="ligne" style={{ gap: 10 }}>
              <span className="doux">Équipes :</span>
              {v.equipes.length === 0 && <span className="doux">aucune</span>}
              {v.equipes.map((e) => (
                <span className="puce or" key={cle(e.cible)}>
                  {e.libelle}{" "}
                  <button className="bouton discret" style={{ height: 22, padding: "0 4px" }} aria-label={`Retirer ${e.libelle}`} onClick={() => agir({ type: "retirerEquipe", id: v.id, equipe: e.cible.id })}>✕</button>
                </span>
              ))}
            </div>
            {v.equipesPossibles.length > 0 && (
              <div className="ligne" style={{ gap: 8 }}>
                <select className="champ" style={{ maxWidth: 320 }} aria-label="Affecter une équipe" value={equipeChoisie[v.id] ?? ""} onChange={(e) => setEquipeChoisie({ ...equipeChoisie, [v.id]: e.target.value })}>
                  <option value="">Affecter une équipe…</option>
                  {v.equipesPossibles.map((e) => (
                    <option key={cle(e.cible)} value={e.cible.id}>{e.libelle}</option>
                  ))}
                </select>
                <button
                  className="bouton"
                  disabled={!equipeChoisie[v.id]}
                  onClick={async () => {
                    const r = await agir({ type: "affecterEquipe", id: v.id, equipe: Number(equipeChoisie[v.id]) });
                    if (r?.ok) setEquipeChoisie({ ...equipeChoisie, [v.id]: "" });
                  }}
                >
                  Affecter
                </button>
              </div>
            )}
            <div className="ligne" style={{ gap: 10 }}>
              <span className="doux">Gestionnaires :</span>
              {v.gestionnaires.length === 0 && <span className="doux">aucun</span>}
              {v.gestionnaires.map((g) => (
                <span className="puce or" key={g.id}>
                  {g.libelle}{" "}
                  <button className="bouton discret" style={{ height: 22, padding: "0 4px" }} aria-label={`Retirer ${g.libelle}`} onClick={() => agir({ type: "retirerGestionnaire", id: v.id, gestionnaire: g.id })}>✕</button>
                </span>
              ))}
            </div>
            {v.gestionnairesPossibles.length > 0 && (
              <div className="ligne" style={{ gap: 8 }}>
                <select className="champ" style={{ maxWidth: 320 }} aria-label="Affecter un gestionnaire" value={gestionnaireChoisi[v.id] ?? ""} onChange={(e) => setGestionnaireChoisi({ ...gestionnaireChoisi, [v.id]: e.target.value })}>
                  <option value="">Affecter un gestionnaire…</option>
                  {v.gestionnairesPossibles.map((g) => (
                    <option key={g.id} value={g.id}>{g.libelle}</option>
                  ))}
                </select>
                <button
                  className="bouton"
                  disabled={!gestionnaireChoisi[v.id]}
                  onClick={async () => {
                    const r = await agir({ type: "affecterGestionnaire", id: v.id, gestionnaire: Number(gestionnaireChoisi[v.id]) });
                    if (r?.ok) setGestionnaireChoisi({ ...gestionnaireChoisi, [v.id]: "" });
                  }}
                >
                  Affecter
                </button>
              </div>
            )}
          </div>
        ))}
      </section>

      <section className="carte section">
        <h2>Acheter une monture</h2>
        <FormulaireAchat
          id="mon"
          modeles={p.modelesMontures}
          libelleCible="Propriétaire"
          cibles={p.proprietaires}
          detenteurs={p.detenteurs}
          payeurs={p.payeurs}
          acheter={(modele, nom, cible, payeur, prix, detenteur) => agir({ type: "acheterMonture", modele, nom, proprietaire: cible, detenteur, payeur, prix })}
        />
      </section>

      <section className="section">
        <h2>Montures</h2>
        {p.montures.length === 0 && <p className="doux">Aucune monture pour l'instant.</p>}
        {p.montures.map((m) => (
          <div className="carte pile" key={m.id}>
            <div className="ligne" style={{ gap: 12 }}>
              <strong style={{ fontSize: 17, color: "var(--texte-fort)" }}>{m.nom}</strong>
              <span className="doux">{m.modele}</span>
              <strong style={{ color: "var(--po)" }} title="Compté dans le patrimoine du propriétaire">{formaterPo(m.prix.pc)}</strong>
              <button className="bouton discret" onClick={() => agir({ type: "retirerMonture", id: m.id })}>Retirer</button>
            </div>
            <div className="ligne" style={{ gap: 10 }}>
              <label className="doux" htmlFor={`mon-prop-${m.id}`}>Propriétaire</label>
              <select
                id={`mon-prop-${m.id}`}
                className="champ"
                style={{ maxWidth: 320 }}
                value={cle(m.proprietaire.cible)}
                onChange={(e) => {
                  const c = trouver(p.proprietaires, e.target.value);
                  if (c) agir({ type: "modifierMonture", id: m.id, nom: m.nom, proprietaire: c, detenteur: m.detenteur?.cible ?? null });
                }}
              >
                {p.proprietaires.map((x) => (
                  <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
                ))}
              </select>
            </div>
            <div className="ligne" style={{ gap: 10 }}>
              <label className="doux" htmlFor={`mon-det-${m.id}`}>Confiée à</label>
              <select
                id={`mon-det-${m.id}`}
                className="champ"
                style={{ maxWidth: 320 }}
                value={m.detenteur ? cle(m.detenteur.cible) : ""}
                onChange={(e) => {
                  const c = e.target.value ? trouver(p.detenteurs, e.target.value) ?? null : null;
                  agir({ type: "modifierMonture", id: m.id, nom: m.nom, proprietaire: m.proprietaire.cible, detenteur: c });
                }}
              >
                <option value="">personne (à l'écurie du propriétaire)</option>
                {p.detenteurs.map((x) => (
                  <option key={cle(x.cible)} value={cle(x.cible)}>{x.libelle}</option>
                ))}
              </select>
            </div>
            {m.details.length > 0 && (
              <ul className="doux" style={{ margin: 0, paddingLeft: 18 }}>
                {m.details.map((d) => <li key={d}>{d}</li>)}
              </ul>
            )}
            <SuiviMonture m={m} payeurs={p.payeurs} agir={agir} outils={outils} />
          </div>
        ))}
      </section>
    </div>
  );
}
