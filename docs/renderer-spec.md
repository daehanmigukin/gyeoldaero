# gyeoldaero — Renderer Specification

What a rendered gyeoldaero site looks like and how it behaves. The approved mockup at
`docs/reference/mockup.html` is the behavioral reference: when this document is silent,
match the mockup. When this document and the mockup disagree, this document wins.

The mockup is a single page with all notes embedded in JavaScript. The real renderer
produces **one static HTML page per note** and adds the same behavior through progressive
enhancement. Copy its design tokens, styles and interaction logic; do not copy its
data-in-JS architecture.

---

## 1. Mental model

Every note is a place. A page shows one note: its own text, then the notes inside it as a
list of titles whose text is collapsed. From a page you can:

- **Read in place**: expand a child to read its body without leaving the page.
- **Step in**: open a child that has notes of its own. Its page replaces the current one.
- **Step out**: go back to the parent page.

Moving in and out should feel like walking directories in yazi.

## 2. Page anatomy

```
┌──────────────────────────────────────────┐
│ ‹  notebook / cpu-architecture / caches  │  path bar (fixed)
├──────────────────────────────────────────┤
│ cpu-architecture/caches/index.md         │  eyebrow: source file
│ caches                                   │  heading (title or slug)
│ Small, fast memory close to the core…    │  body
│                                          │
│ 3 NOTES INSIDE                           │  list heading
│ ──────────────────────────────────────── │
│ › cache-coherence                        │  child row (collapsed, leaf)
│ ──────────────────────────────────────── │
│ ⌄ false-sharing                          │  child row (expanded)
│     Two threads write different…         │    child body
│ ──────────────────────────────────────── │
│ › replacement-policies          (2) ›    │  child row with notes inside
├──────────────────────────────────────────┤
│ Tap a title to read · …     [Flip swipe] │  hint bar (fixed)
└──────────────────────────────────────────┘
```

### Required elements

| Element | Content |
|---|---|
| Back button `‹` | Link to the parent page. Disabled (not a link) on the root page. |
| Path bar | Root name, then each ancestor slug, `/`-separated. Every segment except the current one links to its page. Scrolls horizontally when long, kept scrolled to the end. |
| Eyebrow | `<path>/index.md`, or `~/<root name>/index.md` on the root page. Monospace, muted. |
| Heading | `title` from frontmatter, else the slug. Monospace. |
| Body | Rendered Markdown. Readable sans. Max ~62ch line length. |
| List heading | `N note(s) inside`. Omitted for leaves, replaced by "No notes inside this one yet." |
| Child row | Caret, title, and, when the child has children, an open control showing the count and `›`. |
| Child body | The child's rendered body, collapsed by default. |
| Hint bar | Short instructions that match the current swipe direction, plus the Flip swipe toggle. Rendered by JS only (hidden without JS). |

## 3. Static HTML (no JavaScript)

Without JavaScript the site must be fully usable:

- Each child row is a `<details>` element; the title is its `<summary>`. Expanding works
  natively.
- The open control is a real `<a href="<child>/">`. The back button and path segments are
  real links.
- Wiki links are real relative links.
- Nothing in the layout depends on JS to be visible. The hint bar is the only JS-only part.

Suggested markup (classes are part of the contract with `gyeoldaero.js` and tests):

