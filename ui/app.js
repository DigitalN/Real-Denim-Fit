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
    desiredFit: { room: "regular", leg_shape: null, rise: null, doesnt_matter: [] },
    pairs: [],
    activePairId: null,
    filters: {
      brands: null,
      price_min: null,
      price_max: null,
      try_on_only: false,
      weight_min: null,
      weight_max: null,
      sanforized_only: false,
      no_stretch: false,
      current_only: true,
      show_other_shapes_and_rises: false,
    },
  };
}

let saveTimer = null;
function save() {
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
    S = saved && saved.version === 1 ? saved : freshState(CAT.default_currency);
  } catch (e) {
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

function closenessClass(pct) {
  if (pct >= 80) return "c-close";
  if (pct >= 50) return "c-mid";
  return "c-far";
}

// The waist is shown all the way around; other widths straight across.
function shownValue(m, cm) {
  return m === "waist" ? cm * 2 : cm;
}

// ------------------------------------------------------------ welcome ----

function renderWelcome() {
  $app.innerHTML = `
  <div class="narrow">
    <h1>Real Denim Fit</h1>
    <p>Find raw denim jeans worth looking into, in the right size, from your own measurements.</p>
    <div class="card">
      <h2 style="margin-top:0">Before you start</h2>
      <ul>
        <li>Use a <b>soft tape measure</b>. Keep it snug against you without pressing in.</li>
        <li>Measure over <b>thin underwear</b>, not over jeans.</li>
        <li>Have a few strips of masking tape and a pen to hand for marking spots on your leg.</li>
        <li>It takes about five minutes. Everything you enter stays on this computer.</li>
      </ul>
      <label>Units</label>
      ${choiceGroup("units", [["cm", "Centimetres"], ["in", "Inches"]], S.settings.units)}
    </div>
    <div class="actions"><span></span><button class="primary" id="start">Start measuring</button></div>
  </div>`;
  $app.querySelectorAll('input[name="units"]').forEach((r) =>
    r.addEventListener("change", () => {
      S.settings.units = r.value;
      save();
    }),
  );
  document.getElementById("start").onclick = () => (location.hash = "#/guide/0");
}

// --------------------------------------------------------- body guide ----

const BODY_STEPS = [
  {
    title: "Waist",
    drawing: "waist",
    fields: [["waist", "Around your waist"]],
    lines: [
      "Measure where you wear your jeans. That's often lower than your natural waist, around the top of your hip bones.",
      "Keep the tape level all the way round and breathe out normally.",
    ],
    mistake: "Measuring your natural waist, above the belly button, when your jeans sit lower.",
  },
  {
    title: "Seat",
    drawing: "seat",
    fields: [["seat", "Around the fullest part"]],
    lines: ["Stand with your feet together and measure around the fullest part of your seat.", "Check in a mirror that the tape is level at the back."],
    mistake: "The tape riding up at the back, which makes the seat look smaller than it is.",
  },
  {
    title: "Thigh",
    drawing: "thigh",
    fields: [
      ["thigh_top", "Around the fullest part at the top"],
      ["thigh_1in", "Again, 2.5 cm (1″) lower"],
    ],
    lines: ["Stand with your weight on both feet and measure around the top of one leg, as high as the tape goes.", "Then slide the tape 2.5 cm (1″) down and measure again."],
    mistake: "Tensing the leg or pulling the tape tight, which makes the thigh look smaller.",
  },
  {
    title: "Inseam you want",
    drawing: "inseam",
    fields: [["inseam_wanted", "From the crotch to where you want the hem"]],
    lines: [
      "Measure down the inside of your leg, from the crotch to where you want the jeans to end, in the shoes you'll wear.",
      "Want cuffs or a stack? Add the extra now: about 5–8 cm (2–3″) for a double cuff, 3–8 cm for a stack.",
    ],
    mistake: "Measuring down the outside of the leg, or reading an old pair's inseam that was already too short.",
  },
  {
    title: "Leg",
    drawing: "leg",
    fields: [
      ["leg_12in", "Around the leg 30.5 cm (12″) below the crotch"],
      ["leg_13in", "33 cm (13″) below"],
      ["leg_14in", "35.5 cm (14″) below"],
      ["leg_half_inseam", "Halfway down the inseam"],
    ],
    lines: [
      "Brands measure the knee at different heights, so measure around your leg at each one. Put a strip of tape on the inside of your leg at each spot first.",
      "Stand straight with the knee relaxed, not bent.",
    ],
    mistake: "Measuring the heights from the floor instead of down from the crotch.",
  },
];

function renderBodyGuide(stepText) {
  const i = Math.max(0, Math.min(BODY_STEPS.length - 1, parseInt(stepText || "0", 10)));
  const step = BODY_STEPS[i];
  const halfway = S.body.inseam_wanted ? `<p class="hint">Halfway down your inseam is ${len(S.body.inseam_wanted / 2)} below the crotch.</p>` : "";
  $app.innerHTML = `
  <div class="narrow" style="max-width:820px">
    <div class="progress"><div style="width:${((i + 1) / BODY_STEPS.length) * 100}%"></div></div>
    <p class="muted small">Step ${i + 1} of ${BODY_STEPS.length}</p>
    <div class="guide">
      ${Drawings.body(step.drawing)}
      <div>
        <h1>${step.title}</h1>
        ${step.lines.map((l) => `<p>${l}</p>`).join("")}
        <p class="tip"><b>Common mistake:</b> ${step.mistake}</p>
        ${step.title === "Leg" ? halfway : ""}
        ${step.fields.map(([f, label]) => `<label for="f-${f}">${label}</label>${measureInput(`f-${f}`, S.body[f])}`).join("")}
        <p class="error" id="err" hidden>Please enter every number on this screen.</p>
      </div>
    </div>
    <div class="actions">
      <button id="back">${i === 0 ? "Back" : "Previous"}</button>
      <button class="primary" id="next">${i === BODY_STEPS.length - 1 ? "Done" : "Next"}</button>
    </div>
  </div>`;
  const first = $app.querySelector("input");
  if (first) first.focus();
  document.getElementById("back").onclick = () => (location.hash = i === 0 ? "#/welcome" : `#/guide/${i - 1}`);
  const next = () => {
    const values = step.fields.map(([f]) => [f, readMeasure(`f-${f}`)]);
    if (values.some(([, v]) => v === null)) {
      document.getElementById("err").hidden = false;
      return;
    }
    for (const [f, v] of values) S.body[f] = v;
    save();
    if (i < BODY_STEPS.length - 1) location.hash = `#/guide/${i + 1}`;
    else location.hash = S.onboarded ? "#/body" : "#/fit";
  };
  document.getElementById("next").onclick = next;
  $app.querySelectorAll("input").forEach((el) => el.addEventListener("keydown", (e) => e.key === "Enter" && next()));
}

function renderBodyForm() {
  if (!S.onboarded && !bodyComplete()) {
    location.hash = "#/guide/0";
    return;
  }
  const rows = BODY_STEPS.flatMap((s) => s.fields.map(([f, label]) => [f, s.title === label ? label : `${s.title}: ${label.charAt(0).toLowerCase()}${label.slice(1)}`]));
  $app.innerHTML = `
  <div class="narrow">
    <h1>Body measurements</h1>
    <p class="muted">All the way around your body, except the inseam. Change any number; matches update straight away.</p>
    <div class="card">
      ${rows.map(([f, label]) => `<label for="b-${f}">${esc(label)}</label>${measureInput(`b-${f}`, S.body[f])}`).join("")}
      <p class="error" id="err" hidden></p>
    </div>
    <div class="actions">
      <button id="guide">Go through the measuring guide again</button>
      <button class="primary" id="save">Save</button>
    </div>
  </div>`;
  document.getElementById("guide").onclick = () => (location.hash = "#/guide/0");
  document.getElementById("save").onclick = () => {
    const values = BODY_FIELDS.map((f) => [f, readMeasure(`b-${f}`)]);
    const missing = values.filter(([, v]) => v === null);
    if (missing.length) {
      const err = document.getElementById("err");
      err.hidden = false;
      err.textContent = "Every body measurement is needed. Please fill in the empty or unreadable ones.";
      return;
    }
    for (const [f, v] of values) S.body[f] = v;
    save();
    location.hash = "#/results";
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

function renderFit() {
  const f = S.desiredFit;
  $app.innerHTML = `
  <div class="narrow">
    <h1>Desired fit</h1>
    <p class="muted">How you want your jeans to sit. The waist isn't a choice: it should fit your waist.</p>
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
      <h3 style="margin-top:0">Measurements that don't matter to you</h3>
      <p class="hint">These are left out when working out how close a pair comes. Pairs that are too tight for you are still left out.</p>
      <div class="grid2">${MEASUREMENTS.map((m) => `<label class="checkline"><input type="checkbox" value="${m}" ${f.doesnt_matter.includes(m) ? "checked" : ""}/> ${cap(LABELS[m])} doesn't matter</label>`).join("")}</div>
    </div>
    <div class="actions"><span></span><button class="primary" id="done">${S.onboarded ? "Show matches" : "See my matches"}</button></div>
  </div>`;
  const update = () => {
    f.room = $app.querySelector('input[name="room"]:checked').value;
    f.leg_shape = $app.querySelector('input[name="leg_shape"]:checked').value || null;
    f.rise = $app.querySelector('input[name="rise"]:checked').value || null;
    f.doesnt_matter = [...$app.querySelectorAll('input[type="checkbox"]:checked')].map((c) => c.value);
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

function filtersForRust() {
  const f = S.filters;
  return {
    ...f,
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
    <span>${what === "size" ? `Size ${esc(m.size)}` : `${esc(p.fabric)} · size ${esc(m.size)}`}<div class="offs">${offsText(m)}</div></span>
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
  const brandIds = CAT.brands.map((b) => b.id);
  $app.innerHTML = `
  <div class="results">
    <aside class="card filters" id="filters">
      <h3>Compare with</h3>
      <select id="compare">
        <option value="">Your body and typical ease</option>
        ${pairs.map((p) => `<option value="${esc(p.id)}" ${p.id === S.activePairId ? "selected" : ""}>${esc(p.name)}</option>`).join("")}
      </select>
      <h3>Leg shape and rise</h3>
      <p class="small">${cap(usualName(S.desiredFit))} · <a href="#/fit">change</a></p>
      <label class="checkline"><input type="checkbox" id="f-rest" ${f.show_other_shapes_and_rises ? "checked" : ""}/> Show other leg shapes and rises</label>
      <h3>Brands</h3>
      ${CAT.brands.map((b) => `<label class="checkline"><input type="checkbox" data-brand="${b.id}" ${!f.brands || f.brands.includes(b.id) ? "checked" : ""}/> ${esc(b.name)}</label>`).join("")}
      <label class="checkline"><input type="checkbox" id="f-tryon" ${f.try_on_only ? "checked" : ""} ${S.settings.tryOnBrands.length ? "" : "disabled"}/> Try-on brands only</label>
      ${S.settings.tryOnBrands.length ? "" : `<p class="hint">Mark brands you can try on in <a href="#/settings">Settings</a>.</p>`}
      <h3>Price (${esc(S.settings.currency)})</h3>
      <div class="row2"><input type="number" min="0" id="f-pmin" placeholder="from" value="${f.price_min ?? ""}"/><input type="number" min="0" id="f-pmax" placeholder="to" value="${f.price_max ?? ""}"/></div>
      <h3>Fabric weight (oz)</h3>
      <div class="row2"><input type="number" min="0" id="f-wmin" placeholder="from" value="${f.weight_min ?? ""}"/><input type="number" min="0" id="f-wmax" placeholder="to" value="${f.weight_max ?? ""}"/></div>
      <h3>Fabric</h3>
      <label class="checkline"><input type="checkbox" id="f-sanf" ${f.sanforized_only ? "checked" : ""}/> Sanforized only</label>
      <label class="checkline"><input type="checkbox" id="f-nostretch" ${f.no_stretch ? "checked" : ""}/> No stretch</label>
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
    S.activePairId = document.getElementById("compare").value || null;
    const chosen = [...$app.querySelectorAll("[data-brand]:checked")].map((c) => c.dataset.brand);
    f.brands = chosen.length === brandIds.length ? null : chosen;
    f.try_on_only = document.getElementById("f-tryon").checked;
    f.price_min = num("f-pmin");
    f.price_max = num("f-pmax");
    f.weight_min = num("f-wmin");
    f.weight_max = num("f-wmax");
    f.sanforized_only = document.getElementById("f-sanf").checked;
    f.no_stretch = document.getElementById("f-nostretch").checked;
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
            <div class="title">${esc(row.brand_name)} ${esc(row.cut)} <span class="muted" style="font-weight:normal">· size ${esc(m.size)}</span></div>
            <div class="small muted">${esc(p.fabric)}${p.fabric_weight_oz ? ` · ${p.fabric_weight_oz} oz` : ""}</div>
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
  if (typeof t === "string") return "not stated by the brand";
  return `±${t.value} ${t.unit === "in" ? "″" : "cm"}`.replace(" ″", "″");
}

const BASIS_TEXT = {
  "brand-chart": "the brand's own after-wash chart",
  "sold-washed": "sold already washed, so the chart is how it will be",
  "from-shrinkage": "estimated from the brand's shrinkage",
  "shrinkage-unknown": "shrinkage unknown: the brand doesn't say how much it shrinks, so the chart is used as sold",
};

const POINT_TEXT = {
  crotch: "at the crotch",
  "1in-below-crotch": "1″ below the crotch",
  "12in-below-crotch": "12″ below the crotch",
  "13in-below-crotch": "13″ below the crotch",
  "14in-below-crotch": "14″ below the crotch",
  "half-inseam": "halfway down the inseam",
  "not-stated": "not stated by the brand",
};

async function renderProduct(id, size) {
  $app.innerHTML = `<p class="muted">Loading…</p>`;
  let d;
  try {
    d = await invoke("product_details", { productId: id });
  } catch (e) {
    showError(e);
    return;
  }
  const { brand, product: p, washed } = d;
  const sizes = washed ? washed.sizes : [];
  const current = sizes.find((s) => s.size === size) || sizes[0];
  const chart = p.charts.find((c) => c.state === "after-wash") || p.charts[0];
  const asSoldChart = p.charts.find((c) => c.state === "as-sold");
  const estimated = washed && washed.basis === "from-shrinkage";
  const shown = (m, v) => (v === undefined || v === null ? null : shownValue(m, v));
  const rigid = p.elastane_percent === 0;
  const lp = p.list_price;
  const converted = lp ? priceText({ product_id: p.id, price: convertPrice(lp) }) : "";

  $app.innerHTML = `
  <p><button class="link" onclick="history.back()">← Back</button></p>
  <h1>${esc(brand.name)} ${esc(p.cut)}</h1>
  <p class="muted">${esc(p.fabric)}</p>
  ${!p.on_brands_site ? `<div class="note">No longer on the brand's site.</div>` : ""}
  ${washed && washed.basis === "shrinkage-unknown" ? `<div class="note">Shrinkage unknown: the brand doesn't say how much this shrinks in its first wash, so it's compared as sold and ranked after the rest.</div>` : ""}
  ${!brand.says_where_it_measures ? `<div class="note">Approximate: ${esc(brand.name)} doesn't say exactly where it measures, so the nearest of your readings is used.</div>` : ""}
  <div class="card">
    <div class="grid2">
      <p><b>Fabric weight</b><br>${p.fabric_weight_oz ? `${p.fabric_weight_oz} oz` : "not published"}</p>
      <p><b>Composition</b><br>${esc(p.composition)}${p.elastane_percent > 0 ? ` (${p.elastane_percent}% elastane: stretch)` : ""}</p>
      <p><b>Sanforized</b><br>${p.sanforized === true ? "Yes" : p.sanforized === false ? "No (unsanforized)" : "The brand doesn't say"}</p>
      <p><b>After first wash</b><br>${washed ? esc(BASIS_TEXT[washed.basis]) : "no measurements"}</p>
      ${p.shrinkage ? `<p><b>Shrinkage</b><br>${esc(p.shrinkage.stated)} (<a href="${esc(p.shrinkage.source)}" target="_blank" rel="noopener">source</a>, seen ${esc(p.shrinkage.seen)})</p>` : ""}
      <p><b>Tolerance</b><br>${chart ? describeTolerance(chart.tolerance) : "—"}</p>
      <p><b>List price</b><br>${lp ? `${Units.money(lp.amount, lp.currency)}${lp.currency !== S.settings.currency ? ` (${converted})` : ""}<br><span class="small muted">seen ${esc(lp.seen)} on <a href="${esc(lp.source)}" target="_blank" rel="noopener">the brand's site</a></span>` : "not published"}</p>
      <p><b>Brand page</b><br><a href="${esc(p.url)}" target="_blank" rel="noopener">See it on ${esc(brand.name)}'s site</a></p>
    </div>
    ${rigid ? `<p class="hint">Rigid denim: the waist usually stretches about ½–1″ with wear. That isn't added to the numbers.</p>` : ""}
  </div>
  ${current ? `
  <div class="card">
    <div class="actions" style="margin-top:0"><h2 style="margin:0">Size</h2>
      <select id="size" style="max-width:180px">${sizes.map((s) => `<option ${s === current ? "selected" : ""}>${esc(s.size)}</option>`).join("")}</select>
    </div>
    <div class="table-wrap"><table>
      <thead><tr><th>Measurement</th><th>As sold</th><th>After first wash</th><th>Measuring point</th></tr></thead>
      <tbody>${MEASUREMENTS.map((m) => {
        const point = chart.points[m];
        const after = current.after[m];
        const sold = current.as_sold ? current.as_sold[m] : undefined;
        return `<tr><td>${cap(LABELS[m])}${m === "waist" ? " (around)" : LENGTHS.includes(m) ? "" : " (flat)"}</td>
          <td>${sold !== undefined ? len(shown(m, sold)) : after === undefined ? "not published" : "—"}</td>
          <td>${after !== undefined ? len(shown(m, after)) + (estimated ? "*" : "") : "not published"}</td>
          <td class="small muted">${point ? esc(POINT_TEXT[point.at] || point.at) : ""}</td></tr>`;
      }).join("")}</tbody>
    </table></div>
    ${estimated ? `<p class="hint">* Estimated from the brand's shrinkage.</p>` : ""}
    <p class="hint">Widths are straight across the jeans laid flat; the waist is shown all the way around.</p>
  </div>` : `<div class="card"><p>This product has no published measurements, so it isn't matched.</p></div>`}
  <div class="card">
    <h3 style="margin-top:0">Where the numbers come from</h3>
    <ul class="small">${p.charts.map((c) => `<li>${c.state === "after-wash" ? "After-wash" : "As-sold"} chart: <a href="${esc(c.source)}" target="_blank" rel="noopener">${esc(c.source)}</a>, seen ${esc(c.seen)}, in ${c.unit === "in" ? "inches" : "cm"}${c.from_image ? ", <b>read from an image</b>" : ""}.</li>`).join("")}
    ${asSoldChart && chart !== asSoldChart ? "" : ""}
    ${brand.measuring_guide ? `<li>How ${esc(brand.name)} measures: <a href="${esc(brand.measuring_guide)}" target="_blank" rel="noopener">measuring guide</a>.</li>` : ""}</ul>
    <p class="hint">Real Denim Fit never shows shops, stock or sale prices. Check the brand's page for the jeans themselves.</p>
  </div>
  ${sizes.length ? `<div class="card"><h3 style="margin-top:0">All sizes after the first wash</h3><div class="table-wrap"><table>
    <thead><tr><th>Size</th>${MEASUREMENTS.map((m) => `<th>${cap(LABELS[m])}</th>`).join("")}</tr></thead>
    <tbody>${sizes.map((s) => `<tr class="${s === current ? "current" : ""}"><td>${esc(s.size)}</td>${MEASUREMENTS.map((m) => `<td>${s.after[m] !== undefined ? len(shown(m, s.after[m])) : "—"}</td>`).join("")}</tr>`).join("")}</tbody>
  </table></div></div>` : ""}`;
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
    return p ? `${p.brand.name} ${p.cut}, ${p.fabric}, size ${pair.source.size} · from the brand's chart` : "from the brand's chart (product no longer in the catalogue)";
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
    const pair = { id: newId(), name: "My jeans", source: { kind: "measured", readings: {} }, feel: {}, notes: "" };
    S.pairs.push(pair);
    save();
    location.hash = `#/pair/${pair.id}/measure/0`;
  };
  document.getElementById("pick").onclick = () => (location.hash = "#/pair/new/catalogue");
}

const PAIR_STEPS = [
  {
    title: "Lay it flat",
    drawing: "flat",
    fields: [],
    lines: [
      "Button the pair and lay it on a table, front up.",
      "Smooth it out with your hands without stretching it.",
      "Measure flat, straight across. You can skip anything you can't measure.",
    ],
  },
  { title: "Waist", drawing: "waist", fields: ["waist"], lines: ["Straight across the top of the waistband, from edge to edge. The app doubles it for the waist all the way around."] },
  { title: "Rise", drawing: "front_rise", fields: ["front_rise", "back_rise"], lines: ["Front rise: from the crotch seam straight up to the top of the waistband at the front.", "Back rise: turn the pair over and do the same at the back."] },
  { title: "Seat", drawing: "seat", fields: ["seat"], lines: ["Straight across the widest part of the seat, below the waistband."] },
  { title: "Thigh", drawing: "thigh", fields: ["thigh_crotch", "thigh_1in"], lines: ["Straight across the leg from the end of the crotch seam.", "Then again 2.5 cm (1″) further down the inseam."] },
  { title: "Inseam", drawing: "inseam", fields: ["inseam"], lines: ["Along the inside leg seam, from the crotch seam to the hem."] },
  {
    title: "Knee",
    drawing: "knee",
    fields: ["knee_12in", "knee_13in", "knee_14in", "knee_half_inseam"],
    lines: [
      "Lay the tape down the inseam from the crotch and leave it there.",
      "At each mark, measure straight across the leg: 30.5, 33 and 35.5 cm (12″, 13″ and 14″), and halfway down the inseam.",
    ],
  },
  { title: "Leg opening", drawing: "leg_opening", fields: ["leg_opening"], lines: ["Straight across the bottom of the leg."] },
];

function renderPairMeasure(pair, stepText) {
  const i = Math.max(0, Math.min(PAIR_STEPS.length - 1, parseInt(stepText || "0", 10)));
  const step = PAIR_STEPS[i];
  const r = pair.source.readings;
  const half = r.inseam ? `<p class="hint">Halfway down this pair's inseam is ${len(r.inseam / 2)}.</p>` : "";
  $app.innerHTML = `
  <div class="narrow" style="max-width:820px">
    <div class="progress"><div style="width:${((i + 1) / PAIR_STEPS.length) * 100}%"></div></div>
    <p class="muted small">${esc(pair.name)} · step ${i + 1} of ${PAIR_STEPS.length}</p>
    <div class="guide">
      ${Drawings.jeans(step.drawing)}
      <div>
        <h1>${step.title}</h1>
        ${step.lines.map((l) => `<p>${l}</p>`).join("")}
        ${i === 0 ? `<label for="pname">Name this pair</label><input type="text" id="pname" value="${esc(pair.name)}"/>` : ""}
        ${step.title === "Knee" ? half : ""}
        ${step.fields.map((f) => `<label for="p-${f}">${PAIR_FIELD_LABELS[f]}</label>${measureInput(`p-${f}`, r[f])}`).join("")}
        ${step.title === "Waist" ? `<p class="hint" id="doubled"></p>` : ""}
      </div>
    </div>
    <div class="actions">
      <button id="back">${i === 0 ? "Cancel" : "Previous"}</button>
      <span><button id="skip">${step.fields.length ? "Skip" : ""}</button> <button class="primary" id="next">${i === PAIR_STEPS.length - 1 ? "Done" : "Next"}</button></span>
    </div>
  </div>`;
  const skip = document.getElementById("skip");
  if (!step.fields.length) skip.hidden = true;
  const doubled = document.getElementById("doubled");
  if (doubled) {
    const show = () => {
      const v = readMeasure("p-waist");
      doubled.textContent = v ? `That's a waist of ${len(v * 2)} all the way around.` : "";
    };
    document.getElementById("p-waist").addEventListener("input", show);
    show();
  }
  const store = () => {
    const name = document.getElementById("pname");
    if (name) pair.name = name.value.trim() || "My jeans";
    for (const f of step.fields) {
      const v = readMeasure(`p-${f}`);
      if (v === null) delete r[f];
      else r[f] = v;
    }
    save();
  };
  const go = (to) => (location.hash = to < PAIR_STEPS.length ? `#/pair/${encodeURIComponent(pair.id)}/measure/${to}` : `#/pair/${encodeURIComponent(pair.id)}`);
  document.getElementById("back").onclick = () => {
    if (i === 0) {
      if (!Object.keys(r).length) {
        S.pairs = S.pairs.filter((p) => p.id !== pair.id);
        save();
      }
      location.hash = "#/pairs";
    } else go(i - 1);
  };
  skip.onclick = () => go(i + 1);
  document.getElementById("next").onclick = () => {
    store();
    go(i + 1);
  };
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
    const cuts = b ? [...new Set(b.products.filter((p) => p.sizes.length).map((p) => p.cut))].sort() : [];
    fill(pc, cuts.map((c) => [c, c]));
    fill(pp, []);
    fill(ps, []);
    add.disabled = true;
  };
  pc.onchange = () => {
    const b = BRANDS.get(pb.value);
    fill(pp, b.products.filter((p) => p.cut === pc.value && p.sizes.length).map((p) => [p.id, `${p.fabric}${p.on_brands_site ? "" : " (no longer on the site)"}`]));
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
  document.getElementById("delete").onclick = () => {
    if (!confirm(`Delete "${pair.name}"? This can't be undone.`)) return;
    S.pairs = S.pairs.filter((p) => p.id !== pair.id);
    if (S.activePairId === pair.id) S.activePairId = null;
    save();
    location.hash = "#/pairs";
  };
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
  const place = (p) =>
    p.kind === "room" ? `which is ${p.room}` : p.kind === "tighter-than-snug" ? "which is tighter than snug" : "which is roomier than roomy";
  card.innerHTML = `<h2 style="margin-top:0">This pair's own ease</h2>
    <p class="hint">How much bigger than your body it is, all the way around.</p>
    <ul class="ease-list">${e.points.map((p) => `<li><b>${len(p.ease_around, true)}</b> ${esc(p.at)}${["waist", "seat"].includes(p.measurement) ? "" : ` (${LABELS[p.measurement]})`}, ${place(p.place)}</li>`).join("")}</ul>
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
  document.getElementById("reset").onclick = () => {
    if (!confirm("Delete your body measurements, reference pairs and settings from this computer? This can't be undone.")) return;
    S = freshState(CAT.default_currency);
    save();
    location.hash = "#/welcome";
  };
}

boot();
