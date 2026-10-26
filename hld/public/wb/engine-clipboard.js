// Clipboard, paste, image insertion and drag-and-drop for the Whiteboard (mixed into its prototype).
import { LIMITS, createElement, findItem, newId } from '../../shared/board.js';
import { textSize } from './geometry.js';
import * as ops from './ops.js';
import { clipFromHtml, clipHtml, clipPlain, creationProps, decodeClip, encodeClip, fitSize } from './engine-util.js';
import { prepareImage } from './engine-image.js';

export function installClipboard(P) {
  P._clipElements = function _clipElements() {
    const ids = ops.expandSelection(this._board, this._selIds());
    return ids.map((id) => findItem(this._board, id)).filter(Boolean).map((e) => JSON.parse(JSON.stringify(e)));
  };

  // Copies the selection to the internal clipboard and (best effort) the system clipboard.
  P.copy = function copy() {
    const els = this._clipElements();
    if (!els.length) return false;
    this._clip = els;
    this._pasteN = 0;
    const text = encodeClip(els);
    this._clipText = text;
    this._clipPriority = true; // until the system clipboard is known to hold our payload
    const plain = clipPlain(els);
    try {
      const cb = navigator.clipboard;
      let p = null;
      if (cb && cb.write && typeof ClipboardItem !== 'undefined') {
        // Readable text for other apps, board data hidden in the HTML flavour for this whiteboard.
        p = cb.write([new ClipboardItem({ 'text/plain': new Blob([plain], { type: 'text/plain' }), 'text/html': new Blob([clipHtml(els)], { type: 'text/html' }) })]);
      } else if (cb && cb.writeText) p = cb.writeText(plain);
      if (p && p.then) p.then(() => { this._clipPriority = false; }, () => { /* denied: the internal clipboard still works */ });
    } catch { /* ignore */ }
    return true;
  };

  P.cut = function cut() {
    if (!this._canEdit()) return false;
    if (!this.copy()) return false;
    this.deleteSelected();
    return true;
  };

  P._pasteTarget = function _pasteTarget() {
    if (this._pointerIn && this._pointer && Number.isFinite(this._pointer.x)) return { x: this._pointer.x, y: this._pointer.y };
    return this._viewportCenter();
  };

  // Programmatic paste (context menu / command palette): reads the system clipboard when allowed, else the internal one.
  P.paste = async function paste({ plain = false } = {}) {
    if (!this._canEdit()) return;
    try {
      if (navigator.clipboard && navigator.clipboard.read) {
        try {
          const items = await navigator.clipboard.read();
          for (const it of items) {
            if (!plain && it.types.includes('text/html')) {
              const els = clipFromHtml(await (await it.getType('text/html')).text());
              if (els) { this._pasteElements(els, 'html:' + els.length + ':' + (els[0] && els[0].id), this._pasteTarget()); return; }
            }
            const imgType = it.types.find((t) => t.startsWith('image/'));
            if (imgType && !plain) {
              const blob = await it.getType(imgType);
              await this.addImageFile(new File([blob], 'pasted', { type: imgType }), this._pasteTarget());
              return;
            }
          }
        } catch { /* denied or unsupported: fall through to text */ }
      }
      let text = '';
      try { text = navigator.clipboard && navigator.clipboard.readText ? await navigator.clipboard.readText() : ''; } catch { text = ''; }
      await this._pasteText(text, { plain });
    } catch (err) { this._fail(err); }
  };

  P._pasteText = async function _pasteText(text, { plain = false } = {}) {
    const data = decodeClip(text);
    const at = this._pasteTarget();
    // The system clipboard wins when it holds something we understand; otherwise fall back to what we copied ourselves.
    if (!data || (data.kind === 'text' && (!text.trim() || this._clipPriority))) {
      if (this._clip && this._clip.length) { this._pasteElements(this._clip, '__internal__', at); return; }
      return;
    }
    if (plain && data.kind !== 'text') { this._pasteAsText(data.kind === 'hld' ? this._plainOf(data.elements) : text, at); return; }
    if (data.kind === 'hld') { this._pasteElements(data.elements, text, at); return; }
    if (data.kind === 'excalidraw') {
      try {
        const mod = await import('./interop.js').catch(() => null);
        if (mod && mod.importExcalidraw) {
          const board = mod.importExcalidraw({ type: 'excalidraw', elements: data.json.elements, files: data.json.files || {} });
          if (board && board.items.length) { this._pasteElements(board.items, text, at); return; }
        }
      } catch { /* not importable: paste as text */ }
      return;
    }
    this._pasteAsText(data.text, at);
  };

  P._plainOf = function _plainOf(elements) {
    return elements.map((e) => e.text || e.name || '').filter(Boolean).join('\n');
  };

  P._pasteElements = function _pasteElements(elements, key, at) {
    if (this._lastPasteKey === key) this._pasteN = (this._pasteN || 0) + 1; else { this._lastPasteKey = key; this._pasteN = 0; }
    const off = (this._pasteN || 0) * 20;
    this.insertElements(elements, { at: { x: at.x + off, y: at.y + off } });
  };

  P._pasteAsText = function _pasteAsText(text, at) {
    const t = String(text || '').slice(0, LIMITS.text);
    if (!t.trim()) return;
    this._finishEditor();
    const b = this._board;
    const props = creationProps('text', this._style, { textAlignSet: this._textAlignSet });
    const s = textSize(t, { fontFamily: props.fontFamily, fontSize: props.fontSize });
    const el = createElement('text', { id: newId(b), ...props, text: t, w: Math.max(s.w, 4), h: s.h });
    el.x = at.x - el.w / 2; el.y = at.y - el.h / 2;
    b.items.push(el);
    ops.assignFrames(b, [el.id]);
    this._setSelection([el.id]);
    this._commit();
  };

  // Browser paste event (Ctrl/Cmd+V, context menu of the browser).
  P._onPaste = function _onPaste(e) {
    if (!this._keyboard || !this._canEdit() || this._editor) return;
    if (this._midAt && Date.now() - this._midAt < 1000) return; // X11 middle-click paste
    const t = e.target;
    if (t && t.closest && t.closest('input,textarea,select,[contenteditable=""],[contenteditable="true"],dialog,[role="dialog"]')) return;
    const cd = e.clipboardData;
    const plain = !!this._plainNext;
    this._plainNext = false;
    e.preventDefault();
    const files = cd ? [...(cd.files || [])].filter((f) => f.type && f.type.startsWith('image/')) : [];
    if (files.length && !plain) {
      const at = this._pasteTarget();
      files.slice(0, 5).forEach((f, i) => { this.addImageFile(f, { x: at.x + i * 24, y: at.y + i * 24 }).catch((err) => this._toast(err.message, 'error')); });
      return;
    }
    const fromHtml = !plain && cd ? clipFromHtml(cd.getData('text/html')) : null;
    if (fromHtml) { this._pasteElements(fromHtml, 'html:' + fromHtml.length + ':' + (fromHtml[0] && fromHtml[0].id), this._pasteTarget()); return; }
    const text = cd ? cd.getData('text/plain') : '';
    this._pasteText(text, { plain }).catch((err) => this._fail(err));
  };

  // ---------- images ----------
  P.addImageFile = async function addImageFile(file, at) {
    if (!this._canEdit()) throw new Error('The board is read-only');
    let prepared;
    try { prepared = await prepareImage(file); } catch (err) { this._toast(err.message, 'error'); throw err; }
    const { w, h } = fitSize(prepared.w, prepared.h, 480, 360);
    const b = this._board;
    const target = at && Number.isFinite(at.x) && Number.isFinite(at.y) ? at : this._viewportCenter();
    const el = createElement('image', { id: newId(b), ...creationProps('image', this._style), dataUrl: prepared.dataUrl, x: target.x - w / 2, y: target.y - h / 2, w, h });
    b.items.push(el);
    ops.assignFrames(b, [el.id]);
    this._setSelection([el.id]);
    this._commit();
    return el.id;
  };

  P._onDragOver = function _onDragOver(e) {
    if (!this._canEdit()) return;
    const types = e.dataTransfer && e.dataTransfer.types ? [...e.dataTransfer.types] : [];
    if (types.includes('Files')) { e.preventDefault(); e.dataTransfer.dropEffect = 'copy'; }
  };

  P._onDrop = function _onDrop(e) {
    if (!this._canEdit()) return;
    const files = e.dataTransfer && e.dataTransfer.files ? [...e.dataTransfer.files].filter((f) => f.type && f.type.startsWith('image/')) : [];
    if (!files.length) return;
    e.preventDefault();
    const at = this.clientToWorld(e.clientX, e.clientY);
    files.slice(0, 10).forEach((f, i) => {
      this.addImageFile(f, { x: at.x + i * 24, y: at.y + i * 24 }).catch(() => { /* toast already emitted */ });
    });
  };
}

