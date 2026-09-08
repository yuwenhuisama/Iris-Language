import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';

const [runner, ...extra] = process.argv.slice(2);
assert.ok(runner && extra.length === 0, 'usage: node tests/pure.mjs /absolute/path/to/iris');
const source = ['model', 'application', 'http'].map(name =>
  readFileSync(new URL(`../src/${name}.ir`, import.meta.url), 'utf8')).join('\n');
const tests = readFileSync(new URL('test_http.ir', import.meta.url), 'utf8');

for (const [engine, flags] of [['reference', []], ['vm', ['--vm']]]) {
  const result = spawnSync(runner, [...flags, '-e', `${source}\n${tests}`], { encoding: 'utf8', timeout: 60000 });
  assert.equal(result.status, 0, `${engine}: ${result.error ?? ''}\n${result.stdout}\n${result.stderr}`);
  const passed = result.stdout.match(/^PASS /gm)?.length ?? 0;
  assert.ok(passed > 70, 'all protocol and language fixtures must execute');
  assert.ok(result.stdout.includes(`Advanced HTTP tests passed: ${passed}`));
  console.log(`${engine}: ${passed} pure fixtures PASS`);

  // Change only the annotation argument in trusted source to prove the wire marker comes from real metadata, not another hardcoded constant.
  const changed = source.replace('@PipelineTag(:advanced_v1)', '@PipelineTag(:experiment_v2)');
  assert.notEqual(changed, source);
  const probe = spawnSync(runner, [...flags, '-e', `${changed}\nprint(Http.response("GET /health HTTP/1.1\\r\\nHost: localhost\\r\\n\\r\\n"))`], { encoding: 'utf8', timeout: 10000 });
  assert.equal(probe.status, 0, probe.stderr);
  assert.ok(probe.stdout.includes('X-Iris-Pipeline: experiment_v2\r\n'));
  assert.ok(!probe.stdout.includes('X-Iris-Pipeline: advanced_v1'));
  console.log(`${engine}: decorator argument changes wire header PASS`);

  // Preserve the observed engine difference rather than imply the VM implements the missing Contract argument check.
  const invalidHandler = spawnSync(runner, [...flags, '-e', `${source}\nprint(try { Pipeline.run("not a handler", Request.new("/")) } catch error { error.to_string() })`], { encoding: 'utf8', timeout: 10000 });
  assert.equal(invalidHandler.status, 0, invalidHandler.stderr);
  assert.equal(invalidHandler.stdout.trim(), engine === 'vm' ? 'MessageNotFound' : 'TypeContractError');
  console.log(`${engine}: invalid Contract receiver diagnostic = ${invalidHandler.stdout.trim()} (documented engine difference)`);
}
