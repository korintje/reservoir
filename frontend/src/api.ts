let csrfToken = '';
export function setCsrfToken(value: string) { csrfToken = value; }
export class ApiError extends Error { constructor(message: string, public status: number) { super(message); } }
export async function api<T>(path: string, method = 'GET', data?: unknown): Promise<T> {
  const response = await fetch('/api' + path, {
    method, credentials: 'same-origin',
    headers: { 'X-Reservoir-Request': '1', ...(data === undefined ? {} : { 'Content-Type': 'application/json' }), ...(csrfToken ? { 'X-CSRF-Token': csrfToken } : {}) },
    body: data === undefined ? undefined : JSON.stringify(data),
  }).catch(() => { throw new ApiError('Cannot connect to the server. Check your connection and try again.', 0); });
  const text = await response.text();
  let body: any;
  try { body = text ? JSON.parse(text) : {}; } catch { body = { message: text.length < 200 && !text.includes('<') ? text : 'The server returned an unexpected response.' }; }
  if (!response.ok) throw new ApiError(body.message || body.error || 'The request failed.', response.status);
  return body as T;
}
