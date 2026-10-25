// Standalone whiteboard page. It makes no network requests of its own: no server API, no AI, nothing else.
import { mountWhiteboard } from '/wb/mount.js';

mountWhiteboard(document.getElementById('board'), {
  storageKey: 'hld-wb:scratch',
  menuExtra: [{ label: 'Back to home', action: () => { location.href = '/'; } }],
});
