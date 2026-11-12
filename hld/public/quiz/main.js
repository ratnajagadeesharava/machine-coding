// Boot and routing for the Quizzes page: #/ (subjects), #/subject/<id>, #/topic/<id>, #/play (the player), #/results.
import '../apikey.js';
import { DB, S, bus, loadProgress, setSaveErrorHandler } from './store.js';
import { clearRun, savedRun } from './data.js';
import { renderHome, renderSubject, renderTopic } from './home.js';
import { renderResults } from './results.js';
import { openPlayer } from './player.js';
import { go, openProgressDialog } from './actions.js';
import { $, h, toast } from './dom.js';

const app = $('#app');
const playerHost = $('#player');
let player = null;
setSaveErrorHandler((m) => toast(m, 'err'));

function route() {
  const hash = location.hash || '#/';
  if (player && hash !== '#/play') { player.stop(); player.close(); player = null; }
  if (hash === '#/play') {
    if (!S.run || S.run.result) { go('#/'); return; }
    if (!player) player = openPlayer(playerHost, { onExit: () => { player = null; go('#/'); }, onFinish: () => { player = null; go('#/results'); } });
    return;
  }
  document.body.classList.remove('playing');
  let title = 'Quizzes';
  let m;
  if ((m = /^#\/subject\/(.+)$/.exec(hash))) { renderSubject(app, decodeURIComponent(m[1])); title = $('h1', app)?.textContent || title; }
  else if ((m = /^#\/topic\/(.+)$/.exec(hash))) { renderTopic(app, decodeURIComponent(m[1])); title = $('h1', app)?.textContent || title; }
  else if (hash === '#/results') { renderResults(app); title = 'Results'; }
  else renderHome(app);
  document.title = `${title} - HLD Room`;
}

async function boot() {
  app.replaceChildren(h('div', { class: 'page' }, h('p', { class: 'muted', text: 'Loading quizzes…' })));
  try {
    const [list, health] = await Promise.all([fetch('/api/quizzes').then((r) => r.json()), fetch('/api/health').then((r) => r.json())]);
    S.summaries = list; S.health = health;
  } catch {
    app.replaceChildren(h('div', { class: 'page' }, h('div', { class: 'empty' }, h('b', { text: 'Could not reach the HLD Room server.' }), h('p', { class: 'muted', text: 'Start it with "npm start" in the hld folder, then reload.' }))));
    return;
  }
  await loadProgress();
  if (location.hash === '#/play') location.hash = '#/'; // a reload while playing: the run is saved, offer it on the home page
  window.addEventListener('hashchange', route);
  bus.addEventListener('progress', () => { if (!player) route(); });
  route();
  if (DB.offer) toast('Progress found in this browser. Open "Your progress" to sync it to the database.', '');
  const mode = S.health.ai;
  if (mode === 'mock') document.body.dataset.demo = '1';
}

document.addEventListener('keydown', (e) => {
  if (e.key === '/' && !e.ctrlKey && !e.metaKey && !e.altKey && !player && !document.querySelector('dialog[open]') && !/INPUT|TEXTAREA|SELECT/.test(document.activeElement?.tagName || '')) {
    const box = $('#qsearch'); if (box) { e.preventDefault(); box.focus(); }
  }
});
boot();
export { clearRun, savedRun, openProgressDialog };
