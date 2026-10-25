# HLD Room whiteboard - architecture and module contracts

The whiteboard is a **standalone component**. It must work with no interview question, no AI bot and no server API at all:

- `public/board.html` is a plain whiteboard page (nothing else on it).
- `public/room.html` (the interview room) embeds the same component next to a closable question panel and a closable bot panel.
- **Nothing under `public/wb/` or `shared/` may import from `app.js`/`room.js`/`quiz.js`, mention `/api/`, call `fetch` against the server, or know about questions or the AI.** A test enforces this.

Reference product: Excalidraw (the feature list below was taken from its source: tool types, actions, shortcuts, element properties).
HackerRank's interview whiteboard is built on Excalidraw and adds a shapes library, colour/stroke/edge styling, freehand, an unlimited zoomable
canvas and live cursors. Live multi-user collaboration, encrypted share links and embeds are out of scope (see "Not included").

Everything is vanilla ES modules, no build step, no dependencies. Browser target: current Chrome, Firefox, Safari. Node >= 20 for tests.
No `innerHTML` with user-controlled strings except markup produced by `public/wb/render.js` (which escapes everything).

## 1. File map and owners

| File | Purpose | Owner |
|---|---|---|
| `shared/wbconst.js` | constants (tools, palettes, fonts, defaults, limits) | done |
| `shared/board.js` | element model, factory, sanitize, geometry basics, cloning, `describeBoard` for the AI | done |
| `public/wb/rough.js` | seeded hand-drawn ("sketchy") path generation, fill patterns | A |
| `public/wb/geometry.js` | hit testing, boxes, handles, resizing maths, snapping, text metrics, stroke smoothing | A |
| `public/wb/render.js` | element -> SVG markup string, whole board -> standalone SVG string, dark-theme colour mapping | A |
| `public/wb/ops.js` | pure board transformations (style, z-order, group, align, distribute, flip, duplicate, delete, frames, find) | B2 |
| `public/wb/engine.js`, `engine.css` | the `Whiteboard` class: scene, history, selection, view, tools, pointer/keyboard, text editing, clipboard, snapping | B1 |
| `public/wb/icons.js`, `ui-toolbar.js`, `ui-properties.js`, `ui-zoom.js`, `ui-help.js`, `chrome.css` | toolbar, properties panel, zoom + undo/redo, shortcuts dialog | C |
| `public/wb/menu-icons.js`, `ui-menu.js`, `ui-export.js`, `ui-context.js`, `ui-palette.js`, `ui-library.js`, `menus.css` | main menu, export dialog, context menu, command palette + search, library panel | D |
| `public/wb/interop.js`, `library-data.js` | Excalidraw import/export, scene + library files, built-in system-design library | E |
| `public/wb/mount.js` | assembles engine + UI into one call | F |
| `public/board.html`, `board.js`, `index.html` (home), `home.js`, `room.html`, `room.js`, `server/*` | pages, question decoupling, server | F |

## 2. Elements

Defined in `shared/board.js` / `shared/wbconst.js`. A board is `{ version: 2, items: Element[], nextId }`; `items` is z-order, bottom to top.
Ids are integers (group ids come from the same counter). Always create elements with `createElement(type, props)` and ids with `newId(board)`.

Common fields: `id, type, x, y, w, h, angle (radians), strokeColor, backgroundColor, fillStyle, strokeWidth (1|2|4), strokeStyle, roughness (0|1|2), opacity (0..100), roundness ('sharp'|'round'), seed, groupIds[] (innermost first), frameId|null, locked, link|null`.

