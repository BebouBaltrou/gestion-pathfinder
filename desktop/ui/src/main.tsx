import { createRoot } from "react-dom/client";
import { App } from "./App";
import { autotest } from "./autotest";
import "./style.css";

createRoot(document.getElementById("racine")!).render(<App />);
autotest();
