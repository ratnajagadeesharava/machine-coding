// Inline SVG icon set (24x24 grid, 1.8px strokes, round caps, currentColor) plus two tiny DOM helpers shared by the
// chrome modules: `h()` (element factory) and `popover()` (anchored popover with Esc / outside-click close).
// All markup below is static; nothing dynamic is ever interpolated into HTML.

const NS = 'http://www.w3.org/2000/svg';

// Shared pieces
const HEAD_LINE = (x1, x2) => `<path d="M${x1} 12H${x2}"/>`;
const FONT = (css, ch) =>
  `<text x="12" y="17.5" text-anchor="middle" font-size="16" font-weight="500" fill="currentColor" stroke="none" style="font-family:${css.replace(/"/g, "'")}">${ch}</text>`;
const SANS = 'ui-sans-serif,system-ui,-apple-system,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif';
const HAND = '"Segoe Print","Bradley Hand","Chalkboard SE","Comic Sans MS","Marker Felt",cursive';
const MONO = 'ui-monospace,SFMono-Regular,Menlo,Consolas,monospace';
const BG = 'style="fill:var(--wbui-icon-bg,#fff)"';

export const ICONS = {
  // ---- tools
  lock: '<rect x="5" y="11" width="14" height="9.5" rx="2.2"/><path d="M8 11V8a4 4 0 018 0v3"/><path d="M12 15v2" />',
  unlock: '<rect x="5" y="11" width="14" height="9.5" rx="2.2"/><path d="M8 11V8a4 4 0 017.6-1.7"/><path d="M12 15v2"/>',
  hand: '<path d="M8 12.5V6.2a1.4 1.4 0 012.8 0V11"/><path d="M10.8 10.5V4.7a1.4 1.4 0 012.8 0V11"/><path d="M13.6 10.8V5.8a1.4 1.4 0 012.8 0v6"/><path d="M16.4 11.2V8.8a1.4 1.4 0 012.8 0V15a6.5 6.5 0 01-6.5 6.5h-.6a6.5 6.5 0 01-4.9-2.2l-3.4-4a1.5 1.5 0 012.2-2L8 15.2"/>',
  select: '<path d="M5.5 3.5l13 6.6-5.7 1.8-1.9 5.8z" stroke-linejoin="round"/><path d="M13.2 13.6l5.3 5.4"/>',
  rect: '<rect x="3.5" y="5.5" width="17" height="13" rx="2.2"/>',
  diamond: '<path d="M12 3.2l8.8 8.8-8.8 8.8L3.2 12z" stroke-linejoin="round"/>',
  ellipse: '<ellipse cx="12" cy="12" rx="9" ry="7"/>',
  arrow: '<path d="M5 19L19 5"/><path d="M9 5h10v10"/>',
  line: '<path d="M5 19L19 5"/><circle cx="5" cy="19" r="0.6"/><circle cx="19" cy="5" r="0.6"/>',
  freedraw: '<path d="M4 20l1-4.2L16.3 4.5a2.1 2.1 0 013 3L8.2 19z" stroke-linejoin="round"/><path d="M14.2 6.6l3.2 3.2"/>',
  text: '<path d="M5.5 7V5.2h13V7"/><path d="M12 5.2v13.6"/><path d="M9 18.8h6"/>',
  image: '<rect x="3.5" y="4.5" width="17" height="15" rx="2.2"/><circle cx="9" cy="10" r="1.7"/><path d="M4 17.5l4.8-4.6 3.6 3.4 2.6-2.4 5 4.2"/>',
  eraser: '<path d="M4.8 14.6l8.9-8.9a2 2 0 012.8 0l2.8 2.8a2 2 0 010 2.8l-7 7H8.4z" stroke-linejoin="round"/><path d="M9.6 10.8l5.6 5.6"/><path d="M12.5 19.8H20"/>',
  database: '<ellipse cx="12" cy="6" rx="7" ry="2.8"/><path d="M5 6v12c0 1.55 3.13 2.8 7 2.8s7-1.25 7-2.8V6"/><path d="M5 12c0 1.55 3.13 2.8 7 2.8s7-1.25 7-2.8"/>',
  sticky: '<path d="M4.5 4.5h15v10.2l-5.3 4.8H4.5z" stroke-linejoin="round"/><path d="M19.5 14.7h-5.3v4.8"/>',
  frame: '<path d="M7 3.5v17M17 3.5v17M3.5 7h17M3.5 17h17"/>',
  laser: '<path d="M4 20l8.2-8.2"/><path d="M13.5 10.5l2.3 2.3"/><path d="M15.5 3.5v3M20.5 8.5h-3M19 5l-2 2M11.2 5.2l1.6 1.6"/>',
  bucket: '<path d="M6.2 11.4l6-6 7 7-6 6a2 2 0 01-2.8 0l-4.2-4.2a2 2 0 010-2.8z" stroke-linejoin="round"/><path d="M12.2 5.4L9.5 2.7"/><path d="M5.6 11.4h12.8" opacity=".0"/><path d="M20.2 16.2s1.8 1.9 1.8 3.2a1.8 1.8 0 01-3.6 0c0-1.3 1.8-3.2 1.8-3.2z" fill="currentColor"/>',
  eyedropper: '<path d="M14.5 6.5l3 3"/><path d="M16.2 4.3a2.1 2.1 0 013 3l-2.2 2.2-3-3z" stroke-linejoin="round"/><path d="M14.5 8.5L6.2 16.8a2 2 0 00-.6 1.4V20h1.8a2 2 0 001.4-.6l8.3-8.3"/>',
  library: '<rect x="3.5" y="4" width="4.5" height="16" rx="1"/><rect x="9.5" y="4" width="4.5" height="16" rx="1"/><path d="M16.2 6l3.9-1 3.2 12.5-3.9 1z" transform="translate(-1.6 0)"/>',
  moreTools: '<circle cx="5.5" cy="12" r="1.5" fill="currentColor"/><circle cx="12" cy="12" r="1.5" fill="currentColor"/><circle cx="18.5" cy="12" r="1.5" fill="currentColor"/>',
  // ---- history / view
  undo: '<path d="M8.5 5L4 9.5 8.5 14"/><path d="M4 9.5h10a5.5 5.5 0 010 11h-3.5"/>',
  redo: '<path d="M15.5 5L20 9.5 15.5 14"/><path d="M20 9.5H10a5.5 5.5 0 000 11h3.5"/>',
  zoomIn: '<circle cx="10.8" cy="10.8" r="6.3"/><path d="M20 20l-4.7-4.7"/><path d="M10.8 8v5.6M8 10.8h5.6"/>',
  zoomOut: '<circle cx="10.8" cy="10.8" r="6.3"/><path d="M20 20l-4.7-4.7"/><path d="M8 10.8h5.6"/>',
  zoomPlus: '<path d="M12 5v14M5 12h14"/>',
  zoomMinus: '<path d="M5 12h14"/>',
  fit: '<path d="M4 9V5.8A1.8 1.8 0 015.8 4H9M15 4h3.2A1.8 1.8 0 0120 5.8V9M20 15v3.2a1.8 1.8 0 01-1.8 1.8H15M9 20H5.8A1.8 1.8 0 014 18.2V15"/><rect x="8.5" y="8.5" width="7" height="7" rx="1"/>',
  stats: '<path d="M5 20v-7M12 20V5M19 20v-10"/>',
  // ---- layers
  toFront: `<rect x="3.5" y="3.5" width="10.5" height="10.5" rx="1.5"/><rect x="10" y="10" width="10.5" height="10.5" rx="1.5" fill="currentColor"/>`,
  toBack: `<rect x="3.5" y="3.5" width="10.5" height="10.5" rx="1.5" fill="currentColor"/><rect x="10" y="10" width="10.5" height="10.5" rx="1.5" ${BG}/>`,
  forward: `<rect x="3.5" y="3.5" width="10.5" height="10.5" rx="1.5"/><rect x="10" y="10" width="10.5" height="10.5" rx="1.5" ${BG}/><path d="M13.2 17.2l3-3 3 3"/>`,
  backward: `<rect x="3.5" y="3.5" width="10.5" height="10.5" rx="1.5"/><rect x="10" y="10" width="10.5" height="10.5" rx="1.5" ${BG}/><path d="M13.2 14.2l3 3 3-3"/>`,
  // ---- align / distribute
  alignLeft: '<path d="M4 3.5v17"/><rect x="7.5" y="6" width="12" height="4.2" rx="1"/><rect x="7.5" y="13.8" width="7" height="4.2" rx="1"/>',
  alignRight: '<path d="M20 3.5v17"/><rect x="4.5" y="6" width="12" height="4.2" rx="1"/><rect x="9.5" y="13.8" width="7" height="4.2" rx="1"/>',
  alignTop: '<path d="M3.5 4h17"/><rect x="6" y="7.5" width="4.2" height="12" rx="1"/><rect x="13.8" y="7.5" width="4.2" height="7" rx="1"/>',
  alignBottom: '<path d="M3.5 20h17"/><rect x="6" y="4.5" width="4.2" height="12" rx="1"/><rect x="13.8" y="9.5" width="4.2" height="7" rx="1"/>',
  alignHCenter: '<path d="M12 3.5v17"/><rect x="5" y="6" width="14" height="4.2" rx="1"/><rect x="8" y="13.8" width="8" height="4.2" rx="1"/>',
  alignVCenter: '<path d="M3.5 12h17"/><rect x="6" y="5" width="4.2" height="14" rx="1"/><rect x="13.8" y="8" width="4.2" height="8" rx="1"/>',
  distH: '<path d="M4 3.5v17M20 3.5v17"/><rect x="9.2" y="7" width="5.6" height="10" rx="1"/>',
  distV: '<path d="M3.5 4h17M3.5 20h17"/><rect x="7" y="9.2" width="10" height="5.6" rx="1"/>',
  flipH: '<path d="M12 3v18" stroke-dasharray="2 2.4"/><path d="M9 7.2L4 17h5z" stroke-linejoin="round"/><path d="M15 7.2L20 17h-5z" stroke-linejoin="round" fill="currentColor"/>',
  flipV: '<path d="M3 12h18" stroke-dasharray="2 2.4"/><path d="M7.2 9L17 4v5z" stroke-linejoin="round"/><path d="M7.2 15L17 20v-5z" stroke-linejoin="round" fill="currentColor"/>',
  // ---- actions
  group: '<rect x="3" y="3" width="18" height="18" rx="2" stroke-dasharray="2.4 2.4"/><rect x="6.5" y="6.5" width="5" height="5" rx="1"/><rect x="12.5" y="12.5" width="5" height="5" rx="1"/>',
  ungroup: '<rect x="3.5" y="3.5" width="9" height="9" rx="1.6"/><rect x="11.5" y="11.5" width="9" height="9" rx="1.6" stroke-dasharray="2.4 2.4"/>',
  link: '<path d="M10 14a4 4 0 005.7 0l3-3a4 4 0 00-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 00-5.7 0l-3 3a4 4 0 005.7 5.7l1-1"/>',
  duplicate: '<rect x="8.5" y="8.5" width="12" height="12" rx="2.2"/><path d="M15.5 8.5V6.7a2.2 2.2 0 00-2.2-2.2H6.7a2.2 2.2 0 00-2.2 2.2v6.6a2.2 2.2 0 002.2 2.2h1.8"/>',
  trash: '<path d="M4 7h16M9.5 7V4.5h5V7M6.2 7l.9 12.2a2 2 0 002 1.8h5.8a2 2 0 002-1.8L17.8 7M10 11v6M14 11v6"/>',
  copyStyle: '<rect x="3.5" y="3.5" width="13" height="5.5" rx="1.2"/><path d="M16.5 6h2.8a1.2 1.2 0 011.2 1.2v3.3a1.2 1.2 0 01-1.2 1.2H12v2.3"/><rect x="10.2" y="14" width="3.6" height="6.5" rx="1.2"/>',
  pasteStyle: '<rect x="3.5" y="3.5" width="13" height="5.5" rx="1.2"/><path d="M16.5 6h2.8a1.2 1.2 0 011.2 1.2v3.3a1.2 1.2 0 01-1.2 1.2H12v1"/><path d="M12 14.5v6M9.5 18l2.5 2.5 2.5-2.5"/>',
  // ---- text
  textLeft: '<path d="M4 6h16M4 10.7h10M4 15.3h16M4 20h10"/>',
  textCenter: '<path d="M4 6h16M7 10.7h10M4 15.3h16M7 20h10"/>',
  textRight: '<path d="M4 6h16M10 10.7h10M4 15.3h16M10 20h10"/>',
  vTop: '<path d="M4 4h16"/><rect x="6.5" y="7.5" width="4" height="11" rx="1"/><rect x="13.5" y="7.5" width="4" height="6.5" rx="1"/>',
  vMiddle: '<path d="M4 12h16" stroke-dasharray="2 2.4"/><rect x="6.5" y="5.5" width="4" height="13" rx="1"/><rect x="13.5" y="8" width="4" height="8" rx="1"/>',
  vBottom: '<path d="M4 20h16"/><rect x="6.5" y="5.5" width="4" height="11" rx="1"/><rect x="13.5" y="10" width="4" height="6.5" rx="1"/>',
  fontHand: FONT(HAND, 'Aa'),
  fontNormal: FONT(SANS, 'Aa'),
  fontCode: FONT(MONO, '&lt;/&gt;'),
  // ---- stroke / edges / fill / sloppiness / arrow types
  strokeSolid: '<path d="M3.5 12h17"/>',
  strokeDashed: '<path d="M3.5 12h4M10 12h4M16.5 12h4"/>',
  strokeDotted: '<path d="M4 12h.01M8 12h.01M12 12h.01M16 12h.01M20 12h.01" stroke-width="3"/>',
  widthThin: '<path d="M4 12h16" stroke-width="1.2"/>',
  widthBold: '<path d="M4 12h16" stroke-width="3"/>',
  widthExtra: '<path d="M4 12h16" stroke-width="5"/>',
  edgeSharp: '<path d="M5 19V5h14"/>',
  edgeRound: '<path d="M5 19v-8a6 6 0 016-6h8"/>',
  fillSolid: '<rect x="4" y="4" width="16" height="16" rx="1.5" fill="currentColor"/>',
  fillHachure: '<rect x="4" y="4" width="16" height="16" rx="1.5"/><path d="M4.5 13L13 4.5M4.5 19.5L19.5 4.5M10 20l10-10M16 20l4-4" stroke-width="1.5"/>',
  fillCross: '<rect x="4" y="4" width="16" height="16" rx="1.5"/><path d="M4.5 13L13 4.5M4.5 19.5L19.5 4.5M10 20l10-10M16 20l4-4M11 4.5L19.5 13M4.5 4.5l15 15M4.5 11l9 9" stroke-width="1.4"/>',
  fillZigzag: '<rect x="4" y="4" width="16" height="16" rx="1.5"/><path d="M5.5 9l3-3 3 3 3-3 3 3M5.5 14.5l3-3 3 3 3-3 3 3M5.5 20l3-3 3 3 3-3 3 3" stroke-width="1.4"/>',
  sloppy0: '<path d="M4 12h16"/>',
  sloppy1: '<path d="M4 12.5c3-2.5 5 2.5 8 0s5-2.5 8 0"/>',
  sloppy2: '<path d="M3.5 13c1.5-6 4 5 6.5-1s3.5 5 5.5-1.2c1.2-3.2 3 .6 4.5.7"/>',
  arrowSharp: '<path d="M4 18L19 6"/><path d="M12.5 6H19v6.5"/>',
  arrowRound: '<path d="M4 18c2-11 9-12 15-10"/><path d="M14.5 5.3L19 8l-3.4 4"/>',
  arrowElbow: '<path d="M4 19h8V6h7"/><path d="M15.5 3.2L19 6l-3.5 2.8"/>',
  // ---- misc UI
  menu: '<path d="M4 7h16M4 12h16M4 17h16"/>',
  help: '<circle cx="12" cy="12" r="9"/><path d="M9.4 9.6a2.7 2.7 0 015.2.9c0 1.7-2.6 2.2-2.6 3.8"/><path d="M12 17.3h.01" stroke-width="2.4"/>',
  search: '<circle cx="10.8" cy="10.8" r="6.3"/><path d="M20 20l-4.7-4.7"/>',
  theme: '<circle cx="12" cy="12" r="8.5"/><path d="M12 3.5a8.5 8.5 0 010 17z" fill="currentColor"/>',
  grid: '<rect x="3.5" y="3.5" width="17" height="17" rx="2"/><path d="M3.5 9.2h17M3.5 14.8h17M9.2 3.5v17M14.8 3.5v17"/>',
  sliders: '<path d="M4 7h9M19 7h1M4 17h1M11 17h9"/><circle cx="16" cy="7" r="2.2"/><circle cx="8" cy="17" r="2.2"/>',
  chevronDown: '<path d="M6 9.5l6 6 6-6"/>',
  chevronUp: '<path d="M6 14.5l6-6 6 6"/>',
  close: '<path d="M6 6l12 12M18 6L6 18"/>',
  check: '<path d="M5 12.5l4.5 4.5L19 7.5"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  // ---- arrowheads (previews pointing right; the start picker mirrors them with CSS)
  head_none: `${HEAD_LINE(3, 21)}`,
  head_arrow: `${HEAD_LINE(3, 20.5)}<path d="M14.5 6.5l6 5.5-6 5.5"/>`,
  head_triangle: `${HEAD_LINE(3, 14)}<path d="M21.5 12l-9-5.2v10.4z" fill="currentColor" stroke-linejoin="round"/>`,
  head_triangle_outline: `${HEAD_LINE(3, 12.5)}<path d="M21.5 12l-9-5.2v10.4z" stroke-linejoin="round"/>`,
  head_circle: `${HEAD_LINE(3, 13.5)}<circle cx="17.4" cy="12" r="3.9" fill="currentColor"/>`,
  head_circle_outline: `${HEAD_LINE(3, 13.5)}<circle cx="17.4" cy="12" r="3.9"/>`,
  head_diamond: `${HEAD_LINE(3, 11)}<path d="M22 12l-5.5-4.6L11 12l5.5 4.6z" fill="currentColor" stroke-linejoin="round"/>`,
  head_diamond_outline: `${HEAD_LINE(3, 11)}<path d="M22 12l-5.5-4.6L11 12l5.5 4.6z" stroke-linejoin="round"/>`,
  head_bar: `${HEAD_LINE(3, 20)}<path d="M20 6.5v11"/>`,
  head_crowfoot_one: `${HEAD_LINE(3, 21)}<path d="M16 6.5v11"/>`,
  head_crowfoot_many: `${HEAD_LINE(3, 12)}<path d="M12 12l9-6M12 12h9M12 12l9 6"/>`,
  head_crowfoot_one_or_many: `${HEAD_LINE(3, 14)}<path d="M14 12l7-5.5M14 12l7 5.5M10 6.5v11"/>`,
};