```html
<body class="gyeoldaero" data-path="cpu-architecture/caches">
  <header class="top">
    <a class="back" href="../" aria-label="Go to parent note">‹</a>
    <nav class="crumbs" aria-label="Path">
      <a class="crumb" href="../../">notebook</a><span class="sep">/</span>
      <a class="crumb" href="../">cpu-architecture</a><span class="sep">/</span>
      <span class="crumb cur" aria-current="page">caches</span>
    </nav>
  </header>
  <main class="stage">
    <article class="view" data-path="cpu-architecture/caches">
      <div class="inner">
        <p class="eyebrow">cpu-architecture/caches/index.md</p>
        <h1>caches</h1>
        <div class="prose">…</div>
        <p class="eyebrow list-head">3 notes inside</p>
        <ul class="rows">
          <li class="row has-kids" data-path="cpu-architecture/caches/replacement-policies">
            <details>
              <summary class="toggle"><span class="caret"></span><span class="t">replacement-policies</span></summary>
              <div class="body"><div class="prose">…</div></div>
            </details>
            <a class="enter" href="replacement-policies/" aria-label="Open replacement-policies, 2 notes inside">
              <span class="n">2</span><span class="chev" aria-hidden="true">›</span>
            </a>
          </li>
        </ul>
      </div>
    </article>
  </main>
  <footer class="bar" hidden>…</footer>
  <script src="../../_gyeoldaero/gyeoldaero.js?v=…" defer></script>
</body>
```

The open control sits beside the summary visually (same row), even though it is outside
`<details>` in the DOM; lay it out with CSS grid on `.row`. When JS is active it may
replace `<details>` behavior with its own animated expand/collapse, but the DOM must stay
valid without it.

## 4. Enhanced behavior (JavaScript)

`gyeoldaero.js` is one small dependency-free script, loaded with `defer`. No inline scripts
(the server's CSP forbids them).

### 4.1 Navigation router

- Intercept clicks on internal links that point to note pages (`.enter`, `.back`,
  `.crumb`, wiki links). Fetch the target page, extract its `.view` and path bar, and
  animate the swap. Update history with `pushState` so the URL, reload and the browser's
  back button all work. On `popstate`, perform the matching animated transition.
- Direction: a target deeper than the current path is **in**; a target that is an ancestor
  is **out**; anything else (a wiki link sideways) is **in**.
- If a fetch fails, fall back to a normal navigation.
- Prefetch the parent page and the pages of visible children with children when idle, so
  the back-swipe peek and openings are instant.

### 4.2 Transitions

Match the mockup:

| Direction | Old view | New view |
|---|---|---|
| in | slides 30% toward the opening side, fades to 0.5 | slides in from the opposite edge to 0 |
| out | slides fully off toward the back side | starts 30% offset at 0.5 opacity, slides to 0 |

Duration 280 ms, easing `cubic-bezier(.2,.7,.2,1)`. The view on top casts a soft shadow.
With `prefers-reduced-motion: reduce`, swap instantly with no animation.

### 4.3 Gestures

Gestures use Pointer Events on the stage; views use `touch-action: pan-y` so vertical
scrolling stays native.

- **Axis lock**: a gesture becomes horizontal once `|dx| > 10px` and `|dx| > 1.3 × |dy|`.
  If `|dy| > 10px` first, it is a scroll and is ignored.
- **Open (row swipe)**: a horizontal drag in the *open direction* starting on a row that
  has children drags that row, revealing an "open ›" panel behind it. Rubber-band beyond
  90 px (`90 + (d − 90) × 0.35`). Release past 70 px opens the note; otherwise the row
  springs back over 200 ms. Rows without children don't move.
- **Back (page swipe)**: a horizontal drag in the *back direction* anywhere on a non-root
  page drags the whole view, with the parent page peeking from behind (starting 30%
  offset, 0.5 opacity, both interpolated with progress). Release past
  `min(110px, 28% of width)` completes the back navigation from the current offset;
  otherwise the view springs back and the peek is discarded.
- **Directions**: by default the open direction is a leftward finger movement and back is
  rightward. **Flip swipe** reverses both, and also mirrors the transition animations and
  the reveal panel's alignment. The choice is stored in `localStorage` (guarded with
  try/catch) and reflected in the hint text and `aria-pressed` on the toggle.
- A completed drag suppresses the click that follows it.

### 4.4 Expand and collapse

- Tapping a title toggles its body with a height animation (grid-rows `0fr → 1fr`,
  240 ms) and rotates the caret. Collapsed content is not focusable.
