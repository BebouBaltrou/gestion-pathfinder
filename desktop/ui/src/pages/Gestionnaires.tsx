import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, ActionGestionnaire, formaterPo, PageGestionnaires } from "../api";
import { ChampMontant } from "./Localites";

export function Gestionnaires({ outils }: { outils: Outils }) {
  const [p, setP] = useState<PageGestionnaires | null>(null);
  const [reference, setReference] = useState<string | null>(null);
  const [nom, setNom] = useState("");
  const [salaire, setSalaire] = useState("");
  const [localite, setLocalite] = useState<number | null>(null);
  const [choix, setChoix] = useState<Record<number, string>>({});

  useEffect(() => {
    api.pageGestionnaires().then(setP);
  }, [outils.version]);

  if (!p) return null;
  const loc = localite ?? p.localites[0]?.id ?? null;
  const exemple = p.exemples.find((x) => x.id === reference);
  const agir = (a: ActionGestionnaire) => outils.agir(api.actionGestionnaire(a));
  const cleChoix = (c: { genre: string; id: number }) => `${c.genre}:${c.id}`;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Gestionnaires</h1>
        <p>Employés qualifiés qui gèrent bâtiments et organisations, payés chaque jour en po (Guide de campagne p. 89)</p>
      </div>

      {p.salaires.length > 0 && (
        <section className="section">
          <h2>Salaires quotidiens</h2>
          <div className="ligne" style={{ gap: 14, alignItems: "stretch" }}>
            {p.salaires.map((s) => (
              <div className="carte pile" key={s.localite} style={{ minWidth: 220 }}>
                <strong style={{ color: "var(--texte-fort)" }}>{s.localite}</strong>
                <span>Salaires : <strong style={{ color: "var(--po)" }}>{formaterPo(s.salairesJour.pa)}</strong> / jour</span>
                <span>Trésorerie : <strong style={{ color: s.negative ? "var(--rouge)" : "var(--po)" }}>{formaterPo(s.tresorerie.pa)}</strong></span>
                {s.negative ? (
                  <span style={{ color: "var(--rouge)" }}>trésorerie négative</span>
                ) : (
                  <span className="doux">≈ {s.joursDAvance} jours de salaires d'avance</span>
                )}
              </div>
            ))}
          </div>
        </section>
      )}

      <section className="carte section">
        <h2>Embaucher</h2>
        {p.localites.length === 0 ? (
          <p className="doux">Créez d'abord une localité.</p>
        ) : (
          <form
            className="formulaire"
            onSubmit={async (e) => {
              e.preventDefault();
              if (loc === null) return;
              const r = await agir({ type: "embaucher", reference, nom, salaire, localite: loc });
              if (r?.ok) setNom("");
            }}
          >
            <label htmlFor="ges-ref">Exemple du livre</label>
            <select
              id="ges-ref"
              className="champ"
              value={reference ?? ""}
              onChange={(e) => {
                const id = e.target.value || null;
                setReference(id);
                const x = p.exemples.find((y) => y.id === id);
                if (x) {
                  setSalaire(x.salaire);
                  if (!nom.trim()) setNom(x.nom);
                }
              }}
            >
              <option value="">aucun (personnalisé)</option>
              {p.exemples.map((x) => (
                <option key={x.id} value={x.id}>{x.libelle}</option>
              ))}
            </select>
            <label htmlFor="ges-nom">Nom</label>
            <input id="ges-nom" className="champ" placeholder="ex. Ameiko" value={nom} onChange={(e) => setNom(e.target.value)} />
            <label htmlFor="ges-salaire">Salaire par jour</label>
            <ChampMontant id="ges-salaire" valeur={salaire} changer={setSalaire} />
            <label htmlFor="ges-loc">Payé par la trésorerie de</label>
            <select id="ges-loc" className="champ moyen" value={loc ?? ""} onChange={(e) => setLocalite(Number(e.target.value))}>
              {p.localites.map((l) => (
                <option key={l.id} value={l.id}>{l.libelle}</option>
              ))}
            </select>
            {exemple && (
              <>
                <span className="etiquette-grille">Compétences</span>
                <span className="doux">{exemple.competences}</span>
              </>
            )}
            <span />
            <div>
              <button type="submit" className="bouton principal" disabled={!nom.trim() || !salaire.trim() || loc === null}>Embaucher</button>
            </div>
          </form>
        )}
      </section>

      <section className="section">
        <h2>Gestionnaires employés</h2>
        {p.gestionnaires.length === 0 && <p className="doux">Aucun gestionnaire pour l'instant.</p>}
        {p.gestionnaires.map((g) => (
          <div className="carte pile" key={g.id}>
            <div className="ligne" style={{ gap: 12 }}>
              <strong style={{ fontSize: 17, color: "var(--texte-fort)" }}>{g.nom}</strong>
              {g.exemple && <span className="doux">{g.exemple}</span>}
              <strong style={{ color: "var(--po)" }}>{formaterPo(g.salaireJour.pa)}</strong>
              <span className="doux">/ jour</span>
              {!g.actif && <span className="badge" style={{ border: "1px solid var(--bord-fort)" }}>inactif (non payé)</span>}
            </div>
            <div className="ligne" style={{ gap: 14 }}>
              <label className="choix-radio" title="Un gestionnaire inactif n'est plus payé">
                <input type="checkbox" checked={g.actif} onChange={(e) => agir({ type: "regler", id: g.id, actif: e.target.checked, localite: g.localite })} /> actif
              </label>
              <label className="doux" htmlFor={`ges-paye-${g.id}`}>Payé par</label>
              <select
                id={`ges-paye-${g.id}`}
                className="champ moyen"
                value={g.localite}
                onChange={(e) => agir({ type: "regler", id: g.id, actif: g.actif, localite: Number(e.target.value) })}
              >
                {p.localites.map((l) => (
                  <option key={l.id} value={l.id}>{l.libelle}</option>
                ))}
              </select>
              <button className="bouton discret" onClick={() => agir({ type: "renvoyer", id: g.id })}>Renvoyer</button>
            </div>
            <div className="ligne" style={{ gap: 10 }}>
              <span className="doux">S'occupe de :</span>
              {g.affectations.length === 0 && <span className="doux">rien pour l'instant</span>}
              {g.affectations.map((a) => (
                <span className="puce or" key={cleChoix(a.cible)}>
                  {a.libelle}{" "}
                  <button
                    className="bouton discret"
                    style={{ height: 22, padding: "0 4px" }}
                    aria-label={`Retirer l'affectation : ${a.libelle}`}
                    title="Retirer cette affectation"
                    onClick={() => agir({ type: "desaffecter", id: g.id, cible: a.cible })}
                  >
                    ✕
                  </button>
                </span>
              ))}
            </div>
            {g.possibles.length > 0 && (
              <div className="ligne" style={{ gap: 8 }}>
                <select
                  className="champ"
                  style={{ maxWidth: 320 }}
                  aria-label="Ajouter un bâtiment ou une organisation"
                  value={choix[g.id] ?? ""}
                  onChange={(e) => setChoix({ ...choix, [g.id]: e.target.value })}
                >
                  <option value="">Ajouter un bâtiment ou une organisation…</option>
                  {g.possibles.map((a) => (
                    <option key={cleChoix(a.cible)} value={cleChoix(a.cible)}>{a.libelle}</option>
                  ))}
                </select>
                <button
                  className="bouton"
                  disabled={!choix[g.id]}
                  onClick={async () => {
                    const a = g.possibles.find((x) => cleChoix(x.cible) === choix[g.id]);
                    if (!a) return;
                    const r = await agir({ type: "affecter", id: g.id, cible: a.cible });
                    if (r?.ok) setChoix({ ...choix, [g.id]: "" });
                  }}
                >
                  Affecter
                </button>
              </div>
            )}
          </div>
        ))}
        <p className="doux">{p.noteSalaire}</p>
      </section>
    </div>
  );
}
