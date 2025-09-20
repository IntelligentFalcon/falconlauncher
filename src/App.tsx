import React from 'react';
import FalconLauncher from './FalconLauncher';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

export const queryClient = new QueryClient();

function App() {
  return (
    <React.StrictMode>
      <QueryClientProvider client={queryClient}>
        <div className="min-h-screen antialiased">
          <FalconLauncher />
        </div>
      </QueryClientProvider>
    </React.StrictMode>
  );
}

export default App;
