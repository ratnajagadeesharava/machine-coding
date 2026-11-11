// The git branch the code is checked out on (read from .git/HEAD, no git binary needed), so the page can say which branch your progress belongs to.
import fs from 'node:fs';
import path from 'node:path';

export function currentBranch(start) {
  let dir = start;
  for (let i = 0; i < 6; i++) {
    const git = path.join(dir, '.git');
    try {
      const st = fs.statSync(git);
      let headFile = path.join(git, 'HEAD');
      if (st.isFile()) { // worktree / submodule: ".git" is a file pointing at the real dir
        const m = /^gitdir:\s*(.+)$/m.exec(fs.readFileSync(git, 'utf8'));
        if (m) headFile = path.resolve(dir, m[1].trim(), 'HEAD');
      }
      const head = fs.readFileSync(headFile, 'utf8').trim();
      const ref = /^ref:\s*refs\/heads\/(.+)$/.exec(head);
      return ref ? ref[1] : `detached at ${head.slice(0, 7)}`;
    } catch { /* not here: look one level up */ }
    const up = path.dirname(dir);
    if (up === dir) break;
    dir = up;
  }
  return null;
}
