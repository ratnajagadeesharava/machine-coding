// Site theme: follows the system until you pick Light or Dark with the header button; the choice is remembered in this browser.
// Loaded as a plain script in <head> so the right colours are set before the first paint.
(function () {
  var KEY = 'hld-theme';
  var root = document.documentElement;
  function saved() { try { var v = localStorage.getItem(KEY); return v === 'light' || v === 'dark' ? v : null; } catch (e) { return null; } }
  function systemDark() { try { return window.matchMedia('(prefers-color-scheme: dark)').matches; } catch (e) { return false; } }
  function effective() { return saved() || (systemDark() ? 'dark' : 'light'); }
  function apply() { var v = saved(); if (v) root.setAttribute('data-theme', v); else root.removeAttribute('data-theme'); root.style.colorScheme = effective(); }
  apply();
  window.hldTheme = { get: effective, set: function (v) { try { if (v) localStorage.setItem(KEY, v); else localStorage.removeItem(KEY); } catch (e) { /* storage blocked */ } apply(); window.dispatchEvent(new CustomEvent('hld:theme', { detail: effective() })); } };
  try { window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function () { if (!saved()) { apply(); window.dispatchEvent(new CustomEvent('hld:theme', { detail: effective() })); } }); } catch (e) { /* old browser */ }

  var SUN = '<svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></svg>';
  var MOON = '<svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/></svg>';
  function mount() {
    var bar = document.querySelector('.topbar');
    if (!bar || bar.querySelector('[data-theme-btn]')) return;
    var actions = bar.querySelector('.actions');
    if (!actions) { actions = document.createElement('div'); actions.className = 'actions'; bar.appendChild(actions); }
    var b = document.createElement('button');
    b.type = 'button'; b.className = 'btn themebtn'; b.setAttribute('data-theme-btn', '');
    function paint() {
      var dark = effective() === 'dark';
      b.innerHTML = dark ? SUN : MOON;
      var label = dark ? 'Switch to light mode' : 'Switch to dark mode';
      b.title = label; b.setAttribute('aria-label', label);
    }
    b.addEventListener('click', function () { window.hldTheme.set(effective() === 'dark' ? 'light' : 'dark'); });
    window.addEventListener('hld:theme', paint);
    paint();
    actions.appendChild(b);
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', mount); else mount();
})();
