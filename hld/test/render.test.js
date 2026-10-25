import test from 'node:test';
import assert from 'node:assert/strict';
import { emptyBoard, newId, createElement, ARROWHEADS, FILL_STYLES, ELEMENT_TYPES } from '../shared/board.js';
import { boardToSvgString, elementToSvg, exportBounds, fontCss, frameClipPath, themeColor } from '../public/wb/render.js';

// ---- tiny XML well-formedness checker: tag stack, quoted attributes, escaped text, unique ids ----
function checkXml(xml) {
  const stack = []; const ids = new Set(); let i = 0; let roots = 0;
  const fail = (m) => assert.fail(`${m} at ${i}: ...${xml.slice(Math.max(0, i - 40), i + 40)}...`);
  while (i < xml.length) {
    if (xml[i] !== '<') {
      const j = xml.indexOf('<', i);
      const text = xml.slice(i, j === -1 ? xml.length : j);
      if (/[<>]/.test(text)) fail('raw angle bracket in text');
      if (/&(?!(amp|lt|gt|quot|apos|#\d+|#x[0-9a-f]+);)/i.test(text)) fail('unescaped ampersand');
      if (stack.length === 0 && text.trim()) fail('text outside root');
      i = j === -1 ? xml.length : j; continue;
    }
    const end = xml.indexOf('>', i);
    if (end === -1) fail('unterminated tag');
    const tag = xml.slice(i + 1, end);
    if (tag.startsWith('/')) {
      const name = tag.slice(1).trim();
      if (stack.pop() !== name) fail(`mismatched close </${name}>`);
    } else {
      const m = /^([A-Za-z][\w:-]*)((?:\s+[A-Za-z_:][\w:.-]*="[^"<]*")*)\s*(\/?)$/.exec(tag);
      if (!m) fail(`bad tag <${tag.slice(0, 80)}>`);
      for (const a of m[2].matchAll(/\s([\w:.-]+)="([^"]*)"/g)) {
        if (/&(?!(amp|lt|gt|quot|apos|#\d+|#x[0-9a-f]+);)/i.test(a[2])) fail(`unescaped & in attribute ${a[1]}`);
        if (a[1] === 'id') { if (ids.has(a[2])) fail(`duplicate id ${a[2]}`); ids.add(a[2]); }
      }
      if (!m[3]) stack.push(m[1]); else if (stack.length === 0) roots++;
      if (stack.length === 1 && !m[3]) roots++;
    }
    i = end + 1;
  }
  assert.equal(stack.length, 0, 'unclosed tags');
  assert.equal(roots, 1, 'exactly one root');
  return ids;
}

function make(roughness = 1) {
  const b = emptyBoard();
  const add = (type, props = {}) => { const el = createElement(type, { id: newId(b), seed: 7 + b.nextId, roughness, ...props }); b.items.push(el); return el; };
  return { b, add };
}
const PNG = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==';

test('themeColor maps like invert(93%) hue-rotate(180deg)', () => {
  assert.equal(themeColor('#ffffff', false), '#ffffff');
  assert.equal(themeColor('transparent', true), 'transparent');
  const white = themeColor('#ffffff', true); const black = themeColor('#000000', true);
  const lum = (h) => parseInt(h.slice(1, 3), 16) * 0.3 + parseInt(h.slice(3, 5), 16) * 0.59 + parseInt(h.slice(5, 7), 16) * 0.11;
  assert.ok(lum(white) < 30, white); assert.ok(lum(black) > 220, black);
  assert.equal(themeColor('#1e1e1e', true).length, 7);
  // chromatic colours keep their hue family: blue stays bluish, red stays reddish
  const red = themeColor('#e03131', true); const blue = themeColor('#1971c2', true);
  assert.ok(parseInt(red.slice(1, 3), 16) > parseInt(red.slice(5, 7), 16), red);
  assert.ok(parseInt(blue.slice(5, 7), 16) > parseInt(blue.slice(1, 3), 16), blue);
  assert.match(themeColor('rgba(255,255,255,0.5)', true), /^rgba\(\d+,\d+,\d+,0\.5\)$/);
  assert.equal(themeColor('#fff', true), white);
  assert.equal(themeColor('notacolour', true), 'notacolour');
  // round trip is not exact (93%) but white->dark->? stays bright vs dark ordering
  assert.ok(lum(themeColor(themeColor('#ffffff', true), true)) > 200);
});

test('fontCss and frameClipPath', () => {
  assert.match(fontCss(3), /monospace/); assert.match(fontCss(1), /cursive/); assert.match(fontCss(99), /cursive/);
  const { b, add } = make(); const f = add('frame', { x: 5, y: 6, w: 100, h: 50 });
  const cp = frameClipPath(b, f);
  assert.match(cp, /<clipPath id="wb-fclip-1"><rect x="5" y="6" width="100" height="50"\/><\/clipPath>/);
});

test('every element type x roughness x fill x stroke style renders non-empty valid markup', () => {
  for (const roughness of [0, 1, 2]) for (const type of ELEMENT_TYPES) {
    for (const fillStyle of FILL_STYLES) for (const strokeStyle of ['solid', 'dashed', 'dotted']) {
      const { b, add } = make(roughness);
      const props = { x: 10, y: 10, w: 120, h: 80, fillStyle, strokeStyle, backgroundColor: '#a5d8ff', strokeWidth: 2, text: 'Label' };
      if (type === 'line' || type === 'arrow') Object.assign(props, { points: [[0, 0], [100, 40]], w: 100, h: 40 });
      if (type === 'freedraw') Object.assign(props, { points: [[0, 0], [10, 5], [20, 3], [40, 20]], w: 40, h: 20 });
      if (type === 'image') props.dataUrl = PNG;
      if (type === 'frame') props.name = 'F';
      const el = add(type, props);
      const svg = elementToSvg(b, el, {});
      assert.ok(svg.length > 30, `${type}`);
      assert.match(svg, new RegExp(`^<g data-id="${el.id}" class="wb-el"`));
      checkXml(`<svg xmlns="http://www.w3.org/2000/svg">${svg}</svg>`);
    }
  }
});

test('stroke styles produce dash arrays and round caps', () => {
  const { b, add } = make(0);
  const solid = elementToSvg(b, add('rect', { strokeStyle: 'solid', strokeWidth: 2 }), {});
  const dashed = elementToSvg(b, add('rect', { strokeStyle: 'dashed', strokeWidth: 2 }), {});
  const dotted = elementToSvg(b, add('rect', { strokeStyle: 'dotted', strokeWidth: 2 }), {});
  assert.ok(!solid.includes('stroke-dasharray'));
  assert.ok(dashed.includes('stroke-dasharray="8 10"'));
  assert.ok(dotted.includes('stroke-dasharray="1.5 8"') && dotted.includes('stroke-linecap="round"'));
  assert.ok(solid.includes('stroke-linejoin="round"'));
});

test('opacity and rotation about the centre', () => {
  const { b, add } = make(0);
  const el = add('rect', { x: 0, y: 0, w: 100, h: 50, angle: Math.PI / 2, opacity: 40 });
  const svg = elementToSvg(b, el, {});
  assert.match(svg, /opacity="0.4"/);
  assert.match(svg, /transform="rotate\(90 50 25\)"/);
  assert.ok(!elementToSvg(b, add('rect', {}), {}).includes('transform='));
});

test('fills: solid path, hatch with a unique clipPath per element, transparent has none', () => {
  const { b, add } = make(1);
  const a = add('rect', { fillStyle: 'hachure', backgroundColor: '#ffc9c9' });
  const c = add('ellipse', { fillStyle: 'cross-hatch', backgroundColor: '#ffc9c9' });
  const s = add('rect', { fillStyle: 'solid', backgroundColor: '#ffc9c9' });
  const t = add('rect', { backgroundColor: 'transparent' });
  assert.match(elementToSvg(b, a, {}), /<clipPath id="wb-clip-1">/);
  assert.match(elementToSvg(b, c, {}), /<clipPath id="wb-clip-2">/);
  assert.ok(!elementToSvg(b, s, {}).includes('clipPath'));
  assert.ok(!elementToSvg(b, t, {}).includes('#ffc9c9'));
  assert.ok(elementToSvg(b, s, {}).includes('fill="#ffc9c9"'));
});

test('labels: wrapped tspans, skipLabel, align and escaping', () => {
  const { b, add } = make(0);
  const el = add('rect', { w: 100, h: 100, text: 'one two three four five six seven', fontSize: 16 });
  const svg = elementToSvg(b, el, {});
  assert.ok((svg.match(/<tspan/g) || []).length >= 3, 'wrapped into several lines');
  assert.match(svg, /text-anchor="middle"/);
  assert.ok(!elementToSvg(b, el, { skipLabel: true }).includes('<text'));
  assert.match(elementToSvg(b, { ...el, textAlign: 'left' }, {}), /text-anchor="start"/);
  assert.match(elementToSvg(b, { ...el, textAlign: 'right' }, {}), /text-anchor="end"/);
  const hostile = add('rect', { text: '</text><script>alert(1)</script>"onload="x & <b>' });
  const out = elementToSvg(b, hostile, {});
  assert.ok(!out.includes('<script') && !out.includes('</text><script'));
  assert.ok(out.includes('&lt;/text&gt;&lt;script&gt;'));
  assert.ok(out.includes('&quot;onload=&quot;'));
  checkXml(`<svg xmlns="http://www.w3.org/2000/svg">${out}</svg>`);
});

test('free text: multi-line tspans, font family css, size and colour', () => {
  const { b, add } = make(0);
  const el = add('text', { x: 10, y: 10, text: 'a\nb\nc', fontFamily: 3, fontSize: 24, strokeColor: '#e03131', w: 20, h: 90 });
  const svg = elementToSvg(b, el, {});
  assert.equal((svg.match(/<tspan/g) || []).length, 3);
  assert.ok(svg.includes('font-size="24"') && svg.includes('monospace') && svg.includes('fill="#e03131"'));
  checkXml(`<svg xmlns="http://www.w3.org/2000/svg">${svg}</svg>`);
});

test('arrows: all arrowheads, three arrow types, labels with a halo, bound ends', () => {
  const { b, add } = make(1);
  const from = add('rect', { x: 0, y: 0, w: 100, h: 60 });
  const to = add('rect', { x: 300, y: 0, w: 100, h: 60 });
  for (const kind of ARROWHEADS.map((a) => a.id).filter(Boolean)) {
    for (const arrowType of ['sharp', 'round', 'elbow']) {
      const a = add('arrow', { x: 100, y: 30, points: [[0, 0], [100, 40], [200, 0]], w: 200, h: 40, arrowType, endArrowhead: kind, startArrowhead: kind, from: from.id, to: to.id, text: 'writes' });
      const svg = elementToSvg(b, a, { background: '#ffffff' });
      assert.ok(svg.split('<path').length >= 4, `${kind}/${arrowType} has body and two heads`);
      assert.ok(svg.includes('writes') && svg.includes('<rect') && svg.includes('rx="4"'));
      checkXml(`<svg xmlns="http://www.w3.org/2000/svg">${svg}</svg>`);
    }
  }
  const ln = add('line', { x: 0, y: 0, points: [[0, 0], [10, 10]], w: 10, h: 10, endArrowhead: 'arrow' });
  assert.ok(!elementToSvg(b, ln, {}).includes('stroke-linecap="round" stroke-linejoin="round"/><path'), 'lines get no arrowheads');
});

test('sticky has a shadow and paper; database; link badge; locked renders normally', () => {
  const { b, add } = make(1);
  const st = add('sticky', { text: 'note', w: 160, h: 120 });
  const svg = elementToSvg(b, st, {});
  assert.ok(svg.includes('rgba(0,0,0,0.16)') && svg.includes('#ffec99'));
  assert.match(svg, /text-anchor="start"/);
  const db = add('database', { text: 'DB', locked: true, link: 'https://example.com/?a=1&b=2' });
  const out = elementToSvg(b, db, {});
  assert.ok(out.includes('class="wb-link"') && out.includes('a=1&amp;b=2'));
  assert.ok(!elementToSvg(b, add('rect', {}), {}).includes('wb-link'));
  checkXml(`<svg xmlns="http://www.w3.org/2000/svg">${svg}${out}</svg>`);
});

test('images only embed valid data urls', () => {
  const { b, add } = make(0);
  assert.match(elementToSvg(b, add('image', { dataUrl: PNG }), {}), /<image href="data:image\/png;base64,/);
  const bad = elementToSvg(b, add('image', { dataUrl: 'javascript:alert(1)' }), {});
  assert.ok(!bad.includes('<image') && !bad.includes('javascript'));
  assert.ok(!elementToSvg(b, add('image', { dataUrl: 'data:image/png;base64,AAA"onload="x' }), {}).includes('<image'));
  assert.ok(!elementToSvg(b, add('image', { dataUrl: '' }), {}).includes('<image'));
});

test('dark mode maps stroke, background and text colours, not images', () => {
  const { b, add } = make(0);
  const el = add('rect', { strokeColor: '#1e1e1e', backgroundColor: '#ffffff', text: 'x' });
  const light = elementToSvg(b, el, {}); const dark = elementToSvg(b, el, { dark: true });
  assert.ok(light.includes('#1e1e1e') && light.includes('fill="#ffffff"'));
  assert.ok(!dark.includes('#1e1e1e') && !dark.includes('fill="#ffffff"'));
  assert.ok(dark.includes(themeColor('#1e1e1e', true)) && dark.includes(themeColor('#ffffff', true)));
  const img = add('image', { dataUrl: PNG });
  assert.equal(elementToSvg(b, img, { dark: true }), elementToSvg(b, img, {}));
  assert.ok(elementToSvg(b, add('rect', { backgroundColor: 'transparent' }), { dark: true }).includes('fill="none"'));
});

test('rendering is deterministic', () => {
  const { b, add } = make(2);
  const el = add('ellipse', { fillStyle: 'hachure', backgroundColor: '#a5d8ff', text: 'x' });
  assert.equal(elementToSvg(b, el, {}), elementToSvg(b, { ...el }, {}));
  assert.notEqual(elementToSvg(b, el, {}), elementToSvg(b, { ...el, seed: el.seed + 1 }, {}));
});

test('exportBounds: strokes, arrowheads, rotation, ids subset, frames, empty', () => {
  const { b, add } = make(0);
  assert.equal(exportBounds(b), null);
  const r = add('rect', { x: 100, y: 100, w: 100, h: 50, strokeWidth: 4 });
  const bb = exportBounds(b);
  assert.ok(bb.x < 100 && bb.y < 100 && bb.x + bb.w > 200 && bb.y + bb.h > 150);
  const rot = add('rect', { x: 300, y: 300, w: 100, h: 20, angle: Math.PI / 2 });
  const sub = exportBounds(b, [rot.id]);
  assert.ok(sub.w < 40 && sub.h > 100 && sub.x > 300);
  const arrow = add('arrow', { x: 0, y: 0, points: [[0, 0], [100, 0]], w: 100, h: 0, endArrowhead: 'arrow' });
  const ab = exportBounds(b, [arrow.id]);
  assert.ok(ab.x + ab.w > 100 && ab.h > 10, 'room for the head');
  assert.equal(exportBounds(b, [999]), null);
  assert.ok(r);
});

test('empty board gives a small valid SVG', () => {
  const svg = boardToSvgString(emptyBoard(), {});
  checkXml(svg);
  assert.match(svg, /^<svg xmlns="http:\/\/www.w3.org\/2000\/svg" viewBox="0 0 100 100"/);
  const bg = boardToSvgString(emptyBoard(), { background: '#ffffff', scale: 2 });
  assert.match(bg, /width="200" height="200"/); assert.ok(bg.includes('<rect'));
});

test('board with one of every element type, frames with children, hostile text: well-formed XML with unique ids', () => {
  for (const roughness of [0, 1, 2]) for (const dark of [false, true]) {
    const { b, add } = make(roughness);
    const frame = add('frame', { x: 0, y: 0, w: 300, h: 200, name: '"><script>alert(1)</script>&' });
    const frame2 = add('frame', { x: 400, y: 0, w: 100, h: 100, name: 'B' });
    const kids = [];
    for (const type of ['rect', 'diamond', 'ellipse', 'database', 'sticky']) {
      kids.push(add(type, { x: 20 + kids.length * 40, y: 20, w: 100, h: 80, text: `</text><script>x</script> ${type}`, frameId: frame.id, fillStyle: FILL_STYLES[kids.length % 4], backgroundColor: '#b2f2bb', link: 'https://a.b/?x=1&y="2"' }));
    }
    add('text', { x: 10, y: 250, text: '"onload="alert(1)" <b>&amp;</b>', w: 100, h: 25, frameId: frame2.id });
    add('image', { x: 350, y: 250, w: 40, h: 40, dataUrl: PNG });
    add('freedraw', { x: 20, y: 300, points: [[0, 0], [5, 5], [12, 4], [30, 18]], w: 30, h: 18 });
    add('line', { x: 20, y: 340, points: [[0, 0], [100, 20]], w: 100, h: 20, arrowType: 'round', strokeStyle: 'dashed' });
    add('arrow', { x: 20, y: 380, points: [[0, 0], [100, 20]], w: 100, h: 20, from: kids[0].id, to: kids[1].id, text: '"quoted" <label>' });
    for (const k of ARROWHEADS.map((a) => a.id).filter(Boolean)) add('arrow', { x: 200, y: 300, points: [[0, 0], [80, 0]], w: 80, h: 0, endArrowhead: k });
    const svg = boardToSvgString(b, { dark, background: '#ffffff', padding: 12, scale: 2 });
    const ids = checkXml(svg);
    assert.ok(ids.has('wb-fclip-1') && ids.has('wb-fclip-2'), 'frame clip paths');
    assert.ok(!svg.includes('<script'));
    assert.ok(svg.includes(`clip-path="url(#wb-fclip-${frame.id})"`));
    assert.equal((svg.match(/class="wb-el"/g) || []).length, b.items.length);
    const vb = svg.match(/viewBox="([-\d. ]+)" width="([\d.]+)" height="([\d.]+)"/);
    const [, , , vw, vh] = vb[1].split(' ').map(Number).concat([0]);
    assert.ok(Number(vb[2]) > 0 && Number(vb[3]) > 0);
    assert.ok(vw !== undefined || vh !== undefined);
    const nums = vb[1].split(' ').map(Number);
    assert.ok(Math.abs(Number(vb[2]) - nums[2] * 2) < 0.02 && Math.abs(Number(vb[3]) - nums[3] * 2) < 0.02, 'size = viewBox * scale');
  }
});

test('ids restrict the export; children keep clipping only when their frame is exported', () => {
  const { b, add } = make(0);
  const f = add('frame', { x: 0, y: 0, w: 100, h: 100, name: 'F' });
  const kid = add('rect', { x: 50, y: 50, w: 200, h: 200, frameId: f.id });
  const other = add('rect', { x: 1000, y: 1000, w: 10, h: 10 });
  const only = boardToSvgString(b, { ids: [other.id] });
  assert.ok(only.includes(`data-id="${other.id}"`) && !only.includes(`data-id="${f.id}"`) && !only.includes(`data-id="${kid.id}"`));
  const withFrame = boardToSvgString(b, { ids: [f.id, kid.id] });
  assert.ok(withFrame.includes('wb-fclip-1'));
  const vb = withFrame.match(/viewBox="([-\d. ]+)"/)[1].split(' ').map(Number);
  assert.ok(vb[2] < 200, 'content clipped to the frame bounds');
  const kidOnly = boardToSvgString(b, { ids: [kid.id] });
  assert.ok(!kidOnly.includes('wb-fclip'));
  const vb2 = kidOnly.match(/viewBox="([-\d. ]+)"/)[1].split(' ').map(Number);
  assert.ok(vb2[2] > 200);
  checkXml(withFrame); checkXml(kidOnly); checkXml(only);
});

test('background rect, dark mapping of the background, padding and scale', () => {
  const { b, add } = make(0);
  add('rect', { x: 0, y: 0, w: 100, h: 100, strokeWidth: 2 });
  const light = boardToSvgString(b, { background: '#ffffff', padding: 0 });
  const dark = boardToSvgString(b, { background: '#ffffff', dark: true, padding: 0 });
  assert.ok(light.includes('fill="#ffffff"') && !dark.includes('fill="#ffffff"'));
  assert.ok(dark.includes(`fill="${themeColor('#ffffff', true)}"`));
  assert.ok(!boardToSvgString(b, { padding: 0 }).includes('<rect x="-'), 'no background by default');
  const p0 = Number(boardToSvgString(b, { padding: 0 }).match(/width="([\d.]+)"/)[1]);
  const p20 = Number(boardToSvgString(b, { padding: 20 }).match(/width="([\d.]+)"/)[1]);
  assert.ok(Math.abs(p20 - p0 - 40) < 0.02);
  const s3 = Number(boardToSvgString(b, { padding: 0, scale: 3 }).match(/width="([\d.]+)"/)[1]);
  assert.ok(Math.abs(s3 - p0 * 3) < 0.05);
});
