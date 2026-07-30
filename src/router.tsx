import { createMemoryRouter } from 'react-router';
import Layout from './layout';
import IndexPage from './pages';
import Downloads from './pages/downloads';
import Console from './pages/console.tsx'
import Settings from "@/pages/settings.tsx";
import Mods from "@/pages/mods.tsx";

export const router = createMemoryRouter([
  {
    element: Layout(),
    children: [
      {
        path: '/',
        element: <IndexPage />,
      },
      {
        path: '/downloads',
        element: <Downloads />,
      },
      {
        path: '/settings',
        element: <Settings/>
      },
      {
        path: '/mods',
        element: <Mods/>
      },
      {
        path: '/console',
        element: <Console/>
      }
    ],
  },
]);
