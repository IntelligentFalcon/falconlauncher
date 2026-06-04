import { createMemoryRouter } from 'react-router';
import Layout from './layout';
import IndexPage from './pages';
import Downloads from './pages/downloads';
import Console from './pages/console.tsx'
import Settings from "@/pages/settings.tsx";

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
        path: '/console',
        element: <Console/>
      }
    ],
  },
]);
