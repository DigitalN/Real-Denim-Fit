// Real Denim Fit: the screens. Matching, the catalogue and storage live in
// Rust; this file only draws and collects what the user enters.
"use strict";

// In the app, commands go to Rust. In the browser preview (examples/preview.rs)
// they go to the same Rust code over HTTP.
const invoke = window.__TAURI__
  ? window.__TAURI__.core.invoke
  : async (cmd, args = {}) => {
      const r = await fetch(`/api/${cmd}`, { method: "POST", body: JSON.stringify(args) });
      if (!r.ok) throw new Error(await r.text());
      return r.json();
    };

const $app = document.getElementById("app");

// The app's webview ignores target="_blank" links, so every https link opens
// in the default browser through the opener plugin (https only, per the
// app's capabilities). The browser preview keeps ordinary links.
if (window.__TAURI__) {
  document.addEventListener("click", (e) => {
    const link = e.target.closest && e.target.closest('a[href^="https://"]');
    if (!link) return;
    e.preventDefault();
    invoke("plugin:opener|open_url", { url: link.href }).catch(showError);
  });
}
let S = null; // everything the user has entered
let CAT = null; // brands and products, without charts
const PRODUCTS = new Map();
const BRANDS = new Map();

const BODY_FIELDS = ["waist", "seat", "thigh_top", "thigh_1in", "inseam_wanted", "leg_12in", "leg_13in", "leg_14in", "leg_half_inseam"];
const PAIR_FIELDS = ["waist", "front_rise", "back_rise", "seat", "thigh_crotch", "thigh_1in", "inseam", "knee_12in", "knee_13in", "knee_14in", "knee_half_inseam", "leg_opening"];
const PAIR_FIELD_LABELS = {
  waist: "Waist, straight across",
  front_rise: "Front rise",
  back_rise: "Back rise",
  seat: "Seat",
  thigh_crotch: "Thigh at the crotch",
  thigh_1in: "Thigh 2.5 cm (1″) below the crotch",
  inseam: "Inseam",
  knee_12in: "Knee 30.5 cm (12″) below the crotch",
  knee_13in: "Knee 33 cm (13″) below the crotch",
  knee_14in: "Knee 35.5 cm (14″) below the crotch",
  knee_half_inseam: "Knee halfway down the inseam",
  leg_opening: "Leg opening",
};

function freshState(currency) {
  return {
    version: 1,
    onboarded: false,
    settings: { units: "cm", currency, tryOnBrands: [] },
    body: {},
    desiredFit: { room: "regular", leg_shape: null, rise: null, importance: {} },
    pairs: [],
    activePairId: null,
    filters: {
      excluded_brands: [],
      price_min: null,
      price_max: null,
      try_on_only: false,
      mens: true,
      womens: true,
      weight_min: null,
      weight_max: null,
      sanforized_only: false,
      no_stretch: false,
      include_washed: false,
      current_only: true,
      show_other_shapes_and_rises: false,
    },
  };
}

let saveTimer = null;
function save() {
  if (!S) return;
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => invoke("save_state", { state: S }).catch(showError), 150);
}

function showError(e) {
  console.error(e);
  const box = document.createElement("div");
  box.className = "note";
  box.textContent = `Something went wrong: ${e.message || e}`;
  $app.prepend(box);
}

const units = () => S.settings.units;
const len = (cm, signed) => Units.length(cm, units(), signed);
const unitLabel = () => (units() === "in" ? "in" : "cm");
const bodyComplete = () => BODY_FIELDS.every((f) => typeof S.body[f] === "number");
const activePair = () => S.pairs.find((p) => p.id === S.activePairId) || null;

/** Removes a reference pair, and stops comparing with it if it was chosen. */
function removePair(id) {
  S.pairs = S.pairs.filter((p) => p.id !== id);
  if (S.activePairId === id) S.activePairId = null;
  save();
}

// ---------------------------------------------------------------- boot ----

async function boot() {
  const overlay = document.getElementById("updating");
  const text = document.getElementById("updating-text");
  overlay.hidden = false;
  text.textContent = "Opening…";
  try {
    const version = await invoke("check_for_update");
    if (version) {
      text.textContent = `Updating to ${version}…`;
      await invoke("install_update"); // reopens the app when done
      return;
    }
  } catch (e) {
    console.warn("Update skipped", e);
  }
  try {
    const [saved, index] = await Promise.all([invoke("load_state"), invoke("catalogue_index")]);
    CAT = index;
    for (const b of CAT.brands) {
      BRANDS.set(b.id, b);
      for (const p of b.products) PRODUCTS.set(p.id, { ...p, brand: b });
    }
    if (saved && saved.version !== 1) {
      // Made by another version of the app: leave the file alone.
      overlay.hidden = true;
      $app.innerHTML = `<div class="narrow"><h1>Your saved measurements are from another version</h1>
        <p>They were saved by a different version of Real Denim Fit, so this one won't open or change them. Update the app to the latest version to carry on.</p></div>`;
      return;
    }
    S = saved || freshState(CAT.default_currency);
    // Earlier builds saved the brands to keep; keep the same choice as the
    // brands left out, so brands added since start ticked.
    if (Array.isArray(S.filters.brands)) {
      S.filters.excluded_brands = CAT.brands.map((b) => b.id).filter((id) => !S.filters.brands.includes(id));
    }
    delete S.filters.brands;
    // A pair removed while chosen (by older builds) leaves nothing to compare with.
    if (S.activePairId && !S.pairs.some((p) => p.id === S.activePairId)) S.activePairId = null;
    // Earlier builds had "doesn't matter" checkboxes; those become importance 1.
    if (S.desiredFit.doesnt_matter) {
      S.desiredFit.importance = S.desiredFit.importance || {};
      for (const m of S.desiredFit.doesnt_matter) S.desiredFit.importance[m] = 1;
      delete S.desiredFit.doesnt_matter;
    }
  } catch (e) {
    // S stays unset, so nothing is saved over a file that couldn't be read.
    overlay.hidden = true;
    showError(e);
    return;
  }
  overlay.hidden = true;
  window.addEventListener("hashchange", render);
  if (!S.onboarded || !bodyComplete()) location.hash = location.hash.startsWith("#/body") ? location.hash : "#/welcome";
  else if (!location.hash || location.hash === "#/") location.hash = "#/results";
  render();
}

function render() {
  const [, route, ...rest] = location.hash.split("/");
  const topbar = document.getElementById("topbar");
  topbar.hidden = !S.onboarded;
  document.querySelectorAll("[data-nav]").forEach((a) => a.classList.toggle("active", a.dataset.nav === route));
  window.scrollTo(0, 0);
  const screens = {
    welcome: renderWelcome,
    guide: renderBodyGuide,
    body: renderBodyForm,
    fit: renderFit,
    results: renderResults,
    product: renderProduct,
    pairs: renderPairs,
    pair: renderPair,
    settings: renderSettings,
  };
  (screens[route] || renderResults)(...rest.map(decodeURIComponent));
}

// --------------------------------------------------------- small parts ----

function measureInput(id, cm, attrs = "") {
  return `<div class="measure-input"><input type="text" inputmode="decimal" id="${id}" value="${esc(Units.inputValue(cm, units()))}" ${attrs} autocomplete="off"/><span class="unit">${unitLabel()}</span></div>`;
}

function readMeasure(id) {
  const el = document.getElementById(id);
  return el ? Units.parse(el.value, units()) : null;
}

