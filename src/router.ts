import { createMemoryRouter } from 'react-router';
import Layout from './layout';
import IndexPage from './pages';
import Downloads from './pages/downloads';

export const router = createMemoryRouter([
  {
    element: Layout(),
    children: [
      {
        path: '/',
        element: IndexPage(),
      },
      {
        path: '/downloads',
        element: Downloads(),
      },
    ],
  },
]);
