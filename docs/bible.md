# 결대로 gyeoldaero — Project Bible

This is the source of truth for what gyeoldaero is and why it exists. Every other document
(architecture, renderer spec, PRDs) must agree with this one. If they conflict, this file
wins, and the conflict is a bug to raise with Ellie. Only Ellie changes this file.

---

## Tagline and description

**English**

> **gyeoldaero** — find the grain of what you're learning.
>
> gyeoldaero is a terminal notebook for taking a subject apart and putting it back together your
> own way. Every note is a folder, and every piece lives next to, above, or below the ones it
> belongs with. Over time the pieces line up and you start to see the grain of the whole:
> where one idea splits off and where it runs into the next. Write in neovim, browse in yazi,
> and read it anywhere from a server you run yourself.

**한국어**

> **결대로** — 배움의 결을 따라가는 노트
>
> 결대로는 공부한 것을 잘게 나누고, 내 방식대로 다시 엮는 터미널 노트예요. 모든 노트는 폴더가
> 되고, 조각들이 차곡차곡 쌓이다 보면 어느새 전체의 결이 보여요. 생각이 어디서 갈라지고
> 어디로 이어지는지가요. neovim으로 쓰고 yazi로 둘러보고, 직접 돌리는 서버 덕분에
> 휴대폰에서도 언제든 읽을 수 있어요.

### The name

결대로 means "along the grain," as in 나무를 결대로 쪼개다: splitting wood along its grain,
so it comes apart cleanly along its natural lines instead of being forced. That is what
the project is for: breaking a subject apart along its own lines, then seeing the grain
of the whole.

The name is written `gyeoldaero` in code, crates, binaries, config files and URLs, and
`결대로` or `gyeoldaero` in prose. The binary is `gyeoldaero`; the README suggests short
shell aliases for everyday use.

---

## The idea

Studying something properly means breaking the source material into small pieces and
placing each piece where it belongs: under the idea it grew from, next to its siblings,
above the details that elaborate it. The person studying does this work themselves. That
work is the point; gyeoldaero does not organize notes for you.

gyeoldaero makes that structure literal. **Every note is a folder.** The folder's `index.md` is
the note's own text, and its subfolders are the notes that grow out of it. The filesystem
tree *is* the structure of your understanding.

Because the structure is just folders and Markdown, you keep your own tools: neovim with
your own dotfiles to write, yazi (or any file manager) to walk the tree. gyeoldaero adds the
part those tools can't do: it compiles the tree into a site you can read from anywhere,
where moving through notes feels exactly like walking directories.

## Who it's for

- First user: Ellie. A developer who writes in neovim, navigates with yazi, self-hosts on a
  home server reached over Tailscale, and studies technical and legal material in depth.
- Open-source audience: terminal-native people who want plain-file notes, a hierarchical
  way of thinking, and a reading view on their phone, without a cloud service.

## How it's used (the reference setup)

```
laptop: yazi + nvim  ──edit──▶  ~/notes (plain folders)
                                    │ Syncthing
                                    ▼
server (watchtower): ~/notes ──gyeoldaero serve──▶ HTML site ──Tailscale──▶ phone / any browser
```

gyeoldaero does not do sync. Syncthing, rsync, git, or a network mount are all fine. gyeoldaero only
needs a folder it can watch.

---

## Principles

1. **Plain files forever.** A notebook is folders and Markdown. No database, no lock-in,
   no required metadata. Deleting gyeoldaero leaves a perfectly readable notebook.
2. **The tree is the structure.** Hierarchy comes from folders. gyeoldaero never invents
   structure the user didn't create.
3. **Bring your own editor.** gyeoldaero never edits note content in a UI. Writing happens in
   the user's editor.
4. **Reading should feel like walking.** The web view mirrors directory navigation: step
   into a note to see what's inside it, step back out to its parent. See
   `renderer-spec.md`.
5. **Works without JavaScript.** Every page is a real static HTML page with real links.
   JavaScript adds swipes, animation and keyboard control on top.
6. **Self-hosted and private by default.** No external requests from rendered pages (fonts
   are bundled). The server binds to localhost unless told otherwise.
7. **Small and boring inside.** One binary, few dependencies, predictable behavior. Prefer
   rebuilding a little too much over clever caching that can go stale.

## Non-goals (v1)

- Editing notes in the browser.
- Syncing files between machines.
- Accounts, multi-user, permissions. Access control is the network's job (Tailscale).
- Graph views, tags, search, AI features. Some may come later; none are v1.
- Gemini/gemtext output. Considered and dropped in favor of HTML; may return as an export.

---

## Core concepts (glossary)

| Term | Meaning |
|---|---|
| **Notebook** | The root folder gyeoldaero builds. Contains a `gyeoldaero.toml` (optional) and notes. |
| **Note** | Any folder inside the notebook (including the root). Its text is `index.md`. |
| **Body** | The Markdown content of a note's `index.md`. May be empty or missing. |
| **Child** | A note directly inside another note's folder. |
| **Slug** | A note's folder name without its order prefix, e.g. `cache-coherence`. |
| **Order prefix** | Optional `NNN-` at the start of a folder name that sets sibling order, e.g. `020-caches`. |
| **Path** | Slash-separated slugs from the root, e.g. `cpu-architecture/caches`. |
| **Link** | A wiki link in a body, `[[cpu-architecture/caches]]`, pointing to a note by path. |
| **Page** | One rendered HTML file per note: the note's body plus its children, collapsed. |
| **Asset** | A non-Markdown file in a note folder (e.g. an image) that the note can reference. |

---

## Decisions already made

- Language: **Rust**, as a Cargo workspace producing one binary, `gyeoldaero`.
- License: **MIT**.
- Hosting: **GitHub**, public.
- v1 scope: the compile server (`gyeoldaero serve` / `gyeoldaero build`) plus commands to create
  notes (`gyeoldaero new`). A yazi plugin is a later PRD.
- Output: HTML only. The interaction model is defined by the approved mockup in
  `docs/reference/mockup.html` and specified in `docs/renderer-spec.md`.
- Visual design: the mockup's design (cool grey-green ground, ink-blue accent, JetBrains
  Mono for structure, Atkinson Hyperlegible Next for reading). All colors and fonts are
  CSS custom properties so a theming feature can be added later without restructuring.

## Open questions (Ellie decides)

- Backlinks on pages: in v1 or later?
- Live reload of open pages when a note changes: in v1 or later?
- Should `gyeoldaero new` default to adding order prefixes?
