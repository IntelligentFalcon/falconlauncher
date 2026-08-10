import { createMemoryRouter } from "react-router";
import Mods from "@/pages/mods.tsx";
import Settings from "@/pages/settings.tsx";
import Layout from "./layout";
import IndexPage from "./pages";
import Console from "./pages/console.tsx";
import Downloads from "./pages/downloads";

export const router = createMemoryRouter([
  {
    children: [
      {
        element: <IndexPage />,
        path: "/",
      },
      {
        element: <Downloads />,
        path: "/downloads",
      },
      {
        element: <Settings />,
        path: "/settings",
      },
      {
        element: <Mods />,
        path: "/mods",
      },
      {
        element: <Console />,
        path: "/console",
      },
    ],
    element: Layout(),
  },
]);
