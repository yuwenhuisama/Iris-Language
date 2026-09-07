import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { runInNewContext } from 'node:vm';
import test from 'node:test';
import { checkLocalLinks, checkTutorial, parseTutorial, runExample } from './check-tutorial.mjs';

const checker = fileURLToPath(new URL('./check-tutorial.mjs', import.meta.url));
const hello = { id: '01-hello', mode: 'vm', stdout: 'Hello, Iris!\n' };

function block(annotation = hello, source = 'print("Hello, Iris!")\n') {
  return `<!-- iris-example: ${JSON.stringify(annotation)} -->\n\`\`\`iris\n${source}\`\`\`\n`;
}

function fixture(context, english = block(), chinese = english) {
  const root = mkdtempSync(join(tmpdir(), 'iris-checker-test-'));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  for (const directory of ['tools', 'tutorial/en', 'tutorial/zh-cn', 'scratch']) mkdirSync(join(root, directory), { recursive: true });
  const writePage = (language, body, stem = '01-start') => writeFileSync(join(root, 'tutorial', language, `${stem}.md`), `# Start\n\n${body}`);
  writePage('en', english);
  writePage('zh-cn', chinese);
  writeFileSync(join(root, 'tools/tutorial-routes.json'), JSON.stringify({
    version: 1,
    languages: ['en', 'zh-cn'],
    routes: [{ stem: '01-start', headings: { en: ['# Start'], 'zh-cn': ['# Start'] } }],
  }));
  const script = join(root, 'fake-iris.cjs');
  writeFileSync(script, `const fs = require('node:fs');
const args = process.argv.slice(2);
const source = fs.readFileSync(args.at(-1), 'utf8');
if (source === 'hang\\n') { fs.appendFileSync('hang-pids', process.pid + '\\n'); setInterval(() => {}, 1000); }
else if (source === 'fail\\n') { process.stderr.write('fixture diagnostic: intentional failure\\n'); process.exitCode = 3; }
else if (source === 'vm-only\\n') { process.stdout.write(args[0] === '--vm' ? 'vm\\n' : 'reference\\n'); }
else {
  const match = /^print\\((".*")\\)\\n$/.exec(source);
  if (!match) { process.stderr.write('unexpected source ' + JSON.stringify(source)); process.exitCode = 4; }
  else process.stdout.write(JSON.parse(match[1]) + '\\n');
}
`);
  const binary = join(root, 'fake-iris');
  const shellQuote = value => `'${value.replaceAll("'", "'\\''")}'`;
  writeFileSync(binary, `#!/bin/sh\nexec ${shellQuote(process.execPath)} ${shellQuote(script)} "$@"\n`);
  chmodSync(binary, 0o755);
  return { root, binary, tempRoot: join(root, 'scratch'), writePage };
}

test('executes actual fenced source and selects each engine, with cleanup', context => {
  const body = block({ ...hello, stdout: 'vm\n' }, 'vm-only\n') + block({ id: '01-reference', mode: 'reference', stdout: 'reference\n' }, 'vm-only\n');
  const options = fixture(context, body);
  const result = checkTutorial(options);
  assert.deepEqual(result.errors, []);
  assert.equal(result.runnable, 4);
  assert.deepEqual(result.modeCounts, { vm: 2, reference: 2, 'expected-error': 0, 'spec-only': 0 });
  assert.equal(result.fences, 4);
  assert.deepEqual(readdirSync(options.tempRoot), []);
});

test('wrong output fails exact comparison, including trailing newline', context => {
  const options = fixture(context, block({ ...hello, stdout: 'Hello, Iris!' }));
  const result = checkTutorial(options);
  assert.equal(result.errors.length, 2);
  assert.ok(result.errors.every(error => error.includes('stdout expected')));
  assert.deepEqual(readdirSync(options.tempRoot), []);
});

test('missing translation example is rejected', context => {
  const result = checkTutorial(fixture(context, block(), 'No example yet.\n'));
  assert.ok(result.errors.some(error => error.includes('01-hello: missing translation example in zh-cn')));
});

