// Constants shared by the whiteboard engine, its UI and the server-side board validation. No logic here.

// Tool ids, keyboard shortcuts (letter and number keys, as in Excalidraw) and labels.
export const TOOLS = [
  { id: 'hand', key: 'h', num: null, label: 'Hand (panning tool)' },
  { id: 'select', key: 'v', num: '1', label: 'Selection' },
  { id: 'rect', key: 'r', num: '2', label: 'Rectangle' },
  { id: 'diamond', key: 'd', num: '3', label: 'Diamond' },
  { id: 'ellipse', key: 'o', num: '4', label: 'Ellipse' },
  { id: 'arrow', key: 'a', num: '5', label: 'Arrow' },
  { id: 'line', key: 'l', num: '6', label: 'Line' },
  { id: 'freedraw', key: 'p', num: '7', label: 'Draw' },
  { id: 'text', key: 't', num: '8', label: 'Text' },
  { id: 'image', key: null, num: '9', label: 'Insert image' },
  { id: 'eraser', key: 'e', num: '0', label: 'Eraser' },
  { id: 'database', key: 'c', num: null, label: 'Database (cylinder)' },
  { id: 'sticky', key: 'n', num: null, label: 'Sticky note' },
  { id: 'frame', key: 'f', num: null, label: 'Frame' },
  { id: 'laser', key: 'k', num: null, label: 'Laser pointer' },
  { id: 'bucket', key: 'b', num: null, label: 'Bucket fill' },
  { id: 'eyedropper', key: 'i', num: null, label: 'Eyedropper (pick a colour)' },
];

export const ELEMENT_TYPES = ['rect', 'diamond', 'ellipse', 'database', 'sticky', 'text', 'image', 'frame', 'freedraw', 'line', 'arrow'];
export const CONTAINER_TYPES = ['rect', 'diamond', 'ellipse', 'database', 'sticky']; // shapes that carry a bound text label
export const BINDABLE_TYPES = ['rect', 'diamond', 'ellipse', 'database', 'sticky', 'text', 'image']; // arrows can attach to these
export const LINEAR_TYPES = ['line', 'arrow'];
export const POINT_TYPES = ['line', 'arrow', 'freedraw'];
export const NO_ROTATE_TYPES = ['line', 'arrow', 'freedraw', 'frame'];

export const STROKE_COLORS = ['#1e1e1e', '#e03131', '#2f9e44', '#1971c2', '#f08c00'];
export const BACKGROUND_COLORS = ['transparent', '#ffc9c9', '#b2f2bb', '#a5d8ff', '#ffec99'];
export const STICKY_COLORS = ['#ffec99', '#ffc9c9', '#b2f2bb', '#a5d8ff', '#e5dbff', '#ffd8a8'];
export const CANVAS_BACKGROUNDS = ['#ffffff', '#f8f9fa', '#f5faff', '#fffce8', '#fdf8f6'];
// Extended palette (5 shades per hue) for a "more colours" popover.
export const EXTENDED_COLORS = {
  gray: ['#f8f9fa', '#e9ecef', '#ced4da', '#868e96', '#343a40'],
  red: ['#ffc9c9', '#ffa8a8', '#ff6b6b', '#fa5252', '#e03131'],
  pink: ['#fcc2d7', '#faa2c1', '#f06595', '#e64980', '#c2255c'],
  grape: ['#eebefa', '#e599f7', '#cc5de8', '#be4bdb', '#9c36b5'],
  violet: ['#d0bfff', '#b197fc', '#845ef7', '#7950f2', '#6741d9'],
  blue: ['#a5d8ff', '#74c0fc', '#339af0', '#228be6', '#1971c2'],
  cyan: ['#99e9f2', '#66d9e8', '#22b8cf', '#15aabf', '#0c8599'],
  teal: ['#96f2d7', '#63e6be', '#20c997', '#12b886', '#099268'],
  green: ['#b2f2bb', '#8ce99a', '#51cf66', '#40c057', '#2f9e44'],
  yellow: ['#ffec99', '#ffe066', '#fcc419', '#fab005', '#f08c00'],
  orange: ['#ffd8a8', '#ffc078', '#ff922b', '#fd7e14', '#e8590c'],
};

