import { MotionConfig } from "motion/react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { autotest } from "./autotest";
import "./style.css";
import { appliquerThemeEnregistre } from "./composants/Apparence";

appliquerThemeEnregistre();

createRoot(document.getElementById("racine")!).render(
  <MotionConfig reducedMotion="user">
    <App />
  </MotionConfig>,
);
autotest();
