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

Content order: language positioning (hook), four feature rows with source/output
(explain and prove), quickstart build/run (enable), grouped curriculum
(navigate), VM/reference distinction and runtime status (qualify), frozen
specification (reference). No download, online execution, toolchain compatibility,
production-readiness or toolchain-release claim. Compatible class revisions
describe bounded language semantics, not a released implementation guarantee.

The approved homepage feature introduction extends this baseline, not its visual
direction. Keep the warm-paper palette, editorial typography, existing actions
and teaching-sheet vocabulary. The positioning introduces objects and messages
and dynamic behavior bounded by static promises before setup instructions.

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
hero, feature rows and curriculum groups stack. At 375px no main-content horizontal scroll.
DOM order follows reading order. No hidden content to improve performance.

Feature introduction adopts [StyleGallery stack](https://github.com/changeroa/StyleGallery/blob/main/patterns/stacking/stack.md)
for four repeated vertical editorial rows, not a card dashboard, carousel or
alternating zigzag. The document owns vertical scroll; rows have natural height.
Within each row, prose precedes an adjacent example in two equal
`minmax(0, 1fr)` columns above the existing 760px breakpoint. At or below it,
stack prose then example with `--space-5` between them; desktop gap is
`--space-7`. Reuse page width and gutters, `--space-7` row padding and thin
`--color-border` separators. Keep children shrinkable and prose within
`--measure`; only source/output blocks may scroll horizontally. No fixed-height
rows, clipping, truncation or page-level overflow to fit longer Chinese copy.

## 5. Components

| Primitive | Structure and variants | States and accessibility |
| --- | --- | --- |
| Header | brand link, learning/spec/status links, language/theme buttons | active collection, named toggles, visible focus, wrap on small screens |
| Action | primary anchor, text link, ghost button; space-3/4 | hover underline or surface, pressed contrast, focus ring; no dummy destinations |
| Section heading | kicker, h2, short description | semantic hierarchy; balanced text |
| Teaching sheet | figure, source pre/code, labeled output, caption | selectable real text, no fake editor controls; source scrolls if needed |
| Feature row | article with heading, short prose, tutorial text link and teaching-sheet example variant | four rows in fixed reading order; semantic heading/figure labels, selectable source/output, visible backend text; no whole-row click target |
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

### Homepage feature rows

| Order / group | Explanation boundary | Tutorial route stem |
| --- | --- | --- |
| 1 / Objects and messages | Values are objects; behavior is expressed through message sends. | `01-getting-started` |
| 2 / Dynamic dispatch and static promises | Dispatch remains runtime-dynamic within declared promises; annotations do not introduce static overload dispatch. | `07-types-and-generics` |
| 3 / Class, Module and Contract composition | Single Class inheritance, Module reuse and explicit Contract promises work together. | `06-modules-and-contracts` |
| 4 / Compatible atomic class revisions | Compatible candidate changes publish atomically or roll back; this is not unrestricted mutation or a toolchain compatibility promise. | `09-dynamic-and-static` |

Each row pairs concise translated explanatory prose with one approved complete
example (two stacked figures for Module reuse and Contract promises in the
composition row), its actual output, an explicit backend label and a localized tutorial
link via the existing reader route. Reuse section-heading hierarchy: one feature
section h2 and row h3 titles using the display stack at `--step-2`; body uses
`--step-0`, code `--step--1`, labels `--step--2`, and existing text-link styling.
Apply the existing Chinese display/body stacks and normal tracking to row copy.

The example variant composes teaching-sheet source, output, backend and caption
parts without repeating the hero's shadow or offset paper edge. Use existing
panel/surface colors, syntax tokens, borders, radii and spacing; do not add a
palette, type scale or visual-effect token. Source and output remain visible
together as selectable `pre/code` text, not tabs, screenshots or a fake editor.
No run, copy or backend-switching control is introduced.

Consume the separately approved shared snippet records rather than settling
source or expected output in this contract. Both locales use identical English
source, output and backend values; translate prose, captions, visible field
labels and link text. Each example figure carries its shared record's stable
`data-example-id`, unchanged across locales, themes and presentation order.
That machine label does not replace an accessible figure name or visible
source/output/backend labels. Backend claims must match the snippet's execution
evidence; reference-evaluator success is not VM support.

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

Feature-row reading and focus order must follow prose, tutorial link, then
example. Give horizontally overflowing source/output blocks keyboard access
and localized accessible names; backend meaning must remain explicit without
color. Verify all four rows, translated text, both themes and tutorial targets
at 375/768/1280 and around the existing 760px stacking boundary after UI work.

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

The feature introduction is implemented in the homepage files, with shared
snippet source/output checked against bilingual tutorial metadata by regression tests.
Reported prior snapshot captures were corrupt even for the control capture.
They cannot establish either visual regressions or a visual pass. Fresh usable
rendered evidence is required before visual sign-off; this contract update
makes no screenshot, accessibility-audit or visual-pass claim.