| type | extra fields | notes |
|---|---|---|
| `rect`, `diamond`, `ellipse`, `database` | `text` (bound label), `fontFamily, fontSize, textAlign, verticalAlign` | label is drawn inside, wrapped to the shape; double-click edits it |
| `sticky` | same as above | filled paper note, default `#ffec99`, text top-left aligned |
| `text` | `text, fontFamily, fontSize, textAlign` | free text, `w`/`h` follow the text (see `textSize`) |
| `image` | `dataUrl` (validated `data:image/...;base64,`) | aspect locked on resize |
| `frame` | `name` | labelled region; elements with `frameId === frame.id` are clipped to it and move with it; not rotatable |
| `freedraw` | `points` (relative to `x,y`, first is `[0,0]`) | smoothed pressure-less stroke |
| `line`, `arrow` | `points` (>= 2, relative to `x,y`, first is `[0,0]`), `arrowType` (`sharp`\|`round`\|`elbow`; lines: sharp/round), `startArrowhead`, `endArrowhead` (ids in `ARROWHEADS`), `from`, `to` (bound element ids, arrows only), `text` (label) | `w,h` = bounds of points. Bound ends follow their shapes; use `resolvePoints(board, el)` for absolute points |

Rotation applies to `rect, diamond, ellipse, database, sticky, text, image` (see `NO_ROTATE_TYPES`).

`shared/board.js` helpers every module may use: `emptyBoard, newId, createElement, findItem, cloneBoard, cloneElements, sanitizeBoard,
center, rotatePoint, edgePoint, sideAnchor, elbowRoute, resolvePoints, elementBounds, selectionBounds, pointsBounds, normalizePoints,
describeBoard, boardStats, boardSignature, toAiBoard, isShape, isLinear, isArrow, hasPoints, isContainer, canRotate, labelOf` plus every export of `wbconst.js`.

## 3. Rendering (`public/wb/render.js`, `geometry.js`, `rough.js`) - owner A

All of these are **DOM-free** (strings and numbers), so they run in Node and can be unit-tested.

```js
// render.js
elementToSvg(board, el, { dark?, skipLabel? }) -> string          // one <g data-id="..." class="wb-el"> ... </g>, including its bound text; no selection UI
boardToSvgString(board, { ids?, background?: string|null, dark?, padding = 10, scale = 1 }) -> string   // standalone <svg xmlns> sized to the content; frames clip children; images embedded
exportBounds(board, ids?) -> {x,y,w,h} | null                    // content bounds incl. stroke/arrowheads; null when empty
themeColor(color, dark) -> string                                // the dark-theme colour mapping (invert 93% + hue-rotate 180deg)
frameClipPath(board, frame) -> string                            // <clipPath> markup the engine puts in <defs>
fontCss(fontFamily) -> string                                    // FONT_FAMILIES[id].css
// rough.js (seeded: same element + seed => same drawing)
shapePath(el) -> { fill: string|null, stroke: string, fillKind }   // path data for rect/diamond/ellipse/database/sticky/frame honouring roughness, roundness
linearPath(points, { arrowType, roughness, seed }) -> string      // line/arrow body (sharp polyline, round = smooth curve through points, elbow = orthogonal with small corner radius)
freedrawPath(points, strokeWidth) -> string                       // smoothed variable-width outline
arrowheadPath(kind, tip, direction, strokeWidth) -> { d: string, fill: boolean }
fillPattern(el) -> { kind: 'solid'|'hachure'|'cross-hatch'|'zigzag', d: string }   // hatch lines clipped to the shape by the caller
// geometry.js
hitElement(board, el, point, tol) -> boolean                     // outline hit for hollow shapes, interior hit for filled ones, rotation aware, label area counts
hitTestTop(board, point, tol, { skipLocked?, ids? }) -> el|null
elementsInBox(board, box, { contain: boolean }) -> number[]      // marquee selection (contain=false: intersects)
transformHandles(box, angle, zoom, { rotation = true, linear = false }) -> { nw,n,ne,e,se,s,sw,w, rotate }   // handle centres in world coordinates
handleAt(handles, point, zoom) -> name|null
resizeBox(el, handle, point, { keepAspect, fromCenter }) -> { x,y,w,h }   // honours angle
resizeSelection(board, ids, handle, point, { keepAspect, fromCenter }) -> Map<id, patch>   // scales every element in a multi-selection about the opposite handle
rotateAngle(center, point, { snap }) -> radians                   // snap = 15 degree steps
snapToGrid(v, grid) -> number
snapMove(board, movingIds, dx, dy, { threshold, others }) -> { dx, dy, guides: [{ axis: 'x'|'y', pos, from, to }] }   // snap edges/centres to other elements
nearestBindTarget(board, point, tol, { excludeId }) -> element|null   // for arrow binding
textSize(text, { fontFamily, fontSize, maxWidth? }) -> { w, h, lines: string[] }   // DOM-free estimate: per-family average glyph widths; wraps to maxWidth
simplifyStroke(points, tolerance) -> points                       // Douglas-Peucker for freedraw
```

