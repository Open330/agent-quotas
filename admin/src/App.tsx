import { useHash } from './hooks.ts';
import { hasToken, clearToken } from './api.ts';
import Login from './pages/Login.tsx';
import Stats from './pages/Stats.tsx';
import Users from './pages/Users.tsx';

function navigate(hash: string) {
  window.location.hash = hash;
}

function NavIcon({ type }: { type: 'stats' | 'users' | 'logout' }) {
  if (type === 'stats') {
    return (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <rect x="3" y="3" width="7" height="7" rx="1" />
        <rect x="14" y="3" width="7" height="7" rx="1" />
        <rect x="3" y="14" width="7" height="7" rx="1" />
        <rect x="14" y="14" width="7" height="7" rx="1" />
      </svg>
    );
  }
  if (type === 'users') {
    return (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
        <circle cx="9" cy="7" r="4" />
        <path d="M22 21v-2a4 4 0 0 0-3-3.87" />
        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
      </svg>
    );
  }
  return (
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />
      <polyline points="16 17 21 12 16 7" />
      <line x1="21" y1="12" x2="9" y2="12" />
    </svg>
  );
}

export default function App() {
  const hash = useHash();

  // If no token and not on login page, redirect to login
  if (!hasToken() && hash !== '#/login') {
    window.location.hash = '#/login';
    return null;
  }

  // Login page (no sidebar)
  if (hash === '#/login') {
    return <Login />;
  }

  const handleLogout = () => {
    clearToken();
    navigate('#/login');
  };

  const currentPage = hash === '#/users' ? 'users' : 'stats';

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <h1>Quota Monitor</h1>
          <p>Admin Panel</p>
        </div>

        <nav className="sidebar-nav">
          <button
            className={`nav-link ${currentPage === 'stats' ? 'active' : ''}`}
            onClick={() => navigate('#/')}
          >
            <NavIcon type="stats" />
            Dashboard
          </button>
          <button
            className={`nav-link ${currentPage === 'users' ? 'active' : ''}`}
            onClick={() => navigate('#/users')}
          >
            <NavIcon type="users" />
            Users
          </button>
        </nav>

        <div className="sidebar-footer">
          <button onClick={handleLogout}>
            <NavIcon type="logout" />
            Sign out
          </button>
        </div>
      </aside>

      <main className="main-content">
        {currentPage === 'stats' && <Stats />}
        {currentPage === 'users' && <Users />}
      </main>
    </div>
  );
}
