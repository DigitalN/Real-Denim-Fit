// Nudie: check the user's cookie consent, read the Virtusize widget key, and open the
// widget on a product for spot checks.
//
// WHERE TO RUN: in a browser tab on a https://www.nudiejeans.com/en-US/product/... page
// (built-in browser's javascript tool, or the DevTools console).
//
// BEFORE RUNNING: the USER accepts Nudie's cookies. Ask them to click "Size Guide" on the
// product page, then "Update Cookie Settings", and accept (at least the "Function"
// category with the Virtusize vendor). Never click the consent dialog yourself.
//
// WHAT IT DOES / OUTPUT: returns { consent, apiKey }.
//   consent.ok is true when purpose c51 (Function) and vendor c74506 (Virtusize) are both
//   accepted; apiKey is window.vs.apiKey, which fetch_virtusize.py needs (--api-key). The key
//   only exists once the consent is in place, so a missing key means: ask the user again.
// It also defines window.nudieOpenWidget("<product uri>"), which opens the Virtusize widget
// on that product for a spot check (see EXTRACTING.md): it navigates inside the site, because
// Nudie's Size Guide reads the consent only when it first mounts, and it checks the wrong
// vendor (c74504), so after the user accepts it keeps showing "Please accept cookies" until
// it mounts again. Then pick a size in the widget's dropdown and read the labels.
(() => {
  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  let data = {};
  try { data = window.__cmp ? window.__cmp("getCMPData") : {}; } catch (e) { data = { error: String(e) }; }
  const consent = {
    function_c51: !!data.purposeConsents?.c51,
    virtusize_c74506: !!data.vendorConsents?.c74506,
    frontchat_c74504: !!data.vendorConsents?.c74504, // the vendor Nudie's component actually checks
  };
  consent.ok = consent.function_c51 && consent.virtusize_c74506;

  window.nudieOpenWidget = async (uri) => {
    window.next.router.push(`/en-US/product/${uri}`);
    await sleep(4500);
    [...document.querySelectorAll("button")].find((e) => e.textContent.trim() === "Size Guide")?.click();
    await sleep(3000);
    const b = document.querySelector(".virtusize-button");
    if (!b) return "no 'Open in Virtusize' button: consent missing, or the Size Guide didn't remount";
    b.click(); await sleep(2500);           // first click binds the widget
    if (!document.querySelector('iframe[src*="fit-illustrator"]')) { b.click(); await sleep(3000); }
    [...document.querySelectorAll("#size-guide-dialog button")].find((e) => e.textContent.trim() === "Close")?.click();
    return "widget open: choose a size in its dropdown (bottom centre) and read the labels";
  };

  return { consent, apiKey: window.vs?.apiKey || null };
})();
