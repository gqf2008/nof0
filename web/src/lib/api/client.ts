const envBase = (import.meta.env.VITE_BACKEND_URL as string | undefined) || "";
const normalizedBase = envBase.replace(/\/$/, "");
export const BASE_URL = normalizedBase;

export async function fetcher<T = unknown>(
  url: string,
  init?: RequestInit,
): Promise<T> {
  const target = resolveUrl(url);
  const res = await fetch(target, {
    ...init,
    // Allow the browser HTTP cache to satisfy short‑interval polling.
    // Combined with Cache-Control from our proxy, this avoids hitting Vercel at all
    // when data is fresh, dramatically reducing Fast Data Transfer.
    cache: init?.cache ?? "default",
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`Request failed ${res.status}: ${text || res.statusText}`);
  }
  return res.json();
}

export const apiUrl = (path: string) => {
  const suffix = path.startsWith("/") ? path : `/${path}`;
  return `${BASE_URL}${suffix}`;
};

function resolveUrl(url: string) {
  if (/^https?:/i.test(url) || url.startsWith("//")) return url;
  const base = BASE_URL;
  if (!base) return url;
  return `${base}${url.startsWith("/") ? url : `/${url}`}`;
}
