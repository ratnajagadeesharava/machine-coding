// Quick-connect handles (mixed into the Whiteboard prototype): four "+" buttons around a selected shape. Clicking one adds a connected next shape in that
// direction, joined by an arrow, and opens its label so you can type the name straight away (the draw.io / Lucidchart way of building a flow).
import { isContainer, sideAnchor } from '../../shared/board.js';

const SIDES = [
  { side: 'right', dir: 'ArrowRight', label: 'Add a connected shape to the right' },
  { side: 'bottom', dir: 'ArrowDown', label: 'Add a connected shape below' },
  { side: 'left', dir: 'ArrowLeft', label: 'Add a connected shape to the left' },
  { side: 'top', dir: 'ArrowUp', label: 'Add a connected shape above' },
];
const OUT = { right: [1, 0], bottom: [0, 1], left: [-1, 0], top: [0, -1] };

export function installConnect(P) {
  P._removeConnectors = function _removeConnectors() {
    if (this._connectors) { this._connectors.remove(); this._connectors = null; }
  };

  // Called whenever the overlay is redrawn (selection, zoom, pan, edits), so the handles always sit on the shape's edges.
  P._placeConnectors = function _placeConnectors() {
    const el = this._selection.size === 1 ? this._selEls()[0] : null;
    const show = !!el && !el.locked && this._canEdit() && !this._editor && !this._g && this._tool === 'select' && isContainer(el) && el.type !== 'sticky';
    if (!show) { this._removeConnectors(); return; }
    if (!this._connectors) {
      const box = document.createElement('div');
      box.className = 'wb-connectors';
      for (const s of SIDES) {
        const b = document.createElement('button');
        b.type = 'button'; b.className = `wb-conn wb-conn-${s.side}`; b.dataset.side = s.side;
        b.title = `${s.label} (Ctrl+${s.dir.replace('Arrow', '')} arrow)`; b.setAttribute('aria-label', s.label);
        b.textContent = '+';
        b.addEventListener('pointerdown', (e) => e.stopPropagation());
        b.addEventListener('mousedown', (e) => e.preventDefault());
        b.addEventListener('click', (e) => { e.stopPropagation(); this._connectFrom(s.dir); });
        box.appendChild(b);
      }
      this.root.appendChild(box);
      this._connectors = box;
    }
    const z = this._view.zoom;
    for (const b of this._connectors.children) {
      const side = b.dataset.side;
      const p = sideAnchor(el, side);
      const [vx, vy] = OUT[side];
      b.style.left = `${this._view.x + p.x * z + vx * 20}px`;
      b.style.top = `${this._view.y + p.y * z + vy * 20}px`;
    }
  };

  P._connectFrom = function _connectFrom(direction) {
    const id = this.flowchartAdd(direction);
    if (id != null) this._beginEdit(id, { isNew: false });
  };
}
