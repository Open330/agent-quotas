import { useState, type FormEvent } from 'react';
import { setToken, api } from '../api.ts';

export default function Login() {
  const [pat, setPat] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!pat.trim()) {
      setError('Please enter a token.');
      return;
    }

    setLoading(true);
    setError('');

    // Store token and attempt to validate it
    setToken(pat.trim());

    try {
      await api.validateToken();
      window.location.hash = '#/';
    } catch {
      setError('Invalid or expired token. Please check and try again.');
      setToken('');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="login-page">
      <div className="login-card">
        <h1>Quota Monitor</h1>
        <p className="subtitle">Enter your personal access token to continue.</p>

        {error && (
          <div className="alert alert-error">{error}</div>
        )}

        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label className="form-label" htmlFor="pat-input">
              Access Token
            </label>
            <input
              id="pat-input"
              className="form-input"
              type="password"
              placeholder="Enter your PAT..."
              value={pat}
              onChange={(e) => setPat(e.target.value)}
              autoFocus
              autoComplete="off"
              disabled={loading}
            />
          </div>

          <button
            type="submit"
            className="btn btn-primary"
            disabled={loading || !pat.trim()}
            style={{ width: '100%', justifyContent: 'center', marginTop: '8px' }}
          >
            {loading ? (
              <>
                <span className="spinner" style={{ width: 14, height: 14, borderWidth: 2 }} />
                Validating...
              </>
            ) : (
              'Sign in'
            )}
          </button>
        </form>
      </div>
    </div>
  );
}
