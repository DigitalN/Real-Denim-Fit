// Line drawings for the measuring guides. Each returns an SVG string with
// the tape's position drawn in copper.
"use strict";

const Drawings = (() => {
  const svg = (inner) =>
    `<svg viewBox="0 0 200 400" role="img" aria-hidden="true" xmlns="http://www.w3.org/2000/svg">${inner}</svg>`;
  const across = (y, x1, x2, label) =>
    `<line class="drawing-line" x1="${x1}" y1="${y}" x2="${x2}" y2="${y}" stroke-width="4" stroke-linecap="round"/>` +
    (label ? `<text class="drawing-text" x="${x2 + 5}" y="${y + 4}">${label}</text>` : "");
  const mark = (x, y) => `<rect class="drawing-mark" x="${x - 6}" y="${y - 2}" width="12" height="4" rx="1"/>`;

  // Lower body, front view: waist at 40, crotch at 150, ankles at 385.
  const bodyOutline = `
    <path class="drawing-body" stroke-width="1.5" d="
      M58 40 C54 70 44 95 44 120 C44 140 48 150 50 170 C54 230 62 300 66 385
      L94 385 C96 300 98 230 100 160 C102 230 104 300 106 385 L134 385
      C138 300 146 230 150 170 C152 150 156 140 156 120 C156 95 146 70 142 40 Z"/>`;

  function body(which) {
    const parts = { waist: "", seat: "", thigh: "", leg: "", inseam: "" };
    parts.waist = across(48, 52, 148);
    parts.seat = across(110, 42, 158);
    parts.thigh = across(168, 50, 98, "top") + across(184, 52, 98, "−2.5 cm");
    parts.leg =
      [222, 232, 242].map((y) => across(y, 56, 97)).join("") +
      across(262, 58, 97) +
      [222, 232, 242, 262].map((y) => mark(78, y)).join("") +
      `<text class="drawing-text" x="104" y="236">12″ 13″ 14″</text>` +
      `<text class="drawing-text" x="104" y="266">½ inseam</text>`;
    parts.inseam =
      `<line class="drawing-line" x1="98" y1="160" x2="95" y2="380" stroke-width="4" stroke-linecap="round"/>`;
    return svg(bodyOutline + (parts[which] || ""));
  }

  // Jeans laid flat: waistband 22–40, crotch at 140, hems at 385.
  const jeansOutline = `
    <path class="drawing-body" stroke-width="1.5" d="
      M44 22 L156 22 L160 120 L164 385 L110 385 L100 150 L90 385 L36 385 L40 120 Z"/>
    <line x1="44" y1="40" x2="156" y2="40" stroke="currentColor" stroke-opacity="0.3"/>
    <path d="M100 40 L100 140" stroke="currentColor" stroke-opacity="0.3" fill="none"/>`;

  function jeans(which) {
    const p = {
      waist: across(30, 44, 156),
      front_rise: `<line class="drawing-line" x1="100" y1="22" x2="100" y2="142" stroke-width="4" stroke-linecap="round"/>`,
      back_rise: `<line class="drawing-line" x1="100" y1="22" x2="100" y2="142" stroke-width="4" stroke-dasharray="6 5"/>` +
        `<text class="drawing-text" x="106" y="84">back</text>`,
      seat: across(92, 41, 159),
      thigh: across(146, 38, 99, "crotch") + across(164, 38, 98, "1″ below"),
      knee: [222, 232, 242].map((y) => across(y, 38, 96)).join("") + across(262, 38, 96) +
        `<text class="drawing-text" x="104" y="236">12″ 13″ 14″</text><text class="drawing-text" x="104" y="266">½ inseam</text>`,
      leg_opening: across(381, 36, 90),
      inseam: `<line class="drawing-line" x1="99" y1="148" x2="90" y2="383" stroke-width="4" stroke-linecap="round"/>`,
      flat: "",
    };
    return svg(jeansOutline + (p[which] || ""));
  }

  return { body, jeans };
})();
