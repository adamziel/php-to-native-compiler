#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

process.umask(0o022);

const OUT = process.cwd();
const SOURCE = process.env.SOURCE_REPO || '/home/claude/php-to-native-compiler';
const LANE_ROOT = process.env.LANE_ROOT || '/home/claude';
const REPO = 'https://github.com/adamziel/php-to-native-compiler';
const GENERATED_AT = new Date().toISOString();

const taskMeta = new Map([
  ['String conversion, truthiness, byte buffers', ['strings-conversion-byte-buffers', ['string', 'binary-string', 'type-conversion'], ['source/progress.html', 'source/support.html', 'source/architecture.html']]],
  ['Call operation cleanup and ownership', ['call-operation-cleanup-ownership', ['call', 'function-frame', 'symbol'], ['source/progress.html', 'source/architecture.html', 'source/support.html']]],
  ['Comparison and conversion semantics', ['comparison-conversion-semantics', ['comparison', 'type-conversion'], ['source/progress.html', 'source/support.html']]],
  ['Arrays, lvalues, references, COW', ['arrays-lvalues-references-cow', ['array', 'reference', 'runtime-handles', 'callback-containers', 'dynamic-holders'], ['source/cow-coverage-matrix.html', 'source/progress.html', 'source/next-tasks.html']]],
  ['Symbols, globals, request state', ['symbols-globals-request-state', ['symbol', 'global', 'link-symbol'], ['source/progress.html', 'source/support.html', 'source/architecture.html']]],
  ['Objects, properties, methods', ['objects-properties-methods', ['object', 'property'], ['source/progress.html', 'source/support.html', 'source/architecture.html']]],
  ['Diagnostics and control-flow cleanup', ['diagnostics-control-flow-cleanup', ['diagnostic', 'error', 'control-flow', 'exit', 'backlog'], ['source/progress.html', 'source/support.html']]],
  ['Filesystem/path builtins and request state', ['filesystem-path-request-state', ['sapi', 'wpdb'], ['source/progress.html', 'source/support.html', 'source/architecture.html']]],
  ['Broad composition verification', ['broad-composition-verification', ['regression', 'evaluator', 'backlog', 'integration'], ['source/progress.html', 'source/roadmap.html', 'source/cow-coverage-matrix.html']]],
]);

for (const rel of ['assets', 'lanes', 'source', 'subtasks', 'index.html', '404.html', '.nojekyll', 'README.md']) {
  fs.rmSync(path.join(OUT, rel), { recursive: true, force: true });
}

function ensure(dir) {
  fs.mkdirSync(dir, { recursive: true, mode: 0o755 });
}

function write(rel, text) {
  const file = path.join(OUT, rel);
  ensure(path.dirname(file));
  fs.writeFileSync(file, text, { mode: 0o644 });
}

function src(rel) {
  return fs.readFileSync(path.join(SOURCE, rel), 'utf8');
}

function esc(value) {
  return String(value)
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

function slug(value) {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '');
}

function inline(value) {
  let html = esc(value);
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (_m, label, href) => `<a href="${esc(href)}">${label}</a>`);
  html = html.replace(/\b([0-9a-f]{8,40})\b/g, (hash) => `<a href="${REPO}/commit/${hash}">${hash.slice(0, 8)}</a>`);
  return html;
}

