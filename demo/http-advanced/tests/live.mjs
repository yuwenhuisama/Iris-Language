import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import net from 'node:net';
import { fileURLToPath } from 'node:url';

const [runner, ...extra] = process.argv.slice(2);
assert.ok(runner && extra.length === 0, 'usage: node tests/live.mjs /absolute/path/to/iris');
const demo = fileURLToPath(new URL('../', import.meta.url));
const startupTimeout = Number(process.env.IRIS_STARTUP_TIMEOUT_MS ?? 60000);
assert.ok(Number.isInteger(startupTimeout) && startupTimeout > 0 && startupTimeout <= 2147483647);

function exchange(parts, end = true) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host: '127.0.0.1', port: 8081 });
    const chunks = [];
    socket.setTimeout(10000, () => socket.destroy(new Error('client timeout')));
    socket.on('data', chunk => chunks.push(chunk));
    socket.once('connect', () => {
      for (const part of parts) socket.write(part);
      if (end) socket.end();
    });
    socket.once('error', error => {
      if (error.code === 'ECONNRESET' && chunks.length > 0) resolve(Buffer.concat(chunks));
      else reject(error);
    });
    socket.once('end', () => { resolve(Buffer.concat(chunks)); socket.destroy(); });
  });
}

function expected(status, type, body) {
  const allow = status === '405 Method Not Allowed' ? 'Allow: GET\r\n' : '';
  return Buffer.from(`HTTP/1.1 ${status}\r\nContent-Type: ${type}\r\nContent-Length: ${Buffer.byteLength(body)}\r\nConnection: close\r\n${allow}X-Iris-Pipeline: advanced_v1\r\n\r\n${body}`);
}

