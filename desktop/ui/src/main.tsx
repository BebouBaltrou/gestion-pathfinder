import { createRoot } from "react-dom/client";
import { App } from "./App";
import { autotest } from "./autotest";
import "./style.css";
import { appliquerThemeEnregistre } from "./composants/Apparence";

appliquerThemeEnregistre();

createRoot(document.getElementById("racine")!).render(<App />);
autotest();
