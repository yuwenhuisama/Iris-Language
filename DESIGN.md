# Iris Learning Entrance Design System

## 1. Atmosphere & Identity

An editorial language handbook, not a product launch. Warm paper, dark ink,
bookish display type and precise blue links frame a tangible source/output
sheet. The memorable moment is seeing one complete Iris program beside its
actual result, then discovering exactly how to build and run it.

This is an approved redesign of the existing vanilla site. The extracted
system already had semantic color variables, a four-pixel spacing family,
fluid type steps, shared buttons, chapter links, syntax colors, Markdown
panels, table wrappers and two persisted themes. Those contracts remain;
glowing panels, speculative feature examples, excessive cards and obsolete
implementation claims do not. Reviewed primitives: header, brand, buttons,
hero, code window, chapter cards, chapter navigation, reader, TOC, tables,
translation notices and errors.

Content order: introduce (hook), source/output (prove), build/run (enable),
grouped curriculum (navigate), VM/reference distinction (explain), frozen
specification (reference). No download, online execution, compatibility,
production-readiness or toolchain-release claim.

## 2. Color

| Role / CSS token | Light | Dark |
| --- | --- | --- |
| `--color-canvas` | `#f7f5ef` | `#171b22` |
| `--color-panel` | `#fffdf8` | `#202630` |
| `--color-surface` | `#eeece4` | `#29313d` |
| `--color-text` | `#242a31` | `#f1eee6` |
| `--color-text-muted` | `#47515b` | `#d4d9df` |
| `--color-text-subtle` | `#606974` | `#adb8c5` |
| `--color-accent` | `#2459a6` | `#9ac3ff` |
| `--color-accent-strong` | `#174680` | `#c0d9ff` |
| `--color-on-accent` | `#ffffff` | `#172334` |
| `--color-success` | `#27674a` | `#a0d7b8` |
| `--color-warning` | `#795519` | `#e1bd76` |
| `--color-danger` | `#a33336` | `#ffa5a5` |
| `--color-syntax-keyword` | `#2459a6` | `#9ac3ff` |
| `--color-syntax-type` | `#765530` | `#e0bd88` |
| `--color-syntax-string` | `#27674a` | `#a0d7b8` |
| `--color-syntax-symbol` | `#765530` | `#e0bd88` |
| `--color-syntax-number` | `#795519` | `#e1bd76` |
| `--color-syntax-comment` | `#606974` | `#adb8c5` |

Derived tokens: border uses text at 16% opacity, subtle border 9%, strong
border 28%; accent wash uses accent at 7%. Code background equals surface.
Syntax operator aliases accent, caption/faint aliases subtle. RGB companion
tokens exist only to construct these alpha surfaces. Light is the default;
the stored theme wins. Status always has a text label, never color alone.

## 3. Typography

Display: `Iowan Old Style`, `Palatino Linotype`, `Book Antiqua`, Georgia,
serif. Body: `Avenir Next`, Avenir, `Segoe UI`, sans-serif. Mono:
`SFMono-Regular`, Consolas, `Liberation Mono`, monospace. Chinese body:
`PingFang SC`, `Microsoft YaHei`, sans-serif; Chinese display:
`Songti SC`, `Noto Serif CJK SC`, `SimSun`, serif. System-local stacks avoid
font requests and retain a deliberate serif/sans/code distinction.

| Token | Size | Use |
| --- | --- | --- |
| `--step--2` | .75rem | metadata |
| `--step--1` | .875rem | navigation, code, captions |
| `--step-0` | 1rem to 1.0625rem | body |
| `--step-1` | 1.125rem to 1.25rem | lead, h3/h4 |
| `--step-2` | 1.5rem to 1.875rem | h2 |
| `--step-3` | 2rem to 3rem | section titles |
| `--step-4` | 3.25rem to 5rem | hero |

Body line-height 1.75, Chinese 1.9; heading line-height 1.15 (hero 1.06).
Display weight 400, control weight 600. Display tracking -.035em; Chinese
tracking normal. Headings balance; prose wraps naturally. Never split source
code into arbitrary word fragments. Body measure is 68ch.

## 4. Spacing & Layout

Retain `--space-1` through `--space-9`: 4, 8, 12, 16, 24, 32, 48, 64,
96px. Layout width 1200px; reader width 1440px. Gutters fluid 16-48px.
Header 80px wide, 128px below 640px. Controls at least 44px tall. Border
width 1px, focus width 2px and offset 3px; logo 32px. Reader rails 208px
and 184px. Main content uses `minmax(0, 1fr)`.

