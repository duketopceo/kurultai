import { StrictMode, useState, useEffect } from 'react';
import { createRoot } from 'react-dom/client';
import './styles.css';
import { App } from './App';
import { RepoBrainPage } from './components/RepoBrain';

function Router() {
  const [hash, setHash] = useState(window.location.hash);
  useEffect(() => {
    const handler = () => setHash(window.location.hash);
    window.addEventListener('hashchange', handler);
    return () => window.removeEventListener('hashchange', handler);
  }, []);

  if (hash.startsWith('#/repo')) return <RepoBrainPage />;
  return <App />;
}

const root = document.getElementById('root');
if (!root) throw new Error('No #root element found');
createRoot(root).render(<StrictMode><Router /></StrictMode>);
