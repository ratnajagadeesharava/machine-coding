// Dev harness: mounts only the whiteboard engine full-window and exposes it as window.wb (used by test/engine.e2e.mjs).
import { Whiteboard } from './engine.js';

const q = new URLSearchParams(location.search);
const host = document.getElementById('host');
const wb = new Whiteboard(host, {
  theme: q.get('theme') === 'dark' ? 'dark' : 'light',
  readOnly: q.has('readonly'),
  grid: q.has('grid'),
});
window.wb = wb;
window.__events = [];
for (const t of ['change', 'selection', 'tool', 'style', 'view', 'mode', 'history', 'editing', 'toast', 'contextmenu', 'request']) {
  wb.on(t, (p) => window.__events.push({ t, p: t === 'change' ? undefined : p }));
}
window.__wbReady = true;
