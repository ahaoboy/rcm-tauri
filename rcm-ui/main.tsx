import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { SubmenuApp } from "./components/SubmenuApp";
import "./styles/menu.css";

const root = document.getElementById("root") as HTMLElement;
const isSubmenu = window.location.hash.startsWith("#submenu-");

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    {isSubmenu ? <SubmenuApp /> : <App />}
  </React.StrictMode>,
);
