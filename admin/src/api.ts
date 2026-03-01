import type { AdminStats, UserRecord, SummaryResponse, UserInfo, HourlyUsage } from './types.ts';

const TOKEN_KEY = 'quota-admin-token';

export function getToken(): string {
  return localStorage.getItem(TOKEN_KEY) || '';
}

export function setToken(token: string): void {
  localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
  localStorage.removeItem(TOKEN_KEY);
}

export function hasToken(): boolean {
  return !!localStorage.getItem(TOKEN_KEY);
}

async function apiFetch<T>(path: string, options: RequestInit = {}): Promise<T> {
  const token = getToken();
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...(options.headers as Record<string, string> || {}),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  const res = await fetch(path, {
    ...options,
    headers,
  });

  if (res.status === 401 || res.status === 403) {
    clearToken();
    window.location.hash = '#/login';
    throw new Error('Unauthorized');
  }

  if (!res.ok) {
    const text = await res.text();
    throw new Error(text || `HTTP ${res.status}`);
  }

  const contentType = res.headers.get('content-type');
  if (contentType && contentType.includes('application/json')) {
    return res.json();
  }
  return undefined as T;
}

export const api = {
  // Admin endpoints (require auth when admin middleware is added to the Rust server)
  getStats: () => apiFetch<AdminStats>('/api/admin/stats'),
  getAdminUsers: () => apiFetch<UserRecord[]>('/api/admin/users'),
  createUser: (username: string, is_admin: boolean) =>
    apiFetch<UserRecord>('/api/admin/users', {
      method: 'POST',
      body: JSON.stringify({ username, is_admin }),
    }),
  deleteUser: (id: number) =>
    apiFetch<void>(`/api/admin/users/${id}`, { method: 'DELETE' }),
  regenerateToken: (id: number) =>
    apiFetch<{ token: string }>(`/api/admin/users/${id}/regenerate-token`, {
      method: 'POST',
    }),

  // Public endpoints (currently available on the Rust server)
  getSummary: () => apiFetch<SummaryResponse>('/api/summary'),
  getUsers: () => apiFetch<UserInfo[]>('/api/users'),
  getHourly: (user?: string) =>
    apiFetch<HourlyUsage[]>(`/api/hourly${user ? `?user=${encodeURIComponent(user)}` : ''}`),

  // Validation: test if the token works by hitting any endpoint
  validateToken: () => apiFetch<SummaryResponse>('/api/summary'),
};