test('missing translation page and baseline route are rejected', context => {
  const options = fixture(context);
  rmSync(join(options.root, 'tutorial/zh-cn/01-start.md'));
  const result = checkTutorial(options);
  assert.ok(result.errors.some(error => error.includes('missing baseline route')));
  assert.ok(result.errors.some(error => error.includes('missing translation page')));
});

test('unclassified, detached, malformed and unterminated fences fail', () => {
  const bare = parseTutorial('```iris\nprint(1)\n```\n');
  assert.match(bare.errors.join('\n'), /unclassified iris fence/);
  const detached = parseTutorial(block().replace('-->\n', '-->\n\n'));
  assert.match(detached.errors.join('\n'), /unclassified iris fence/);
  assert.match(detached.errors.join('\n'), /annotation must immediately precede/);
  assert.match(parseTutorial('<!-- iris-example: {bad} -->\n```iris\n1\n```\n').errors.join('\n'), /invalid annotation JSON/);
  assert.match(parseTutorial(block().slice(0, -4)).errors.join('\n'), /unclosed code fence/);
  assert.match(parseTutorial(block().replace('```iris', '~~~iris').replace(/```\n$/, '~~~\n')).errors.join('\n'), /exact opening fence/);
});

test('expected-error validates engine, exit and stderr without weakening stdout', context => {
  const annotation = { id: '01-failure', mode: 'expected-error', engine: 'reference', exit: 3, stdout: '', stderr: 'intentional failure' };
  const options = fixture(context, block(annotation, 'fail\n'));
  const result = checkTutorial(options);
  assert.deepEqual(result.errors, []);
  assert.deepEqual(result.modeCounts, { vm: 0, reference: 0, 'expected-error': 2, 'spec-only': 0 });
  options.writePage('en', block({ ...annotation, stderr: 'other error', exit: 2, stdout: 'noise' }, 'fail\n'));
  const errors = checkTutorial(options).errors.join('\n');
  assert.match(errors, /exit expected 2, got 3/);
  assert.match(errors, /stdout expected/);
  assert.match(errors, /stderr must contain/);
  for (const field of ['engine', 'exit', 'stderr']) {
    const incomplete = { ...annotation };
    delete incomplete[field];
    assert.ok(parseTutorial(block(incomplete, 'fail\n')).errors.length > 0, field);
  }
});

test('invalid annotation fields are rejected instead of silently ignored', () => {
  for (const annotation of [
    { ...hello, mode: 'unknown' },
    { ...hello, stdout: 1 },
    { ...hello, exit: 1.5 },
    { ...hello, exit: 1 },
    { ...hello, stduot: 'typo' },
    { ...hello, id: '' },
  ]) assert.ok(parseTutorial(block(annotation)).errors.length > 0, JSON.stringify(annotation));
});

test('spec-only requires reason and visible adjacent prose, but permits translated reasons', context => {
  const annotation = { id: '01-future', mode: 'spec-only', reason: 'Not implemented by either engine.' };
  assert.match(parseTutorial(block(annotation)).errors.join('\n'), /visible explanatory prose/);
  assert.match(parseTutorial(`<!-- Hidden explanation. -->\n\n${block(annotation)}`).errors.join('\n'), /visible explanatory prose/);
  assert.match(parseTutorial(block({ ...annotation, reason: '' })).errors.join('\n'), /nonempty reason/);
  const options = fixture(context, `This form is specified but not implemented.\n\n${block(annotation)}`, `${block({ ...annotation, reason: 'Different language explanation.' })}\nA translated visible explanation.\n`);
  rmSync(options.binary);
  const result = checkTutorial(options);
  assert.deepEqual(result.errors, []);
  assert.equal(result.specOnly, 2);
  assert.deepEqual(result.modeCounts, { vm: 0, reference: 0, 'expected-error': 0, 'spec-only': 2 });
  assert.equal(result.runnable, 0);
});

test('duplicate IDs are rejected across pages within one language', context => {
  const options = fixture(context);
  options.writePage('en', block(), '02-next');
  options.writePage('zh-cn', block(), '02-next');
  assert.match(checkTutorial(options).errors.join('\n'), /duplicate id 01-hello within en/);
});

test('bilingual source and expectation drift are rejected even when both programs pass', context => {
  const options = fixture(context, block(), block({ ...hello, stdout: 'Different\n' }, 'print("Different")\n'));
  const errors = checkTutorial(options).errors.join('\n');
  assert.match(errors, /bilingual source mismatch/);
  assert.match(errors, /bilingual expectations mismatch/);
  options.writePage('zh-cn', block({ ...hello, exit: 0 }));
  assert.deepEqual(checkTutorial(options).errors, []);
});

test('baseline headings are an exact prefix; new headings may only append', context => {
  const options = fixture(context);
  const baselinePath = join(options.root, 'tools/tutorial-routes.json');
  const baseline = JSON.parse(readFileSync(baselinePath, 'utf8'));
  baseline.routes[0].headings.en.push('## First', '## Second');
  writeFileSync(baselinePath, JSON.stringify(baseline));
  options.writePage('en', `## First\n\n## Second\n\n## Extra\n\n${block()}`);
  assert.deepEqual(checkTutorial(options).errors, []);
  for (const headings of [
    '## Extra\n\n## First\n\n## Second',
    '## First\n\n## Extra\n\n## Second',
    '## Second\n\n## First',
    '## First',
  ]) {
    options.writePage('en', `${headings}\n\n${block()}`);
    assert.match(checkTutorial(options).errors.join('\n'), /baseline heading prefix mismatch/);
  }
});

test('optional root README executes examples and checks links outside bilingual inventory', context => {
  const options = fixture(context);
  const readme = join(options.root, 'README.md');
  writeFileSync(readme, `# Project\n\n${block({ ...hello, id: 'readme-hello' })}\n[tools](tools/)\n`);
  const result = checkTutorial(options);
  assert.deepEqual(result.errors, []);
  assert.equal(result.pages, 3);
  assert.equal(result.fences, 3);
  assert.deepEqual(result.modeCounts, { vm: 3, reference: 0, 'expected-error': 0, 'spec-only': 0 });
  writeFileSync(readme, `${block({ ...hello, id: 'readme-hello', stdout: 'wrong' })}\n[missing](missing.md)\n`);
  const errors = checkTutorial(options).errors;
  assert.equal(errors.length, 2);
  assert.ok(errors.every(error => error.includes('README.md')));
  assert.match(errors.join('\n'), /stdout expected/);
  assert.match(errors.join('\n'), /broken local link/);
  writeFileSync(readme, '```iris\nprint(1)\n```\n');
  assert.match(checkTutorial(options).errors.join('\n'), /README.md:1: unclassified iris fence/);
  rmSync(readme);
  assert.deepEqual(checkTutorial(options).errors, []);
});

test('broken local links fail, including links whose labels contain inline code', context => {
  const options = fixture(context, `${block()}\n[\`missing\`](missing.md)\n`);
  const errors = checkTutorial(options).errors;
  assert.equal(errors.length, 2);
  assert.ok(errors.every(error => error.includes('broken local link "missing.md"')));
});

test('local file and directory links resolve relative to markdown, ignoring code examples', context => {
  const options = fixture(context);
  const file = join(options.root, 'tutorial/en/01-start.md');
  writeFileSync(join(options.root, 'tools/a (b).md'), '# File\n');
  const markdown = [
    '[tools](../../tools/)',
    '[file](../../tools/a%20%28b%29.md#section)',
    '[file](<../../tools/a (b).md>)',
    '[file](../../tools/a%20(b).md)',
    '[root](/tools/)',
    '[self](#start)',
    '[external](https://example.invalid/missing)',
    '[reference][tools]',
    '[tools]: ../../tools/ "Tools"',
    '`[not a link](missing.md)`',
    '``[not a `link`](missing.md)``',
    '```text\n[not a link](missing.md)\n```',
    '<!-- [hidden](missing.md) -->',
  ].join('\n');
  assert.deepEqual(checkLocalLinks(parseTutorial(markdown).prose, file, options.root), []);
  assert.match(checkLocalLinks('[bad][missing]', file, options.root).join('\n'), /undefined link reference/);
  assert.match(checkLocalLinks('[bad](%not-encoded)', file, options.root).join('\n'), /invalid local link/);
});

test('inline operators and generic type spelling do not consume headings', () => {
  const document = parseTutorial('Use `<` and `>` or `List<Object>`.\n\n## Truthiness\n');
  assert.deepEqual(document.headings, ['## Truthiness']);
});

test('timeout fails and removes temporary program directories', context => {
  const options = fixture(context, block({ ...hello, stdout: '' }, 'hang\n'));
  assert.deepEqual(runExample(parseTutorial(block()).examples[0], options), []);
  const started = Date.now();
  const result = checkTutorial({ ...options, timeout: 100 });
  assert.equal(result.errors.length, 2);
  assert.ok(result.errors.every(error => error.includes('timeout after 100ms')));
  assert.ok(Date.now() - started < 5000);
  const pids = readFileSync(join(options.root, 'hang-pids'), 'utf8').trim().split('\n');
  assert.equal(pids.length, 2);
  for (const pid of pids) assert.throws(() => process.kill(Number(pid), 0), { code: 'ESRCH' });
  assert.deepEqual(readdirSync(options.tempRoot), []);
});

test('missing binary fails and still cleans up', context => {
  const options = fixture(context);
  rmSync(options.binary);
  const example = parseTutorial(block()).examples[0];
  assert.match(runExample(example, options).join('\n'), /cannot execute/);
  assert.deepEqual(readdirSync(options.tempRoot), []);
});

test('CLI supports root and binary overrides and meaningful exit codes', context => {
  const options = fixture(context);
  const invoke = arguments_ => spawnSync(process.execPath, [checker, ...arguments_], { cwd: dirname(options.root), encoding: 'utf8', timeout: 5000 });
  const arguments_ = ['--root', options.root, '--binary', options.binary];
  const passed = invoke(arguments_);
  assert.equal(passed.status, 0, passed.stderr);
  assert.match(passed.stdout, /2 vm, 0 reference, 0 expected-error, 0 spec-only, 0 errors/);
  options.writePage('en', block({ ...hello, stdout: 'wrong' }));
  assert.equal(invoke(arguments_).status, 1);
  assert.equal(invoke(['--unknown']).status, 2);
  assert.equal(invoke(['--root']).status, 2);
  assert.match(invoke(['--help']).stdout, /Usage:/);
});

function landingHtml(language) {
  const node = () => ({
    addEventListener() {}, setAttribute() {},
    classList: { toggle() {} }, querySelectorAll: () => [],
  });
  const nodes = Object.fromEntries(['main', 'lang-toggle', 'theme-toggle', 'sidebar-toggle'].map(id => [id, node()]));
  const document = {
    documentElement: { dataset: {} },
    getElementById: id => nodes[id],
    querySelector: () => node(), querySelectorAll: () => [], addEventListener() {},
  };
  runInNewContext(readFileSync(new URL('../assets/app.js', import.meta.url), 'utf8'), {
    document,
    localStorage: { getItem: key => key === 'iris-site-lang' ? language : null, setItem() {} },
    location: { hash: '#/' },
    window: { addEventListener() {}, scrollTo() {} },
  }, { filename: 'assets/app.js' });
  assert.equal(document.documentElement.lang, language === 'en' ? 'en' : 'zh-CN');
  assert.equal(nodes.main.className, 'landing');
  return nodes.main.innerHTML;
}

function classElements(html, className) {
  const elements = [];
  for (const opening of html.matchAll(/<([a-z][\w-]*)\b[^>]*>/g)) {
    const classes = /\sclass=["']([^"']*)["']/.exec(opening[0]);
    if (!classes?.[1].split(/\s+/).includes(className)) continue;
    const tags = new RegExp(`<\\/?${opening[1]}\\b[^>]*>`, 'g');
    tags.lastIndex = opening.index + opening[0].length;
    let depth = 1;
    let closing;
    while (depth && (closing = tags.exec(html))) depth += closing[0].startsWith('</') ? -1 : 1;
    assert.equal(depth, 0, `unclosed .${className}`);
    elements.push({ index: opening.index, end: tags.lastIndex, body: html.slice(opening.index + opening[0].length, closing.index) });
  }
  return elements;
}

function decodeHtml(html) {
  const entities = { amp: '&', lt: '<', gt: '>', quot: '"', apos: "'", nbsp: '\u00a0' };
  return html.replace(/&(#x[\da-f]+|#\d+|amp|lt|gt|quot|apos|nbsp);/gi, (_, entity) => {
    if (!entity.startsWith('#')) return entities[entity.toLowerCase()];
    return String.fromCodePoint(entity[1].toLowerCase() === 'x' ? parseInt(entity.slice(2), 16) : Number(entity.slice(1)));
  });
}

const withoutTrailingNewline = text => text.replace(/\n$/, '');
const featureIds = ['01-operators', '07-union-bindings', '06-composition', '06-contracts-declare-promises', '09-open-class-reopen'];

for (const language of ['en', 'zh-cn']) {
  test(`landing ${language}: exactly four feature rows precede build and run`, () => {
    const html = landingHtml(language);
    const rows = classElements(html, 'feature-row');
    const starts = classElements(html, 'start-section');
    assert.equal(starts.length, 1, 'landing must retain one .start-section');
    assert.equal(rows.length, 4, 'landing must render exactly four .feature-row groups');
    for (const row of rows) {
      assert.ok(row.end <= starts[0].index, 'each complete feature row must precede .start-section');
      assert.match(row.body, /<figure\b[^>]*\sdata-example-id=["'][^"']+["']/, 'each feature row must contain an example');
    }
  });

  test(`landing ${language}: rendered feature source and output match tutorial VM examples`, () => {
    const directory = new URL(`../tutorial/${language}/`, import.meta.url);
    const examples = readdirSync(directory).filter(file => file.endsWith('.md')).flatMap(file => {
      const parsed = parseTutorial(readFileSync(new URL(file, directory), 'utf8'), file);
      assert.deepEqual(parsed.errors, []);
      return parsed.examples;
    });
    const html = landingHtml(language);
    const figures = [...html.matchAll(/<figure\b([^>]*)>([\s\S]*?)<\/figure>/g)].flatMap(match => {
      const id = /\sdata-example-id=["']([^"']+)["']/.exec(match[1]);
      return id ? [{ id: id[1], body: match[2], index: match.index, end: match.index + match[0].length }] : [];
    });
    assert.deepEqual(figures.map(figure => figure.id).sort(), [...featureIds].sort(), 'landing must render each of the five tutorial examples exactly once');
    const rows = classElements(html, 'feature-row');
    for (const figure of figures) {
      const expected = examples.filter(example => example.id === figure.id);
      assert.equal(expected.length, 1, `${figure.id}: unique tutorial metadata`);
      assert.equal(expected[0].mode, 'vm', `${figure.id}: tutorial must classify the example as VM-supported`);
      assert.ok(rows.some(row => row.index < figure.index && figure.end < row.end), `${figure.id}: figure must be inside a feature row`);
      const visibleText = decodeHtml(figure.body.replace(/<!--[\s\S]*?-->|<[^>]*>/g, ''));
      assert.match(visibleText, /--vm\b/, `${figure.id}: visible VM backend label`);
      for (const [className, field] of [['feature-source', 'source'], ['feature-output', 'stdout']]) {
        const containers = classElements(figure.body, className);
        assert.equal(containers.length, 1, `${figure.id}: one .${className}`);
        const codes = [...containers[0].body.matchAll(/<code\b[^>]*>([\s\S]*?)<\/code>/g)];
        assert.equal(codes.length, 1, `${figure.id}: one .${className} code`);
        const actual = decodeHtml(codes[0][1].replace(/<\/?span\b[^>]*>/g, ''));
        assert.equal(withoutTrailingNewline(actual), withoutTrailingNewline(expected[0][field]), `${language} ${figure.id}: rendered ${field} must match tutorial metadata`);
      }
    }
  });
}