function choiceGroup(name, options, current) {
  return `<div class="choices" role="radiogroup">${options
    .map(([value, label]) => `<label class="choice"><input type="radio" name="${name}" value="${value}" ${String(current) === String(value) ? "checked" : ""}/>${label}</label>`)
    .join("")}</div>`;
}

/**
 * A confirmation step on the page itself: the macOS webview has no
 * confirm() dialog (it always answers no). Clicking `button` swaps it for
 * the question with a Yes and a No.
 */
function askInPage(button, question, yesLabel, onYes) {
  button.onclick = () => {
    const box = document.createElement("span");
    box.className = "ask";
    box.innerHTML = `<span>${esc(question)}</span> <button class="danger">${esc(yesLabel)}</button> <button>Keep it</button>`;
    const [yes, no] = box.querySelectorAll("button");
    yes.onclick = onYes;
    no.onclick = () => box.replaceWith(button);
    button.replaceWith(box);
    no.focus();
  };
}

function closenessClass(pct) {
  if (pct >= 80) return "c-close";
  if (pct >= 50) return "c-mid";
  return "c-far";
}

// The waist is shown all the way around; other widths straight across.
function shownValue(m, cm) {
  return m === "waist" ? cm * 2 : cm;
}

// ---------------------------------------------------------- the guide ----
// The guide's words and drawings come from ui/guide/steps.json and its SVGs
// (written separately); this file only maps each screen to the value it
// stores.

let GUIDE = null;
const SVG_CACHE = new Map();

async function loadGuide() {
  if (GUIDE) return GUIDE;
  try {
    const r = await fetch("guide/steps.json");
    GUIDE = r.ok ? await r.json() : [];
  } catch {
    GUIDE = [];
  }
  return GUIDE;
}

async function guideDrawing(image) {
  if (!image) return "";
  if (!SVG_CACHE.has(image)) {
    try {
      const r = await fetch(`guide/${image}`);
      SVG_CACHE.set(image, r.ok ? await r.text() : "");
    } catch {
      SVG_CACHE.set(image, "");
    }
  }
  return SVG_CACHE.get(image);
}

/** Where a screen's named field is stored, such as the body's inseam_extra. */
function fieldKey(kind, id) {
  if (kind === "body" && id === "inseam") return "inseam_wanted";
  return id; // inseam_extra, and the pair's inseam
}

/** The stored value a guide screen fills in, or null for intro screens. */
function guideField(step) {
  const m = step.measurement;
  if (!m) return null;
  const point = step.point;
  if (step.for === "body") {
    if (m === "thigh") return point === "1in-below-crotch" ? "thigh_1in" : "thigh_top";
    if (m === "knee") return { "12in-below-crotch": "leg_12in", "13in-below-crotch": "leg_13in", "14in-below-crotch": "leg_14in", "half-inseam": "leg_half_inseam" }[point];
    if (m === "inseam") return "inseam_wanted";
    return m; // waist, seat
  }
  if (m === "thigh") return point === "1in-below-crotch" ? "thigh_1in" : "thigh_crotch";
  if (m === "knee") return { "12in-below-crotch": "knee_12in", "13in-below-crotch": "knee_13in", "14in-below-crotch": "knee_14in", "half-inseam": "knee_half_inseam" }[point];
  return m; // waist, front_rise, back_rise, seat, leg_opening, inseam
}

/**
 * Draws one guide screen. `values` is the object the screen writes into;
 * `opts` gives the progress text, whether the value can be skipped, and
 * where Back and Next go.
 */
async function renderGuideScreen(kind, index, values, opts) {
  const all = (await loadGuide()).filter((s) => s.for === kind);
  if (!all.length) {
    $app.innerHTML = `<div class="narrow"><h1>The measuring guide isn't available</h1>
      <p>Its screens are missing from this build. You can still type your measurements into the form.</p>
      <div class="actions"><span></span><a class="button primary" href="${opts.formHref}">Open the form</a></div></div>`;
    return;
  }
  const i = Math.max(0, Math.min(all.length - 1, index));
  const step = all[i];
  // A screen asks for one value, or for each of its `fields` (such as the
  // inseam to the hem and an optional extra for cuffs or stacking).
  const field = guideField(step);
  const inputs = step.fields
    ? step.fields.map((f) => ({ key: fieldKey(step.for, f.id), label: f.label, optional: !!f.optional || opts.skippable, id: `g-${f.id}` }))
    : field
      ? [{ key: field, label: opts.fieldLabel(field), optional: opts.skippable, id: "g-value" }]
      : [];
  const drawing = await guideDrawing(step.image);
  const hint = opts.hint ? opts.hint(step) : "";
  $app.innerHTML = `
  <div class="narrow" style="max-width:860px">
    <div class="progress"><div style="width:${((i + 1) / all.length) * 100}%"></div></div>
    <p class="muted small">${esc(opts.label)} · ${i + 1} of ${all.length}</p>
    <div class="guide">
      <div class="guide-drawing">${drawing}</div>
      <div>
        <h1>${esc(step.title)}</h1>
        <ol class="steps">${step.steps.map((l) => `<li>${esc(l)}</li>`).join("")}</ol>
        ${step.mistake ? `<p class="tip"><b>Common mistake:</b> ${esc(step.mistake)}</p>` : ""}
        ${opts.extra ? opts.extra(step, i) : ""}
        ${inputs.map((f) => `<label for="${f.id}">${esc(f.label)}${f.optional && step.fields ? ` <span class="hint">(optional)</span>` : ""}</label>${measureInput(f.id, values[f.key])}`).join("")}${inputs.length ? hint : ""}
        <p class="error" id="err" hidden>Please enter a number, or skip this one.</p>
      </div>
    </div>
    <div class="actions">
      <button id="back" ${i === 0 && !opts.firstBack ? 'style="visibility:hidden"' : ""}>${i === 0 ? opts.firstBack : "Previous"}</button>
      <span>${field && opts.skippable ? `<button id="skip">Skip</button> ` : ""}<button class="primary" id="next">${i === all.length - 1 ? "Done" : "Next"}</button></span>
    </div>
  </div>`;
  if (inputs.length) document.getElementById(inputs[0].id).focus();
  if (opts.afterRender) opts.afterRender(step);
  document.getElementById("back").onclick = () => opts.go(i - 1);
  const skip = document.getElementById("skip");
  if (skip)
    skip.onclick = () => {
      for (const f of inputs) delete values[f.key];
      opts.save();
      opts.go(i + 1, all.length);
    };
  const next = () => {
    if (opts.beforeNext) opts.beforeNext(step);
    const read = inputs.map((f) => [f, readMeasure(f.id)]);
    if (read.some(([f, v]) => v === null && !f.optional)) {
      document.getElementById("err").hidden = false;
      return;
    }
    for (const [f, v] of read) {
      if (v === null) delete values[f.key];
      else values[f.key] = v;
    }
    opts.save();
    opts.go(i + 1, all.length);
  };
  document.getElementById("next").onclick = next;
  for (const f of inputs) document.getElementById(f.id).addEventListener("keydown", (e) => e.key === "Enter" && next());
}

const BODY_FIELD_LABELS = {
  waist: "Waist, where you wear your jeans",
  seat: "Seat",
  thigh_top: "Thigh at the top",
  thigh_1in: "Thigh 2.5 cm (1″) lower",
  leg_12in: "Leg 30.5 cm (12″) below the crotch",
  leg_13in: "Leg 33 cm (13″) below the crotch",
  leg_14in: "Leg 35.5 cm (14″) below the crotch",
  leg_half_inseam: "Leg halfway down the inseam",
  inseam_wanted: "Inseam, to where the hem should sit",
  inseam_extra: "Extra inseam for cuffs or stacking",
};