## 4. Board operations (`public/wb/ops.js`) - owner B2

Pure functions. They **mutate the `board` passed in** (the engine passes a working copy and handles history) and return what changed. None touch the DOM.

```js
STYLE_PROPS: { strokeColor, backgroundColor, fillStyle, strokeWidth, strokeStyle, roughness, opacity, roundness, fontFamily, fontSize, textAlign,
  verticalAlign, arrowType, startArrowhead, endArrowhead }  // each -> list of element types it applies to
applyStyle(board, ids, patch) -> number[]               // only applies props meaningful for each element; text elements resize via textSize; returns changed ids
styleSummary(board, ids, fallbackStyle) -> { values: { [prop]: value | MIXED }, applies: { [prop]: boolean } }   // MIXED = Symbol.for('wb.mixed') exported as MIXED
expandSelection(board, ids) -> number[]                 // whole groups, and every child of a selected frame
reorder(board, ids, 'front'|'forward'|'backward'|'back') -> boolean   // groups move together; Excalidraw semantics
group(board, ids) -> number|null; ungroup(board, ids) -> number[]
setLocked(board, ids, locked) -> number[]; unlockAll(board) -> number[]
align(board, ids, 'left'|'right'|'top'|'bottom'|'hcenter'|'vcenter') -> number[]   // by selection bounds; bound arrows follow automatically
distribute(board, ids, 'h'|'v') -> number[]             // needs >= 3 elements
flip(board, ids, 'h'|'v') -> number[]                   // mirrors about the selection centre; swaps arrowheads/points for linear
duplicate(board, ids, { dx = 20, dy = 20 }) -> number[] // new ids, groups/bindings remapped (uses cloneElements)
deleteElements(board, ids) -> number[]                  // unbinds arrows (they keep their drawn ends); deleting a frame deletes its children
moveElements(board, ids, dx, dy) -> void                // frames drag their children along; selected arrows with bound ends only move free ends
assignFrames(board, ids) -> void                        // set/clear frameId by containment of the element centre
bindArrow(board, arrowId, { start?: point|null, end?: point|null, tol }) -> void   // (re)bind an arrow's ends to the shape under each end
findText(board, query, { caseSensitive = false }) -> number[]   // ids whose text/name/arrow label contains the query, in z-order
copyStyle(el) -> style object; pasteStyle(board, ids, style) -> number[]
```

## 5. Engine (`public/wb/engine.js`) - owner B1

```js
import { Whiteboard } from './engine.js';
const wb = new Whiteboard(containerEl, { board?, readOnly?, theme?: 'light'|'dark', background?, grid?: false, keyboard?: true });
```
The engine owns an `<svg>` inside the container, a text-editing `<textarea>`, a laser-trail layer and selection/handle overlays, and **all pointer and
keyboard handling** (shortcuts below) plus wheel zoom/pan and touch pinch. Keyboard shortcuts only fire when focus is not in an input/textarea/dialog
and `options.keyboard !== false` (a host may disable them while a chat box has focus; `wb.setKeyboard(bool)`).
It never renders any chrome: toolbars and panels are separate modules that talk to it through this API.

