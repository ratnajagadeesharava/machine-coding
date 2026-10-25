// Element -> SVG markup. DOM-free: every function returns strings. All user text is escaped.
import {
  DATA_URL_RE, FONT_FAMILIES, elementBounds, findItem, hasPoints, isLinear, resolvePoints,
} from '../../shared/board.js';
import {
  frameNameBox, labelLayout, linearLabelLayout, textSize,
} from './geometry.js';
import {
  arrowheadPath, fillPattern, freedrawPath, linearPath, num, shapePath,
} from './rough.js';

// ---------- escaping ----------
// eslint-disable-next-line no-control-regex
const BAD_XML = /[\u0000-\u0008\u000b\u000c\u000e-\u001f￾￿]/g;
export function esc(s) {
  return String(s ?? '').replace(BAD_XML, '').replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}

// ---------- colours ----------
const NAMED = {
  black: '#000000', white: '#ffffff', red: '#ff0000', green: '#008000', blue: '#0000ff', yellow: '#ffff00', orange: '#ffa500',
  purple: '#800080', gray: '#808080', grey: '#808080', pink: '#ffc0cb', brown: '#a52a2a', cyan: '#00ffff', magenta: '#ff00ff',
};

function parseColor(c) {
  if (typeof c !== 'string') return null;
  const s = c.trim().toLowerCase();
  let m = /^#([0-9a-f]{3,8})$/.exec(s);
  if (m) {
    let h = m[1];
    if (h.length === 3 || h.length === 4) h = [...h].map((x) => x + x).join('');
    if (h.length !== 6 && h.length !== 8) return null;
    return { r: parseInt(h.slice(0, 2), 16), g: parseInt(h.slice(2, 4), 16), b: parseInt(h.slice(4, 6), 16), a: h.length === 8 ? parseInt(h.slice(6, 8), 16) / 255 : 1, hex: true };
  }
  m = /^rgba?\(\s*([\d.]+)%?\s*,\s*([\d.]+)%?\s*,\s*([\d.]+)%?\s*(?:,\s*([\d.]+)\s*)?\)$/.exec(s);
  if (m) return { r: +m[1], g: +m[2], b: +m[3], a: m[4] === undefined ? 1 : +m[4], hex: false };
  if (NAMED[s]) return parseColor(NAMED[s]);
  return null;
}

// CSS `filter: invert(93%) hue-rotate(180deg)` applied to one colour (what Excalidraw does for its dark theme).
export function themeColor(color, dark) {
  if (!dark || color == null || color === 'transparent' || color === 'none') return color;
  const c = parseColor(color);
  if (!c) return color;
  const inv = (v) => v * 0.07 + (255 - v) * 0.93;
  const [r, g, b] = [inv(c.r), inv(c.g), inv(c.b)];
  const cos = Math.cos(Math.PI); const sin = Math.sin(Math.PI);
  const m = [
    0.213 + cos * 0.787 - sin * 0.213, 0.715 - cos * 0.715 - sin * 0.715, 0.072 - cos * 0.072 + sin * 0.928,
    0.213 - cos * 0.213 + sin * 0.143, 0.715 + cos * 0.285 + sin * 0.14, 0.072 - cos * 0.072 - sin * 0.283,
    0.213 - cos * 0.213 - sin * 0.787, 0.715 - cos * 0.715 + sin * 0.715, 0.072 + cos * 0.928 + sin * 0.072,
  ];
  const q = (v) => Math.max(0, Math.min(255, Math.round(v)));
  const R = q(m[0] * r + m[1] * g + m[2] * b); const G = q(m[3] * r + m[4] * g + m[5] * b); const B = q(m[6] * r + m[7] * g + m[8] * b);
  if (!c.hex && c.a < 1) return `rgba(${R},${G},${B},${c.a})`;
  const hex = (v) => v.toString(16).padStart(2, '0');
  return `#${hex(R)}${hex(G)}${hex(B)}${c.hex && c.a < 1 ? hex(Math.round(c.a * 255)) : ''}`;
}

