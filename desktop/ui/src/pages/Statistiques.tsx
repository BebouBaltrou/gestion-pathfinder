import { useEffect, useState } from "react";
import type { Outils } from "../App";
import { api, PageStatistiques, Somme } from "../api";

const texte = (s: Somme) => s?.texte ?? "—";

function BarresComparatives({ lignes, titre }: { lignes: { nom: string; valeur: number; texte: string }[]; titre: string }) {
  const max = Math.max(1, ...lignes.map((l) => l.valeur));
  return (
    <section className="carte">
      <h2>{titre}</h2>
      {lignes.length === 0 && <p className="detail">Aucune donnée.</p>}
      {lignes.map((l) => (
        <div key={l.nom} className="ligne-barre">
          <span className="nom-barre">{l.nom}</span>
          <div className="jauge">
            <div style={{ width: `${(Math.max(0, l.valeur) / max) * 100}%`, background: "var(--po)" }} />
          </div>
          <span className="nombre">{l.texte}</span>
        </div>
      ))}
    </section>
  );
}

export function Statistiques({ outils }: { outils: Outils }) {
  const [s, setS] = useState<PageStatistiques | null>(null);

  useEffect(() => {
    api.statistiques().then((r) => setS(r && Array.isArray(r.personnages) ? r : null));
  }, [outils.version]);

  if (!s) return null;

  return (
    <div className="page">
      <div className="titre-page">
        <h1>Statistiques</h1>
        <p>Patrimoine, objets, localités et organisations</p>
      </div>

      <section className="grille-chiffres" aria-label="Chiffres clés">
        <div className="carte chiffre">
          <span className="etiquette">Patrimoine total</span>
          <span className="valeur" style={{ color: "var(--po)" }}>{texte(s.patrimoineTotal)}</span>
          <span className="detail">Comptes, objets, trésoreries et capitaux</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Personnages</span>
          <span className="valeur">{texte(s.totalPersonnages)}</span>
          <span className="detail">Comptes et objets des PJ</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Localités</span>
          <span className="valeur">{texte(s.totalLocalites)}</span>
          <span className="detail">Trésoreries et capitaux</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Objets hors PJ</span>
          <span className="valeur">{texte(s.objetsAutres)}</span>
          <span className="detail">Autres inventaires</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Effectif des organisations</span>
          <span className="valeur">{s.effectifTotal}</span>
          <span className="detail">{s.effectifAVenirTotal} à recruter ou en recrutement</span>
        </div>
        <div className="carte chiffre">
          <span className="etiquette">Masse salariale</span>
          <span className="valeur">{texte(s.masseSalarialeJour)}</span>
          <span className="detail">Par jour, gestionnaires actifs</span>
        </div>
      </section>

      <BarresComparatives
        titre="Patrimoine par personnage"
        lignes={s.personnages.map((p) => ({ nom: p.nom, valeur: p.total.pc, texte: texte(p.total) }))}
      />
      <BarresComparatives
        titre="Patrimoine par localité"
        lignes={s.localites.map((l) => ({ nom: l.nom, valeur: l.total.pc, texte: texte(l.total) }))}
      />

      <section className="carte">
        <h2>Détail des personnages</h2>
        <table className="tableau">
          <thead>
            <tr><th>Nom</th><th>Compte</th><th>Objets</th><th>Total</th></tr>
          </thead>
          <tbody>
            {s.personnages.map((p) => (
              <tr key={p.id}><td>{p.nom}</td><td>{texte(p.compte)}</td><td>{texte(p.objets)}</td><td>{texte(p.total)}</td></tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="carte">
        <h2>Détail des localités</h2>
        <table className="tableau">
          <thead>
            <tr><th>Nom</th><th>Trésorerie</th><th>Capitaux</th><th>Total</th></tr>
          </thead>
          <tbody>
            {s.localites.map((l) => (
              <tr key={l.id}><td>{l.nom}</td><td>{texte(l.tresorerie)}</td><td>{texte(l.capitaux)}</td><td>{texte(l.total)}</td></tr>
            ))}
          </tbody>
        </table>
      </section>

      <section className="carte">
        <h2>Organisations</h2>
        <table className="tableau">
          <thead>
            <tr><th>Organisation</th><th>Localité</th><th>Équipes</th><th>Effectif</th><th>À venir</th><th>Gestionnaires</th><th>Salaires / jour</th></tr>
          </thead>
          <tbody>
            {s.organisations.map((o) => (
              <tr key={o.id}>
                <td>{o.nom}</td><td>{o.localite}</td><td>{o.equipes}</td><td>{o.effectif}</td>
                <td>{o.effectifAVenir}</td><td>{o.gestionnaires}</td><td>{texte(o.masseSalarialeJour)}</td>
              </tr>
            ))}
            <tr className="total">
              <td colSpan={3}>Total</td><td>{s.effectifTotal}</td><td>{s.effectifAVenirTotal}</td><td></td><td>{texte(s.masseSalarialeJour)}</td>
            </tr>
          </tbody>
        </table>
      </section>
    </div>
  );
}