**State (read-only getters):** `board` (live; do not mutate), `tool`, `style` (current item style, see `DEFAULT_STYLE`), `selection` (`Set<number>`), `view` (`{x,y,zoom}`), `mode`
(`{ grid, gridSize, snap, zen, view, theme, background }`), `editingId`, `toolLocked`.

**Scene:** `getBoard()` (deep copy), `setBoard(board, { resetHistory = true, fit = false })`, `isEmpty()`, `clear()` (undoable), `insertElements(elements, { at?: {x,y} | 'viewport-center', select = true }) -> ids` (clones with new ids, offsets, undoable), `addImageFile(file, at?) -> Promise<id>` (downscales large images to max 1600px, rejects non-images and > 5 MB), `toTemplate() -> { elements }` (selection normalised to 0,0, for the library), `find(query) -> ids`, `focusElement(id)` (select + scroll/zoom into view), `focusNext/Prev` for search stepping is done by the UI via `find` + `focusElement`.

**Tools:** `setTool(id, { lock = false })` (ids in `TOOLS`), `toggleToolLock()`. After creating one element the tool returns to `select` unless locked; `arrow`/`line` stay active while the tool lock is on. Dragging from the toolbar (library items) is handled by `insertElements`.

**Style:** `setStyle(patch)` (applies to selected elements and updates `style`), `styleSummary()` -> `ops.styleSummary` for the selection (or tool defaults when nothing is selected), `copyStyle()`, `pasteStyle()`.

**Actions** (each is a method; all undoable unless noted): `undo() redo() canUndo() canRedo()`, `copy() cut() paste()` (async, uses system clipboard for text/images and an internal JSON clipboard
with a `hld-whiteboard:` marker; paste at pointer or viewport centre, offset on repeats), `copyAsPng()`, `copyAsSvg()`, `duplicate()`, `deleteSelected()`, `selectAll()`, `clearSelection()`,
`group() ungroup()`, `toggleLock()`, `unlockAll()`, `bringToFront() bringForward() sendBackward() sendToBack()`, `align(dir)`, `distribute(axis)`, `flip(axis)`,
`increaseFontSize() decreaseFontSize()`, `setLink(url|null)`, `flowchartAdd(direction)` (Ctrl+Arrow: add a connected copy of the selected shape and arrow in that direction).
View (not undoable): `zoomIn() zoomOut() resetZoom() zoomToFit() zoomToSelection() setZoom(z, center?) pan(dx, dy)`.
Modes: `setMode(patch)`, `toggleGrid() toggleSnap() toggleZen() toggleViewMode() toggleTheme()`.

**Export:** `toSVG(opts) -> string`, `toPNG(opts) -> Promise<Blob>`; `opts = { selectionOnly = false, background = true, dark = (mode.theme === 'dark'), scale = 2, padding = 10 }`. Uses `render.boardToSvgString`.

**Events:** `const off = wb.on(type, fn)`. Types and payloads:
`change {board}` (scene committed), `selection {ids}`, `tool {tool, locked}`, `style {style}`, `view {x,y,zoom}`, `mode {mode}`, `history {canUndo, canRedo}`,
`editing {id|null}`, `toast {message, kind: 'info'|'error'}`, `contextmenu {clientX, clientY, targetId|null}` (right-click; the engine calls `preventDefault`), 
`request {name}` where `name` is one of `'search'` (Ctrl/Cmd+F), `'palette'` (Ctrl/Cmd+/ or Ctrl/Cmd+Shift+P), `'help'` (`?`), `'export'` (Ctrl/Cmd+Shift+E), `'open'` (Ctrl/Cmd+O), `'save'` (Ctrl/Cmd+S), `'library'` (`Q` is lock; library has no key), `'image'` (tool 9 / paste-less image insert).

**Test hooks:** the svg has `data-wb-canvas`; every element group has `data-id` and `class="wb-el"`; the engine sets `containerEl.__wb = wb`.

