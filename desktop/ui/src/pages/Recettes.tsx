import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, Capitaux, EntreeRecettes, formaterPo, Points, RessourceId, VueRecettes } from "../api";
import { COULEUR_RESSOURCE, ChoixPayeur, NOM_RESSOURCE } from "./Batiments";
import { BlocCapitaux } from "./Tableau";

const ZERO: Capitaux = { marchandises: 0, influence: 0, travail: 0, magie: 0 };
const NEUVE: EntreeRecettes = { repartition: [], jets: [], conserves: ZERO, payeur: null };

const cle = (c: { genre: string; id: number }) => `${c.genre}:${c.id}`;

export function Recettes({ outils }: { outils: Outils }) {
  const [entree, setEntree] = useState<EntreeRecettes>(NEUVE);
  const [v, setV] = useState<VueRecettes | null>(null);
  const [detail, setDetail] = useState(false);

  useEffect(() => {
    let actuel = true;
    api.vueRecettes(entree).then((x) => actuel && setV(x));
    return () => {
      actuel = false;
    };
  }, [entree, outils.version]);

  if (!v) return null;
  const maj = (c: Partial<EntreeRecettes>) => setEntree((e) => ({ ...e, ...c }));

  // la saisie complète, telle que le cœur l'a retenue (valeurs par défaut comprises)
  const base = (): EntreeRecettes => ({
    ...entree,
    localite: v.localite,
    jours: v.jours,
    batiments: v.batiments.filter((b) => b.coche).map((b) => b.id),
    organisations: v.organisations.filter((o) => o.coche).map((o) => o.id),
    repartition: v.unites.map((u) => ({ cle: u.cle, points: u.points })),
    jets: v.tests.map((t) => ({ ressource: t.ressource, prendre10: t.prendre10, des: t.des })),
    conserverTout: v.conserverTout,
  });
  const basculer = (liste: "batiments" | "organisations", id: number, coche: boolean) => {
    const b = base();
    const actuels = new Set(b[liste] ?? []);
    if (coche) actuels.add(id);
    else actuels.delete(id);
    setEntree({ ...b, [liste]: [...actuels] });
  };
  const changerPoints = (indice: number, points: Points[]) => {
    const b = base();
    b.repartition = b.repartition.map((r, i) => (i === indice ? { ...r, points } : r));
    setEntree(b);
  };
  const changerJet = (ressource: RessourceId, c: Partial<{ prendre10: boolean; des: string }>) => {
    const b = base();
    b.jets = b.jets.map((j) => (j.ressource === ressource ? { ...j, ...c } : j));
    setEntree(b);
  };

  const encaisser = async () => {
    const r = await outils.agir(api.encaisserRecettes(base()));
    if (r?.ok) setEntree({ ...NEUVE, localite: v.localite });
  };

  if (v.localite === null) {
    return (
      <div className="page">
        <div className="titre-page"><h1>Recettes</h1></div>
        <div className="carte vide">
          <p>Créez d'abord une localité.</p>
          <button className="bouton principal" onClick={() => outils.aller("localites")}>Créer une localité</button>
        </div>
      </div>
    );
  }
  const bilan = v.bilan;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Recettes</h1>
        <p>Tests de capitaux des bâtiments et organisations (Guide de campagne p. 79, 84 et 94)</p>
      </div>

      <section className="carte ligne operation" style={{ gap: 16 }}>
        <div>
          <label className="etiquette-champ" htmlFor="rec-loc">Localité</label>
          <select id="rec-loc" className="champ moyen" value={v.localite} onChange={(e) => setEntree({ ...NEUVE, localite: Number(e.target.value) })}>
            {v.localites.map((l) => (
              <option key={l.id} value={l.id}>{l.libelle}</option>
            ))}
          </select>
        </div>
        <div>
          <label className="etiquette-champ" htmlFor="rec-jours">Jours à traiter</label>
          <input id="rec-jours" className="champ mini" inputMode="numeric" value={v.jours} onChange={(e) => maj({ ...base(), jours: e.target.value })} />
        </div>
        <span className="doux">
          {v.joursEnAttente === 0
            ? "aucun jour en attente : avancez le temps pour produire"
            : `${v.joursEnAttente} jour(s) en attente, à traiter dans l'ordre à partir du plus ancien`}
        </span>
      </section>

      <section className="section">
        <h2>Qui produit</h2>
        <div className="carte pile">
          {v.batiments.length + v.organisations.length === 0 ? (
            <p className="doux">
              Aucune salle construite ni équipe recrutée ne produit de recettes ici (ou leurs revenus sont désactivés).
            </p>
          ) : (
            <>
              <div className="ligne" style={{ gap: 18 }}>
                {v.batiments.map((b) => (
                  <label key={`b${b.id}`} className="choix-radio">
                    <input type="checkbox" checked={b.coche} onChange={(e) => basculer("batiments", b.id, e.target.checked)} /> {b.nom}
                  </label>
                ))}
                {v.organisations.map((o) => (
                  <label key={`o${o.id}`} className="choix-radio">
                    <input type="checkbox" checked={o.coche} onChange={(e) => basculer("organisations", o.id, e.target.checked)} /> {o.nom}
                  </label>
                ))}
              </div>
              <span className="doux">Une équipe participe si son organisation ou le bâtiment où elle travaille est coché.</span>
            </>
          )}
          {v.sansType.length > 0 && (
            <span className="doux">
              Sans type de revenu coché, donc écartées : {v.sansType.join(", ")} (cochez « Génère » dans les pages Bâtiments ou Organisations)
            </span>
          )}
        </div>
      </section>

      {v.unites.length > 0 && (
        <>
          <section className="section">
            <div className="section-tete">
              <h2>Répartition des bonus</h2>
              <span className={"badge " + (v.repartitionStricte ? "perso" : "")} style={v.repartitionStricte ? undefined : { border: "1px solid var(--bord-fort)" }}>
                {v.repartitionStricte ? "Personnalisé" : "Officiel"}
              </span>
            </div>
            <span className="doux">
              {v.repartitionStricte
                ? "Règle personnalisée : chaque unité donne tout son bonus à un seul type par jour."
                : "Règle officielle : le bonus peut aller à un type ou être réparti entre les types admis (GC p. 94)."}
            </span>
            <div className="ligne" style={{ gap: 8 }}>
              <button
                className="bouton"
                onClick={() => setEntree({ ...base(), repartition: v.unites.map((u) => ({ cle: u.cle, points: u.defaut })) })}
              >
                Tout en po quand c'est possible
              </button>
              <button
                className="bouton"
                title="Celle des dernières recettes perçues pour chaque unité"
                onClick={() => setEntree({ ...base(), repartition: v.unites.map((u) => ({ cle: u.cle, points: u.precedente ?? u.points })) })}
              >
                Reprendre la répartition précédente
              </button>
            </div>
            <div className="carte tableau-repartition">
              {v.unites.map((u, i) => {
                const pts = (res: RessourceId) => u.points.find((p) => p.ressource === res)?.points ?? 0;
                const choisi = u.points.find((p) => p.points > 0)?.ressource ?? null;
                return (
                  <div className="ligne-repartition" key={cle(u.cle)}>
                    <span className="nom">{u.libelle}</span>
                    <strong style={{ color: "var(--or-clair)" }}>+{u.bonus}</strong>
                    <div className="ligne" style={{ gap: 14 }}>
                      {v.repartitionStricte ? (
                        <>
                          {u.bonus === 0 && <span className="doux">bonus nul</span>}
                          {u.admises.map((x) => (
                            <label key={x} className="choix-radio" style={{ color: COULEUR_RESSOURCE[x] }}>
                              <input type="radio" name={`rep-${cle(u.cle)}`} checked={choisi === x} onChange={() => changerPoints(i, [{ ressource: x, points: u.bonus }])} />
                              {NOM_RESSOURCE[x]}
                            </label>
                          ))}
                        </>
                      ) : (
                        <>
                          {u.admises.map((x) => (
                            <label key={x} className="choix-radio" style={{ color: COULEUR_RESSOURCE[x] }}>
                              {NOM_RESSOURCE[x]}
                              <input
                                className="champ mini"
                                type="number"
                                min={0}
                                max={u.bonus}
                                aria-label={`${u.libelle} : ${NOM_RESSOURCE[x]}`}
                                value={pts(x)}
                                onChange={(e) => {
                                  const n = Math.max(0, Math.min(u.bonus, Number(e.target.value) || 0));
                                  changerPoints(i, u.admises.map((y) => ({ ressource: y, points: y === x ? n : pts(y) })).filter((p) => p.points > 0));
                                }}
                              />
                            </label>
                          ))}
                          {u.somme !== u.bonus && <span style={{ color: "var(--rouge)" }}>{u.somme}/{u.bonus} affectés</span>}
                        </>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          </section>

          <section className="section">
            <h2>Jets</h2>
            <div className="carte pile">
              {v.tests.length === 0 && <span className="doux">Aucun point affecté.</span>}
              {v.tests.map((t) => (
                <div className="ligne" style={{ gap: 14 }} key={t.ressource}>
                  <strong style={{ color: COULEUR_RESSOURCE[t.ressource], minWidth: 150 }}>Test de {NOM_RESSOURCE[t.ressource]}</strong>
                  <label className="choix-radio" title="Autorisé pour les tests de capitaux (GC p. 79 et 84)">
                    <input type="checkbox" checked={t.prendre10} onChange={(e) => changerJet(t.ressource, { prendre10: e.target.checked })} /> faire 10
                  </label>
                  {!t.prendre10 && (
                    <>
                      <input
                        className={"champ" + (t.erreur ? " invalide" : "")}
                        style={{ maxWidth: 300 }}
                        aria-label={`Résultats de d20 pour ${NOM_RESSOURCE[t.ressource]}`}
                        placeholder="un résultat de d20 par jour, ex. 12 5 18"
                        value={t.des}
                        onChange={(e) => changerJet(t.ressource, { des: e.target.value })}
                      />
                      {t.erreur && <span style={{ color: "var(--rouge)" }}>{t.erreur}</span>}
                    </>
                  )}
                </div>
              ))}
            </div>
          </section>
        </>
      )}

      {v.erreur && v.unites.length > 0 && <div className="alerte grave"><span className="point" /><span className="texte">{v.erreur}</span></div>}

      {bilan && (
        <section className="section">
          <h2>Résultat</h2>
          <div className="carte pile">
            {bilan.premierJour.map((t) => (
              <div key={t.ressource}>
                <div className="ligne" style={{ gap: 10 }}>
                  <strong style={{ color: COULEUR_RESSOURCE[t.ressource] }}>{NOM_RESSOURCE[t.ressource]}</strong>
                  <span className="doux">{t.lignes}</span>
                </div>
                {bilan.jours === 1 && <span className="doux">{t.prendre10 ? "faire 10 : " : "d20 : "}{t.formule}</span>}
              </div>
            ))}
            {bilan.jours > 1 && (
              <>
                <label className="choix-radio">
                  <input type="checkbox" checked={detail} onChange={(e) => setDetail(e.target.checked)} /> Détail des {bilan.jours} jours
                </label>
                {detail && (
                  <div className="liste-detail">
                    {bilan.detailJours.map((l) => (
                      <span className="doux" key={l}>{l}</span>
                    ))}
                  </div>
                )}
              </>
            )}
            <hr className="separateur" />
            <div className="ligne" style={{ gap: 14 }}>
              <span>Total sur {bilan.jours} jour(s) :</span>
              <strong style={{ color: "var(--po)" }}>{formaterPo(bilan.poBrut.pa)}</strong>
              <BlocCapitaux c={bilan.capitauxBruts} />
            </div>
            {bilan.absence && <span style={{ color: "var(--orange)" }}>{bilan.absence}</span>}
            <div className="ligne" style={{ gap: 14 }}>
              <strong>Gagné :</strong>
              <strong style={{ color: "var(--po)", fontSize: 18 }}>{formaterPo(bilan.poNet.pa)}</strong>
              <BlocCapitaux c={bilan.capitauxNets} />
            </div>

            {bilan.conservables.length > 0 && (
              <div className="pile">
                <strong style={{ color: "var(--or)" }}>
                  {v.gratuits ? "Capitaux produits encaissés" : "Capitaux produits à payer (Coût produit)"}
                </strong>
                {v.gratuits ? (
                  <span className="doux">
                    <span className="badge perso" style={{ marginLeft: 0 }}>Personnalisé</span> Option des Paramètres : encaissés sans payer. Vous pouvez quand même en garder moins.
                  </span>
                ) : (
                  <span className="doux">
                    Le test donne le droit de produire ces capitaux ; chaque unité gardée se paie au Coût produit (Table 2-1, GC p. 78-79 et 94). Vous pouvez en garder moins.
                  </span>
                )}
                <label className="choix-radio">
                  <input type="checkbox" checked={v.conserverTout} onChange={(e) => setEntree({ ...base(), conserverTout: e.target.checked })} /> tout garder
                </label>
                <div className="ligne" style={{ gap: 18 }}>
                  {bilan.conservables.map((c) => (
                    <label key={c.capital} className="choix-radio" style={{ color: COULEUR_RESSOURCE[c.capital] }}>
                      {NOM_RESSOURCE[c.capital]}
                      <input
                        className="champ mini"
                        type="number"
                        min={0}
                        max={c.max}
                        aria-label={`Garder : ${NOM_RESSOURCE[c.capital]}`}
                        value={c.garde}
                        onChange={(e) => {
                          const n = Math.max(0, Math.min(c.max, Number(e.target.value) || 0));
                          const garde = Object.fromEntries(bilan.conservables.map((x) => [x.capital, x.garde]));
                          garde[c.capital] = n;
                          setEntree({
                            ...base(),
                            conserverTout: false,
                            conserves: { ...ZERO, ...garde } as Capitaux,
                          });
                        }}
                      />
                      <span className="doux">/ {c.max}{v.gratuits ? "" : ` (${c.coutUnitePo} po l'unité)`}</span>
                    </label>
                  ))}
                </div>
                {!v.gratuits && (
                  <div className="ligne" style={{ gap: 10 }}>
                    <span className="doux">Payés par</span>
                    <ChoixPayeur id="rec-payeur" valeur={entree.payeur} changer={(p) => setEntree({ ...base(), payeur: p })} tresorerie={v.tresorerie} payeurs={v.payeurs} />
                  </div>
                )}
              </div>
            )}

            <div className="ligne" style={{ gap: 14 }}>
              {bilan.coutConserves.pa > 0 && (
                <>
                  <span>Coût des capitaux gardés :</span>
                  <strong style={{ color: "var(--po)" }}>{formaterPo(bilan.coutConserves.pa)}</strong>
                </>
              )}
              <button className="bouton principal" disabled={!v.encaissable} onClick={encaisser}>
                Encaisser {bilan.jours} jour{bilan.jours > 1 ? "s" : ""}
              </button>
            </div>
          </div>
        </section>
      )}
    </div>
  );
}
