/* Kurultai Brain bridge for Agent Zero embedding.
 *
 * - Rewrites daemon API calls (/api/status|graph|activity|ontology|search|ask|touch|open)
 *   to the same-origin whitelisting proxy /api/plugins/kurultai/kproxy?ep=...
 *   The daemon is loopback-only inside the container and not reachable from
 *   the browser directly.
 * - Rewrites /ui/assets/* references to the plugin-served copy.
 * - Adds the Agent Zero CSRF token (header + cookie) to proxied POSTs.
 */
(() => {
  const API_EP = new Set(["status", "graph", "activity", "ontology", "search", "ask", "touch", "open"]);
  const PROXY = "/api/plugins/kurultai/kproxy";
  const ASSET_PREFIX = "/ui/assets/";
  const ASSET_TARGET = "/plugins/kurultai/webui/brainapp/assets/";
  let csrfPromise = null;

  function getCsrf() {
    if (!csrfPromise) {
      csrfPromise = fetch("/api/csrf_token", { credentials: "same-origin" })
        .then((r) => r.json())
        .then((j) => {
          if (j && j.ok && j.token && j.runtime_id) {
            document.cookie = `csrf_token_${j.runtime_id}=${j.token}; path=/; SameSite=Lax`;
          }
          return j && j.ok ? j.token : null;
        })
        .catch(() => null);
    }
    return csrfPromise;
  }

  function rewriteUrl(input) {
    try {
      const raw = typeof input === "string" ? input : String(input && input.url ? input.url : input);
      const url = new URL(raw, window.location.origin);
      if (url.pathname.startsWith(ASSET_PREFIX)) {
        return ASSET_TARGET + url.pathname.slice(ASSET_PREFIX.length);
      }
      const ep = url.pathname.replace(/^\/api\//, "");
      if (API_EP.has(ep)) {
        const proxied = new URL(PROXY, window.location.origin);
        proxied.searchParams.set("ep", ep);
        url.searchParams.forEach((v, k) => proxied.searchParams.append(k, v));
        return proxied.pathname + proxied.search;
      }
    } catch (_) {
      /* leave untouched */
    }
    return input;
  }

  const origFetch = window.fetch.bind(window);
  window.fetch = async (input, init = {}) => {
    const rewritten = rewriteUrl(input);
    const nextInit = { ...init, credentials: "same-origin" };
    const method = String(
      nextInit.method || (typeof input === "object" && input && input.method) || "GET"
    ).toUpperCase();
    if (method !== "GET" && method !== "HEAD") {
      const token = await getCsrf();
      if (token) {
        nextInit.headers = { ...(nextInit.headers || {}), "X-CSRF-Token": token };
      }
    }
    return origFetch(rewritten, nextInit);
  };

  // Workers and XHR bypass window.fetch — patch them too (fdg.worker + model loaders).
  const OrigWorker = window.Worker;
  if (OrigWorker) {
    function PatchedWorker(url, opts) {
      let target = url;
      try {
        target = rewriteUrl(typeof url === "string" ? url : url && url.url);
      } catch (_) {
        /* keep original */
      }
      return new OrigWorker(target, opts);
    }
    PatchedWorker.prototype = OrigWorker.prototype;
    window.Worker = PatchedWorker;
  }

  const OrigXhrOpen = XMLHttpRequest.prototype.open;
  XMLHttpRequest.prototype.open = function (method, url, ...rest) {
    let target = url;
    try {
      target = rewriteUrl(typeof url === "string" ? url : String(url));
    } catch (_) {
      /* keep original */
    }
    return OrigXhrOpen.call(this, method, target, ...rest);
  };
})();