### Keyboard and mouse (engine implements, `ui-help.js` documents)

Tools: `H` hand, `V`/`1` select, `R`/`2` rectangle, `D`/`3` diamond, `O`/`4` ellipse, `A`/`5` arrow, `L`/`6` line, `P`/`7` draw, `T`/`8` text, `9` image, `E`/`0` eraser, `C` database, `N` sticky note, `F` frame, `K` laser, `B` bucket, `I` eyedropper, `Q` tool lock.
Edit: `Enter` edit label/text, `Esc` finish/deselect, `Delete`/`Backspace` delete, `Ctrl/Cmd+A` select all, `Ctrl/Cmd+D` or Alt-drag duplicate, `Ctrl/Cmd+C/X/V`, `Ctrl/Cmd+Shift+V` paste as plain text,
`Ctrl/Cmd+Z` undo, `Ctrl/Cmd+Shift+Z`/`Ctrl+Y` redo, `Ctrl/Cmd+G` group, `Ctrl/Cmd+Shift+G` ungroup, `Ctrl/Cmd+Shift+L` lock/unlock, `Ctrl/Cmd+[`/`]` send backward/bring forward, `Ctrl/Cmd+Shift+[`/`]` back/front,
`Ctrl/Cmd+Shift+Arrow` align, `Shift+H`/`Shift+V` flip, `Ctrl/Cmd+Alt+C`/`V` copy/paste style, `Ctrl/Cmd+Shift+<`/`>` font size, `Ctrl/Cmd+K` link, `Ctrl/Cmd+Arrow` add connected shape, `Arrow keys` nudge (Shift = 10px).
View: `Ctrl/Cmd+=`/`-`/`0` zoom, `Shift+1` fit, `Shift+2` zoom to selection, `Space`+drag or middle-drag pan, wheel scroll pans, `Ctrl/Cmd`+wheel / pinch zooms, `Ctrl/Cmd+'` grid, `Alt+S` object snapping, `Alt+Z` zen, `Alt+R` view mode, `Alt+Shift+D` theme, `Alt+/` stats.
Mouse: click selects, `Shift`+click toggles, drag on empty space marquee-selects (`Shift` adds), drag moves, `Alt`-drag duplicates, handles resize (`Shift` keeps aspect, `Alt` from centre) and rotate (`Shift` snaps 15 degrees),
double-click edits text/label or creates text on empty canvas, drag from a shape's border with the arrow tool binds to it, multi-click arrow/line adds points (click-click-click = curved with `round` type; double-click or Esc/Enter finishes),
hold `Ctrl/Cmd` while dragging an arrow end to prevent binding, Shift while drawing constrains to 45 degrees / squares / circles.

## 6. UI modules - owners C and D

Every UI module exports `mountXxx(wb, hostEl, options?) -> { destroy() }`, builds its DOM with `document.createElement` (no `innerHTML` of dynamic strings), is keyboard accessible
(buttons have `aria-label`/`title`, popovers close on Esc, focus is restored), supports light and dark app themes via the CSS variables already used in `public/styles.css`
(`--card --ink --muted --line --accent --accent-soft --danger --shadow`), and puts stable test hooks on controls: tools `data-tool="rect"`, style controls
`data-prop="strokeColor" data-value="#e03131"`, menu entries `data-action="export-image"`.

