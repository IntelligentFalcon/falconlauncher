import { createRoot } from "react-dom/client";
import "./lib/i18n";
import React from "react";
import { RouterProvider } from "react-router";
import { router } from "./router";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Root element not found");
}

createRoot(rootElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>
);

document.addEventListener("DOMContentLoaded", () => {
  document.addEventListener("contextmenu", (e) => {
    e.preventDefault();
  });
});