export const ICON_NAMES = Object.keys(ICONS);

/** Icon name for an arrowhead id (null -> none). */
export function headIconName(id) { return 'head_' + (id || 'none'); }

const tpl = typeof document !== 'undefined' ? document.createElement('template') : null;
const cache = new Map();

/** icon(name, size = 20) -> SVGElement. Unknown names give an empty (but sized) svg so layout never breaks. */
export function icon(name, size = 20) {
  let proto = cache.get(name);
  if (!proto) {
    const body = ICONS[name] || '';
    tpl.innerHTML = `<svg xmlns="${NS}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">${body}</svg>`;
    proto = tpl.content.firstChild;
    cache.set(name, proto);
  }
  const svg = proto.cloneNode(true);
  svg.setAttribute('width', String(size));
  svg.setAttribute('height', String(size));
  svg.classList.add('wbui-icon');
  return svg;
}

/** h('button', { class: 'x', dataset: {tool: 'rect'}, onclick: fn, 'aria-label': 'Foo' }, child | text, ...) */
export function h(tag, attrs, ...children) {
  const el = document.createElement(tag);
  if (attrs) {
    for (const [k, v] of Object.entries(attrs)) {
      if (v == null || v === false) continue;
      if (k === 'class') el.className = v;
      else if (k === 'dataset') for (const [dk, dv] of Object.entries(v)) el.dataset[dk] = String(dv);
      else if (k === 'text') el.textContent = v;
      else if (k.startsWith('on') && typeof v === 'function') el.addEventListener(k.slice(2), v);
      else if (k === 'style' && typeof v === 'object') Object.assign(el.style, v);
      else el.setAttribute(k, v === true ? '' : String(v));
    }
  }
  for (const c of children.flat()) {
    if (c == null || c === false) continue;
    el.append(c.nodeType ? c : document.createTextNode(String(c)));
  }
  return el;
}