| Module | Export | Contents |
|---|---|---|
| `ui-toolbar.js` (C) | `mountToolbar(wb, host, { library?: boolean })` | tool buttons in Excalidraw order (lock, hand, select, rect, diamond, ellipse, arrow, line, draw, text, image, eraser, + "more tools": database, sticky, frame, laser, bucket, eyedropper), tool-lock toggle, library button (emits `request:library` via `options.onLibrary`), active tool highlight, number-key hints in titles |
| `ui-properties.js` (C) | `mountProperties(wb, host)` | the left panel Excalidraw shows for the active tool or selection: stroke colour (5 swatches + extended palette + hex input), background (swatches + transparent + extended + hex), fill style, stroke width, stroke style, sloppiness, edges, arrow type + start/end arrowheads, font family, font size (S/M/L/XL), text align + vertical align, opacity slider, layers (4 buttons), align (6) and distribute (2) when >= 2/3 selected, actions row (duplicate, delete, group/ungroup, lock, link, flip H/V, copy style, paste style). Shows only properties that apply (`wb.styleSummary().applies`), mixed values shown as no selection |
| `ui-zoom.js` (C) | `mountZoomBar(wb, host)` | `-  100%  +` (click % resets), zoom-to-fit, undo/redo buttons, stats readout toggle (element count, selection size/angle) |
| `ui-help.js` (C) | `openHelp(wb?)`, `mountHelp(wb, host)` | modal listing every shortcut from section 5 grouped Tools / View / Editor, opened by `?` (`request:help`) |
| `ui-menu.js` (D) | `mountMenu(wb, host, hooks)` | hamburger menu: Open (file), Save to file, Export image..., Copy as PNG, Find on canvas, Command palette, Clear canvas (confirm), Reset the library, Theme (light/dark), Canvas background swatches, Grid, Object snapping, Zen mode, View mode, Shortcuts. `hooks = { onOpen(board, meta), onSave(), extra?: [{label, action}] }`; file handling uses `interop.js` |
| `ui-export.js` (D) | `openExportDialog(wb, hooks)` | preview (SVG), options (background, dark mode, scale 1x/2x/3x, selection only, include scene), buttons: PNG download, SVG download, Copy PNG, Copy SVG text, Save .hld.json, Save .excalidraw |
| `ui-context.js` (D) | `mountContextMenu(wb, host)` | right-click menu (canvas: paste, select all, grid, zen, view mode; element: cut/copy/paste, copy as PNG/SVG, copy/paste style, duplicate, delete, group/ungroup, lock, add/edit link, layers, flip, align) driven by `contextmenu` events |
| `ui-palette.js` (D) | `mountPalette(wb, host, { getCommands? })`, `mountSearch(wb, host)` | `Ctrl/Cmd+/` command palette (fuzzy list of every action and tool, with shortcut hints), `Ctrl/Cmd+F` find-on-canvas bar with next/prev/count that selects and scrolls to matches |
| `ui-library.js` (D) | `mountLibrary(wb, host, { storageKey })` | right-side panel: built-in "System design" library (from `library-data.js`) + "My library" (persisted; "Add selection to library"), click or drag an item onto the canvas, remove item, export/import `.hldlib.json`/`.excalidrawlib` |

## 7. Mounting (`public/wb/mount.js`) - owner F

```js
import { mountWhiteboard } from './wb/mount.js';
const app = mountWhiteboard(containerEl, {
  board?,                // initial board (already sanitized) or null
  onChange?(board),      // debounced ~300ms; hosts persist here
  storageKey?,           // if set, mount.js autosaves/restores itself under this localStorage key and handles quota errors with a toast
  library?: true, menu?: true, toolbar?: true, properties?: true, zoom?: true, stats?: true,
  keyboard?: true, theme?, readOnly?,
  menuExtra?: [],        // extra menu entries a host wants (e.g. "Back to home")
});
app.wb;  app.destroy();  app.setBoard(board);  app.getBoard();  app.setKeyboard(bool);  app.on(type, fn)
```
It injects `engine.css`, `chrome.css`, `menus.css` once, lays chrome out around the canvas (toolbar top-centre, properties left, zoom/undo bottom-left, menu top-left, library right), hides all chrome in zen mode,
and forwards `request` events to the right UI module. Hosts must give it a positioned container with a size.

## 8. Interop (`public/wb/interop.js`, `library-data.js`) - owner E