Adopt [StyleGallery sticky-aside](https://github.com/changeroa/StyleGallery/blob/main/patterns/split-sidebar/sticky-aside.md):
the document owns the article's vertical scroll; aligned, sticky rails
provide navigation. Rails alone may scroll within the viewport when their
link lists exceed it. Code and tables own horizontal overflow, not the page.
At 1100px, TOC becomes an inline disclosure above the article; below 900px,
chapter navigation becomes a keyboard-accessible disclosure. At 760px the
hero and curriculum groups stack. At 375px no main-content horizontal scroll.
DOM order follows reading order. No hidden content to improve performance.

## 5. Components

| Primitive | Structure and variants | States and accessibility |
| --- | --- | --- |
| Header | brand link, learning/spec/status links, language/theme buttons | active collection, named toggles, visible focus, wrap on small screens |
| Action | primary anchor, text link, ghost button; space-3/4 | hover underline or surface, pressed contrast, focus ring; no dummy destinations |
| Section heading | kicker, h2, short description | semantic hierarchy; balanced text |
| Teaching sheet | figure, source pre/code, labeled output, caption | selectable real text, no fake editor controls; source scrolls if needed |
| Command step | numbered heading and pre/code | build then run; instructions remain selectable; no simulated execution |
| Curriculum group | heading, description, ordered chapter links | Start 01-02, Core 03-06, Advanced 07-10; titles and route stems stable |
| Chapter link | number, title, directional mark | hover/focus, current page; full-row link target |
| Spec directory | native details/summary and all fourteen artifact links | native Enter/Space disclosure, expanded content remains accessible |
| Reader | collection/source masthead, article, chapter rail, TOC | loading announcement, missing/fetch-error panel, source link, previous/next |
| Markdown | headings, prose, pre, blockquote, horizontally wrapped table | ordinal IDs and text slugs unchanged; keyboard-scrollable code/tables |
| TOC | details/summary with heading links | desktop open, compact optional; current section and anchors |
| Notice | named aside / role alert for failure | text explanation and recovery navigation; not color-only |

The real homepage/reader form the state harness for reused primitives;
verify focus/hover/current, disclosure open/closed, loading/error, and both
themes at 375/768/1280. No disabled or empty-control variants are needed.

## 6. Motion & Interaction

Quiet, native interaction. No entrance animation, decorative hover or scroll
hijacking. Focus, underlines and surface shifts provide immediate feedback.
Reduced motion uses automatic scrolling; normal anchors may scroll smoothly.
Language/theme persist in the existing storage keys. Reader navigation
invalidates stale requests, including navigation back home. Mobile chapter
disclosure closes on Escape and returns focus to its control. Hidden chapter
navigation must not remain in keyboard order. Anchor and language navigation
retain section identity; normal chapter navigation starts at the top.

## 7. Depth & Surface

Mixed paper treatment: thin rules for directories, tonal inset for code, one
elevated teaching sheet. `--radius-1` 4px, `--radius-2` 8px. Sheet shadow:
`0 2px 4px` text at 4%, `0 20px 48px` text at 8%; a second offset paper edge
uses surface and a border. No gradients, neon glows, browser traffic lights,
background grids, or unrelated decorative imagery. Preserve the Iris logo.

## 8. Accessibility Constraints & Accepted Debt

WCAG 2.2 AA target: 4.5:1 body contrast, visible keyboard focus, 44px primary
controls, semantic landmarks, one document h1, no clipped Chinese glyphs,
no inaccessible hidden rail. Skip link focuses main without changing the
hash route. Error recovery offers a real collection-index link.

Personas: a first-time programmer must identify build/run steps without
reading the spec; a Chinese reader must follow identical chapter paths and
switch language without losing their section; a keyboard/mobile reader must
open/close chapters, use TOC and recover from a missing document.

Accepted constraints: retain the pinned marked CDN and trusted checked-in
Markdown model; no framework or new dependency. The website is static and
requires HTTP for reader fetches. No new accessibility debt is accepted.
Automated scores and independent visual review are reported as measured,
never assumed. Only DESIGN.md, index.html, assets/app.js and assets/style.css
belong to this redesign; tutorial and runtime owners maintain teaching prose.