- Expanded state is remembered per page for the session (`sessionStorage`, keyed by note
  path), so stepping out and back restores it.

### 4.5 Returning to a parent

- Scroll position of each page is remembered for the session and restored when returning.
- After stepping out, the row of the note you came from is scrolled into view if needed
  and briefly highlighted (accent-soft background fading over ~1.4 s).

### 4.6 Keyboard (yazi-style)

| Key | Action |
|---|---|
| `j` / `↓` | Move focus to next row |
| `k` / `↑` | Move focus to previous row |
| `space` / `Enter` | Expand or collapse the focused row (when focus isn't on a link or button) |
| `l` / `→` | Open the focused row if it has children |
| `h` / `←` | Go to parent |

Ignore keys with Ctrl/Meta/Alt held, and while focus is in a text field. The focused row
gets a surface-colored background.

### 4.7 Wiki links

A resolved wiki link navigates to the target note's page using the router (direction
**in**). A broken link renders as plain text with a dotted underline in the muted color and
`title="Missing note: <path>"`.

## 5. Visual design

Copy the token block and component styles from the mockup into `gyeoldaero.css`. Tokens:

| Token | Light | Dark | Use |
|---|---|---|---|
| `--bg` | `#EEF1EF` | `#121615` | Page ground |
| `--surface` | `#F8FAF9` | `#1A201E` | Inline code, focused row, toggle button |
| `--fg` | `#1A201F` | `#E2E8E6` | Text |
| `--muted` | `#5A6764` | `#8D9A96` | Eyebrows, carets, hints |
| `--line` | `#D1D9D6` | `#29312F` | Hairlines |
| `--accent` | `#2350B8` | `#8FAEF6` | Links, open controls |
| `--accent-soft` | `#DCE4F6` | `#1E2944` | Badges, reveal panel, came-from flash |
| `--shadow` | `rgba(20,30,28,.12)` | `rgba(0,0,0,.45)` | Top view during transitions |
| `--ui` | Atkinson Hyperlegible Next, fallback system sans | | Reading text |
| `--mono` | JetBrains Mono, fallback ui-monospace | | Paths, slugs, headings, badges |

- Dark mode follows `prefers-color-scheme`. All component rules use tokens only, so a
  theme only has to replace the token block.
- **Fonts are bundled** as woff2 files under `_gyeoldaero/fonts/` with their OFL license files.
  No Google Fonts requests.
- Layout: one column, max width 680 px, side gutter `max(16px, env(safe-area-inset-*))`.
  Path bar and hint bar fixed; the stage scrolls. Works from 320 px wide up.
- Touch targets at least 40 px tall. Visible `:focus-visible` outlines in the accent color.

## 6. Accessibility

- Semantic structure: one `h1` per page, `nav` for the path, `aria-current="page"` on the
  current segment, real buttons and links.
- Open controls have an `aria-label` naming the note and its child count.
- Every gesture has a tap or key equivalent.
- Text contrast meets WCAG AA in both themes.

## 7. Acceptance checklist (used by PRD-002 and PRD-003)

- [ ] Every note in the fixture notebook has a page at its path, with correct path bar,
      eyebrow, heading, body and children in sibling order.
- [ ] Without JS: expand via `<details>`, open via links, back via links all work.
- [ ] With JS: expand/collapse animates; open via control, row swipe and `l`; back via
      button, page swipe, `h`, path segments and browser back.
- [ ] Back swipe shows the parent peeking and completes from the release offset.
- [ ] Flip swipe reverses gestures, animations and reveal alignment, and persists.
- [ ] Expanded rows and scroll position are restored when stepping back out; the
      came-from row flashes.
- [ ] Reduced motion disables all animation.
- [ ] No requests leave the server; no inline scripts; no console errors.
- [ ] Light and dark themes match the mockup.