/**
 * Anchored popover. `root` must be a positioned element (the module's wrapper); coordinates are computed relative to it.
 * side: 'bottom' (below, left aligned), 'right' (right of the anchor), 'top'.
 * Closes on Esc, outside pointerdown, or the returned close(). Focus returns to the anchor.
 */
export function popover(root, anchor, content, { side = 'bottom', className = '', label = '', onClose } = {}) {
  const pop = h('div', { class: 'wbui-popover ' + className, role: 'dialog', 'aria-label': label || null }, content);
  root.append(pop);
  const place = () => {
    const rr = root.getBoundingClientRect();
    const ar = anchor.getBoundingClientRect();
    const pw = pop.offsetWidth;
    const ph = pop.offsetHeight;
    let left;
    let top;
    if (side === 'right') { left = ar.right - rr.left + 8; top = ar.top - rr.top; }
    else if (side === 'top') { left = ar.left - rr.left; top = ar.top - rr.top - ph - 8; }
    else { left = ar.left - rr.left; top = ar.bottom - rr.top + 8; }
    left = Math.max(8, Math.min(left, rr.width - pw - 8));
    top = Math.max(8, Math.min(top, rr.height - ph - 8));
    pop.style.left = left + 'px';
    pop.style.top = top + 'px';
  };
  place();
  let closed = false;
  const close = (restoreFocus = true) => {
    if (closed) return;
    closed = true;
    document.removeEventListener('pointerdown', onDown, true);
    document.removeEventListener('keydown', onKey, true);
    window.removeEventListener('resize', place);
    pop.remove();
    anchor.setAttribute('aria-expanded', 'false');
    if (restoreFocus && anchor.isConnected) anchor.focus({ preventScroll: true });
    if (onClose) onClose();
  };
  const onDown = (e) => { if (!pop.contains(e.target) && !anchor.contains(e.target)) close(false); };
  const onKey = (e) => {
    if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); close(true); }
  };
  document.addEventListener('pointerdown', onDown, true);
  document.addEventListener('keydown', onKey, true);
  window.addEventListener('resize', place);
  anchor.setAttribute('aria-expanded', 'true');
  return { el: pop, close, place };
}

