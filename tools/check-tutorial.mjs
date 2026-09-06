#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repositoryRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const languages = ['en', 'zh-cn'];
const modes = ['vm', 'reference', 'expected-error', 'spec-only'];
const annotationPattern = /^<!-- iris-example: (\{.*\}) -->$/;

function maskInlineCode(text) {
  return text.replace(/(`+)([\s\S]*?)\1(?!`)/g, match => match.replace(/[^\n]/g, ' '));
}

function visibleText(text) {
  return text.replace(/<!--[\s\S]*?-->/g, '');
}

function nearbyProse(lines, start, direction) {
  const paragraph = [];
  for (let index = start; index >= 0 && index < lines.length; index += direction) {
    const line = lines[index];
    if (!line.trim()) {
      if (paragraph.length) break;
      continue;
    }
    if (/^\s*(#{1,6}\s|`{3,}|~{3,}|<!-- iris-example:)/.test(line)) break;
    paragraph.push(line);
  }
  return /\p{L}/u.test(visibleText(maskInlineCode(paragraph.join('\n'))));
}

function validateAnnotation(annotation) {
  if (!annotation || typeof annotation !== 'object' || Array.isArray(annotation)) return 'annotation must be an object';
  if (typeof annotation.id !== 'string' || !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(annotation.id)) return 'id must be a nonempty lowercase hyphenated identifier';
  if (!modes.includes(annotation.mode)) return `mode must be one of ${modes.join(', ')}`;
  const allowed = new Set(['id', 'mode']);
  if (annotation.mode === 'spec-only') {
    allowed.add('reason');
    if (typeof annotation.reason !== 'string' || !annotation.reason.trim()) return 'spec-only needs a nonempty reason';
  } else {
    allowed.add('stdout');
    allowed.add('exit');
    if (typeof annotation.stdout !== 'string') return 'runnable examples need exact stdout (use "" for silence)';
    if (annotation.exit !== undefined && (!Number.isInteger(annotation.exit) || annotation.exit < 0 || annotation.exit > 255)) return 'exit must be an integer from 0 to 255';
    if (annotation.mode === 'expected-error') {
      allowed.add('engine');
      allowed.add('stderr');
      if (!['vm', 'reference'].includes(annotation.engine)) return 'expected-error needs engine vm or reference';
      if (!Number.isInteger(annotation.exit) || annotation.exit === 0) return 'expected-error needs an explicit nonzero integer exit';
      if (typeof annotation.stderr !== 'string' || !annotation.stderr.trim()) return 'expected-error needs a nonempty stderr substring';
    } else if (annotation.exit !== undefined && annotation.exit !== 0) {
      return 'nonzero exit requires expected-error mode';
    }
  }
  const unknown = Object.keys(annotation).filter(key => !allowed.has(key));
  return unknown.length ? `unknown annotation fields: ${unknown.join(', ')}` : null;
}

export function parseTutorial(markdown, file = '<markdown>') {
  const lines = markdown.replace(/\r\n/g, '\n').split('\n');
  const prose = [...lines];
  const examples = [];
  const errors = [];
  const usedAnnotations = new Set();
  let fenceCount = 0;
  let inComment = false;
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (inComment || line.trimStart().startsWith('<!--')) {
      if (line.includes('<!--')) inComment = true;
      if (line.includes('-->')) inComment = false;
      continue;
    }
    const opening = /^ {0,3}(`{3,}|~{3,})([^\n]*)$/.exec(line);
    if (!opening) continue;
    const start = index;
    const marker = opening[1];
    const closing = new RegExp(`^ {0,3}${marker[0]}{${marker.length},}\\s*$`);
    index += 1;
    while (index < lines.length && !closing.test(lines[index])) index += 1;
    const closed = index < lines.length;
    for (let hidden = start; hidden <= Math.min(index, lines.length - 1); hidden += 1) prose[hidden] = '';
    if (!closed) errors.push(`${file}:${start + 1}: unclosed code fence`);
    if (opening[2].trim().split(/\s+/)[0].toLowerCase() !== 'iris') continue;
    fenceCount += 1;
    const location = `${file}:${start + 1}`;
    if (line !== '```iris') errors.push(`${location}: Iris examples must use the exact opening fence \`\`\`iris`);
    const annotationLine = lines[start - 1] ?? '';
    const match = annotationPattern.exec(annotationLine);
    if (!match) {
      errors.push(`${location}: unclassified iris fence; add <!-- iris-example: {"id":"01-hello","mode":"vm","stdout":"Hello, Iris!\\n"} --> immediately before it`);
      continue;
    }
    usedAnnotations.add(start - 1);
    let annotation;
    try {
      annotation = JSON.parse(match[1]);
    } catch (error) {
      errors.push(`${location}: invalid annotation JSON: ${error.message}`);
      continue;
    }
    const problem = validateAnnotation(annotation);
    if (problem) {
      errors.push(`${location}: ${problem}`);
      continue;
    }
    if (annotation.mode === 'spec-only' && !nearbyProse(lines, start - 2, -1) && !nearbyProse(lines, index + 1, 1)) {
      errors.push(`${location}: spec-only needs visible explanatory prose adjacent to the example`);
    }
    if (closed) examples.push({ ...annotation, source: lines.slice(start + 1, index).join('\n') + (index > start + 1 ? '\n' : ''), file, line: start + 1 });
  }
  for (let index = 0; index < prose.length; index += 1) {
    if (prose[index].includes('<!-- iris-example:') && !usedAnnotations.has(index)) errors.push(`${file}:${index + 1}: annotation must immediately precede an iris fence`);
  }
  const text = visibleText(prose.join('\n'));
  const headings = text.split('\n').filter(line => /^ {0,3}#{1,6}\s+/.test(line)).map(line => line.trim().replace(/\s+#+\s*$/, ''));
  return { examples, errors, headings, prose: text, fenceCount };
}

export function checkLocalLinks(prose, file, root) {
  const text = maskInlineCode(prose);
  const errors = [];
  const definitions = new Map();
  const normalize = label => label.trim().replace(/\s+/g, ' ').toLowerCase();
  const targets = [];
  for (const match of text.matchAll(/^ {0,3}\[([^\]]+)\]:\s*(?:<([^>]+)>|(\S+))/gm)) {
    definitions.set(normalize(match[1]), match[2] ?? match[3]);
    targets.push(match[2] ?? match[3]);
  }
  const withoutDefinitions = text.replace(/^ {0,3}\[[^\]]+\]:.*$/gm, '');
  const inline = /!?\[[^\]\n]*\]\(\s*(?:<([^>]+)>|((?:[^\s()\\]|\\.|\([^()]*\))*))(?:\s+(?:"[^"]*"|'[^']*'|\([^)]*\)))?\s*\)/g;
  for (const match of withoutDefinitions.matchAll(inline)) targets.push(match[1] ?? match[2]);
  const remaining = withoutDefinitions.replace(inline, '');
  for (const match of remaining.matchAll(/!?\[([^\]\n]*)\](?:\[([^\]\n]*)\])?/g)) {
    const label = normalize(match[2] || match[1]);
    if (definitions.has(label)) targets.push(definitions.get(label));
    else if (match[2] !== undefined) errors.push(`${file}: undefined link reference [${label}]`);
  }
  for (const target of new Set(targets)) {
    if (!target || /^(?:[a-z][a-z0-9+.-]*:|\/\/|#)/i.test(target)) continue;
    try {
      const path = decodeURIComponent(target.split(/[?#]/, 1)[0].replace(/\\([() ])/g, '$1'));
      const destination = path.startsWith('/') ? resolve(root, `.${path}`) : resolve(dirname(file), path);
      if (!existsSync(destination)) errors.push(`${file}: broken local link ${JSON.stringify(target)}`);
    } catch (error) {
      errors.push(`${file}: invalid local link ${JSON.stringify(target)}: ${error.message}`);
    }
  }
  return errors;
}

export function runExample(example, { binary, root = repositoryRoot, timeout = 5000, tempRoot = tmpdir() }) {
  if (example.mode === 'spec-only') return [];
  const directory = mkdtempSync(join(tempRoot, 'iris-tutorial-'));
  try {
    const sourcePath = join(directory, 'example.iris');
    writeFileSync(sourcePath, example.source);
    const engine = example.mode === 'expected-error' ? example.engine : example.mode;
    const arguments_ = engine === 'vm' ? ['--vm', sourcePath] : [sourcePath];
    const result = spawnSync(binary, arguments_, { cwd: root, encoding: 'utf8', timeout, killSignal: 'SIGKILL', maxBuffer: 1024 * 1024 });
    const location = `${example.file}:${example.line} (${example.id}, ${engine})`;
    if (result.error) return [`${location}: ${result.error.code === 'ETIMEDOUT' ? `timeout after ${timeout}ms` : `cannot execute: ${result.error.message}`}`];
    const errors = [];
    if (result.status !== (example.exit ?? 0)) errors.push(`${location}: exit expected ${example.exit ?? 0}, got ${result.status ?? result.signal}; stderr ${JSON.stringify(result.stderr)}`);
    if (result.stdout !== example.stdout) errors.push(`${location}: stdout expected ${JSON.stringify(example.stdout)}, got ${JSON.stringify(result.stdout)}`);
    if (example.mode === 'expected-error' && !result.stderr.includes(example.stderr)) errors.push(`${location}: stderr must contain ${JSON.stringify(example.stderr)}, got ${JSON.stringify(result.stderr)}`);
    return errors;
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

function markdownFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap(entry => {
    const path = join(directory, entry.name);
    return entry.isDirectory() ? markdownFiles(path) : entry.name.endsWith('.md') ? [path] : [];
  }).sort();
}

function expectations(example) {
  return example.mode === 'spec-only'
    ? { mode: example.mode }
    : { mode: example.mode, engine: example.engine ?? example.mode, stdout: example.stdout, exit: example.exit ?? 0, stderr: example.stderr ?? null };
}

export function checkTutorial({ root = repositoryRoot, binary, timeout = 5000, tempRoot = tmpdir() } = {}) {
  root = resolve(root);
  binary = resolve(binary ?? join(root, 'target/debug/iris'));
  const baseline = JSON.parse(readFileSync(join(root, 'tools/tutorial-routes.json'), 'utf8'));
  if (baseline.version !== 1 || JSON.stringify(baseline.languages) !== JSON.stringify(languages) || !Array.isArray(baseline.routes) || !baseline.routes.length) throw new Error('invalid tutorial-routes.json baseline');
  const errors = [];
  const inventories = new Map();
  let pages = 0;
  let fences = 0;
  const modeCounts = Object.fromEntries(modes.map(mode => [mode, 0]));
  function checkDocument(file) {
    const document = parseTutorial(readFileSync(file, 'utf8'), file);
    pages += 1;
    fences += document.fenceCount;
    errors.push(...document.errors, ...checkLocalLinks(document.prose, file, root));
    for (const example of document.examples) {
      modeCounts[example.mode] += 1;
      errors.push(...runExample(example, { binary, root, timeout, tempRoot }));
    }
    return document;
  }
  const readme = join(root, 'README.md');
  if (existsSync(readme)) checkDocument(readme);
  for (const language of languages) {
    const directory = join(root, 'tutorial', language);
    const files = existsSync(directory) ? markdownFiles(directory) : [];
    const documents = new Map();
    const examples = new Map();
    for (const file of files) {
      const stem = relative(directory, file).replace(/\.md$/, '').split('\\').join('/');
      const document = checkDocument(file);
      documents.set(stem, document);
      for (const example of document.examples) {
        if (examples.has(example.id)) errors.push(`${file}:${example.line}: duplicate id ${example.id} within ${language}`);
        examples.set(example.id, { ...example, stem });
      }
    }
    for (const route of baseline.routes) {
      const document = documents.get(route.stem);
      if (!document) {
        errors.push(`tutorial/${language}/${route.stem}.md: missing baseline route`);
        continue;
      }
      for (const [index, heading] of route.headings[language].entries()) {
        if (document.headings[index] !== heading) errors.push(`tutorial/${language}/${route.stem}.md: baseline heading prefix mismatch at position ${index + 1}; expected ${JSON.stringify(heading)}, got ${JSON.stringify(document.headings[index] ?? null)}; new headings may only append`);
      }
    }
    inventories.set(language, { documents, examples });
  }
  const english = inventories.get('en');
  const chinese = inventories.get('zh-cn');
  for (const stem of new Set([...english.documents.keys(), ...chinese.documents.keys()])) {
    for (const language of languages) if (!inventories.get(language).documents.has(stem)) errors.push(`tutorial/${language}/${stem}.md: missing translation page`);
  }
  for (const id of new Set([...english.examples.keys(), ...chinese.examples.keys()])) {
    const original = english.examples.get(id);
    const translation = chinese.examples.get(id);
    if (!original || !translation) {
      errors.push(`${id}: missing translation example in ${original ? 'zh-cn' : 'en'}`);
      continue;
    }
    if (original.stem !== translation.stem) errors.push(`${id}: bilingual route mismatch`);
    if (original.source !== translation.source) errors.push(`${id}: bilingual source mismatch`);
    if (JSON.stringify(expectations(original)) !== JSON.stringify(expectations(translation))) errors.push(`${id}: bilingual expectations mismatch`);
  }
  const specOnly = modeCounts['spec-only'];
  const runnable = modeCounts.vm + modeCounts.reference + modeCounts['expected-error'];
  return { errors, pages, fences, runnable, specOnly, modeCounts };
}

export function main(arguments_ = process.argv.slice(2)) {
  const options = {};
  for (let index = 0; index < arguments_.length; index += 1) {
    const argument = arguments_[index];
    if (argument === '--help') {
      console.log('Usage: node tools/check-tutorial.mjs [--root PATH] [--binary PATH]\nChecks bilingual tutorials and optional root README.md annotations, exact program output, and local file/directory links.\nTutorial baseline headings must remain an exact prefix; new headings may only append.\nEach program has a 5000ms timeout. Build first: cargo build -p iris-cli');
      return 0;
    }
    if (!['--root', '--binary'].includes(argument) || !arguments_[index + 1] || arguments_[index + 1].startsWith('--')) {
      console.error(`Invalid argument: ${argument}. Use --help.`);
      return 2;
    }
    options[argument.slice(2)] = arguments_[index + 1];
    index += 1;
  }
  try {
    const result = checkTutorial(options);
    for (const error of result.errors) console.error(error);
    const modeSummary = modes.map(mode => `${result.modeCounts[mode]} ${mode}`).join(', ');
    console.log(`Tutorial check: ${result.pages} pages, ${result.fences} Iris fences, ${modeSummary}, ${result.errors.length} errors.`);
    return result.errors.length ? 1 : 0;
  } catch (error) {
    console.error(`Tutorial check: ${error.message}`);
    return 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) process.exitCode = main();
