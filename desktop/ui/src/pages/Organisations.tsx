import { useEffect, useState } from "react";
import type { Outils } from "../App";
import {
  Acquisition,
  ActionOrganisation,
  api,
  ApercuCreation,
  FicheEquipe,
  FicheOrganisation,
  PageOrganisations,
  RessourceId,
  SaisieBatiment,
} from "../api";
import { icones } from "../icones";
import { texteCapitaux } from "../texte";
import { BlocVente, ChoixFacteur, ChoixPayeur, COULEUR_RESSOURCE, NOM_RESSOURCE, SAISIE_VIDE } from "./Batiments";
import { ChampMontant } from "./Localites";
import { BlocCapitaux } from "./Tableau";


const ACQUISITIONS: [Acquisition, string][] = [
  ["plan", "Seulement la liste (équipes à recruter, rien n'est lancé)"],
  ["possede", "Déjà possédée (gratuit, équipes recrutées)"],
  ["construire", "Recruter avec les capitaux de la localité"],
  ["acheterCapitaux", "Acheter les capitaux en po, puis recruter"],
  ["acheterBatiment", "Acheter l'organisation constituée (prix en po, sans durée)"],
];


export function Organisations({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageOrganisations | null>(null);
  const [selection, setSelection] = useState<number | null>(null);
  const [creation, setCreation] = useState(false);

  useEffect(() => {
    api.pageOrganisations().then(setP);
  }, [outils.version]);

  if (!p) return null;
  const toutes = p.groupes.flatMap((g) => g.organisations);
  const choisie = toutes.find((o) => o.id === selection) ?? (creation ? undefined : toutes[0]);
  const peutCreer = p.localites.length > 0;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Organisations</h1>
        <p>Équipes d'employés : appartenance, lieu de travail, recrutement (Guide de campagne p. 92-95)</p>
      </div>

      <div className="maitre-detail">
        <aside className="liste-maitre" aria-label="Liste des organisations">
          <button
            className={"bouton nouvelle" + (creation ? " actif" : "")}
            disabled={!peutCreer}
            onClick={() => {
              setCreation(true);
              setSelection(null);
            }}
          >
            {icones.plus({})} Nouvelle organisation
          </button>
          {!peutCreer && <p className="doux">Créez d'abord une localité.</p>}
          {p.groupes.map((g) => (
            <div className="groupe-liste" key={g.localite}>
              <span className="titre-groupe">{g.localite}</span>
              {g.organisations.map((o) => {
                const actif = !creation && choisie?.id === o.id;
                return (
                  <button
                    key={o.id}
                    className={"element-liste" + (actif ? " actif" : "")}
                    aria-current={actif ? "true" : undefined}
                    onClick={() => {
                      setSelection(o.id);
                      setCreation(false);
                    }}
                  >
                    <span className="nom">{o.nom}</span>
                    <span className="doux">{o.equipes} équipe(s), {o.effectif} pers.</span>
                  </button>
                );
              })}
            </div>
          ))}
        </aside>

        <div className="detail">
          {creation ? (
            <Creation
              p={p}
              outils={outils}
              existants={new Set(toutes.map((o) => o.id))}
              apres={async (avant) => {
                const apres = await api.pageOrganisations();
                setCreation(false);
                setSelection(apres?.groupes.flatMap((g) => g.organisations).find((o) => !avant.has(o.id))?.id ?? null);
              }}
            />
          ) : choisie ? (
            <Detail key={choisie.id} id={choisie.id} p={p} outils={outils} />
          ) : (
            <div className="carte vide">
              <p>Aucune organisation pour l'instant. Créez-en une pour y recruter des équipes.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------ création

function Creation(props: { p: PageOrganisations; outils: Outils; existants: Set<number>; apres: (avant: Set<number>) => void }) {
  const { p } = props;
  const [s, setS] = useState<SaisieBatiment>({ ...SAISIE_VIDE, localite: p.localites[0]?.id ?? null });
  const [ap, setAp] = useState<ApercuCreation | null>(null);
  const maj = (c: Partial<SaisieBatiment>) => setS((x) => ({ ...x, ...c }));

  useEffect(() => {
    let actuel = true;
    if (!s.modele) setAp(null);
    else api.apercuCreationOrganisation(s).then((x) => actuel && setAp(x));
    return () => {
      actuel = false;
    };
  }, [s.localite, s.modele, s.acquisition, s.facteur, s.seulementManquants]);

  const modele = p.modeles.find((m) => m.id === s.modele);
  const avecFacteur = s.acquisition === "construire" || s.acquisition === "acheterCapitaux";

  return (
    <section className="carte section">
      <h2>Nouvelle organisation</h2>
      <form
        className="formulaire"
        onSubmit={async (e) => {
          e.preventDefault();
          const r = await props.outils.agir(api.creerOrganisation(s));
          if (r?.ok) props.apres(props.existants);
        }}
      >
        <label htmlFor="org-loc">Localité</label>
        <select id="org-loc" className="champ moyen" value={s.localite ?? ""} onChange={(e) => maj({ localite: Number(e.target.value), payeur: null })}>
          {p.localites.map((l) => (
            <option key={l.id} value={l.id}>{l.libelle}</option>
          ))}
        </select>

        <label htmlFor="org-modele">Modèle</label>
        <div className="pile">
          <select id="org-modele" className="champ" value={s.modele ?? ""} onChange={(e) => maj({ modele: e.target.value || null })}>
            <option value="">aucun (organisation vide)</option>
            {p.modeles.map((m) => (
              <option key={m.id} value={m.id}>{m.libelle}</option>
            ))}
          </select>
          {modele?.personnel && (
            <span className="ligne" style={{ gap: 10 }}>
              <span className="badge perso" style={{ marginLeft: 0 }}>Personnalisé</span>
              <span className="doux">Modèle personnel : équipes comme enregistrées.</span>
              <button
                type="button"
                className="bouton discret"
                onClick={async () => {
                  const r = await props.outils.agir(api.supprimerModeleOrganisation(modele.id));
                  if (r?.ok) maj({ modele: null });
                }}
              >
                Supprimer ce modèle
              </button>
            </span>
          )}
        </div>

        <label htmlFor="org-nom">Nom</label>
        <input id="org-nom" className="champ" placeholder="ex. Les Ombres du port" value={s.nom} onChange={(e) => maj({ nom: e.target.value })} />
        <label htmlFor="org-classement">Classement</label>
        <input id="org-classement" className="champ" placeholder="guilde, culte, compagnie…" value={s.classement} onChange={(e) => maj({ classement: e.target.value })} />

        {ap && (
          <>
            <span className="etiquette-grille">Équipes</span>
            <span className="doux">{ap.composition} <em>({ap.source})</em></span>
            <span className="etiquette-grille">Acquisition</span>
            <div className="pile" role="radiogroup" aria-label="Acquisition">
              {ACQUISITIONS.map(([a, nom]) => (
                <label key={a} className="choix-radio">
                  <input type="radio" name="acq-org" checked={s.acquisition === a} onChange={() => maj({ acquisition: a })} /> {nom}
                </label>
              ))}
              {s.acquisition === "plan" && (
                <span className="doux">Les équipes sont créées « à recruter », sans lieu de travail ; lancez-les ensuite une par une.</span>
              )}
              {s.acquisition === "possede" && <span className="doux">Toutes les équipes sont créées recrutées, sans rien payer.</span>}
              {avecFacteur && (
                <>
                  <span className="ligne" style={{ gap: 10 }}>
                    <span className="doux">Accélération</span>
                    <ChoixFacteur id="org-facteur" valeur={s.facteur} changer={(f) => maj({ facteur: f })} capital="Influence" />
                  </span>
                  <span>Capitaux nécessaires : <strong>{texteCapitaux(ap.besoin)}</strong></span>
                  <span className="doux">
                    Un PJ doit être présent. Chaque jour, les recrutements puisent dans les réserves de la localité, dans la limite de dépense (partagée avec les chantiers).
                  </span>
                </>
              )}
              {s.acquisition === "acheterCapitaux" && (
                <>
                  <label className="choix-radio">
                    <input type="checkbox" checked={s.seulementManquants} onChange={(e) => maj({ seulementManquants: e.target.checked })} /> N'acheter que ce qui manque dans les réserves
                  </label>
                  <span>À acheter : <strong>{texteCapitaux(ap.aAcheter)}</strong> = <strong>{ap.prixCapitauxPo} po</strong></span>
                </>
              )}
              {s.acquisition === "acheterBatiment" && (
                <span className="ligne" style={{ gap: 10 }}>
                  <span className="doux">Prix payé</span>
                  <ChampMontant id="org-prix" valeur={s.prix} changer={(v) => maj({ prix: v })} />
                  <span className="doux">prix du livre : {ap.prixLivrePo} po (vide = ce prix)</span>
                </span>
              )}
              {(s.acquisition === "acheterCapitaux" || s.acquisition === "acheterBatiment") && (
                <span className="ligne" style={{ gap: 10 }}>
                  <span className="doux">Payé par</span>
                  <ChoixPayeur id="org-payeur" valeur={s.payeur} changer={(x) => maj({ payeur: x })} tresorerie={ap.tresorerie} payeurs={p.payeurs} />
                </span>
              )}
            </div>
          </>
        )}

        <span />
        <div>
          <button type="submit" className="bouton principal" disabled={!s.nom.trim() || s.localite === null}>
            Créer l'organisation
          </button>
        </div>
      </form>
    </section>
  );
}

// ------------------------------------------------------------------ fiche

function Detail({ id, p, outils }: { id: number; p: PageOrganisations; outils: Outils }) {
  const [o, setO] = useState<FicheOrganisation | null>(null);
  const [facteur, setFacteur] = useState(1);
  const [payeur, setPayeur] = useState<number | null>(null);
  const [ajout, setAjout] = useState(false);
  const [recherche, setRecherche] = useState("");
  const [choix, setChoix] = useState("");
  const [deja, setDeja] = useState(false);
  const [delocOrg, setDelocOrg] = useState(false);

  useEffect(() => {
    api.ficheOrganisation(id).then(setO);
  }, [id, outils.version]);

  if (!o) return null;
  const agir = (a: ActionOrganisation) => outils.agir(api.actionOrganisation(a));
  const rech = recherche.trim().toLowerCase();
  const equipes = p.equipes.filter((e) => !rech || e.nom.toLowerCase().includes(rech));

  return (
    <>
      <section className="carte fiche-localite">
        <div className="tete">
          <h2 className="nom">{o.nom}</h2>
          {o.classement && <span className="doux">{o.classement}</span>}
          {o.modele && <span className="badge perso">modèle : {o.modele}</span>}
        </div>
        <span className="doux">Siège : {o.siege}</span>
        <strong style={{ color: "var(--or-clair)" }}>{o.effectif}</strong>
        <div>
          <span className="etiquette-champ">Reste à dépenser</span>
          <BlocCapitaux c={o.reste} />
        </div>
        <div className="ligne" style={{ gap: 10 }}>
          <span className="doux">Recettes (bonus maximum par type) :</span>
          {o.bonus.length === 0 && <span className="doux">aucune recette active</span>}
          {o.bonus.map((x) => (
            <span key={x.ressource} className="puce" style={{ color: COULEUR_RESSOURCE[x.ressource] }}>
              {NOM_RESSOURCE[x.ressource]} +{x.valeur}
            </span>
          ))}
        </div>
        <span className="doux">{o.gestionnaires}</span>
        <BlocVente v={o.vente} outils={outils} />
        <div>
          <button
            className="bouton discret"
            title="Équipes de cette organisation, réutilisables dans toutes vos campagnes (un modèle du même nom est remplacé)"
            onClick={() => outils.agir(api.enregistrerModeleOrganisation(o.id))}
          >
            ★ Enregistrer comme modèle personnel
          </button>
        </div>
      </section>

      <section className="carte ligne" style={{ gap: 14 }}>
        <label className="doux" htmlFor="org-det-facteur">Accélération des prochains recrutements</label>
        <ChoixFacteur id="org-det-facteur" valeur={facteur} changer={setFacteur} capital="Influence" />
        <label className="doux" htmlFor="org-det-payeur">Achats payés par</label>
        <ChoixPayeur id="org-det-payeur" valeur={payeur} changer={setPayeur} tresorerie={o.tresorerie} payeurs={o.payeurs} />
      </section>

      {o.equipes.map((e) => (
        <Equipe key={e.id} e={e} o={o} facteur={facteur} payeur={payeur} agir={agir} outils={outils} />
      ))}

      {o.jetUnique && o.equipesRecrutees > 1 && (
        <section className="carte pile">
          {!delocOrg ? (
            <div>
              <button className="bouton" onClick={() => setDelocOrg(true)}>
                Délocaliser toute l'organisation ({o.equipesRecrutees} équipes recrutées)…
              </button>
            </div>
          ) : (
            <FormulaireDeloc
              titre="Délocaliser toute l'organisation"
              perso="Règle personnalisée : un seul jet de 1d6 pour la préparation de toutes les équipes recrutées. Prix proposé : la moitié de leurs prix de recrutement (GC p. 95)."
              localites={o.localites}
              prixInitial={o.prixDelocOrganisation}
              annuler={() => setDelocOrg(false)}
              valider={async (vers, preparation, voyage, prix) => {
                const r = await agir({ type: "delocaliserOrganisation", organisation: o.id, vers, preparation, voyage, payeur, prix });
                if (r?.ok) setDelocOrg(false);
              }}
            />
          )}
        </section>
      )}

      {ajout ? (
        <section className="carte ligne operation">
          <input className="champ moyen" aria-label="Chercher une équipe" placeholder="chercher une équipe" value={recherche} onChange={(e) => setRecherche(e.target.value)} />
          <select className="champ" aria-label="Équipe à ajouter" style={{ maxWidth: 460 }} value={choix} onChange={(e) => setChoix(e.target.value)}>
            <option value="">Choisir…</option>
            {equipes.map((x) => (
              <option key={x.id} value={x.id}>{x.libelle}</option>
            ))}
          </select>
          <label className="choix-radio"><input type="checkbox" checked={deja} onChange={(e) => setDeja(e.target.checked)} /> déjà recrutée</label>
          <button className="bouton principal" disabled={!choix} onClick={() => agir({ type: "ajouterEquipe", organisation: o.id, reference: choix, deja })}>
            Ajouter l'équipe
          </button>
          <button className="bouton" onClick={() => setAjout(false)}>Fermer</button>
        </section>
      ) : (
        <div>
          <button className="bouton" onClick={() => { setAjout(true); setChoix(""); }}>{icones.plus({})} Ajouter une équipe</button>
        </div>
      )}
    </>
  );
}

function FormulaireDeloc(props: {
  titre: string;
  perso?: string;
  localites: { id: number; libelle: string }[];
  prixInitial: string;
  annuler: () => void;
  valider: (vers: number, preparation: string, voyage: string, prix: string) => void;
}) {
  const [vers, setVers] = useState<number | "">("");
  const [prep, setPrep] = useState("");
  const [voyage, setVoyage] = useState("");
  const [prix, setPrix] = useState(props.prixInitial);
  return (
    <form
      className="pile"
      onSubmit={(e) => {
        e.preventDefault();
        if (vers !== "") props.valider(vers, prep, voyage, prix);
      }}
    >
      <strong style={{ color: "var(--or)" }}>
        {props.titre} {props.perso && <span className="badge perso">Personnalisé</span>}
      </strong>
      <span className="doux">
        {props.perso ??
          "Délocaliser (GC p. 95) : moitié du prix de recrutement, 1d6 jours de préparation par groupe (½ recettes), puis le temps du voyage (aucune recette)."}
      </span>
      <div className="ligne operation">
        <div>
          <label className="etiquette-champ" htmlFor="deloc-vers">Vers</label>
          <select id="deloc-vers" className="champ moyen" value={vers} onChange={(e) => setVers(e.target.value === "" ? "" : Number(e.target.value))}>
            <option value="">Choisir…</option>
            {props.localites.map((l) => (
              <option key={l.id} value={l.id}>{l.libelle}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="deloc-prep">Préparation (jet de 1d6)</label>
          <input id="deloc-prep" className="champ mini" inputMode="numeric" placeholder="1 à 6" value={prep} onChange={(e) => setPrep(e.target.value)} />
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="deloc-voyage">Voyage (jours)</label>
          <input id="deloc-voyage" className="champ mini" inputMode="numeric" placeholder="ex. 4" value={voyage} onChange={(e) => setVoyage(e.target.value)} />
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="deloc-prix">Prix</label>
          <ChampMontant id="deloc-prix" valeur={prix} changer={setPrix} />
        </div>
      </div>
      <div className="ligne" style={{ gap: 8 }}>
        <button type="submit" className="bouton principal" disabled={vers === "" || !prep.trim() || !voyage.trim()}>Lancer la délocalisation</button>
        <button type="button" className="bouton" onClick={props.annuler}>Annuler</button>
        <span className="doux">Payé par le choix « Achats payés par » ci-dessus.</span>
      </div>
    </form>
  );
}

// ------------------------------------------------------------------ équipe

function Equipe(props: {
  e: FicheEquipe;
  o: FicheOrganisation;
  facteur: number;
  payeur: number | null;
  agir: (a: ActionOrganisation) => Promise<unknown>;
  outils: Outils;
}) {
  const { e, o, agir } = props;
  const [amelioration, setAmelioration] = useState("");
  const [deloc, setDeloc] = useState(false);
  const choisie = e.ameliorations.find((a) => a.id === amelioration);
  const regler = (c: Partial<{ ressource: RessourceId | null; revenusActifs: boolean }>) =>
    agir({ type: "reglerEquipe", equipe: e.id, ressource: e.ressource, revenusActifs: e.revenusActifs, ...c });
  const couleur =
    e.statut === "recrutee" ? "var(--vert)" : e.statut === "a_recruter" ? "var(--texte-doux)" : e.statut === "en_delocalisation" ? "var(--travail)" : "var(--orange)";

  return (
    <section className="carte salle-carte pile">
      <div className="ligne" style={{ gap: 10 }}>
        <strong style={{ fontSize: 16, color: "var(--texte-fort)" }}>{e.nom}</strong>
        {e.typeEquipe && <span className="doux">{e.typeEquipe}</span>}
        <span className="badge" style={{ color: couleur, border: `1px solid ${couleur}`, marginLeft: 0 }}>{e.statutTexte}</span>
        {e.ailleurs && <span className="badge perso">à {e.ailleurs}</span>}
        <span className="badge" style={{ color: "var(--or)", border: "1px solid var(--or)" }}>{e.effectif} pers.</span>
        <span className="doux">{e.source}</span>
        {e.pointAConfirmer && <span className="badge confirmer">À confirmer : {e.pointAConfirmer}</span>}
      </div>
      <span className="doux">{e.infos}</span>

      <div className="ligne" style={{ gap: 10 }}>
        <label className="doux" htmlFor={`eq-org-${e.id}`}>Membre de</label>
        <select
          id={`eq-org-${e.id}`}
          className="champ moyen"
          value={e.organisation}
          onChange={(ev) => agir({ type: "affecterEquipe", equipe: e.id, organisation: Number(ev.target.value), lieu: e.lieu })}
        >
          {o.organisations.map((x) => (
            <option key={x.id} value={x.id}>{x.libelle}</option>
          ))}
        </select>
        <label className="doux" htmlFor={`eq-lieu-${e.id}`}>travaille</label>
        <select
          id={`eq-lieu-${e.id}`}
          className="champ moyen"
          disabled={e.statut === "en_delocalisation"}
          value={e.lieu ?? ""}
          onChange={(ev) => agir({ type: "affecterEquipe", equipe: e.id, organisation: e.organisation, lieu: ev.target.value === "" ? null : Number(ev.target.value) })}
        >
          <option value="">{e.moitieSansBatiment ? "sans bâtiment (recettes ÷ 2)" : "sans bâtiment"}</option>
          {e.lieux.map((l) => (
            <option key={l.id} value={l.id}>{l.libelle}</option>
          ))}
        </select>
      </div>

      {e.admises.length > 0 && (
        <div className="ligne" style={{ gap: 12 }}>
          <span className="doux">Génère :</span>
          {e.admises.map((x) => (
            <label key={x} className="choix-radio" style={{ color: COULEUR_RESSOURCE[x] }}>
              {/* un seul type de revenu à la fois ; recliquer le type choisi le décoche */}
              <input type="radio" name={`eq-res-${e.id}`} checked={e.ressource === x} onClick={() => regler({ ressource: e.ressource === x ? null : x })} onChange={() => undefined} />
              {NOM_RESSOURCE[x]}
            </label>
          ))}
          <label className="choix-radio"><input type="checkbox" checked={e.revenusActifs} onChange={(ev) => regler({ revenusActifs: ev.target.checked })} /> revenus actifs</label>
        </div>
      )}

      {e.statut === "a_recruter" && (
        <div className="ligne" style={{ gap: 8 }}>
          <button className="bouton" disabled={!e.constructible} onClick={() => agir({ type: "lancerRecrutement", equipe: e.id, facteur: props.facteur })}>
            Lancer le recrutement{props.facteur > 1 ? ` (×${props.facteur})` : ""}
          </button>
          {e.prixAchatPo !== null && (
            <button className="bouton" title="Payé par le choix « Achats payés par » ci-dessus (GC p. 93)" onClick={() => agir({ type: "acheterEquipe", equipe: e.id, payeur: props.payeur })}>
              Acquérir d'un coup ({e.prixAchatPo} po)
            </button>
          )}
          <button className="bouton" title="Marque l'équipe recrutée sans rien payer" onClick={() => agir({ type: "equipePossedee", equipe: e.id })}>Déjà recrutée</button>
          <button className="bouton discret" onClick={() => agir({ type: "retirerEquipe", equipe: e.id })}>Retirer</button>
        </div>
      )}

      {e.chantier && (
        <div className="pile">
          <div className="ligne" style={{ gap: 16 }}>
            <span className="doux">capitaux {e.chantier.capitauxFaits}/{e.chantier.capitauxTotal}</span>
            <div className="barre" style={{ width: 160 }}><div style={{ width: `${Math.min(100, (e.chantier.capitauxFaits / Math.max(1, e.chantier.capitauxTotal)) * 100)}%` }} /></div>
            <span className="doux">jours {e.chantier.jours}/{e.chantier.duree}</span>
            <div className="barre" style={{ width: 160 }}><div style={{ width: `${e.chantier.duree === 0 ? 100 : Math.min(100, (e.chantier.jours / e.chantier.duree) * 100)}%` }} /></div>
            <span className="doux">{e.chantier.estimation}</span>
          </div>
          {e.chantier.manque && <span className="doux">{e.chantier.manque}</span>}
        </div>
      )}

      {e.statut === "recrutee" && (
        <div className="ligne" style={{ gap: 8 }}>
          {e.ameliorations.length > 0 && (
            <>
              <select className="champ moyen" aria-label="Améliorer en…" value={amelioration} onChange={(ev) => setAmelioration(ev.target.value)}>
                <option value="">Améliorer en…</option>
                {e.ameliorations.map((a) => (
                  <option key={a.id} value={a.id}>{a.nom}</option>
                ))}
              </select>
              {choisie?.detail && <span className="doux">{choisie.detail}</span>}
              {choisie && (
                <button className="bouton" onClick={() => agir({ type: "ameliorerEquipe", equipe: e.id, vers: choisie.id, facteur: props.facteur })}>
                  Lancer l'amélioration
                </button>
              )}
            </>
          )}
          <button className="bouton" onClick={() => setDeloc(!deloc)}>Délocaliser…</button>
          <button className="bouton discret" onClick={() => agir({ type: "retirerEquipe", equipe: e.id })}>Retirer</button>
        </div>
      )}
      {e.statut === "en_delocalisation" && (
        <span className="doux">L'équipe arrivera sans lieu de travail ; affectez-la ensuite à un bâtiment.</span>
      )}

      {e.vente && <BlocVente v={e.vente} outils={props.outils} />}

      {deloc && e.delocalisable && (
        <div className="carte encadre">
          <FormulaireDeloc
            titre="Délocaliser"
            localites={o.localites.filter((l) => l.id !== o.localite || e.ailleurs !== null)}
            prixInitial={e.prixDeloc}
            annuler={() => setDeloc(false)}
            valider={async (vers, preparation, voyage, prix) => {
              const r = (await agir({ type: "delocaliser", equipe: e.id, vers, preparation, voyage, payeur: props.payeur, prix })) as { ok?: boolean } | null;
              if (r?.ok) setDeloc(false);
            }}
          />
        </div>
      )}
    </section>
  );
}
