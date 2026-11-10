import { Component, type ReactNode } from "react";

type Props = { cle: unknown; children: ReactNode };
type Etat = { erreur: Error | null; cle: unknown };

/**
 * Empêche une erreur d'affichage dans une page de faire disparaître toute la fenêtre
 * (écran noir). Le message s'affiche à la place ; il se réinitialise quand `cle` change
 * (autre onglet, autre personnage).
 */
export class LimiteErreur extends Component<Props, Etat> {
  state: Etat = { erreur: null, cle: this.props.cle };

  static getDerivedStateFromError(erreur: Error): Partial<Etat> {
    return { erreur };
  }

  static getDerivedStateFromProps(props: Props, etat: Etat): Partial<Etat> | null {
    return props.cle !== etat.cle ? { erreur: null, cle: props.cle } : null;
  }

  componentDidCatch(erreur: Error): void {
    console.error("Erreur d'affichage :", erreur);
  }

  render(): ReactNode {
    if (!this.state.erreur) return this.props.children;
    return (
      <section className="carte alerte grave" role="alert">
        <span className="point" />
        <div className="texte">
          <strong>Cette page n'a pas pu s'afficher.</strong>
          <p className="doux">Détail : {this.state.erreur.message}</p>
          <button className="bouton principal" onClick={() => this.setState({ erreur: null })}>
            Réessayer
          </button>
        </div>
      </section>
    );
  }
}
