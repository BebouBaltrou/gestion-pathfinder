import { MotionConfig } from "motion/react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { autotest } from "./autotest";
import "./style.css";
import { appliquerThemeEnregistre } from "./composants/Apparence";
import { api } from "./api";

appliquerThemeEnregistre();

// F11 : plein écran ou fenêtre, comme dans la plupart des logiciels.
window.addEventListener("keydown", (e) => {
  if (e.key === "F11") {
    e.preventDefault();
    api.basculerPleinEcran();
  }
});

createRoot(document.getElementById("racine")!).render(
  <MotionConfig reducedMotion="user">
    <App />
  </MotionConfig>,
);
autotest();
