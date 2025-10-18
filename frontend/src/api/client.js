// Base URL — relative so it works with nginx reverse proxy
const API_BASE = '/api';

export async function apiGet(path) {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: 'include', // allows cookies/sessions
    headers: { 'Accept': 'application/json' },
  });
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return res.json();
}

export async function apiPost(path, body) {
  const res = await fetch(`${API_BASE}${path}`, {
    method: 'POST',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json'
    },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`API error ${res.status}`);
  return res.json();
}