function renderWelcome() {
  location.hash = "#/guide/0";
}

function renderBodyGuide(stepText) {
  const index = parseInt(stepText || "0", 10);
  renderGuideScreen("body", index, S.body, {
    label: "Body measurements",
    firstBack: S.onboarded ? "Back" : "",
    skippable: false,
    formHref: "#/body",
    fieldLabel: (f) => (f.startsWith("inseam") ? BODY_FIELD_LABELS[f] : `${BODY_FIELD_LABELS[f]}, all the way around`),
    hint: (step) =>
      step.point === "half-inseam" && S.body.inseam_wanted ? `<p class="hint">Half of your inseam to the hem is ${len(S.body.inseam_wanted / 2)} below the crotch.</p>` : "",
    extra: (step, i) =>
      i === 0
        ? `${S.onboarded ? "" : `<div class="card"><p><b>Real Denim Fit</b> finds raw denim jeans worth looking into, in the right size, from your own measurements. Everything you enter stays on this computer.</p></div>`}
           <label>Units</label>${choiceGroup("units", [["cm", "Centimetres"], ["in", "Inches"]], S.settings.units)}`
        : "",
    afterRender: () =>
      $app.querySelectorAll('input[name="units"]').forEach((r) =>
        r.addEventListener("change", () => {
          S.settings.units = r.value;
          save();
        }),
      ),
    save,
    go: (to, total) => {
      if (to < 0) location.hash = S.onboarded ? "#/body" : "#/guide/0";
      else if (total !== undefined && to >= total) location.hash = S.onboarded ? "#/body" : bodyComplete() ? "#/fit" : "#/body";
      else location.hash = `#/guide/${to}`;
    },
  });
}

function renderBodyForm() {
  $app.innerHTML = `
  <div class="narrow">
    <h1>Body measurements</h1>
    <p class="muted">All the way around your body, except the inseam. Change any number; matches update straight away.</p>
    <div class="card">
      ${BODY_FIELDS.map((f) => `<label for="b-${f}">${esc(BODY_FIELD_LABELS[f])}</label>${measureInput(`b-${f}`, S.body[f])}`).join("")}
      <label for="b-inseam_extra">${esc(BODY_FIELD_LABELS.inseam_extra)} <span class="hint">(optional)</span></label>${measureInput("b-inseam_extra", S.body.inseam_extra || null)}
      <p class="error" id="err" hidden></p>
    </div>
    <div class="actions">
      <button id="guide">Go through the measuring guide</button>
      <button class="primary" id="save">Save</button>
    </div>
  </div>`;
  document.getElementById("guide").onclick = () => (location.hash = "#/guide/0");
  document.getElementById("save").onclick = () => {
    const values = BODY_FIELDS.map((f) => [f, readMeasure(`b-${f}`)]);
    if (values.some(([, v]) => v === null)) {
      const err = document.getElementById("err");
      err.hidden = false;
      err.textContent = "Every body measurement is needed. Please fill in the empty or unreadable ones.";
      return;
    }
    for (const [f, v] of values) S.body[f] = v;
    const extra = readMeasure("b-inseam_extra");
    if (extra === null) delete S.body.inseam_extra;
    else S.body.inseam_extra = extra;
    save();
    location.hash = S.onboarded ? "#/results" : "#/fit";
  };
}

// --------------------------------------------------------- desired fit ----

function usualName(fit) {
  const room = { snug: "slim", regular: "regular", roomy: "relaxed" }[fit.room];
  const leg = { tapered: "tapered", straight: "straight", wide: "wide leg" }[fit.leg_shape] || "fit";
  const name = `${room} ${leg}`;
  if (fit.rise === "low") return `low-rise ${name}`;
  if (fit.rise === "high") return `high-rise ${name}`;
  return name;
}

function importanceOf(m) {
  return S.desiredFit.importance?.[m] ?? 3;
}

function importanceControl(m) {
  const v = importanceOf(m);
  return `<div class="importance-row">
    <span class="importance-name">${cap(LABELS[m])}</span>
    <span class="importance" role="radiogroup" aria-label="Importance of the ${LABELS[m]}, from least to most">
      <span class="importance-end">Least</span>
      ${[1, 2, 3, 4, 5].map((n) => `<label class="dot ${n <= v ? "on" : ""}" title="${n}"><input type="radio" name="imp-${m}" value="${n}" ${n === v ? "checked" : ""} aria-label="${n}"/></label>`).join("")}
      <span class="importance-end">Most</span>
    </span>
  </div>`;
}

function renderFit() {
  const f = S.desiredFit;
  if (!f.importance) f.importance = {};
  $app.innerHTML = `
  <div class="fit-page">
    <h1>Desired fit</h1>
    <p class="muted">How you want your jeans to sit. The waist isn't a choice: it should fit your waist.</p>
    <div class="fit-layout">
      <div class="card">
        <h2 style="margin-top:0" id="usual">${cap(usualName(f))}</h2>
        <label>Room through the seat and thigh</label>
        ${choiceGroup("room", [["snug", "Snug"], ["regular", "Regular"], ["roomy", "Roomy"]], f.room)}
        <label>Leg shape below the knee</label>
        ${choiceGroup("leg_shape", [["tapered", "Tapered"], ["straight", "Straight"], ["wide", "Wide"], ["", "Any"]], f.leg_shape ?? "")}
        <label>Rise</label>
        ${choiceGroup("rise", [["low", "Low"], ["mid", "Mid"], ["high", "High"], ["", "Any"]], f.rise ?? "")}
        <p class="hint">Leg shape and rise are worked out from each size's own numbers, not from what brands call their cuts.</p>
      </div>
      <div class="card">
        <h2 style="margin-top:0">Importance</h2>
        <p class="hint">How much each measurement counts when ranking matches. Pairs too tight for you are left out at any importance.</p>
        ${MEASUREMENTS.map(importanceControl).join("")}
      </div>
    </div>
    <div class="actions"><span></span><button class="primary" id="done">${S.onboarded ? "Show matches" : "See my matches"}</button></div>
  </div>`;
  const update = () => {
    f.room = $app.querySelector('input[name="room"]:checked').value;
    f.leg_shape = $app.querySelector('input[name="leg_shape"]:checked').value || null;
    f.rise = $app.querySelector('input[name="rise"]:checked').value || null;
    for (const m of MEASUREMENTS) {
      const v = Number($app.querySelector(`input[name="imp-${m}"]:checked`).value);
      if (v === 3) delete f.importance[m];
      else f.importance[m] = v;
      $app.querySelectorAll(`input[name="imp-${m}"]`).forEach((r) => r.parentElement.classList.toggle("on", Number(r.value) <= v));
    }
    document.getElementById("usual").textContent = cap(usualName(f));
    save();
  };
  $app.querySelectorAll("input").forEach((el) => el.addEventListener("change", update));
  document.getElementById("done").onclick = () => {
    S.onboarded = true;
    save();
    location.hash = "#/results";
  };
}

// ------------------------------------------------------------- results ----

/** What the results were matched on: a pair only counts when it has readings. */
function compareHint(results) {
  if (!results) return "";
  return results.from_body_only
    ? "Matched from your body and typical ease"
    : "Matched to this pair. Your body still rules out anything too tight.";
}