export const FILL_STYLES = ['solid', 'hachure', 'cross-hatch', 'zigzag'];
export const STROKE_STYLES = ['solid', 'dashed', 'dotted'];
export const STROKE_WIDTHS = { thin: 1, bold: 2, extra: 4 };
export const ROUGHNESS = { architect: 0, artist: 1, cartoonist: 2 };
export const ROUNDNESS = ['sharp', 'round'];
export const ARROW_TYPES = ['sharp', 'round', 'elbow'];
export const TEXT_ALIGNS = ['left', 'center', 'right'];
export const VERTICAL_ALIGNS = ['top', 'middle', 'bottom'];

export const FONT_FAMILIES = {
  1: { id: 1, name: 'Hand-drawn', css: '"Segoe Print","Bradley Hand","Chalkboard SE","Comic Sans MS","Marker Felt",cursive' },
  2: { id: 2, name: 'Normal', css: 'ui-sans-serif,system-ui,-apple-system,"Segoe UI",Roboto,"Helvetica Neue",Arial,sans-serif' },
  3: { id: 3, name: 'Code', css: 'ui-monospace,SFMono-Regular,Menlo,Consolas,monospace' },
};
export const FONT_SIZES = { S: 16, M: 20, L: 28, XL: 36 };

// Arrowhead ids. Includes the ER "crow's foot" family, handy for database design.
export const ARROWHEADS = [
  { id: null, label: 'None' },
  { id: 'arrow', label: 'Arrow' },
  { id: 'triangle', label: 'Triangle' },
  { id: 'triangle_outline', label: 'Triangle (outline)' },
  { id: 'circle', label: 'Circle' },
  { id: 'circle_outline', label: 'Circle (outline)' },
  { id: 'diamond', label: 'Diamond' },
  { id: 'diamond_outline', label: 'Diamond (outline)' },
  { id: 'bar', label: 'Bar' },
  { id: 'crowfoot_one', label: 'One (ER)' },
  { id: 'crowfoot_many', label: 'Many (ER)' },
  { id: 'crowfoot_one_or_many', label: 'One or many (ER)' },
];
export const ARROWHEAD_IDS = ARROWHEADS.map((a) => a.id).filter(Boolean);

// What a newly drawn element looks like before the user changes anything ("current item style").
export const DEFAULT_STYLE = {
  strokeColor: '#1e1e1e',
  backgroundColor: 'transparent',
  fillStyle: 'solid',
  strokeWidth: 2,
  strokeStyle: 'solid',
  roughness: 1,
  opacity: 100,
  roundness: 'round',
  fontFamily: 2,
  fontSize: 20,
  textAlign: 'center',
  verticalAlign: 'middle',
  arrowType: 'sharp',
  startArrowhead: null,
  endArrowhead: 'arrow',
};

// [w, h] used when a shape is created by a single click.
export const DEFAULT_SIZES = {
  rect: [160, 90], diamond: [140, 100], ellipse: [140, 90], database: [110, 110], sticky: [160, 160],
  text: [40, 28], image: [240, 160], frame: [360, 240],
};
export const DEFAULT_LABELS = { rect: 'Service', diamond: 'Load Balancer', ellipse: 'Client', database: 'Database', sticky: '' };
export const MIN_SIZE = { w: 12, h: 12 };

export const GRID_SIZE = 20;
export const ZOOM = { min: 0.1, max: 30, step: 1.1 };

export const LIMITS = {
  items: 1500, label: 400, text: 4000, frameName: 80, coord: 1_000_000, size: 100_000, points: 4000, link: 500,
  dataUrl: 3_000_000, groupDepth: 8, fontSize: [8, 200], opacity: [0, 100],
};
export const DATA_URL_RE = /^data:image\/(png|jpeg|gif|webp|svg\+xml);base64,[A-Za-z0-9+/=]+$/;
