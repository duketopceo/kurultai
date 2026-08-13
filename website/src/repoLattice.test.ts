import assert from 'node:assert/strict';
import { latticeOf, isJunkLatticeName, isCodeSource, codeLatticeOf } from './repoLattice.ts';

assert.equal(latticeOf({ source: 'notes', source_id: 'learnings/foo.md#c0' }), 'notes');
assert.equal(codeLatticeOf({ source: 'notes', source_id: 'learnings/foo.md#c0' }), null);
assert.equal(codeLatticeOf({ source: 'pond', source_id: '019e37c3-e879-7611-8c68-72a3f8963671:001423' }), null);
assert.equal(codeLatticeOf({ source: 'dayflow', source_id: '7689' }), null);
assert.equal(
  codeLatticeOf({ source: 'code', source_id: 'duketopceo/kurultai/src/http/mod.rs' }),
  'duketopceo/kurultai',
);
assert.equal(
  codeLatticeOf({ source: 'github', source_id: 'Bartlett-Roofs/help-dashboard/app/page.tsx' }),
  'Bartlett-Roofs/help-dashboard',
);
assert.equal(codeLatticeOf({ source: 'code', source_id: 'src/http/mod.rs' }), 'code');
assert.equal(isCodeSource('code'), true);
assert.equal(isCodeSource('notes'), false);
assert.equal(isJunkLatticeName('7689'), true);
assert.equal(isJunkLatticeName('README.md#c0'), true);

console.log('repoLattice tests ok');
