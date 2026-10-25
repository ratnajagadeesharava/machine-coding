// Browser-only image helpers for the whiteboard engine: reading image files into validated data URLs and rasterising SVG exports.
import { DATA_URL_RE, LIMITS } from '../../shared/board.js';

export const MAX_IMAGE_BYTES = 5 * 1024 * 1024;
export const MAX_IMAGE_DIM = 1600;

export function readAsDataUrl(blob) {
  return new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(String(r.result));
    r.onerror = () => reject(new Error('Could not read the file'));
    r.readAsDataURL(blob);
  });
}

export function loadImage(src) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error('Could not decode the image'));
    img.src = src;
  });
}

// File -> { dataUrl, w, h }. Large images are downscaled so the longest side is <= MAX_IMAGE_DIM and the data URL fits the board limit.
export async function prepareImage(file) {
  if (!file || typeof file.type !== 'string' || !file.type.startsWith('image/')) throw new Error('That is not an image file');
  if (file.size > MAX_IMAGE_BYTES) throw new Error('Image is larger than 5 MB');
  const raw = await readAsDataUrl(file);
  const img = await loadImage(raw);
  const nw = img.naturalWidth || img.width || 1;
  const nh = img.naturalHeight || img.height || 1;
  const isSvg = file.type === 'image/svg+xml';
  let dataUrl = raw;
  let w = nw;
  let h = nh;
  const big = Math.max(nw, nh) > MAX_IMAGE_DIM;
  const unsupported = !/^data:image\/(png|jpeg|gif|webp|svg\+xml);base64,/.test(raw);
  if (!isSvg && (big || unsupported || raw.length > LIMITS.dataUrl)) {
    const k = Math.min(1, MAX_IMAGE_DIM / Math.max(nw, nh));
    w = Math.max(1, Math.round(nw * k));
    h = Math.max(1, Math.round(nh * k));
    const canvas = document.createElement('canvas');
    canvas.width = w; canvas.height = h;
    canvas.getContext('2d').drawImage(img, 0, 0, w, h);
    const keepAlpha = file.type === 'image/png' || file.type === 'image/gif' || file.type === 'image/webp';
    dataUrl = canvas.toDataURL(keepAlpha ? 'image/png' : 'image/jpeg', 0.9);
    let q = 0.8;
    while (dataUrl.length > LIMITS.dataUrl && q > 0.3) { dataUrl = canvas.toDataURL('image/jpeg', q); q -= 0.15; }
  }
  if (dataUrl.length > LIMITS.dataUrl || !DATA_URL_RE.test(dataUrl)) throw new Error('Image is too large to store on the board');
  return { dataUrl, w, h };
}

// Parses width/height from the root <svg> tag of an export.
export function svgSize(svg) {
  const tag = /<svg\b[^>]*>/.exec(svg)?.[0] || '';
  const num = (name) => {
    const m = new RegExp(`\\s${name}="([0-9.]+)(?:px)?"`).exec(tag);
    return m ? Number(m[1]) : NaN;
  };
  let w = num('width');
  let h = num('height');
  if (!(w > 0 && h > 0)) {
    const vb = /viewBox="([-0-9.\s,]+)"/.exec(tag);
    if (vb) { const p = vb[1].trim().split(/[\s,]+/).map(Number); w = p[2]; h = p[3]; }
  }
  return { w: w > 0 ? w : 300, h: h > 0 ? h : 150 };
}

export async function svgToPngBlob(svg, scale = 2) {
  const { w, h } = svgSize(svg);
  const s = Math.max(0.1, Math.min(8, Number.isFinite(scale) ? scale : 2));
  const cw = Math.min(8192, Math.max(1, Math.round(w * s)));
  const ch = Math.min(8192, Math.max(1, Math.round(h * s)));
  const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml;charset=utf-8' }));
  try {
    const img = await loadImage(url);
    const canvas = document.createElement('canvas');
    canvas.width = cw; canvas.height = ch;
    const ctx = canvas.getContext('2d');
    ctx.drawImage(img, 0, 0, cw, ch);
    return await new Promise((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error('PNG export failed'))), 'image/png'));
  } finally {
    URL.revokeObjectURL(url);
  }
}