export const fontCss = (fontFamily) => (FONT_FAMILIES[fontFamily] || FONT_FAMILIES[1]).css;

// ---------- helpers ----------
const SAFE_COLOR = /^(transparent|none|#[0-9a-fA-F]{3,8}|rgba?\([0-9 ,.%]+\)|[a-zA-Z]{3,20})$/;
const colour = (c, dark, fallback = '#1e1e1e') => esc(themeColor(typeof c === 'string' && SAFE_COLOR.test(c) ? c : fallback, dark));
const FRAME_FONT = 14;

function dashAttrs(style, w) {
  if (style === 'dashed') return ` stroke-dasharray="8 ${num(8 + w)}"`;
  if (style === 'dotted') return ` stroke-dasharray="1.5 ${num(6 + w)}"`;
  return '';
}

function strokeAttrs(el, dark) {
  return `fill="none" stroke="${colour(el.strokeColor, dark)}" stroke-width="${num(el.strokeWidth || 2)}" stroke-linecap="round" stroke-linejoin="round"${dashAttrs(el.strokeStyle, el.strokeWidth || 2)}`;
}

function textMarkup(lay, el, dark, extra = '') {
  if (!lay || !lay.lines.some((l) => l.length)) return '';
  const fill = colour(el.strokeColor, dark);
  const tspans = lay.lines.map((line, i) => (line === '' ? '' : `<tspan x="${num(lay.anchorX)}" y="${num(lay.top + i * lay.lineHeight + lay.lineHeight / 2 + lay.fontSize * 0.34)}">${esc(line)}</tspan>`)).join('');
  return `<text font-family="${esc(fontCss(lay.fontFamily))}" font-size="${num(lay.fontSize)}" fill="${fill}" text-anchor="${lay.anchor}" xml:space="preserve"${extra}>${tspans}</text>`;
}

function linkBadge(b, el, dark) {
  const cx = b.x + b.w - 12; const cy = b.y + 12;
  const stroke = colour('#1971c2', dark);
  return `<g class="wb-link"><title>${esc(el.link)}</title><circle cx="${num(cx)}" cy="${num(cy)}" r="9" fill="${colour('#ffffff', dark)}" stroke="${stroke}" stroke-width="1.5"/>`
    + `<path d="M${num(cx - 4)} ${num(cy + 1)}l-1.5 1.5a2.4 2.4 0 0 0 3.4 3.4l1.5 -1.5M${num(cx + 4)} ${num(cy - 1)}l1.5 -1.5a2.4 2.4 0 0 0 -3.4 -3.4l-1.5 1.5M${num(cx - 2)} ${num(cy + 2)}l4 -4" fill="none" stroke="${stroke}" stroke-width="1.4" stroke-linecap="round"/></g>`;
}

function fillMarkup(el, sp, dark) {
  if (!sp.fill) return '';
  const bg = colour(el.backgroundColor, dark, 'transparent');
  const kind = sp.fillKind;
  if (kind === 'solid') return `<path d="${sp.fill}" fill="${bg}" stroke="none"/>`;
  const pat = fillPattern(el);
  const id = `wb-clip-${el.id}`;
  const sw = Math.max(1, (el.strokeWidth || 2) * 0.75);
  return `<clipPath id="${id}"><path d="${sp.fill}"/></clipPath><g clip-path="url(#${id})"><path d="${pat.d}" fill="none" stroke="${bg}" stroke-width="${num(sw)}" stroke-linecap="round" stroke-linejoin="round"/></g>`;
}

function arrowheads(board, el, pts, dark) {
  const stroke = colour(el.strokeColor, dark);
  const sw = el.strokeWidth || 2;
  let body = pts.map((p) => ({ ...p }));
  let heads = '';
  const add = (kind, tipIdx, otherIdx) => {
    if (!kind) return;
    const tip = body[tipIdx]; const other = body[otherIdx];
    const dir = Math.atan2(tip.y - other.y, tip.x - other.x);
    const h = arrowheadPath(kind, tip, dir, sw);
    if (!h.d) return;
    heads += `<path d="${h.d}" fill="${h.fill ? stroke : 'none'}" stroke="${stroke}" stroke-width="${num(sw)}" stroke-linecap="round" stroke-linejoin="round"/>`;
    if (h.inset > 0) { // stop the body at the base of closed heads
      const len = Math.hypot(tip.x - other.x, tip.y - other.y);
      if (len > h.inset * 1.2) body[tipIdx] = { x: tip.x - Math.cos(dir) * h.inset, y: tip.y - Math.sin(dir) * h.inset };
    }
  };
  if (el.type === 'arrow' && body.length >= 2) {
    const last = body.length - 1;
    // direction of each head uses the untrimmed neighbour
    add(el.endArrowhead, last, last - 1);
    add(el.startArrowhead, 0, 1);
  }
  return { body, heads };
}

// ---------- one element ----------
export function elementToSvg(board, el, { dark = false, skipLabel = false, background } = {}) {
  const parts = [];
  const bgPaper = colour(background || '#ffffff', dark);
  let b;
  try { b = elementBounds(board, el); } catch { b = { x: el.x, y: el.y, w: el.w, h: el.h }; }
  const stroke = strokeAttrs(el, dark);

  switch (el.type) {
    case 'rect': case 'diamond': case 'ellipse': case 'database': {
      const sp = shapePath(el);
      parts.push(fillMarkup(el, sp, dark));
      parts.push(`<path d="${sp.stroke}" ${stroke}/>`);
      if (!skipLabel) parts.push(textMarkup(labelLayout(el), el, dark));
      break;
    }
    case 'sticky': {
      const sp = shapePath({ ...el, roundness: 'sharp', roughness: 0 });
      const paper = colour(el.backgroundColor, dark, '#ffec99');
      parts.push(`<rect x="${num(el.x + 4)}" y="${num(el.y + 5)}" width="${num(el.w)}" height="${num(el.h)}" fill="${dark ? 'rgba(0,0,0,0.45)' : 'rgba(0,0,0,0.16)'}"/>`);
      if (el.fillStyle && el.fillStyle !== 'solid') parts.push(`<path d="${sp.silhouette}" fill="${paper}" opacity="0.5"/>`, fillMarkup({ ...el, roughness: 0 }, { ...sp, fill: sp.silhouette, fillKind: el.fillStyle }, dark));
      else parts.push(`<path d="${sp.silhouette}" fill="${paper}"/>`);
      parts.push(`<path d="${sp.silhouette}" fill="none" stroke="${dark ? 'rgba(255,255,255,0.18)' : 'rgba(0,0,0,0.14)'}" stroke-width="1"/>`);
      if (!skipLabel) parts.push(textMarkup(labelLayout(el), el, dark));
      break;
    }
    case 'frame': {
      const sp = shapePath({ ...el, backgroundColor: 'transparent' });
      parts.push(`<path d="${sp.stroke}" ${stroke}/>`);
      if (!skipLabel && el.name) {
        const bx = frameNameBox(el);
        parts.push(`<text x="${num(bx.x + 2)}" y="${num(el.y - 8)}" font-family="${esc(fontCss(2))}" font-size="${FRAME_FONT}" fill="${colour(el.strokeColor, dark)}" fill-opacity="0.8" xml:space="preserve">${esc(el.name)}</text>`);
      }
      break;
    }
    case 'text': {
      if (el.backgroundColor && el.backgroundColor !== 'transparent') parts.push(`<rect x="${num(el.x)}" y="${num(el.y)}" width="${num(el.w)}" height="${num(el.h)}" fill="${colour(el.backgroundColor, dark, 'transparent')}"/>`);
      if (!skipLabel) {
        const align = el.textAlign || 'left';
        const size = textSize(el.text, { fontFamily: el.fontFamily, fontSize: el.fontSize });
        const anchorX = align === 'left' ? el.x : align === 'right' ? el.x + el.w : el.x + el.w / 2;
        const lay = { lines: size.lines, lineHeight: (el.fontSize || 20) * 1.25, fontSize: el.fontSize || 20, fontFamily: el.fontFamily, anchor: align === 'left' ? 'start' : align === 'right' ? 'end' : 'middle', anchorX, top: el.y };
        parts.push(textMarkup(lay, el, dark));
      }
      break;
    }
    case 'image': {
      if (el.dataUrl && DATA_URL_RE.test(el.dataUrl)) parts.push(`<image href="${esc(el.dataUrl)}" x="${num(el.x)}" y="${num(el.y)}" width="${num(el.w)}" height="${num(el.h)}" preserveAspectRatio="none"/>`);
      else parts.push(`<rect x="${num(el.x)}" y="${num(el.y)}" width="${num(el.w)}" height="${num(el.h)}" fill="none" stroke="${colour(el.strokeColor, dark)}" stroke-width="1" stroke-dasharray="6 4"/>`);
      break;
    }
    case 'freedraw': {
      const abs = (el.points || []).map(([px, py]) => [el.x + px, el.y + py]);
      const c = colour(el.strokeColor, dark);
      parts.push(`<path d="${freedrawPath(abs, el.strokeWidth || 2)}" fill="${c}" stroke="${c}" stroke-width="0.5" stroke-linejoin="round"/>`);
      break;
    }
    case 'line': case 'arrow': {
      const pts = resolvePoints(board, el);
      if (pts.length >= 2) {
        const { body, heads } = arrowheads(board, el, pts, dark);
        parts.push(`<path d="${linearPath(body, { arrowType: el.arrowType, roughness: el.roughness, seed: el.seed })}" ${stroke}/>`);
        parts.push(heads);
        if (!skipLabel && el.text) {
          const lay = linearLabelLayout(board, el);
          if (lay) {
            const pad = 4;
            parts.push(`<rect x="${num(lay.box.x - pad)}" y="${num(lay.box.y - 1)}" width="${num(lay.box.w + pad * 2)}" height="${num(lay.box.h + 2)}" rx="4" fill="${bgPaper}"/>`);
            parts.push(textMarkup(lay, el, dark));
          }
        }
      }
      break;
    }
    default: break;
  }
  if (el.link) parts.push(linkBadge(b, el, dark));

  const attrs = [`data-id="${esc(el.id)}"`, 'class="wb-el"'];
  const op = el.opacity;
  if (Number.isFinite(op) && op < 100) attrs.push(`opacity="${num(Math.max(0, op) / 100)}"`);
  if (el.angle && !hasPoints(el)) {
    const cx = el.x + el.w / 2; const cy = el.y + el.h / 2;
    attrs.push(`transform="rotate(${num((el.angle * 180) / Math.PI)} ${num(cx)} ${num(cy)})"`);
  }
  return `<g ${attrs.join(' ')}>${parts.join('')}</g>`;
}

export function frameClipPath(board, frame) {
  return `<clipPath id="wb-fclip-${esc(frame.id)}"><rect x="${num(frame.x)}" y="${num(frame.y)}" width="${num(frame.w)}" height="${num(frame.h)}"/></clipPath>`;
}

// ---------- bounds ----------
function inflate(b, m) { return { x: b.x - m, y: b.y - m, w: b.w + m * 2, h: b.h + m * 2 }; }
const union = (a, b) => {
  if (!a) return b;
  const x = Math.min(a.x, b.x); const y = Math.min(a.y, b.y);
  return { x, y, w: Math.max(a.x + a.w, b.x + b.w) - x, h: Math.max(a.y + a.h, b.y + b.h) - y };
};
const intersect = (a, b) => {
  const x = Math.max(a.x, b.x); const y = Math.max(a.y, b.y);
  const x2 = Math.min(a.x + a.w, b.x + b.w); const y2 = Math.min(a.y + a.h, b.y + b.h);
  return x2 >= x && y2 >= y ? { x, y, w: x2 - x, h: y2 - y } : null;
};

function extent(board, el) {
  const sw = el.strokeWidth || 2;
  let b = elementBounds(board, el);
  const wobble = (el.roughness || 0) * 2;
  if (el.type === 'image' || el.type === 'text') b = inflate(b, 1);
  else if (el.type === 'freedraw') b = inflate(b, sw * 1.2 + 1);
  else b = inflate(b, sw / 2 + wobble + 1);
  if (isLinear(el)) {
    const head = 10 + 3 * sw;
    if (el.type === 'arrow' && (el.startArrowhead || el.endArrowhead)) b = inflate(b, head * 0.6 + 4);
    if (el.text) {
      const lay = linearLabelLayout(board, el);
      if (lay) b = union(b, inflate(lay.box, 5));
    }
  }
  if (el.type === 'sticky') b = { ...b, w: b.w + 5, h: b.h + 6 };
  if (el.type === 'frame' && el.name) b = union(b, frameNameBox(el));
  if ((el.type === 'rect' || el.type === 'diamond' || el.type === 'ellipse' || el.type === 'database') && el.angle) b = inflate(b, 0);
  return b;
}

export function exportBounds(board, ids) {
  const set = ids ? new Set(ids) : null;
  const items = board.items.filter((e) => !set || set.has(e.id));
  const inSet = new Set(items.map((e) => e.id));
  let acc = null;
  for (const el of items) {
    let b = extent(board, el);
    const frame = el.frameId != null && inSet.has(el.frameId) ? findItem(board, el.frameId) : null;
    if (frame) { b = intersect(b, extent(board, frame)); if (!b) continue; }
    acc = union(acc, b);
  }
  return acc;
}

// ---------- whole board ----------
export function boardToSvgString(board, { ids, background = null, dark = false, padding = 10, scale = 1 } = {}) {
  const set = ids ? new Set(ids) : null;
  const items = board.items.filter((e) => !set || set.has(e.id));
  const inSet = new Set(items.map((e) => e.id));
  const bounds = exportBounds(board, ids);
  const pad = Number.isFinite(padding) ? padding : 10;
  const sc = Number.isFinite(scale) && scale > 0 ? scale : 1;
  const view = bounds ? { x: bounds.x - pad, y: bounds.y - pad, w: bounds.w + pad * 2, h: bounds.h + pad * 2 } : { x: 0, y: 0, w: 100, h: 100 };
  const bg = background ? colour(background, dark, '#ffffff') : null;
  const defs = [];
  const body = [];
  const frameUsed = new Set();
  for (const el of items) {
    const frame = el.frameId != null && inSet.has(el.frameId) ? findItem(board, el.frameId) : null;
    const markup = elementToSvg(board, el, { dark, background: background || undefined });
    if (frame && frame.type === 'frame') {
      if (!frameUsed.has(frame.id)) { frameUsed.add(frame.id); defs.push(frameClipPath(board, frame)); }
      body.push(`<g clip-path="url(#wb-fclip-${esc(frame.id)})">${markup}</g>`);
    } else body.push(markup);
  }
  const w = num(view.w * sc); const h = num(view.h * sc);
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="${num(view.x)} ${num(view.y)} ${num(view.w)} ${num(view.h)}" width="${w}" height="${h}">`
    + (defs.length ? `<defs>${defs.join('')}</defs>` : '')
    + (bg ? `<rect x="${num(view.x)}" y="${num(view.y)}" width="${num(view.w)}" height="${num(view.h)}" fill="${bg}"/>` : '')
    + body.join('') + '</svg>';
}