for (const [engine, flags, shutdownOnly] of [
  ['reference', [], false], ['vm', ['--vm'], false],
  ['reference', [], true], ['vm', ['--vm'], true],
]) {
  const child = spawn(runner, ['package', 'run', demo, '--allow', 'native.load,native.blocking,network.tcp', ...flags], { stdio: ['ignore', 'pipe', 'pipe'] });
  let stdout = '';
  let stderr = '';
  child.stdout.on('data', chunk => { stdout += chunk; });
  child.stderr.on('data', chunk => { stderr += chunk; });
  const exited = new Promise(resolve => child.once('close', (code, signal) => resolve([code, signal])));
  let startupTimer;
  let shutdownTimer;
  let ready;
  let earlyExit;
  let spawnError;
  try {
    try {
      await new Promise((resolve, reject) => {
        startupTimer = setTimeout(() => reject(new Error(`startup timeout: ${stderr}`)), startupTimeout);
        ready = () => { if (stdout.includes('http://127.0.0.1:8081')) resolve(); };
        earlyExit = code => reject(new Error(`startup exit ${code}: ${stderr}`));
        spawnError = reject;
        child.stdout.on('data', ready);
        child.once('close', earlyExit);
        child.once('error', spawnError);
      });
    } finally {
      clearTimeout(startupTimer);
      child.stdout.off('data', ready);
      child.off('close', earlyExit);
      child.off('error', spawnError);
    }
    let clients = 0;
    const plain = 'text/plain; charset=utf-8';
    const request = target => `GET ${target} HTTP/1.1\r\nHost: localhost\r\n\r\n`;
    async function check(name, parts, status, type, body, end = true) {
      const actual = await exchange(parts, end);
      clients += 1;
      assert.deepEqual(actual, expected(status, type, body), `${engine}: ${name}`);
    }
    async function bad(name, parts, end = true) {
      await check(name, parts, '400 Bad Request', plain, 'Bad Request\n', end);
    }
    if (!shutdownOnly) {
    await check('unicode root', [request('/')], '200 OK', plain, '你好，Iris!\n');
    await check('health', [request('/health')], '200 OK', 'application/json', '{"status":"ok"}\n');
    await check('double', [request('/double?value=21')], '200 OK', 'application/json', '{"doubled":42}\n');
    await check('negative bound', [request('/double?value=-1000')], '200 OK', 'application/json', '{"doubled":-2000}\n');
    for (const value of ['true', 'null', '"21"', '[]', '1001']) {
      await check(`type/range ${value}`, [request(`/double?value=${value}`)], '422 Unprocessable Content', 'application/json', '{"error":"integer -1000..1000 required"}\n');
    }
    for (const target of ['/double', '/double?value=oops', '/double?value=1.5', '/double?value=%32', '/double?value=1&x=2']) {
      await bad('invalid query', [request(target)]);
    }
    await check('controlled error', [request('/fail')], '500 Internal Server Error', plain, 'Controlled Failure\n');
    await check('after error', [request('/')], '200 OK', plain, '你好，Iris!\n');
    await check('missing', [request('/missing')], '404 Not Found', plain, 'Not Found\n');
    for (const method of ['POST', '1GET', '123', 'nil']) {
      await check(method, [`${method} / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 10\r\n\r\n`], '405 Method Not Allowed', plain, 'Method Not Allowed\n');
    }
    await check('HEAD', ['HEAD / HTTP/1.1\r\nHost: localhost\r\n\r\n'], '405 Method Not Allowed', plain, '');
    await bad('source-like method', ['${raise :injected} / HTTP/1.1\r\nHost: localhost\r\n\r\n']);
    await check('after method errors', [request('/')], '200 OK', plain, '你好，Iris!\n');
    for (const header of ['', 'Host: \r\n', 'Host: a\r\nHost: b\r\n', 'Host: a b\r\n', 'Host: a\r\nTransfer-Encoding: chunked\r\n', 'Host: a\r\nContent-Length: 0\r\nContent-Length: 0\r\n', 'Host: a\r\nContent-Length: 1\r\n', 'Host: a\r\nX: a\nb\r\n']) {
      await bad('header validation', [`GET / HTTP/1.1\r\n${header}\r\n`]);
    }
    await bad('invalid UTF8', [Buffer.from('GET / HTTP/1.1\r\nHost: a\r\nX: '), Buffer.from([255]), '\r\n\r\n']);
    await bad('non ASCII', ['GET / HTTP/1.1\r\nHost: a\r\nX: é\r\n\r\n']);
    await check('split writes', ['GET /heal', 'th HTTP/1.1\r\nHost: a\r\n\r', '\n'], '200 OK', 'application/json', '{"status":"ok"}\n');
    await check('discard pipeline', [request('/') + request('/fail')], '200 OK', plain, '你好，Iris!\n');
    const prefix = 'GET / HTTP/1.1\r\nHost: localhost\r\nX: ';
    await check('8192 bytes', [prefix + 'a'.repeat(8192 - prefix.length - 4) + '\r\n\r\n'], '200 OK', plain, '你好，Iris!\n');
    await bad('8193 bytes', [prefix + 'a'.repeat(8192 - prefix.length - 3) + '\r\n\r\n']);
    const fields = 'GET / HTTP/1.1\r\nHost: a\r\n' + 'X: a\r\n'.repeat(63);
    await check('64 fields', [fields + '\r\n'], '200 OK', plain, '你好，Iris!\n');
    await bad('65 fields', [fields + 'X: a\r\n\r\n']);
    await bad('incomplete EOF', ['GET / HTTP/1.1\r\n']);
    await bad('idle read', [], false);
    assert.ok(clients < 64);
    } else {
      // Use separate short requests to verify normal 64-connection exit without earlier large requests exhausting the VM's total instruction budget.
      while (clients < 64) await check('64-client cleanup', [request('/health')], '200 OK', 'application/json', '{"status":"ok"}\n');
    }
    const scenarios = clients;
    // Stop only the process this test started after boundary cases; a separate short-request process verifies normal 64-client exit.
    // The current VM's total instruction budget prevents a long suite with 8KiB boundary cases from reliably reaching 64 connections.
    if (!shutdownOnly) {
      assert.equal(child.exitCode, null, stderr);
      assert.equal(child.signalCode, null, stderr);
      child.kill('SIGTERM');
    }
    const [code, signal] = await Promise.race([
      exited,
      new Promise((resolve, reject) => { shutdownTimer = setTimeout(() => reject(new Error('server shutdown timeout')), 5000); }),
    ]);
    if (shutdownOnly) {
      assert.equal(code, 0, `${signal}: ${stderr}`);
      console.log(`${engine}: 64 short clients, normal exit/finally cleanup PASS`);
    } else {
      assert.equal(signal, 'SIGTERM', `unexpected exit ${code}: ${stderr}`);
      console.log(`${engine}: ${scenarios} live scenarios, exact UTF-8 wire/metadata, owned SIGTERM shutdown PASS`);
    }
  } catch (error) {
    console.error(`${engine} output: ${stdout}\n${stderr}`);
    throw error;
  } finally {
    clearTimeout(shutdownTimer);
    if (child.exitCode === null && child.signalCode === null) child.kill('SIGTERM');
    await exited;
  }
}
