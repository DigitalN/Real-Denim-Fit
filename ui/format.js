// Units: everything is stored in cm; inches are shown in ⅛″ steps.
"use strict";

const Units = (() => {
  const CM = 2.54;
  const FRACTIONS = ["", "⅛", "¼", "⅜", "½", "⅝", "¾", "⅞"];

  function eighths(inches) {
    return Math.round(inches * 8);
  }

  function inchText(inches, signed) {
    const sign = inches < 0 ? "−" : signed ? "+" : "";
    const total = Math.abs(eighths(inches));
    const whole = Math.floor(total / 8);
    const frac = FRACTIONS[total % 8];
    if (total === 0) return (signed ? "±" : "") + "0″";
    return `${sign}${whole || !frac ? whole : ""}${frac}″`;
  }

  function cmText(cm, signed) {
    const v = Math.round(cm * 10) / 10;
    const sign = v < 0 ? "−" : signed && v > 0 ? "+" : "";
    return `${sign}${Math.abs(v).toFixed(1).replace(/\.0$/, "")} cm`;
  }

  /** A length in the user's units, such as 32½″ or 82.5 cm. */
  function length(cm, units, signed = false) {
    if (cm === null || cm === undefined || Number.isNaN(cm)) return "—";
    return units === "in" ? inchText(cm / CM, signed) : cmText(cm, signed);
  }

  /** The value to put in an input box. */
  function inputValue(cm, units) {
    if (cm === null || cm === undefined) return "";
    if (units === "in") {
      const total = eighths(cm / CM);
      const whole = Math.floor(total / 8);
      const rest = total % 8;
      if (!rest) return String(whole);
      const [n, d] = [[1, 8], [1, 4], [3, 8], [1, 2], [5, 8], [3, 4], [7, 8]][rest - 1];
      return `${whole} ${n}/${d}`;
    }
    return String(Math.round(cm * 10) / 10);
  }

  /** Reads "32", "32.5", "32 1/2", "32½" or "82,5" into cm, or null. */
  function parse(text, units) {
    if (text === null || text === undefined) return null;
    let s = String(text).trim().replace(",", ".").replace(/[″"]|cm|in/g, "").trim();
    if (!s) return null;
    let value = null;
    const glyph = { "⅛": 0.125, "¼": 0.25, "⅜": 0.375, "½": 0.5, "⅝": 0.625, "¾": 0.75, "⅞": 0.875 };
    const g = s.match(/^(\d+(?:\.\d+)?)?\s*([⅛¼⅜½⅝¾⅞])$/);
    const f = s.match(/^(\d+)\s+(\d+)\/(\d+)$/);
    const f2 = s.match(/^(\d+)\/(\d+)$/);
    if (g) value = (g[1] ? parseFloat(g[1]) : 0) + glyph[g[2]];
    else if (f) value = parseInt(f[1], 10) + parseInt(f[2], 10) / parseInt(f[3], 10);
    else if (f2) value = parseInt(f2[1], 10) / parseInt(f2[2], 10);
    else if (/^\d+(\.\d+)?$/.test(s)) value = parseFloat(s);
    if (value === null || !Number.isFinite(value) || value <= 0) return null;
    return units === "in" ? Math.round(value * 8) / 8 * CM : Math.round(value * 10) / 10;
  }

  function money(amount, currency) {
    if (amount === null || amount === undefined) return "";
    try {
      return new Intl.NumberFormat(undefined, { style: "currency", currency, maximumFractionDigits: 0 }).format(amount);
    } catch {
      return `${Math.round(amount)} ${currency}`;
    }
  }

  return { length, inputValue, parse, money, CM };
})();

const LABELS = {
  waist: "waist",
  front_rise: "front rise",
  back_rise: "back rise",
  seat: "seat",
  thigh: "thigh",
  knee: "knee",
  leg_opening: "leg opening",
  inseam: "inseam",
};
const MEASUREMENTS = Object.keys(LABELS);
const LINE_TEXT = { men: "Men's", women: "Women's", unisex: "Unisex" };
const LENGTHS = ["front_rise", "back_rise", "inseam"];
const cap = (s) => s.charAt(0).toUpperCase() + s.slice(1);
const esc = (s) =>
  String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
