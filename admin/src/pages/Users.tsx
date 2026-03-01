import { useState, useCallback, type FormEvent } from 'react';
import { api } from '../api.ts';
import { usePolling } from '../hooks.ts';
import type { UserRecord, UserInfo } from '../types.ts';

function relativeTime(isoStr: string): string {
  if (!isoStr) return 'never';
  const diff = Date.now() - new Date(isoStr).getTime();
  const m = Math.floor(diff / 60000);
  if (m < 1) return 'just now';
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  return `${Math.floor(h / 24)}d ago`;
}

interface ConfirmDialogProps {
  title: string;
  message: string;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
  danger?: boolean;
}

function ConfirmDialog({ title, message, confirmLabel, onConfirm, onCancel, danger }: ConfirmDialogProps) {
  return (
    <div className="overlay" onClick={onCancel}>
      <div className="dialog" onClick={(e) => e.stopPropagation()}>
        <h3>{title}</h3>
        <p>{message}</p>
        <div className="dialog-actions">
          <button className="btn btn-ghost" onClick={onCancel}>Cancel</button>
          <button className={`btn ${danger ? 'btn-danger' : 'btn-primary'}`} onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

export default function Users() {
  // Try admin endpoint first, fall back to public endpoint
  const [useAdminEndpoint, setUseAdminEndpoint] = useState(true);

  const fetchUsers = useCallback(async (): Promise<(UserRecord | UserInfo)[]> => {
    if (useAdminEndpoint) {
      try {
        return await api.getAdminUsers();
      } catch {
        setUseAdminEndpoint(false);
        return await api.getUsers();
      }
    }
    return await api.getUsers();
  }, [useAdminEndpoint]);

  const { data: users, loading, error, refresh } = usePolling(fetchUsers, 30000);

  // Create user form state
  const [newUsername, setNewUsername] = useState('');
  const [newIsAdmin, setNewIsAdmin] = useState(false);
  const [creating, setCreating] = useState(false);
  const [createError, setCreateError] = useState('');
  const [createdToken, setCreatedToken] = useState<string | null>(null);
  const [createdUsername, setCreatedUsername] = useState('');

  // Regenerated token display
  const [regeneratedToken, setRegeneratedToken] = useState<string | null>(null);
  const [regeneratedUser, setRegeneratedUser] = useState('');

  // Confirmation dialogs
  const [deleteConfirm, setDeleteConfirm] = useState<{ id: number; username: string } | null>(null);
  const [regenConfirm, setRegenConfirm] = useState<{ id: number; username: string } | null>(null);

  const handleCreateUser = async (e: FormEvent) => {
    e.preventDefault();
    if (!newUsername.trim()) return;

    setCreating(true);
    setCreateError('');
    setCreatedToken(null);

    try {
      const result = await api.createUser(newUsername.trim(), newIsAdmin);
      if (result.token) {
        setCreatedToken(result.token);
        setCreatedUsername(result.username);
      }
      setNewUsername('');
      setNewIsAdmin(false);
      refresh();
    } catch (err) {
      setCreateError(err instanceof Error ? err.message : String(err));
    } finally {
      setCreating(false);
    }
  };

  const handleDeleteUser = async (id: number) => {
    try {
      await api.deleteUser(id);
      setDeleteConfirm(null);
      refresh();
    } catch (err) {
      alert('Failed to delete user: ' + (err instanceof Error ? err.message : String(err)));
      setDeleteConfirm(null);
    }
  };

  const handleRegenerateToken = async (id: number, username: string) => {
    try {
      const result = await api.regenerateToken(id);
      setRegeneratedToken(result.token);
      setRegeneratedUser(username);
      setRegenConfirm(null);
    } catch (err) {
      alert('Failed to regenerate token: ' + (err instanceof Error ? err.message : String(err)));
      setRegenConfirm(null);
    }
  };

  // Check if we have admin-level user records (with id field)
  const isAdminData = users && users.length > 0 && 'id' in users[0];

  return (
    <>
      <div className="page-header">
        <h2>User Management</h2>
        <p>Manage users and access tokens</p>
      </div>

      {/* Token display alerts */}
      {createdToken && (
        <div className="alert alert-success" style={{ marginBottom: 20 }}>
          <strong>User "{createdUsername}" created successfully.</strong>
          <br />
          Copy this token now -- it will not be shown again:
          <div className="token-display">{createdToken}</div>
          <button
            className="btn btn-ghost btn-sm"
            style={{ marginTop: 10 }}
            onClick={() => setCreatedToken(null)}
          >
            Dismiss
          </button>
        </div>
      )}

      {regeneratedToken && (
        <div className="alert alert-warning" style={{ marginBottom: 20 }}>
          <strong>Token regenerated for "{regeneratedUser}".</strong>
          <br />
          Copy this token now -- it will not be shown again:
          <div className="token-display">{regeneratedToken}</div>
          <button
            className="btn btn-ghost btn-sm"
            style={{ marginTop: 10 }}
            onClick={() => setRegeneratedToken(null)}
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Create user form (only if admin endpoints are available) */}
      {useAdminEndpoint && (
        <div className="card" style={{ marginBottom: 24 }}>
          <p className="section-title" style={{ marginBottom: 16 }}>Add New User</p>

          {createError && (
            <div className="alert alert-error" style={{ marginBottom: 12 }}>{createError}</div>
          )}

          <form onSubmit={handleCreateUser} style={{ display: 'flex', gap: 12, alignItems: 'flex-end', flexWrap: 'wrap' }}>
            <div className="form-group" style={{ flex: '1 1 200px', marginBottom: 0 }}>
              <label className="form-label" htmlFor="username-input">Username</label>
              <input
                id="username-input"
                className="form-input"
                type="text"
                placeholder="Enter username..."
                value={newUsername}
                onChange={(e) => setNewUsername(e.target.value)}
                disabled={creating}
              />
            </div>

            <div className="form-group" style={{ marginBottom: 0 }}>
              <div className="form-checkbox-row" style={{ height: 40, display: 'flex', alignItems: 'center' }}>
                <input
                  id="admin-checkbox"
                  type="checkbox"
                  checked={newIsAdmin}
                  onChange={(e) => setNewIsAdmin(e.target.checked)}
                  disabled={creating}
                />
                <label htmlFor="admin-checkbox">Admin</label>
              </div>
            </div>

            <button
              type="submit"
              className="btn btn-primary"
              disabled={creating || !newUsername.trim()}
              style={{ height: 40 }}
            >
              {creating ? 'Creating...' : 'Create User'}
            </button>
          </form>
        </div>
      )}

      {error && (
        <div className="alert alert-error" style={{ marginBottom: 20 }}>
          Failed to load users: {error}
        </div>
      )}

      {/* Users table */}
      <div className="table-card">
        <div className="table-card-header">
          <span className="table-card-title">
            {isAdminData ? 'Registered Users' : 'Active Users'}
          </span>
          <span style={{ fontSize: '0.78rem', color: 'var(--text-dim)' }}>
            {users ? `${users.length} user${users.length !== 1 ? 's' : ''}` : ''}
          </span>
        </div>

        {loading ? (
          <div className="loading-state">
            <div className="spinner" />
            Loading users...
          </div>
        ) : !users || users.length === 0 ? (
          <div className="loading-state" style={{ color: 'var(--text-dim)' }}>
            No users found.
          </div>
        ) : (
          <div style={{ overflowX: 'auto' }}>
            <table>
              <thead>
                <tr>
                  <th>Username</th>
                  {isAdminData && <th>Role</th>}
                  {isAdminData ? <th>Created</th> : <th>Last Active</th>}
                  {!isAdminData && <th>Reports</th>}
                  {isAdminData && <th>Actions</th>}
                </tr>
              </thead>
              <tbody>
                {users.map((user) => {
                  if (isAdminData) {
                    const u = user as UserRecord;
                    return (
                      <tr key={u.id}>
                        <td style={{ fontWeight: 600, color: 'var(--text)' }}>{u.username}</td>
                        <td>
                          {u.is_admin ? (
                            <span className="badge badge-admin">Admin</span>
                          ) : (
                            <span className="badge badge-muted">User</span>
                          )}
                        </td>
                        <td style={{ color: 'var(--text-dim)' }}>{relativeTime(u.created_at)}</td>
                        <td>
                          <div style={{ display: 'flex', gap: 6 }}>
                            <button
                              className="btn btn-ghost btn-sm"
                              onClick={() => setRegenConfirm({ id: u.id, username: u.username })}
                            >
                              Regenerate Token
                            </button>
                            <button
                              className="btn btn-danger btn-sm"
                              onClick={() => setDeleteConfirm({ id: u.id, username: u.username })}
                            >
                              Delete
                            </button>
                          </div>
                        </td>
                      </tr>
                    );
                  } else {
                    const u = user as UserInfo;
                    return (
                      <tr key={u.username}>
                        <td style={{ fontWeight: 600, color: 'var(--text)' }}>{u.username}</td>
                        <td style={{ color: 'var(--text-dim)' }}>{relativeTime(u.last_active)}</td>
                        <td>{u.total_reports.toLocaleString()}</td>
                      </tr>
                    );
                  }
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Confirmation dialogs */}
      {deleteConfirm && (
        <ConfirmDialog
          title="Delete User"
          message={`Are you sure you want to delete "${deleteConfirm.username}"? This action cannot be undone.`}
          confirmLabel="Delete"
          danger
          onConfirm={() => handleDeleteUser(deleteConfirm.id)}
          onCancel={() => setDeleteConfirm(null)}
        />
      )}

      {regenConfirm && (
        <ConfirmDialog
          title="Regenerate Token"
          message={`This will invalidate the current token for "${regenConfirm.username}" and generate a new one. The user will need the new token to authenticate.`}
          confirmLabel="Regenerate"
          onConfirm={() => handleRegenerateToken(regenConfirm.id, regenConfirm.username)}
          onCancel={() => setRegenConfirm(null)}
        />
      )}
    </>
  );
}