/** Roving-tabindex helper for a group of buttons: arrows/Home/End move focus. */
export function rovingKeys(container, selector = 'button:not([disabled])') {
  container.addEventListener('keydown', (e) => {
    const keys = ['ArrowRight', 'ArrowLeft', 'ArrowDown', 'ArrowUp', 'Home', 'End'];
    if (!keys.includes(e.key) || e.ctrlKey || e.metaKey || e.altKey) return;
    const items = [...container.querySelectorAll(selector)].filter((b) => !b.closest('[hidden]') && b.offsetParent !== null);
    const i = items.indexOf(document.activeElement);
    if (i < 0) return;
    e.preventDefault();
    let n = i;
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown') n = (i + 1) % items.length;
    else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') n = (i - 1 + items.length) % items.length;
    else if (e.key === 'Home') n = 0;
    else n = items.length - 1;
    items.forEach((b) => b.setAttribute('tabindex', b === items[n] ? '0' : '-1'));
    items[n].focus();
  });
}

/** Inject chrome.css once (idempotent). Hosts that already link it can ignore this; mount.js does it too. */
export function ensureChromeCss() {
  if (typeof document === 'undefined') return;
  if (document.querySelector('link[data-wbui-css], link[href$="/wb/chrome.css"], link[href$="chrome.css"]')) return;
  const link = document.createElement('link');
  link.rel = 'stylesheet';
  link.href = new URL('./chrome.css', import.meta.url).href;
  link.dataset.wbuiCss = '';
  document.head.append(link);
}