```js
serializeScene(board, { appState? }) -> string                    // our .hld.json: { type: 'hld-whiteboard', version: 2, source, elements/items, appState }
parseScene(text) -> { board, appState } | throws Error(message)   // accepts .hld.json, .excalidraw, and bare JSON boards; always sanitizes
exportExcalidraw(board) -> object                                  // valid .excalidraw v2 JSON (bound text as separate text elements with containerId, arrowheads mapped, groups, frames, images in `files`)
importExcalidraw(json) -> board                                    // rectangle/diamond/ellipse/arrow/line/freedraw/text/image/frame, containerId labels, bindings, roundness, fillStyle, roughness...; unknown types skipped
serializeLibrary(items) / parseLibrary(text)                        // items: [{ id, name, category?, elements: Element[] }]; also reads .excalidrawlib (v1/v2)
BUILTIN_LIBRARY (library-data.js)                                   // ~30 system-design stencils across Clients, Compute, Storage, Messaging, Networking, Observability, Patterns
downloadText(filename, text, mime) / downloadBlob(filename, blob) / pickFile(accept) -> Promise<File|null>   // browser helpers
```
Round-trip tests (`test/interop.test.js`) must show our board survives export -> import with the same labels, bindings and geometry within tolerance.

## 9. Feature matrix (what "parity" means here)

| Excalidraw / HackerRank feature | Where |
|---|---|
| Tools: hand, selection, rectangle, diamond, ellipse, arrow, line, freedraw, text, image, eraser, frame, sticky note, laser, bucket fill, eyedropper, tool lock | engine + toolbar |
| Extra for HLD: database cylinder, system-design library, arrows with ER crow's-foot heads | engine, library |
| Hand-drawn look: sloppiness 0/1/2 (architect/artist/cartoonist), fill styles solid/hachure/cross-hatch/zigzag, stroke width/style, round/sharp edges, opacity, 3 fonts, sizes, alignment | rough + render + properties |
| Arrow types sharp/curved/elbow, start/end arrowheads (12), labelled arrows, binding with follow, multi-point lines | engine + render |
| Text in shapes (bound label), auto-grow text, text align | engine |
| Select: click, shift, marquee, select all, groups (select group, deep select), resize (8 handles, multi), rotate, nudge, duplicate (Ctrl+D, Alt-drag), lock, flip, align, distribute, layers | engine + ops |
| Frames that clip and carry children | engine + ops |
| Copy/cut/paste, paste images and text, copy/paste style, copy as PNG/SVG | engine |
| Infinite canvas: pan (hand, space, wheel), zoom (wheel, pinch, keys), zoom to fit/selection, grid, object snapping with guides, dark theme, canvas background, zen mode, view mode, stats | engine + ui-zoom |
| Undo/redo with grouped gestures | engine |
| Find on canvas, command palette, shortcuts dialog, context menu, hyperlinks on elements | ui-* |
| Export PNG/SVG/clipboard (scale, background, dark, selection), save/open `.hld.json`, import/export `.excalidraw` | ui-export, interop |
| Library of reusable shapes, personal library, import/export | ui-library, interop |
| Local-first autosave | mount.js |

**Not included:** real-time collaboration and live cursors, end-to-end-encrypted shareable links, embeddable web pages, Excalidraw's AI "magic frame", crop tool, laser sharing to other users.

## Deviations (render / geometry / rough)
- `shapePath(el)` also returns `silhouette` (clean closed outline; `fill` is that path or `null` when transparent). `arrowheadPath` also returns `inset` (how far the line body stops short of the tip for closed heads; render.js trims with it).
- `elementToSvg` additionally accepts `background` (colour behind arrow-label halos, default white). Frame clip ids are `wb-fclip-<frameId>`, hatch clip ids `wb-clip-<id>`.
- Extra exports: geometry `innerBox, labelLayout, linearLabelLayout, polylineMidpoint, frameNameBox`; rough `makeRng, curveSamples, cornerRadius`; render `esc`. `transformHandles(..., { linear: true })` returns only the four corner handles.