function filtersForRust() {
  const f = S.filters;
  const excluded = f.excluded_brands || [];
  const { excluded_brands: _, ...rest } = f;
  return {
    ...rest,
    brands: excluded.length ? CAT.brands.map((b) => b.id).filter((id) => !excluded.includes(id)) : null,
    include_washed: f.include_washed ?? false,
    mens: f.mens ?? true,
    womens: f.womens ?? true,
    currency: S.settings.currency,
    try_on_brands: S.settings.tryOnBrands,
  };
}

function profileForRust() {
  return { body: S.body, desired_fit: S.desiredFit, reference_pair: activePair() };
}

function offsText(match) {
  const offs = match.measurements
    .filter((m) => m.closeness !== null && m.off_by !== null && Math.abs(m.off_by) >= 0.15 && m.closeness < 99.5)
    .sort((a, b) => a.closeness - b.closeness)
    .slice(0, 4)
    .map((m) => {
      const shown = m.target.target.kind === "at-least" ? `${LABELS[m.measurement]} ${len(m.off_by, true)}` : `${LABELS[m.measurement]} ${len(shownValue(m.measurement, m.off_by), true)}`;
      return `<span class="off ${closenessClass(m.closeness)}">${esc(shown)}</span>`;
    });
  return offs.length ? offs.join(", ") : `<span class="c-close">on target everywhere it's compared</span>`;
}

function matchTags(m) {
  const tags = [];
  if (m.basis === "shrinkage-unknown") tags.push(["shrinkage unknown", true]);
  if (m.basis === "from-shrinkage") tags.push(["after wash estimated from the brand's shrinkage", false]);
  if (m.approximate) tags.push(["approximate", true]);
  if (m.stretch) tags.push(["stretch", false]);
  if (!m.on_brands_site) tags.push(["no longer on the brand's site", true]);
  if (!m.fits_shape_and_rise) tags.push([`${m.leg_shape || "leg shape unknown"}, ${m.rise ? `${m.rise} rise` : "rise unknown"}`, false]);
  return `<div class="tags">${tags.map(([t, warn]) => `<span class="tag ${warn ? "warn" : ""}">${esc(t)}</span>`).join("")}</div>`;
}

function priceText(m) {
  const p = PRODUCTS.get(m.product_id);
  if (m.price === null || !p.list_price) return "";
  const same = p.list_price.currency === S.settings.currency;
  return `${same ? "" : "about "}${Units.money(m.price, S.settings.currency)}`;
}

function subRow(m, what) {
  const p = PRODUCTS.get(m.product_id);
  return `<div class="sub" data-open="${esc(m.product_id)}" data-size="${esc(m.size)}" style="cursor:pointer">
    <span class="pct ${closenessClass(m.closeness)}" style="font-size:1rem">${Math.round(m.closeness)}%</span>
    <span>${what === "size" ? `Size ${esc(m.size)}` : `${esc(p.name)} · size ${esc(m.size)}`}<div class="offs">${offsText(m)}</div></span>
    <span class="price">${priceText(m)}</span>
  </div>`;
}

let lastResults = null;

async function renderResults() {
  if (!bodyComplete()) {
    location.hash = "#/guide/0";
    return;
  }
  const pairs = S.pairs;
  const f = S.filters;
  $app.innerHTML = `
  <div class="results">
    <aside class="card filters" id="filters">
      <h3>Compare with</h3>
      <div role="radiogroup" aria-label="Compare with">
        <label class="checkline wrap"><input type="radio" name="compare" value="" ${S.activePairId ? "" : "checked"}/> <span>My body</span></label>
        ${pairs.map((p) => `<label class="checkline wrap"><input type="radio" name="compare" value="${esc(p.id)}" ${p.id === S.activePairId ? "checked" : ""}/> <span>${esc(p.name)}</span></label>`).join("")}
      </div>
      <p class="hint" id="compare-hint"></p>
      <h3>Leg shape and rise</h3>
      <p class="small">${cap(usualName(S.desiredFit))} · <a href="#/fit">change</a></p>
      <label class="checkline"><input type="checkbox" id="f-rest" ${f.show_other_shapes_and_rises ? "checked" : ""}/> Show other leg shapes and rises</label>
      <h3>Line</h3>
      <label class="checkline"><input type="checkbox" id="f-mens" ${f.mens ?? true ? "checked" : ""}/> Men's</label>
      <label class="checkline"><input type="checkbox" id="f-womens" ${f.womens ?? true ? "checked" : ""}/> Women's</label>
      <p class="hint">Unisex cuts show with either. Cuts the brand doesn't file as men's or women's always show.</p>
      <h3>Brands</h3>
      ${CAT.brands.map((b) => `<label class="checkline"><input type="checkbox" data-brand="${b.id}" ${(f.excluded_brands || []).includes(b.id) ? "" : "checked"}/> ${esc(b.name)}</label>`).join("")}
      <label class="checkline"><input type="checkbox" id="f-tryon" ${f.try_on_only ? "checked" : ""} ${S.settings.tryOnBrands.length ? "" : "disabled"}/> Try-on brands only</label>
      ${S.settings.tryOnBrands.length ? "" : `<p class="hint">Mark brands you can try on in <a href="#/settings">Settings</a>.</p>`}
      <h3>Price (${esc(S.settings.currency)})</h3>
      <div class="row2"><input type="number" min="0" id="f-pmin" placeholder="from" value="${f.price_min ?? ""}"/><input type="number" min="0" id="f-pmax" placeholder="to" value="${f.price_max ?? ""}"/></div>
      <h3>Fabric weight (oz)</h3>
      <div class="row2"><input type="number" min="0" id="f-wmin" placeholder="from" value="${f.weight_min ?? ""}"/><input type="number" min="0" id="f-wmax" placeholder="to" value="${f.weight_max ?? ""}"/></div>
      <h3>Fabric</h3>
      <label class="checkline"><input type="checkbox" id="f-sanf" ${f.sanforized_only ? "checked" : ""}/> Sanforized only</label>
      <label class="checkline"><input type="checkbox" id="f-nostretch" ${f.no_stretch ? "checked" : ""}/> No stretch</label>
      <label class="checkline"><input type="checkbox" id="f-washed" ${f.include_washed ? "checked" : ""}/> Include washed denim</label>
      <label class="checkline"><input type="checkbox" id="f-current" ${f.current_only ? "checked" : ""}/> Current products only</label>
    </aside>
    <section>
      <div id="summary"></div>
      <div id="rows"><p class="muted">Working it out…</p></div>
    </section>
  </div>`;

  const num = (id) => {
    const v = parseFloat(document.getElementById(id).value);
    return Number.isFinite(v) ? v : null;
  };
  const onChange = () => {
    S.activePairId = $app.querySelector('input[name="compare"]:checked')?.value || null;
    // The brands left out are saved, so a brand added in a later release
    // starts ticked.
    f.excluded_brands = [...$app.querySelectorAll("[data-brand]:not(:checked)")].map((c) => c.dataset.brand);
    f.try_on_only = document.getElementById("f-tryon").checked;
    f.mens = document.getElementById("f-mens").checked;
    f.womens = document.getElementById("f-womens").checked;
    f.price_min = num("f-pmin");
    f.price_max = num("f-pmax");
    f.weight_min = num("f-wmin");
    f.weight_max = num("f-wmax");
    f.sanforized_only = document.getElementById("f-sanf").checked;
    f.no_stretch = document.getElementById("f-nostretch").checked;
    f.include_washed = document.getElementById("f-washed").checked;
    f.current_only = document.getElementById("f-current").checked;
    f.show_other_shapes_and_rises = document.getElementById("f-rest").checked;
    save();
    refreshResults();
  };
  document.getElementById("filters").addEventListener("change", onChange);
  refreshResults();
}