/** Mirror wb.mode.theme onto a chrome root as data-wbui-theme (chrome.css then picks light/dark tokens). Returns an unsubscribe. */
export function followTheme(wb, root) {
  const apply = () => {
    const t = wb.mode && wb.mode.theme;
    if (t === 'dark' || t === 'light') root.dataset.wbuiTheme = t; else delete root.dataset.wbuiTheme;
  };
  apply();
  return wb.on('mode', apply);
}

/** Stop keys that a focused chrome control uses from reaching the engine's document-level shortcuts. */
export function guardKeys(root) {
  root.addEventListener('keydown', (e) => {
    const t = e.target;
    if (!t || !t.closest) return;
    if (t.closest('input, textarea, select')) { if (e.key !== 'Escape') e.stopPropagation(); return; }
    if (['Enter', ' ', 'ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Home', 'End'].includes(e.key) && t.closest('button')) e.stopPropagation();
  });
}

/** Toggle `is-compact` on `root` while `host` is narrower than `breakpoint` px. Returns a disconnect fn. */
export function watchWidth(host, root, breakpoint) {
  const apply = () => root.classList.toggle('is-compact', host.clientWidth > 0 && host.clientWidth < breakpoint);
  apply();
  if (typeof ResizeObserver === 'undefined') return () => {};
  const ro = new ResizeObserver(apply);
  ro.observe(host);
  return () => ro.disconnect();
}
