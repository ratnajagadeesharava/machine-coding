import test from 'node:test';
import assert from 'node:assert/strict';
import { buildExpectedDiagram } from '../server/diagram.js';
import { cleanEvaluation } from '../server/ai.js';

const nodes = [
  { id: 'u', label: 'User', kind: 'client' }, { id: 'lb', label: 'Load balancer', kind: 'balancer' },
  { id: 'a', label: 'API', kind: 'service' }, { id: 'd', label: 'DB', kind: 'database' },
];
const edges = [{ from: 'u', to: 'lb', label: 'HTTPS' }, { from: 'lb', to: 'a', label: '' }, { from: 'a', to: 'd', label: 'SQL' }];

test('expected diagram lays boxes out left to right and binds arrows to them', () => {
  const { board } = buildExpectedDiagram({ nodes, edges, caption: 'Flow.' });
  const shapes = board.items.filter((i) => i.type !== 'arrow');
  const arrows = board.items.filter((i) => i.type === 'arrow');
  assert.equal(shapes.length, 4);
  assert.equal(arrows.length, 3);
  assert.ok(shapes[0].x < shapes[1].x && shapes[1].x < shapes[2].x && shapes[2].x < shapes[3].x);
  assert.ok(arrows.every((a) => a.from != null && a.to != null));
});

test('expected diagram drops bad input instead of failing', () => {
  assert.equal(buildExpectedDiagram(null), null);
  assert.equal(buildExpectedDiagram({ nodes: [nodes[0]], edges: [] }), null);
  const out = buildExpectedDiagram({
    nodes: [...nodes, { id: 'u', label: 'Dup' }, { id: 'x', label: '', kind: 'service' }],
    edges: [...edges, { from: 'a', to: 'missing' }, { from: 'a', to: 'a' }, { from: 'u', to: 'lb' }],
  });
  assert.equal(out.board.items.filter((i) => i.type !== 'arrow').length, 4);
  assert.equal(out.board.items.filter((i) => i.type === 'arrow').length, 3);
});

test('a cyclic or disconnected description still gets a place for every box', () => {
  const out = buildExpectedDiagram({ nodes: [{ id: 'a', label: 'A' }, { id: 'b', label: 'B' }, { id: 'c', label: 'C' }], edges: [{ from: 'a', to: 'b' }, { from: 'b', to: 'a' }] });
  assert.equal(out.board.items.filter((i) => i.type !== 'arrow').length, 3);
});

test('cleanEvaluation keeps the review parts and tolerates their absence', () => {
  const full = cleanEvaluation({
    verdict: 'Good', summary: 's', areas: [], strengths: [], gaps: [], next_steps: [],
    expectation: [{ what: 'Storage', status: 'weird', note: 'n' }, { what: '', status: 'shown', note: '' }],
    simulation: [{ scenario: 'Normal request', steps: [{ text: 'It arrives.', works: true }, { text: 'It fails.', works: 'yes' }], takeaway: 't' }, { scenario: '', steps: [] }],
    diagram: { nodes, edges, caption: 'c' },
  });
  assert.deepEqual(full.expectation, [{ what: 'Storage', status: 'missing', note: 'n' }]);
  assert.equal(full.simulation.length, 1);
  assert.deepEqual(full.simulation[0].steps.map((s) => s.works), [true, false]);
  assert.ok(full.diagram.board.items.length >= 4);
  const bare = cleanEvaluation({ verdict: 'Good', summary: '', areas: [] });
  assert.deepEqual([bare.expectation, bare.simulation, bare.diagram], [[], [], null]);
});
