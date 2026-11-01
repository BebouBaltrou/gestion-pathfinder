import { useEffect, useState } from "react";
import type { Outils, Page } from "../App";
import { api, Capitaux, formaterPo, TableauDeBord } from "../api";

const CAPITAUX: [keyof Capitaux, string, string][] = [
  ["marchandises", "Marchandises", "var(--marchandises)"],
  ["influence", "Influence", "var(--influence)"],
  ["travail", "Travail", "var(--travail)"],
  ["magie", "Magie", "var(--magie)"],
];

export function BlocCapitaux({ c }: { c: Capitaux }) {
  const max = Math.max(20, ...CAPITAUX.map(([k]) => c[k]));
  return (
    <div className="capitaux">
      {CAPITAUX.map(([k, nom, couleur]) => (
        <div className="capital" key={k}>
          <span className="nom-capital">{nom}</span>
          <span className="nombre" style={{ color: couleur }}>{c[k]}</span>
          <div className="jauge">
            <div style={{ width: `${(c[k] / max) * 100}%`, background: couleur }} />
          </div>
        </div>
      ))}
    </div>
  );
}

const pluriel = (n: number, un: string, plusieurs: string) => `${n.toLocaleString("fr-FR")} ${n > 1 ? plusieurs : un}`;

export function Tableau({ outils }: { outils: Outils }) {
  const [t, setT] = useState<TableauDeBord | null>(null);

  useEffect(() => {
    api.tableauDeBord().then(setT);
  }, [outils.version]);

  if (!t) return null;
  const totalCap = t.capitaux.marchandises + t.capitaux.influence + t.capitaux.travail + t.capitaux.magie;
  const attente = Math.max(0, ...t.localites.map((l) => l.recettesEnAttente));

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Tableau de bord</h1>
        <p>Ce qui demande votre attention aujourd'hui</p>
      </div>

      <section className="grille-chiffres" aria-label="Chiffres clés">
        <div className="carte chiffre">
          <span className="etiquette">Trésoreries des localités</span>
          <span className="valeur" style={{ color: "var(--po)" }}>{formaterPo(t.tresorerieLocalites.pc)}</span>
          <span className="detail">Comptes des PJ : {formaterPo(t.comptesPersonnages.pc)}</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Capitaux en réserve</span>
          <span className="valeur">{totalCap}</span>
          <span className="detail">{pluriel(t.localites.length, "localité", "localités")}</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Employés</span>
          <span className="valeur">{t.employes.toLocaleString("fr-FR")}</span>
          <span className="detail">{t.employesAVenir > 0 ? `+${t.employesAVenir} en recrutement` : "selon les tailles d'équipe du livre"}</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Recettes en attente</span>
          <span className="valeur" style={{ color: attente > 0 ? "var(--orange)" : undefined }}>
            {attente > 0 ? pluriel(attente, "jour", "jours") : "à jour"}
          </span>
          <span className="detail">
            {t.transferts > 0 ? pluriel(t.transferts, "transfert en route", "transferts en route") : "aucun transfert en route"}
          </span>
        </div>
      </section>

      {t.prestige.map((p) => (
        <div className="carte prestige" key={p.localite}>
          <strong>Test de prestige à faire à {p.nom} : DD {p.dd}</strong>
          <p className="doux">
            Retour après 30 jours ou plus sans gestionnaire : 1d20 + Prestige contre ce DD. En cas d'échec, les entreprises
            de la localité risquent d'échapper à votre contrôle (GC p. 82) ; l'application note le résultat, à vous d'en
            tirer les conséquences avec le MJ.
          </p>
          <div className="puces">
            <button className="bouton" onClick={() => outils.agir(api.resoudrePrestige(p.localite, true))}>Réussi</button>
            <button className="bouton danger" onClick={() => outils.agir(api.resoudrePrestige(p.localite, false))}>Raté</button>
          </div>
        </div>
      ))}

      <div className="deux-colonnes">
        <section className="colonne-large">
          <h2>Localités</h2>
          {t.localites.length === 0 && (
            <div className="carte vide">
              <p>Aucune localité pour l'instant. Créez-en une pour y construire bâtiments et organisations.</p>
              <button className="bouton principal" onClick={() => outils.aller("localites")}>Créer une localité</button>
            </div>
          )}
          {t.localites.map((l) => (
            <article className="carte carte-localite" key={l.id}>
              <div className="tete">
                <span className="nom">{l.nom}</span>
                <span className="doux">{l.categorie}</span>
                <span className="espace" />
                <span className="doux">
                  {l.presents.length > 0 ? `Présents : ${l.presents.join(", ")}` : "aucun PJ présent"}
                </span>
              </div>
              <div className="ligne">
                <div className="tresor">
                  <span className="etiquette">Trésorerie</span>
                  <span className={"montant" + (l.tresorerie.pc < 0 ? " negatif" : "")}>{formaterPo(l.tresorerie.pc)}</span>
                  {l.salairesJour && <span className="sous">salaires −{formaterPo(l.salairesJour.pc)}/jour</span>}
                </div>
                <BlocCapitaux c={l.capitaux} />
              </div>
              <div className="puces">
                {l.batiments > 0 && <span className="puce">{pluriel(l.batiments, "bâtiment", "bâtiments")}</span>}
                {l.organisations > 0 && <span className="puce">{pluriel(l.organisations, "organisation", "organisations")}</span>}
                {l.employes > 0 && <span className="puce">{pluriel(l.employes, "employé", "employés")}</span>}
                {l.recettesEnAttente > 0 && (
                  <button className="puce or" style={{ cursor: "pointer" }} onClick={() => outils.aller("recettes")}>
                    {pluriel(l.recettesEnAttente, "jour", "jours")} de recettes à encaisser
                  </button>
                )}
                {l.absence && <span className="puce">{l.absence}</span>}
                {l.enRoute.map((x) => (
                  <span className="puce" key={x}>{x}</span>
                ))}
              </div>
            </article>
          ))}
        </section>

        <aside className="colonne-etroite">
          <section className="section">
            <h2>Alertes</h2>
            {t.alertes.length === 0 && <p className="doux">Rien à signaler.</p>}
            {t.alertes.map((a, i) => (
              <div className={"alerte" + (a.grave ? " grave" : "")} key={i}>
                <span className="point" />
                <span className="texte">{a.texte}</span>
                <button className="bouton discret" onClick={() => outils.aller(a.page as Page)}>Voir</button>
              </div>
            ))}
          </section>

          <section className="section">
            <div className="section-tete">
              <h2>En cours</h2>
              {t.travaux.length > 0 && <span className="aide">fins estimées si les réserves suffisent</span>}
            </div>
            {t.travaux.length === 0 && <p className="doux">Aucun chantier ni recrutement en cours.</p>}
            {t.travaux.length > 0 && (
              <div className="carte liste-travaux">
                {t.travaux.map((x, i) => (
                  <div className="travail" key={i}>
                    <div className="tete">
                      <button className="titre" onClick={() => outils.aller(x.page as Page)}>{x.titre}</button>
                      <span className={"fin" + (x.joursRestants === null ? " bloque" : "")}>
                        {x.joursRestants === null ? "bloqué : limite de dépense nulle" : `≈ ${x.joursRestants} j · ${x.fin}`}
                      </span>
                    </div>
                    <div className="barre">
                      <div style={{ width: `${Math.round(x.avancement * 100)}%` }} />
                    </div>
                    <span className="doux">{x.detail}</span>
                  </div>
                ))}
              </div>
            )}
          </section>

          {t.personnages.length > 0 && (
            <section className="section">
              <h2>Personnages</h2>
              <div className="carte liste-travaux">
                {t.personnages.map((p) => (
                  <div className="travail" key={p.id}>
                    <div className="tete">
                      <span className="portrait" aria-hidden="true">{p.nom.charAt(0).toUpperCase()}</span>
                      <span style={{ flex: 1, fontWeight: 500 }}>{p.nom}</span>
                      <span style={{ color: "var(--po)", fontWeight: 600 }}>{formaterPo(p.compte.pc)}</span>
                    </div>
                    <span className="doux">{p.lieu ?? "en voyage / hors localité"}</span>
                  </div>
                ))}
              </div>
            </section>
          )}
        </aside>
      </div>
    </div>
  );
}
