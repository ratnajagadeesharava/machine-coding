// One look everywhere: the cream and amber theme, in every browser and system setting. (There used to be a light/dark switch; the two modes looked like
// two different apps, so the dark one was removed.) Loaded as a plain script in <head> so the colours are set before the first paint.
(function () {
  var root = document.documentElement;
  root.setAttribute('data-theme', 'light');
  root.style.colorScheme = 'light';
  try { localStorage.removeItem('hld-theme'); } catch (e) { /* storage blocked */ }
  // Kept so pages that ask for the theme (the whiteboard) always get "light".
  window.hldTheme = { get: function () { return 'light'; }, set: function () { /* fixed */ } };
})();
