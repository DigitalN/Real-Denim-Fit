// Momotaro scrape: product size charts, titles, prices, colours and sizes.
//
// WHERE TO RUN: in a browser tab on https://www.momotaro-jeans.com (the site answers
// plain curl with 403, but same-origin fetch() from a loaded page works). Use the
// built-in browser's javascript tool or the DevTools console.
//
// BEFORE RUNNING: set the flag menu (top left) to "Ship to: United States" and Save,
// so prices come back in USD. The snippet refuses to finish if they don't.
//
// WHAT IT DOES: searches the store for jeans (q = straight, tapered, wide, slim, womens,
// jeans), then fetches each product page one at a time, 2 s apart, and keeps every
// product whose size chart has an "inseam" column. Set ONLY to a list of product ids
// (e.g. ["M200ST"]) to fetch just those.
//
// OUTPUT: window.__momotaro = { done, raw, log }. `raw` is
//   { "<pid>": { t: title, p: "<content>|<price text>", c: [colours], s: [sizes offered],
//                r: ["<header csv>", "<size>,<v>,...", ...], url } }
// which is the input build.py expects. When done it also offers momotaro-raw.json as a
// download (DevTools: or run copy(JSON.stringify(window.__momotaro.raw))).
// It runs in the background: poll `window.__momotaro.done`.
(() => {
  const ONLY = null; // e.g. ["M200ST", "M410ST"]
  const QUERIES = ["straight", "tapered", "wide", "slim", "womens", "jeans"];
  const LOCALE = "en_US";
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const state = (window.__momotaro = { done: false, raw: {}, log: [] });
  const log = (m) => { state.log.push(m); console.log("[momotaro]", m); };

  (async () => {
    const urls = {};
    for (const q of QUERIES) {
      const h = await (await fetch(`/${LOCALE}/search/?q=${q}&sz=300`)).text();
      const d = new DOMParser().parseFromString(h, "text/html");
      d.querySelectorAll(".product[data-pid]").forEach((p) => {
        const a = p.querySelector('a[href*="/product/"]');
        if (a) urls[p.dataset.pid] = new URL(a.getAttribute("href"), location.origin).href.split("?")[0]
          .replace(/\/[a-z]{2}_[A-Z]{2}\//, `/${LOCALE}/`);
      });
      await sleep(1500);
    }
    const pids = Object.keys(urls).filter((pid) => !ONLY || ONLY.includes(pid));
    log(`${Object.keys(urls).length} products found, fetching ${pids.length}`);
    for (const pid of pids) {
      try {
        const h = await (await fetch(urls[pid])).text();
        const d = new DOMParser().parseFromString(h, "text/html");
        const t = d.querySelector("table.sizechart");
        const rows = t ? [...t.querySelectorAll("tr")].map((tr) => [...tr.children].map((c) => c.textContent.trim()).join(",")) : [];
        if (!rows.length || !/inseam/i.test(rows[0])) { await sleep(2000); continue; }
        const price = [...d.querySelectorAll(".prices .price .sales .value, .product-detail .price .value")]
          .map((e) => (e.getAttribute("content") || "") + "|" + e.textContent.trim())[0] || "";
        state.raw[pid] = {
          t: d.querySelector("h1")?.textContent.trim(),
          p: price,
          c: [...d.querySelectorAll('[data-attr="color"] [aria-label]')].map((b) => b.getAttribute("aria-label").replace("Select Color ", "")),
          s: [...new Set([...d.querySelectorAll('[data-attr="size"] [data-attr-value]')].map((b) => b.textContent.trim()))],
          r: rows,
          url: urls[pid],
        };
      } catch (e) { log(`${pid}: ${e}`); }
      await sleep(2000);
    }
    const nonUsd = Object.entries(state.raw).filter(([, v]) => !v.p.split("|")[1]?.startsWith("$"));
    if (nonUsd.length) {
      log(`NOT USD (${nonUsd.length} products, e.g. ${nonUsd[0][1].p}): set Ship to: United States and run again`);
    } else {
      const a = document.createElement("a");
      a.href = URL.createObjectURL(new Blob([JSON.stringify(state.raw, null, 1)], { type: "application/json" }));
      a.download = "momotaro-raw.json";
      a.click();
    }
    log(`done: ${Object.keys(state.raw).length} products with an inseam chart`);
    state.done = true;
  })();
  return "started: poll window.__momotaro.done";
})();
