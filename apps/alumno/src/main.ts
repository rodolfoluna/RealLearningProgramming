import "@rlp/ui-comun/tema.css";
import "./app.css";
import { mount } from "svelte";
import App from "./App.svelte";

// Sin menú contextual del navegador (el editor tiene el suyo, sin "Pegar").
document.addEventListener("contextmenu", (e) => {
  const t = e.target as HTMLElement;
  if (!t.closest("input, textarea")) e.preventDefault();
});

export default mount(App, { target: document.getElementById("app")! });