async function refreshResults() {
  let results;
  try {
    results = await invoke("match_catalogue", { profile: profileForRust(), filters: filtersForRust() });
  } catch (e) {
    showError(e);
    return;
  }
  lastResults = results;
  const hint = document.getElementById("compare-hint");
  if (hint) hint.textContent = compareHint(results);
  const summary = document.getElementById("summary");
  const rowsEl = document.getElementById("rows");
  if (!summary) return;
  const notes = [];
  if (results.from_body_only)
    notes.push(`<div class="note">Matched from your body and typical ease. Adding a pair you know makes this sharper. <a href="#/pairs">Add a reference pair</a></div>`);
  if (results.hidden_by_shape_or_rise && !S.filters.show_other_shapes_and_rises)
    notes.push(`<p class="muted small">${results.hidden_by_shape_or_rise} more cut${results.hidden_by_shape_or_rise === 1 ? "" : "s"} with another leg shape or rise. <button class="link" id="show-rest">Show them</button></p>`);
  summary.innerHTML = `<h1>${results.rows.length} cut${results.rows.length === 1 ? "" : "s"} worth looking into</h1>${notes.join("")}`;
  const showRest = document.getElementById("show-rest");
  if (showRest)
    showRest.onclick = () => {
      S.filters.show_other_shapes_and_rises = true;
      document.getElementById("f-rest").checked = true;
      save();
      refreshResults();
    };
  if (!results.rows.length) {
    rowsEl.innerHTML = `<div class="card"><p>Nothing fits with these filters. Try loosening the filters, choosing "any" leg shape or rise, or checking your body measurements.</p></div>`;
    return;
  }
  rowsEl.innerHTML = results.rows
    .slice(0, 150)
    .map((row, i) => {
      const m = row.best;
      const p = PRODUCTS.get(m.product_id);
      return `<div class="card match-row">
        <div class="match-head" data-row="${i}">
          <span class="pct ${closenessClass(m.closeness)}">${Math.round(m.closeness)}%</span>
          <div>
            <div class="title">${esc(row.brand_name)} ${esc(row.cut)}${row.line ? ` <span class="tag">${LINE_TEXT[row.line]}</span>` : ""} <span class="muted" style="font-weight:normal">· size ${esc(m.size)}${row.brand_label ? ` · the brand calls it “${esc(row.brand_label)}”` : ""}</span></div>
            <div class="small muted">${esc(p.name)}${p.fabric_weight_oz ? ` · ${p.fabric_weight_oz} oz` : ""}</div>
            <div class="offs">${offsText(m)}</div>
            ${matchTags(m)}
          </div>
          <span class="price">${priceText(m)}</span>
        </div>
        <div class="match-more" id="more-${i}" hidden></div>
      </div>`;
    })
    .join("");
  rowsEl.querySelectorAll("[data-row]").forEach((head) =>
    head.addEventListener("click", () => {
      const i = Number(head.dataset.row);
      const more = document.getElementById(`more-${i}`);
      if (!more.hidden) {
        more.hidden = true;
        return;
      }
      const row = lastResults.rows[i];
      more.innerHTML = `
        <div class="actions" style="margin:0.3rem 0 0.4rem"><span class="small muted">${row.best.compared} measurement${row.best.compared === 1 ? "" : "s"} compared${notPublished(row.best)}</span><button data-open="${esc(row.best.product_id)}" data-size="${esc(row.best.size)}">Product details</button></div>
        ${row.other_products.length ? `<h3>Other fabrics of the ${esc(row.cut)}</h3>${row.other_products.map((m) => subRow(m, "product")).join("")}` : ""}
        ${row.other_sizes.length ? `<h3>Other sizes</h3>${row.other_sizes.map((m) => subRow(m, "size")).join("")}` : ""}`;
      more.hidden = false;
      more.querySelectorAll("[data-open]").forEach((el) =>
        el.addEventListener("click", (e) => {
          e.stopPropagation();
          location.hash = `#/product/${encodeURIComponent(el.dataset.open)}/${encodeURIComponent(el.dataset.size)}`;
        }),
      );
    }),
  );
}

function notPublished(m) {
  const missing = m.measurements.filter((x) => x.value === null && x.target).map((x) => LABELS[x.measurement]);
  return missing.length ? ` · ${missing.join(", ")} not published` : "";
}

// ------------------------------------------------------------- product ----

function describeTolerance(t) {
  if (!t) return "not stated by the brand";
  return t.unit === "in" ? `±${t.value}″` : `±${t.value} cm`;
}

const BASIS_TEXT = {
  "brand-chart": "the brand's own after-wash chart",
  "sold-washed": "sold already washed, so the chart is how it will be",
  "from-shrinkage": "estimated from the brand's shrinkage",
  "shrinkage-unknown": "shrinkage unknown: the brand doesn't say how much it shrinks, so the chart is used as sold",
};

const POINT_TEXT = {
  "at-crotch": "at the crotch",
  "1in-below-crotch": "2.5 cm (1″) below the crotch",
  "12in-below-crotch": "30.5 cm (12″) below the crotch",
  "13in-below-crotch": "33 cm (13″) below the crotch",
  "14in-below-crotch": "35.5 cm (14″) below the crotch",
  "half-inseam": "halfway down the inseam",
  fullest: "at the fullest part",
  "16cm-below-waistband": "16 cm below the waistband",
  unknown: "the brand doesn't say where",
};

function compositionText(fabric) {
  if (!fabric.composition) return "not stated";
  return fabric.composition.map((f) => `${f.percent}% ${f.fibre}`).join(", ");
}

function shrinkageText(s) {
  if (s.kind === "percent") return `${s.width}% in width, ${s.length}% in length`;
  const u = s.unit === "in" ? "″" : " cm";
  const parts = [];
  if (s.waist !== null && s.waist !== undefined) parts.push(`${s.waist}${u} off the waist`);
  if (s.inseam !== null && s.inseam !== undefined) parts.push(`${s.inseam}${u} off the inseam`);
  return parts.join(", ");
}