function table(lines) {
  const rows = lines
    .filter((line) => !/^\s*\|?\s*:?-{3,}/.test(line))
    .map((line) => line.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((cell) => cell.trim()));
  if (!rows.length) return '';
  const [head, ...body] = rows;
  return `<div class="table-wrap"><table><thead><tr>${head.map((cell) => `<th>${inline(cell)}</th>`).join('')}</tr></thead><tbody>${body.map((row) => `<tr>${row.map((cell) => `<td>${inline(cell)}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
}

function markdown(md, opts = {}) {
  const shift = opts.shift || 0;
  const maxLines = opts.maxLines || 0;
  const all = md.replace(/\r\n/g, '\n').split('\n');
  const lines = maxLines && all.length > maxLines ? all.slice(0, maxLines) : all;
  const out = [];
  let para = [];
  let list = '';
  let code = null;

  const closePara = () => {
    if (para.length) out.push(`<p>${inline(para.join(' '))}</p>`);
    para = [];
  };
  const closeList = () => {
    if (list) out.push(`</${list}>`);
    list = '';
  };
  const closeCode = () => {
    if (code) out.push(`<pre><code>${esc(code.join('\n'))}</code></pre>`);
    code = null;
  };

  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (line.startsWith('```')) {
      if (code) closeCode();
      else {
        closePara();
        closeList();
        code = [];
      }
      continue;
    }
    if (code) {
      code.push(line);
      continue;
    }
    if (!line.trim()) {
      closePara();
      closeList();
      continue;
    }
    const h = /^(#{1,6})\s+(.*)$/.exec(line);
    if (h) {
      closePara();
      closeList();
      const level = Math.min(6, h[1].length + shift);
      out.push(`<h${level} id="${slug(h[2])}">${inline(h[2])}</h${level}>`);
      continue;
    }
    if (line.includes('|') && i + 1 < lines.length && /^\s*\|?\s*:?-{3,}/.test(lines[i + 1])) {
      closePara();
      closeList();
      const chunk = [line, lines[i + 1]];
      i += 2;
      while (i < lines.length && lines[i].includes('|') && lines[i].trim()) {
        chunk.push(lines[i]);
        i += 1;
      }
      i -= 1;
      out.push(table(chunk));
      continue;
    }
    const ul = /^\s*[-*]\s+(.*)$/.exec(line);
    if (ul) {
      closePara();
      if (list !== 'ul') {
        closeList();
        list = 'ul';
        out.push('<ul>');
      }
      out.push(`<li>${inline(ul[1])}</li>`);
      continue;
    }
    const ol = /^\s*\d+\.\s+(.*)$/.exec(line);
    if (ol) {
      closePara();
      if (list !== 'ol') {
        closeList();
        list = 'ol';
        out.push('<ol>');
      }
      out.push(`<li>${inline(ol[1])}</li>`);
      continue;
    }
    para.push(line.trim());
  }
  closeCode();
  closePara();
  closeList();
  if (maxLines && all.length > maxLines) {
    out.push(`<p class="note">Showing the first ${maxLines.toLocaleString('en-US')} lines of ${all.length.toLocaleString('en-US')} total lines. Open the GitHub source link for the complete document.</p>`);
  }
  return out.join('\n');
}

function section(md, title) {
  const re = new RegExp(`^##\\s+${title.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\s*$`, 'm');
  const match = re.exec(md);
  if (!match) return '';
  const rest = md.slice(match.index + match[0].length);
  const next = /\n##\s+/.exec(rest);
  return (next ? rest.slice(0, next.index) : rest).trim();
}

function bullets(block, limit = 8) {
  const found = [];
  let cur = '';
  for (const line of block.split('\n')) {
    const top = /^-\s+(.*)$/.exec(line);
    const cont = /^\s{2,}(\S.*)$/.exec(line);
    if (top) {
      if (cur) found.push(cur.trim());
      cur = top[1];
    } else if (cont && cur) {
      cur += ` ${cont[1].trim()}`;
    }
  }
  if (cur) found.push(cur.trim());
  return found.slice(0, limit);
}

function statusBars(md) {
  const block = /```([\s\S]*?)```/.exec(section(md, 'Overall Status'))?.[1] || '';
  return block.split('\n').map((line) => /^(.+?)\s+\[[#-]+\]\s+(\d+)%/.exec(line)).filter(Boolean).map((m) => [m[1].trim(), Number(m[2])]);
}

function activeRows(md) {
  return section(md, 'Active Roadmap Estimates')
    .split('\n')
    .filter((line) => line.trim().startsWith('|') && !/^\s*\|?\s*:?-{3,}/.test(line))
    .slice(1)
    .map((line) => line.trim().replace(/^\|/, '').replace(/\|$/, '').split('|').map((cell) => cell.trim()))
    .map((cells) => {
      const meta = taskMeta.get(cells[0]);
      return {
        item: cells[0],
        primary: Number(cells[1].replace('%', '')),
        lane: Number(cells[2].replace('%', '')),
        read: cells[3],
        slug: meta?.[0] || slug(cells[0]),
        keywords: meta?.[1] || [],
        docs: meta?.[2] || ['source/progress.html'],
      };
    });
}

function checklist(md) {
  return section(md, 'Done / In Progress / Not Done')
    .split('\n')
    .map((line) => /^-\s+\[([ xX])\]\s+(.*)$/.exec(line))
    .filter(Boolean)
    .map((m) => {
      const text = m[2].trim();
      let state = m[1].toLowerCase() === 'x' ? 'done' : 'open';
      if (/^In progress:/i.test(text)) state = 'in-progress';
      if (/^Not done:/i.test(text)) state = 'not-done';
      return { state, text };
    });
}

function recent(md) {
  const out = [];
  const re = /^-\s+`([0-9a-f]+)`\s+(.+?)\n\s+-\s+([\s\S]*?)(?=\n-\s+`[0-9a-f]+`|\n##\s+|$)/gm;
  let m;
  while ((m = re.exec(section(md, 'Recent Primary-Integrated Work'))) !== null) {
    out.push({ hash: m[1], title: m[2].trim(), detail: m[3].replace(/\n\s+/g, ' ').trim() });
  }
  return out;
}

function git(cwd, args) {
  try {
    return execFileSync('git', ['-C', cwd, ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
  } catch {
    return '';
  }
}

function latest(md) {
  const match = /^##\s+(\d{4}-\d{2}-\d{2})\s*$/m.exec(md);
  if (!match) return { date: '', md: md.split('\n').slice(0, 80).join('\n') };
  const rest = md.slice(match.index);
  const next = /\n##\s+\d{4}-\d{2}-\d{2}\s*$/m.exec(rest.slice(match[0].length));
  return { date: match[1], md: (next ? rest.slice(0, match[0].length + next.index) : rest).trim() };
}

function collectLanes() {
  if (!fs.existsSync(LANE_ROOT)) return [];
  return fs.readdirSync(LANE_ROOT, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && entry.name.startsWith('phpc-lane-'))
    .map((entry) => {
      const dir = path.join(LANE_ROOT, entry.name);
      const file = path.join(dir, 'docs', 'PROGRESS.md');
      if (!fs.existsSync(file)) return null;
      const l = latest(fs.readFileSync(file, 'utf8'));
      return {
        name: entry.name,
        slug: slug(entry.name),
        branch: git(dir, ['branch', '--show-current']) || 'detached',
        head: git(dir, ['rev-parse', '--short', 'HEAD']),
        date: l.date || 'n/a',
        md: l.md,
        summary: bullets(l.md, 4),
      };
    })
    .filter(Boolean)
    .sort((a, b) => a.name.localeCompare(b.name));
}

function meter(value, label) {
  return `<div class="meter"><div><span>${esc(label)}</span><strong>${value}%</strong></div><progress max="100" value="${value}">${value}%</progress></div>`;
}

function mini(value, label) {
  return `<div class="mini"><span>${esc(label)}</span><progress max="100" value="${value}">${value}%</progress><strong>${value}%</strong></div>`;
}

function cls(value) {
  return value >= 70 ? 'good' : value >= 35 ? 'mixed' : 'early';
}

function shell(title, body, depth = 0) {
  const prefix = '../'.repeat(depth);
  return `<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${esc(title)}</title>
  <link rel="stylesheet" href="${prefix}assets/site.css">
  <script defer src="${prefix}assets/site.js"></script>
</head>
<body>
  <header class="topbar">
    <a class="brand" href="${prefix}index.html">PHP Native Compiler Progress</a>
    <nav><a href="${REPO}/blob/master/PROGRESS.md">PROGRESS.md</a><a href="${REPO}">Repository</a><a href="${prefix}source/progress.html">Source Snapshot</a></nav>
  </header>
  <main>${body}</main>
  <footer><span>Generated ${esc(GENERATED_AT)}</span><span>Source branch: <a href="${REPO}/tree/master">master</a></span></footer>
</body>
</html>`;
}

write('assets/site.css', `:root{--bg:#f7f8f5;--panel:#fff;--soft:#eef5f1;--ink:#17211d;--muted:#5d6a64;--line:#d9ded8;--accent:#007f73;--accent2:#4969a8;--good:#0c7a42;--mixed:#a36200;--early:#9f3434;--shadow:0 1px 2px rgba(20,28,24,.08),0 10px 24px rgba(20,28,24,.06)}*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;background:var(--bg);color:var(--ink);font-family:Inter,ui-sans-serif,system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;line-height:1.5}a{color:var(--accent);text-underline-offset:.18em}.topbar{position:sticky;top:0;z-index:10;display:flex;justify-content:space-between;align-items:center;gap:1rem;padding:.75rem clamp(1rem,3vw,2rem);border-bottom:1px solid var(--line);background:rgba(247,248,245,.94);backdrop-filter:blur(12px)}.brand{color:var(--ink);font-weight:800;text-decoration:none}.topbar nav{display:flex;flex-wrap:wrap;justify-content:flex-end;gap:.75rem;font-size:.92rem}main{width:min(1180px,calc(100% - 2rem));margin:0 auto;padding:1.4rem 0 3rem}.hero{display:grid;grid-template-columns:minmax(0,1.25fr) minmax(280px,.75fr);gap:1rem;align-items:stretch;margin-bottom:1rem}.hero-main,.panel,.card,details,.source{background:var(--panel);border:1px solid var(--line);border-radius:8px;box-shadow:var(--shadow)}.hero-main,.panel,.source{padding:clamp(1rem,3vw,1.6rem)}h1,h2,h3{line-height:1.15;letter-spacing:0}h1{max-width:860px;margin:0 0 .75rem;font-size:clamp(2rem,4vw,3.3rem)}h2{margin:2rem 0 .8rem;font-size:clamp(1.45rem,2vw,1.8rem)}h3{margin:1.2rem 0 .55rem;font-size:1.08rem}.lede{max-width:880px;color:var(--muted);font-size:1.05rem}.eyebrow,.metric span{display:block;color:var(--muted);font-size:.78rem;font-weight:700;text-transform:uppercase;letter-spacing:.08em}.meta,.stats,.docs,.tasks{display:grid;gap:.8rem}.meta{grid-template-columns:repeat(3,minmax(0,1fr));margin-top:1rem}.stats{grid-template-columns:repeat(4,minmax(0,1fr))}.docs,.tasks{grid-template-columns:repeat(auto-fit,minmax(230px,1fr))}.metric{padding:.85rem;background:var(--soft);border-radius:8px;border:1px solid var(--line)}.metric strong{display:block;margin-top:.25rem;font-size:1.05rem;overflow-wrap:anywhere}.overall{display:grid;gap:1rem;align-content:space-between}.number{display:block;font-size:clamp(3rem,8vw,5rem);font-weight:850;line-height:.95}.refresh,.toolbar{display:flex;flex-wrap:wrap;align-items:center;gap:.6rem}.refresh{margin-top:.75rem;color:var(--muted);font-size:.9rem}.toolbar{justify-content:space-between;margin:1rem 0}button,.button,.search{border:1px solid var(--line);border-radius:6px;background:#fff;color:var(--ink);min-height:2.25rem;padding:.45rem .7rem;font:inherit}.search{min-width:min(100%,280px)}button:hover,.button:hover{border-color:var(--accent);cursor:pointer}progress{width:100%;height:.7rem;appearance:none}progress::-webkit-progress-bar{background:#e4e8e2;border-radius:999px}progress::-webkit-progress-value{background:linear-gradient(90deg,var(--accent),var(--accent2));border-radius:999px}progress::-moz-progress-bar{background:var(--accent);border-radius:999px}.meter{display:grid;gap:.35rem;padding:.65rem 0;border-bottom:1px solid var(--line)}.meter:last-child{border-bottom:0}.meter>div{display:flex;justify-content:space-between;gap:1rem}.card{display:block;padding:1rem;text-decoration:none}.card h3{margin-top:0}.tag{display:inline-flex;border-radius:999px;border:1px solid var(--line);background:var(--soft);color:var(--muted);padding:.2rem .5rem;font-size:.82rem;font-weight:700}.tag.good{color:var(--good)}.tag.mixed{color:var(--mixed)}.tag.early{color:var(--early)}details{margin:.75rem 0;padding:.1rem 1rem}summary{cursor:pointer;font-weight:800;padding:.9rem 0}.table-wrap{width:100%;overflow-x:auto}table{width:100%;border-collapse:collapse;background:var(--panel)}th,td{border-bottom:1px solid var(--line);padding:.75rem;text-align:left;vertical-align:top}th{color:var(--muted);font-size:.8rem;text-transform:uppercase;letter-spacing:.07em}.roadmap td:first-child,.lane-table td:first-child{min-width:240px}.mini{display:grid;grid-template-columns:5.5rem 1fr 3rem;gap:.45rem;align-items:center;margin-bottom:.35rem;font-size:.85rem}.mini progress{height:.45rem}.note{color:var(--muted);font-size:.92rem}pre{overflow-x:auto;padding:1rem;border-radius:8px;border:1px solid var(--line);background:#101815;color:#eaf2ed}code{font-family:"SFMono-Regular",Consolas,"Liberation Mono",monospace;font-size:.92em}footer{display:flex;flex-wrap:wrap;justify-content:center;gap:1rem;padding:1.5rem;border-top:1px solid var(--line);color:var(--muted);font-size:.9rem}@media(max-width:820px){.hero{grid-template-columns:1fr}.meta,.stats{grid-template-columns:1fr}.topbar{align-items:flex-start;flex-direction:column}.topbar nav{justify-content:flex-start}.mini{grid-template-columns:1fr}}`);

write('assets/site.js', `(()=>{const secs=900,btn=document.querySelector('[data-refresh-toggle]'),out=document.querySelector('[data-refresh-countdown]');let on=localStorage.getItem('phpc-report-refresh')!=='off',left=secs;function time(s){const m=Math.floor(s/60),r=String(s%60).padStart(2,'0');return \`\${m}:\${r}\`}function draw(){if(btn)btn.textContent=on?'Pause refresh':'Resume refresh';if(out)out.textContent=on?\`Auto refresh in \${time(left)}\`:'Auto refresh paused'}btn?.addEventListener('click',()=>{on=!on;localStorage.setItem('phpc-report-refresh',on?'on':'off');left=secs;draw()});setInterval(()=>{if(on){left-=1;if(left<=0)location.reload()}draw()},1000);draw();document.querySelectorAll('[data-expand-all]').forEach(b=>b.addEventListener('click',()=>document.querySelectorAll('details').forEach(d=>d.open=true)));document.querySelectorAll('[data-collapse-all]').forEach(b=>b.addEventListener('click',()=>document.querySelectorAll('details').forEach(d=>d.open=false)));document.querySelectorAll('[data-filter]').forEach(i=>{const t=document.querySelector(i.dataset.filter);i.addEventListener('input',()=>{const q=i.value.trim().toLowerCase();t?.querySelectorAll('[data-filter-row]').forEach(r=>{r.hidden=q&&!r.textContent.toLowerCase().includes(q)})})})})();`);

const progress = src('PROGRESS.md');
const updated = /^Updated:\s*(.+)$/m.exec(progress)?.[1] || 'Unknown';
const marker = /^Evaluation marker:\s*(.+)$/m.exec(progress)?.[1] || 'Unknown';
const overall = Number(/generalized PHP native compiler:\s+\*\*(\d+)%\*\*/.exec(progress)?.[1] || 0);
const bars = statusBars(progress);
const rows = activeRows(progress);
const checks = checklist(progress);
const commits = recent(progress);
const lanes = collectLanes();
const counts = checks.reduce((a, x) => ((a[x.state] = (a[x.state] || 0) + 1), a), {});
const primary = bullets(section(progress, 'Current Primary State'), 8);
const candidates = bullets(section(progress, 'Current Lane-Local Candidate Work Not Yet Counted'), 8);
const steering = section(progress, 'Near-Term Steering').split('\n').map((line) => /^\d+\.\s+(.*)$/.exec(line)).filter(Boolean).map((m) => m[1]);

function matched(row) {
  return lanes.filter((lane) => row.keywords.some((kw) => lane.name.includes(kw) || lane.branch.includes(kw))).slice(0, 8);
}

const roadmapHtml = rows.map((row) => {
  const laneLinks = matched(row).map((lane) => `<a href="lanes/${lane.slug}.html">${esc(lane.name.replace(/^phpc-lane-/, ''))}</a>`).join(', ') || '<span class="note">No keyword match.</span>';
  return `<tr data-filter-row><td><a href="subtasks/${row.slug}.html"><strong>${esc(row.item)}</strong></a><div><span class="tag ${cls(row.primary)}">primary ${row.primary}%</span> <span class="tag ${cls(row.lane)}">lane ${row.lane}%</span></div></td><td>${mini(row.primary, 'Primary')}${mini(row.lane, 'Lane-local')}</td><td>${inline(row.read)}</td><td>${laneLinks}</td></tr>`;
}).join('\n');

const laneHtml = lanes.map((lane) => `<tr data-filter-row><td><a href="lanes/${lane.slug}.html"><strong>${esc(lane.name.replace(/^phpc-lane-/, ''))}</strong></a></td><td>${esc(lane.branch)}</td><td><code>${esc(lane.head)}</code></td><td>${esc(lane.date)}</td><td>${lane.summary[0] ? inline(lane.summary[0]) : '<span class="note">No latest summary.</span>'}</td></tr>`).join('\n');

const index = `
<section class="hero">
  <div class="hero-main">
    <span class="eyebrow">Public dashboard</span>
    <h1>PHP Native Compiler Progress</h1>
    <p class="lede">A generated HTML view over the candid progress report, with primary-integrated capability separated from lane-local candidates. Percentages are engineering estimates, not pass rates.</p>
    <div class="meta"><div class="metric"><span>Report updated</span><strong>${esc(updated)}</strong></div><div class="metric"><span>Evaluation marker</span><strong>${esc(marker)}</strong></div><div class="metric"><span>Pages generated</span><strong>${esc(GENERATED_AT)}</strong></div></div>
    <div class="refresh"><button type="button" data-refresh-toggle>Pause refresh</button><span data-refresh-countdown>Auto refresh in 15:00</span></div>
  </div>
  <aside class="panel overall"><div><span class="eyebrow">Overall estimate</span><span class="number">${overall}%</span><progress max="100" value="${overall}">${overall}%</progress></div><div class="stats"><div class="metric"><span>Done</span><strong>${counts.done || 0}</strong></div><div class="metric"><span>In progress</span><strong>${counts['in-progress'] || 0}</strong></div><div class="metric"><span>Not done</span><strong>${counts['not-done'] || 0}</strong></div><div class="metric"><span>Lane reports</span><strong>${lanes.length}</strong></div></div></aside>
</section>
<section class="panel"><h2>Capability Snapshot</h2>${bars.map(([label, value]) => meter(value, label)).join('\n')}</section>
<section><div class="toolbar"><h2 id="roadmap">Active Roadmap Estimates</h2><input class="search" type="search" placeholder="Filter roadmap" data-filter="#roadmap-table"></div><div class="table-wrap"><table id="roadmap-table" class="roadmap"><thead><tr><th>Sub-task</th><th>Progress</th><th>Current read</th><th>Related lane pages</th></tr></thead><tbody>${roadmapHtml}</tbody></table></div></section>
<section><h2>Current Primary State</h2><details open><summary>Integrated baseline and resource notes</summary><ul>${primary.map((x) => `<li>${inline(x)}</li>`).join('\n')}</ul></details><details><summary>Done, in progress, and not done</summary><ul>${checks.map((x) => `<li><span class="tag ${x.state === 'done' ? 'good' : x.state === 'in-progress' ? 'mixed' : 'early'}">${esc(x.state)}</span> ${inline(x.text)}</li>`).join('\n')}</ul></details><details><summary>Lane-local candidates not yet counted</summary><ul>${candidates.map((x) => `<li>${inline(x)}</li>`).join('\n')}</ul></details><details><summary>Near-term steering</summary><ol>${steering.map((x) => `<li>${inline(x)}</li>`).join('\n')}</ol></details><div class="toolbar"><button type="button" data-expand-all>Expand all sections</button><button type="button" data-collapse-all>Collapse all sections</button></div></section>
<section><h2>Sub-task Documents</h2><div class="tasks">${rows.map((row) => `<article class="card"><h3><a href="subtasks/${row.slug}.html">${esc(row.item)}</a></h3>${mini(row.primary, 'Primary')}${mini(row.lane, 'Lane-local')}</article>`).join('\n')}</div></section>
<section><h2>Recent Primary-Integrated Work</h2>${commits.map((c, i) => `<details ${i < 2 ? 'open' : ''}><summary><a href="${REPO}/commit/${c.hash}"><code>${esc(c.hash)}</code></a> ${esc(c.title)}</summary><p>${inline(c.detail)}</p></details>`).join('\n')}</section>
<section><div class="toolbar"><h2 id="lanes">Read-only Lane Reports</h2><input class="search" type="search" placeholder="Filter lanes" data-filter="#lane-table"></div><div class="table-wrap"><table id="lane-table" class="lane-table"><thead><tr><th>Lane</th><th>Branch</th><th>Head</th><th>Latest log date</th><th>Recent note</th></tr></thead><tbody>${laneHtml}</tbody></table></div></section>
<section><h2>Source Documents</h2><div class="docs"><a class="card" href="source/progress.html"><strong>Canonical progress report</strong><br><span class="note">Generated from root PROGRESS.md.</span></a><a class="card" href="source/roadmap.html"><strong>Roadmap</strong><br><span class="note">Milestone framing.</span></a><a class="card" href="source/cow-coverage-matrix.html"><strong>COW coverage matrix</strong><br><span class="note">Copy-on-write audit.</span></a><a class="card" href="source/next-tasks.html"><strong>Next tasks excerpt</strong><br><span class="note">Queue excerpt with GitHub link to the full file.</span></a><a class="card" href="source/support.html"><strong>Support excerpt</strong><br><span class="note">Support-boundary excerpt.</span></a><a class="card" href="source/architecture.html"><strong>Architecture excerpt</strong><br><span class="note">Pipeline and runtime notes.</span></a></div></section>`;

write('index.html', shell('PHP Native Compiler Progress', index));
write('404.html', shell('PHP Native Compiler Progress', index));
write('.nojekyll', '');
write('README.md', `# PHP Native Compiler Progress Pages

Static GitHub Pages report for ${REPO}.

Regenerate locally with:

\`\`\`sh
SOURCE_REPO=/home/claude/php-to-native-compiler LANE_ROOT=/home/claude node tools/build-site.mjs
\`\`\`
`);

for (const row of rows) {
  const ms = matched(row);
  const body = `<section class="source"><p><a href="../index.html#roadmap">Back to dashboard</a></p><span class="eyebrow">Sub-task report</span><h1>${esc(row.item)}</h1><div class="meta"><div class="metric"><span>Primary-integrated</span><strong>${row.primary}%</strong><progress max="100" value="${row.primary}">${row.primary}%</progress></div><div class="metric"><span>Lane-local candidate maturity</span><strong>${row.lane}%</strong><progress max="100" value="${row.lane}">${row.lane}%</progress></div><div class="metric"><span>Status class</span><strong>${row.primary >= 70 ? 'Strong primary path' : row.primary >= 35 ? 'Partial primary path' : 'Early primary path'}</strong></div></div><h2>Current Read</h2><p>${inline(row.read)}</p><h2>Related Source Documents</h2><ul>${row.docs.map((doc) => `<li><a href="../${doc}">${esc(doc.replace('source/', '').replace('.html', ''))}</a></li>`).join('\n')}</ul><h2>Related Lane Pages</h2>${ms.length ? `<ul>${ms.map((lane) => `<li><a href="../lanes/${lane.slug}.html">${esc(lane.name)}</a> <span class="note">${esc(lane.branch)} @ ${esc(lane.head)}</span></li>`).join('\n')}</ul>` : '<p class="note">No lane progress page matched this sub-task by keyword.</p>'}<details open><summary>Canonical active-estimate row</summary><p>${inline(row.read)}</p></details></section>`;
  write(`subtasks/${row.slug}.html`, shell(`${row.item} - PHP Native Compiler Progress`, body, 1));
}

for (const lane of lanes) {
  const body = `<section class="source"><p><a href="../index.html#lanes">Back to lane table</a></p><span class="eyebrow">Lane-local report</span><h1>${esc(lane.name)}</h1><div class="meta"><div class="metric"><span>Branch</span><strong>${esc(lane.branch)}</strong></div><div class="metric"><span>Head</span><strong><code>${esc(lane.head)}</code></strong></div><div class="metric"><span>Latest log date</span><strong>${esc(lane.date)}</strong></div></div><p class="note">Generated from the latest dated section of the lane progress log. Lane-local work is candidate material until selected, gated, committed, and pushed to the primary branch.</p><h2>Recent Lane Notes</h2>${markdown(lane.md, { shift: 1 })}</section>`;
  write(`lanes/${lane.slug}.html`, shell(`${lane.name} - PHP Native Compiler Progress`, body, 1));
}

const sourceDocs = [
  ['PROGRESS.md', 'source/progress.html', 'Canonical Progress Report', 0],
  ['docs/ROADMAP.md', 'source/roadmap.html', 'Roadmap', 0],
  ['docs/COW_COVERAGE_MATRIX.md', 'source/cow-coverage-matrix.html', 'COW Coverage Matrix', 0],
  ['docs/NEXT_TASKS.md', 'source/next-tasks.html', 'Next Tasks Excerpt', 450],
  ['docs/SUPPORT.md', 'source/support.html', 'Support Excerpt', 600],
  ['docs/ARCHITECTURE.md', 'source/architecture.html', 'Architecture Excerpt', 600],
];

for (const [rel, out, title, maxLines] of sourceDocs) {
  const body = `<section class="source"><p><a href="../index.html">Back to dashboard</a> | <a href="${REPO}/blob/master/${rel}">Open on GitHub</a></p><span class="eyebrow">Source document</span><h1>${esc(title)}</h1>${markdown(src(rel), { shift: 1, maxLines })}</section>`;
  write(out, shell(`${title} - PHP Native Compiler Progress`, body, 1));
}

write('source/PROGRESS.md', progress);
console.log(`Generated ${rows.length} sub-task pages, ${lanes.length} lane pages, and ${sourceDocs.length} source pages.`);
