// Tiny .env loader (no dependency). Existing environment variables always win.
import fs from 'node:fs';

export function parseEnv(text) {
  const out = {};
  for (const raw of String(text).split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith('#')) continue;
    const eq = line.indexOf('=');
    if (eq < 1) continue;
    const key = line.slice(0, eq).trim().replace(/^export\s+/, '');
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(key)) continue;
    let value = line.slice(eq + 1).trim();
    const quote = value[0];
    if ((quote === '"' || quote === "'") && value.endsWith(quote) && value.length >= 2) {
      value = value.slice(1, -1);
    } else {
      const hash = value.search(/\s#/);
      if (hash >= 0) value = value.slice(0, hash).trim();
    }
    out[key] = value;
  }
  return out;
}

export function loadEnvFile(file, env = process.env) {
  let text;
  try {
    text = fs.readFileSync(file, 'utf8');
  } catch {
    return false;
  }
  for (const [k, v] of Object.entries(parseEnv(text))) {
    if (env[k] === undefined || env[k] === '') env[k] = v;
  }
  return true;
}