async function renderProduct(id, size) {
  $app.innerHTML = `<p class="muted">Loading…</p>`;
  let d;
  try {
    d = await invoke("product_details", { productId: id });
  } catch (e) {
    showError(e);
    return;
  }
  const { brand, product: p, washed, effective } = d;
  const sizes = washed ? washed.sizes : [];
  const current = sizes.find((s) => s.size === size) || sizes[0];
  const mainIndex = Math.max(0, p.charts.findIndex((c) => c.state === "after-wash"));
  const chartIndex = p.charts.some((c) => c.state === "after-wash") ? mainIndex : 0;
  const eff = effective[chartIndex];
  const estimated = washed && washed.basis === "from-shrinkage";
  const shown = (m, v) => (v === undefined || v === null ? null : shownValue(m, v));
  const elastane = (p.fabric.composition || [])
    .filter((f) => ["elastane", "polyurethane", "spandex", "lycra", "pu"].includes(f.fibre.toLowerCase()))
    .reduce((a, f) => a + f.percent, 0);
  const lp = p.list_price;
  const converted = lp ? priceText({ product_id: id, price: convertPrice(lp) }) : "";
  const pointFor = (m) => {
    if (m === "thigh" || m === "knee" || m === "seat") return POINT_TEXT[eff.measuring_points[m]] || eff.measuring_points[m] || "";
    if (LENGTHS.includes(m)) return "length";
    return eff.conventions[m] === "around" ? "all the way around" : "flat";
  };

  $app.innerHTML = `
  <p><button class="link" id="back">← Back</button></p>
  <h1>${esc(brand.name)} ${esc(d.cut)}${d.line ? ` <span class="tag">${LINE_TEXT[d.line]}</span>` : ""}</h1>
  <p class="muted">${esc(p.name)}${d.brand_label ? ` · the brand calls the cut “${esc(d.brand_label)}”` : ""}</p>
  ${p.status === "discontinued" ? `<div class="note">No longer on the brand's site (last checked ${esc(p.status_seen)}).</div>` : ""}
  ${washed && washed.basis === "shrinkage-unknown" ? `<div class="note">Shrinkage unknown: the brand doesn't say how much this shrinks in its first wash, so it's compared as sold and ranked after the rest.</div>` : ""}
  ${washed && washed.approximate ? `<div class="note">Approximate: ${esc(brand.name)} doesn't say exactly where it measures the thigh or knee, so the nearest of your readings is used.</div>` : ""}
  <div class="card">
    <div class="grid2">
      <p><b>Fabric weight</b><br>${p.fabric.weight_oz ? `${p.fabric.weight_oz} oz` : "not stated"}</p>
      <p><b>Composition</b><br>${esc(compositionText(p.fabric))}${elastane > 0 ? ` (${elastane}% elastane: stretch)` : ""}</p>
      <p><b>Sanforized</b><br>${p.fabric.sanforized === true ? "Yes" : p.fabric.sanforized === false ? "No (unsanforized)" : "The brand doesn't say"}</p>
      <p><b>As sold</b><br>${p.sold === "one-washed" ? "Already washed by the brand" : p.sold === "raw" ? "Raw (unwashed)" : "The brand doesn't say"}</p>
      <p><b>Finish</b><br>${{ raw: "Raw: never washed", rinsed: "Rinsed: washed once, not faded", washed: "Washed: faded on purpose at the factory" }[p.finish] || "Not clear from the brand's page"}</p>
      <p><b>After first wash</b><br>${washed ? esc(BASIS_TEXT[washed.basis]) : "no measurements"}</p>
      ${p.shrinkage ? `<p><b>Shrinkage</b><br>${esc(shrinkageText(p.shrinkage))}${p.shrinkage.note ? `<br><span class="small muted">“${esc(p.shrinkage.note)}”</span>` : ""}<br><span class="small muted"><a href="${esc(p.shrinkage.source.url)}" target="_blank" rel="noopener">source</a>, seen ${esc(p.shrinkage.source.seen)}</span></p>` : ""}
      <p><b>Tolerance</b><br>${describeTolerance(eff ? eff.tolerance : null)}</p>
      <p><b>List price</b><br>${lp ? `${Units.money(lp.amount, lp.currency)}${lp.currency !== S.settings.currency && converted ? ` (${converted})` : ""}<br><span class="small muted">seen ${esc(lp.seen)} on <a href="${esc(lp.url)}" target="_blank" rel="noopener">the brand's site</a></span>` : "not published"}</p>
      <p><b>Brand page</b><br><a href="${esc(p.url)}" target="_blank" rel="noopener">See it on ${esc(brand.name)}'s site</a></p>
    </div>
    ${elastane === 0 ? `<p class="hint">Rigid denim: the waist usually stretches about ½–1″ with wear. That isn't added to the numbers.</p>` : ""}
  </div>
  ${current ? `
  <div class="card">
    <div class="actions" style="margin-top:0"><h2 style="margin:0">Size</h2>
      <select id="size" style="max-width:180px">${sizes.map((s) => `<option ${s === current ? "selected" : ""}>${esc(s.size)}</option>`).join("")}</select>
    </div>
    <div class="table-wrap"><table>
      <thead><tr><th>Measurement</th><th>As sold</th><th>After first wash</th><th>How it's measured</th></tr></thead>
      <tbody>${MEASUREMENTS.map((m) => {
        const after = current.after[m];
        const sold = current.as_sold ? current.as_sold[m] : undefined;
        return `<tr><td>${cap(LABELS[m])}</td>
          <td>${sold !== undefined ? len(shown(m, sold)) : after === undefined ? "not published" : "—"}</td>
          <td>${after !== undefined ? len(shown(m, after)) + (estimated ? "*" : "") : "not published"}</td>
          <td class="small muted">${after !== undefined || sold !== undefined ? esc(pointFor(m)) : ""}</td></tr>`;
      }).join("")}</tbody>
    </table></div>
    ${estimated ? `<p class="hint">* Estimated from the brand's shrinkage.</p>` : ""}
    <p class="hint">Widths are shown straight across the jeans laid flat, except the waist, shown all the way around, whatever way the brand published them.</p>
  </div>` : `<div class="card"><p>This product has no published measurements, so it isn't matched.</p></div>`}
  <div class="card">
    <h3 style="margin-top:0">Where the numbers come from</h3>
    <ul class="small">${p.charts.map((c) => `<li>${c.state === "after-wash" ? `After-wash chart${c.estimated_by_brand ? " (the brand calls it an estimate)" : ""}` : "As-sold chart"}: <a href="${esc(c.source.url)}" target="_blank" rel="noopener">${esc(c.source.url)}</a>, seen ${esc(c.source.seen)}, in ${c.unit === "in" ? "inches" : "cm"}${c.from_image ? ", <b>read from an image</b>" : ""}.</li>`).join("")}
    <li>How ${esc(brand.name)} measures: <a href="${esc(brand.method_source.url)}" target="_blank" rel="noopener">${esc(brand.method_source.url)}</a>, seen ${esc(brand.method_source.seen)}.</li></ul>
    <p class="hint">Real Denim Fit never shows shops, stock or sale prices. Check the brand's page for the jeans themselves.</p>
  </div>
  ${sizes.length ? `<div class="card"><h3 style="margin-top:0">All sizes after the first wash</h3><div class="table-wrap"><table>
    <thead><tr><th>Size</th>${MEASUREMENTS.map((m) => `<th>${cap(LABELS[m])}</th>`).join("")}</tr></thead>
    <tbody>${sizes.map((s) => `<tr class="${s === current ? "current" : ""}"><td>${esc(s.size)}</td>${MEASUREMENTS.map((m) => `<td>${s.after[m] !== undefined ? len(shown(m, s.after[m])) : "—"}</td>`).join("")}</tr>`).join("")}</tbody>
  </table></div></div>` : ""}`;
  document.getElementById("back").onclick = () => history.back();
  const sel = document.getElementById("size");
  if (sel) sel.onchange = () => (location.hash = `#/product/${encodeURIComponent(id)}/${encodeURIComponent(sel.value)}`);
}

function convertPrice(lp) {
  const rates = CAT.rates;
  if (!rates) return null;
  const rate = (c) => (c === "EUR" ? 1 : rates[c]);
  if (!rate(lp.currency) || !rate(S.settings.currency)) return null;
  return (lp.amount / rate(lp.currency)) * rate(S.settings.currency);
}

// -------------------------------------------------------------- pairs ----

function newId() {
  return `pair-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 6)}`;
}

function pairSourceText(pair) {
  if (pair.source.kind === "from-catalogue") {
    const p = PRODUCTS.get(pair.source.product_id);
    return p ? `${p.brand.name} ${p.cut}, ${p.name}, size ${pair.source.size} · from the brand's chart` : "from the brand's chart (product no longer in the catalogue)";
  }
  const n = Object.values(pair.source.readings).filter((v) => typeof v === "number").length;
  return `measured by you · ${n} reading${n === 1 ? "" : "s"}`;
}

function renderPairs() {
  $app.innerHTML = `
  <div class="narrow">
    <h1>Reference pairs</h1>
    <p class="muted">A pair of jeans whose fit you know. Where it has a reading, matches aim at that pair, shifted by how it feels. Optional, and as many as you like.</p>
    ${S.pairs.length ? S.pairs.map((p) => `
      <div class="card">
        <div class="actions" style="margin:0">
          <div><b>${esc(p.name)}</b>${p.id === S.activePairId ? ` <span class="tag">compared with</span>` : ""}<div class="small muted">${esc(pairSourceText(p))}</div></div>
          <a class="button" href="#/pair/${encodeURIComponent(p.id)}">Open</a>
        </div>
      </div>`).join("") : `<div class="card"><p>No reference pairs yet.</p></div>`}
    <div class="actions">
      <button id="measure">Add a pair I can measure</button>
      <button id="pick">Add a pair from the catalogue</button>
    </div>
    <p class="hint">Picked from the catalogue works for a pair you've only tried on in a shop too.</p>
  </div>`;
  document.getElementById("measure").onclick = () => {
    const pair = { id: newId(), name: "Reference pair", source: { kind: "measured", readings: {} }, feel: {}, notes: "" };
    S.pairs.push(pair);
    if (!S.activePairId) S.activePairId = pair.id;
    save();
    location.hash = `#/pair/${pair.id}/measure/0`;
  };
  document.getElementById("pick").onclick = () => (location.hash = "#/pair/new/catalogue");
}

function renderPairMeasure(pair, stepText) {
  const r = pair.source.readings;
  const base = `#/pair/${encodeURIComponent(pair.id)}`;
  renderGuideScreen("pair", parseInt(stepText || "0", 10), r, {
    label: pair.name,
    firstBack: "Cancel",
    skippable: true,
    formHref: base,
    fieldLabel: (f) => `${PAIR_FIELD_LABELS[f]}, flat`.replace("Front rise, flat", "Front rise").replace("Back rise, flat", "Back rise").replace("Inseam, flat", "Inseam"),
    hint: (step) => {
      if (step.measurement === "waist") return `<p class="hint" id="doubled"></p>`;
      if (step.point === "half-inseam" && r.inseam) return `<p class="hint">Half of this pair's inseam is ${len(r.inseam / 2)}.</p>`;
      return "";
    },
    extra: (step, i) => (i === 0 ? `<label for="pname">Name this pair</label><input type="text" id="pname" value="${esc(pair.name)}"/>` : ""),
    afterRender: (step) => {
      const doubled = document.getElementById("doubled");
      if (!doubled) return;
      const show = () => {
        const v = readMeasure("g-value");
        doubled.textContent = v ? `That's a waist of ${len(v * 2)} all the way around.` : "";
      };
      document.getElementById("g-value").addEventListener("input", show);
      show();
    },
    beforeNext: () => {
      const name = document.getElementById("pname");
      if (name) pair.name = name.value.trim() || "Reference pair";
    },
    save,
    go: (to, total) => {
      if (to < 0) {
        if (!Object.keys(r).length) {
          removePair(pair.id);
        }
        location.hash = "#/pairs";
      } else if (total !== undefined && to >= total) location.hash = base;
      else location.hash = `${base}/measure/${to}`;
    },
  });
}

function renderPairPick() {
  const brands = CAT.brands;
  $app.innerHTML = `
  <div class="narrow">
    <h1>Pick a pair from the catalogue</h1>
    <p class="muted">Its size's after-wash measurements become the readings, marked "from the brand's chart".</p>
    <div class="card">
      <label for="pb">Brand</label><select id="pb"><option value="">Choose…</option>${brands.map((b) => `<option value="${b.id}">${esc(b.name)}</option>`).join("")}</select>
      <label for="pc">Cut</label><select id="pc" disabled></select>
      <label for="pp">Fabric</label><select id="pp" disabled></select>
      <label for="ps">Size</label><select id="ps" disabled></select>
      <label for="pn">Name this pair</label><input type="text" id="pn" placeholder="For example: my old 634S"/>
    </div>
    <div class="actions"><a class="button" href="#/pairs">Cancel</a><button class="primary" id="add" disabled>Add pair</button></div>
  </div>`;
  const [pb, pc, pp, ps, pn, add] = ["pb", "pc", "pp", "ps", "pn", "add"].map((x) => document.getElementById(x));
  const fill = (sel, options) => {
    sel.innerHTML = `<option value="">Choose…</option>` + options.map(([v, l]) => `<option value="${esc(v)}">${esc(l)}</option>`).join("");
    sel.disabled = !options.length;
  };
  pb.onchange = () => {
    const b = BRANDS.get(pb.value);
    const cuts = b ? [...new Map(b.products.filter((p) => p.sizes.length).map((p) => [p.cut_id, p.line ? `${p.cut} (${LINE_TEXT[p.line]})` : p.cut])).entries()] : [];
    fill(pc, cuts.sort((x, y) => x[1].localeCompare(y[1])));
    fill(pp, []);
    fill(ps, []);
    add.disabled = true;
  };
  pc.onchange = () => {
    const b = BRANDS.get(pb.value);
    fill(pp, b.products.filter((p) => p.cut_id === pc.value && p.sizes.length).map((p) => [p.id, `${p.name}${p.current ? "" : " (no longer on the brand's site)"}`]));
    fill(ps, []);
    add.disabled = true;
  };
  pp.onchange = () => {
    const p = PRODUCTS.get(pp.value);
    fill(ps, p ? p.sizes.map((s) => [s, s]) : []);
    add.disabled = true;
  };
  ps.onchange = () => {
    add.disabled = !ps.value;
    const p = PRODUCTS.get(pp.value);
    if (p && !pn.value) pn.placeholder = `${p.brand.name} ${p.cut} ${ps.value}`;
  };
  add.onclick = () => {
    const p = PRODUCTS.get(pp.value);
    const pair = {
      id: newId(),
      name: pn.value.trim() || `${p.brand.name} ${p.cut} ${ps.value}`,
      source: { kind: "from-catalogue", product_id: pp.value, size: ps.value },
      feel: {},
      notes: "",
    };
    S.pairs.push(pair);
    if (!S.activePairId) S.activePairId = pair.id;
    save();
    location.hash = `#/pair/${encodeURIComponent(pair.id)}`;
  };
}

async function renderPair(id, sub, step) {
  if (id === "new" && sub === "catalogue") return renderPairPick();
  const pair = S.pairs.find((p) => p.id === id);
  if (!pair) {
    location.hash = "#/pairs";
    return;
  }
  if (sub === "measure") return renderPairMeasure(pair, step);
  const measured = pair.source.kind === "measured";
  const r = measured ? pair.source.readings : {};
  const feelRow = (m) => {
    const lengthy = LENGTHS.includes(m);
    const opts = lengthy
      ? [["too-short", "Too short"], ["just-right", "Just right"], ["too-long", "Too long"]]
      : [["too-tight", "Too tight"], ["just-right", "Just right"], ["too-loose", "Too loose"]];
    return `<div><label>${cap(LABELS[m])}</label>${choiceGroup(`feel-${m}`, opts, pair.feel[m] || "just-right")}</div>`;
  };
  $app.innerHTML = `
  <div class="narrow">
    <p><a href="#/pairs">← Reference pairs</a></p>
    <h1>${esc(pair.name)}</h1>
    <p class="muted">${esc(pairSourceText(pair))}</p>
    <div class="card">
      <label class="checkline"><input type="checkbox" id="active" ${S.activePairId === pair.id ? "checked" : ""}/> Compare matches with this pair</label>
      <label for="name">Name</label><input type="text" id="name" value="${esc(pair.name)}"/>
    </div>
    ${measured ? `
    <div class="card">
      <div class="actions" style="margin-top:0"><h2 style="margin:0">Readings</h2><a class="button" href="#/pair/${encodeURIComponent(pair.id)}/measure/0">Measuring guide</a></div>
      <p class="hint">Measured flat, straight across. Leave a box empty to skip it.</p>
      <div class="grid2">${PAIR_FIELDS.map((f) => `<div><label for="r-${f}">${PAIR_FIELD_LABELS[f]}</label>${measureInput(`r-${f}`, r[f])}</div>`).join("")}</div>
    </div>` : ""}
    <div class="card">
      <h2 style="margin-top:0">How it feels</h2>
      <p class="hint">Each "too" nudges the target ¼″ (0.6 cm) the other way.</p>
      <div class="grid2">${MEASUREMENTS.map(feelRow).join("")}</div>
    </div>
    <div class="card" id="ease-card"><h2 style="margin-top:0">This pair's own ease</h2><p class="muted">Working it out…</p></div>
    <div class="card">
      <label for="notes">Notes, for you only</label>
      <textarea id="notes" placeholder="Never used in matching.">${esc(pair.notes)}</textarea>
    </div>
    <div class="actions"><button id="delete">Delete this pair</button><button class="primary" id="done">Done</button></div>
  </div>`;

  const collect = () => {
    pair.name = document.getElementById("name").value.trim() || pair.name;
    pair.notes = document.getElementById("notes").value;
    if (measured)
      for (const f of PAIR_FIELDS) {
        const v = readMeasure(`r-${f}`);
        if (v === null) delete r[f];
        else r[f] = v;
      }
    for (const m of MEASUREMENTS) {
      const v = $app.querySelector(`input[name="feel-${m}"]:checked`).value;
      if (v === "just-right") delete pair.feel[m];
      else pair.feel[m] = v;
    }
    const active = document.getElementById("active").checked;
    if (active) S.activePairId = pair.id;
    else if (S.activePairId === pair.id) S.activePairId = null;
    save();
  };
  $app.querySelectorAll("input, textarea").forEach((el) => el.addEventListener("change", () => {
    collect();
    showPairEase(pair);
  }));
  document.getElementById("done").onclick = () => {
    collect();
    location.hash = "#/pairs";
  };
  askInPage(document.getElementById("delete"), `Delete "${pair.name}"? This can't be undone.`, "Delete", () => {
    removePair(pair.id);
    location.hash = "#/pairs";
  });
  showPairEase(pair);
}

async function showPairEase(pair) {
  const card = document.getElementById("ease-card");
  if (!card) return;
  let e = null;
  try {
    e = await invoke("reference_pair_ease", { body: S.body, pair });
  } catch (err) {
    console.warn(err);
  }
  if (!e || !e.points.length) {
    card.innerHTML = `<h2 style="margin-top:0">This pair's own ease</h2><p class="muted">Add a waist, seat, thigh or knee reading to see how much bigger than your body this pair is.</p>`;
    return;
  }
  // Every room has the same waist ease, so the waist is only ever usual,
  // tighter or looser.
  const place = (pt) => {
    const p = pt.place;
    if (pt.measurement === "waist")
      return p.kind === "room" ? "which is the usual waist ease" : p.kind === "tighter-than-snug" ? "which is tighter than usual" : "which is looser than usual";
    return p.kind === "room" ? `which is ${p.room}` : p.kind === "tighter-than-snug" ? "which is tighter than snug" : "which is roomier than roomy";
  };
  card.innerHTML = `<h2 style="margin-top:0">This pair's own ease</h2>
    <p class="hint">How much bigger than your body it is, all the way around.</p>
    <ul class="ease-list">${e.points.map((p) => `<li><b>${len(p.ease_around, true)}</b> ${esc(p.at)}${["waist", "seat"].includes(p.measurement) ? "" : ` (${LABELS[p.measurement]})`}, ${place(p)}</li>`).join("")}</ul>
    ${e.suggested_room ? `<p>Its seat and thigh suggest <b>${e.suggested_room}</b> room${S.desiredFit.room === e.suggested_room ? ", which is what you've chosen." : `. You've chosen ${S.desiredFit.room}. <button class="link" id="use-room">Use ${e.suggested_room}</button>`}</p>` : ""}`;
  const use = document.getElementById("use-room");
  if (use)
    use.onclick = () => {
      S.desiredFit.room = e.suggested_room;
      save();
      showPairEase(pair);
    };
}

// ----------------------------------------------------------- settings ----

function renderSettings() {
  $app.innerHTML = `
  <div class="narrow">
    <h1>Settings</h1>
    <div class="card">
      <label>Units</label>
      ${choiceGroup("units", [["cm", "Centimetres"], ["in", "Inches (⅛″ steps)"]], S.settings.units)}
      <label for="currency">Currency</label>
      <select id="currency" style="max-width:200px">${CAT.currencies.map((c) => `<option ${c === S.settings.currency ? "selected" : ""}>${c}</option>`).join("")}</select>
      <p class="hint">Prices are the brand's own list price, converted at the European Central Bank's rates of ${esc(CAT.rates_date)}, so they're shown as "about".</p>
    </div>
    <div class="card">
      <h3 style="margin-top:0">Try-on brands</h3>
      <p class="hint">Brands you can try on locally. Use "Try-on brands only" on the results to see just these.</p>
      ${CAT.brands.map((b) => `<label class="checkline"><input type="checkbox" data-tryon="${b.id}" ${S.settings.tryOnBrands.includes(b.id) ? "checked" : ""}/> ${esc(b.name)}</label>`).join("")}
    </div>
    <div class="card">
      <h3 style="margin-top:0">Your data</h3>
      <p>Everything you enter is stored on this computer only. The app's only network use is checking GitHub for updates when it opens: no analytics, no crash reports.</p>
      <button id="reset">Start again from scratch</button>
    </div>
    <p class="small muted">Real Denim Fit is free software under the GPL-3.0. <a href="https://github.com/DigitalN/Real-Denim-Fit" target="_blank" rel="noopener">Source and releases on GitHub</a>.</p>
  </div>`;
  $app.querySelectorAll('input[name="units"]').forEach((r) => r.addEventListener("change", () => {
    S.settings.units = r.value;
    save();
  }));
  document.getElementById("currency").onchange = (e) => {
    S.settings.currency = e.target.value;
    save();
  };
  $app.querySelectorAll("[data-tryon]").forEach((c) => c.addEventListener("change", () => {
    S.settings.tryOnBrands = [...$app.querySelectorAll("[data-tryon]:checked")].map((x) => x.dataset.tryon);
    if (!S.settings.tryOnBrands.length) S.filters.try_on_only = false;
    save();
  }));
  askInPage(
    document.getElementById("reset"),
    "Delete your body measurements, reference pairs and settings from this computer? This can't be undone.",
    "Delete everything",
    () => {
      S = freshState(CAT.default_currency);
      save();
      location.hash = "#/welcome";
    },
  );
}

boot();
