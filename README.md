# 결대로 gyeoldaero

*결대로: "along the grain," the way wood splits cleanly when you follow its lines.*

**find the grain of what you're learning.**

gyeoldaero is a terminal notebook for taking a subject apart and putting it back together your
own way. Every note is a folder, and every piece lives next to, above, or below the ones it
belongs with. Over time the pieces line up and you start to see the grain of the whole:
where one idea splits off and where it runs into the next. Write in neovim, browse in yazi,
and read it anywhere from a server you run yourself.

**배움의 결을 따라가는 노트.**

'결대로'는 공부한 것을 잘게 나누고, 내 방식으로 다시 엮는 터미널 노트예요. 모든 노트는 폴더가 되고, 조각들이 차곡차곡 쌓이다 보면 어느새 전체의 결이 보여요. 생각이 어디서 갈라지고 어디로 이어지는지가요. neovim으로 쓰고 yazi로 둘러보고, 직접 돌리는 서버 덕분에 휴대폰에서도 언제든 읽을 수 있어요.

> **Status: early development.** Only `gyeoldaero check` works so far. Follow the plan
> in [`docs/prd/`](docs/prd/README.md).

## How it works

```
notes/
├── index.md                      ← the notebook's own note
├── cpu-architecture/
│   ├── index.md                  ← "cpu-architecture"
│   ├── 010-caches/
│   │   ├── index.md
│   │   └── cache-coherence/
│   │       └── index.md
│   └── 020-pipelining/
│       └── index.md
└── inbox/
    └── index.md
```

- A note is a folder; its text is `index.md`; the folders inside it are its child notes.
- An optional number prefix (`010-`) sets the order of siblings.
- `gyeoldaero serve` turns the folder into a site where each page shows one note and the notes
  inside it, collapsed. Tap to read in place, open a note to step into it, swipe back to
  step out, like walking directories in yazi.
- Plain files only. Sync them however you like (Syncthing, git, rsync).

## Commands

```sh
gyeoldaero check [path]      # report problems in a notebook (available now)
gyeoldaero new <slug>        # create a note (folder + index.md) here (planned)
gyeoldaero build [path]      # write the static site (planned)
gyeoldaero serve [path]      # build, watch and serve (planned)
```

Suggested aliases:

```sh
alias gy='gyeoldaero'
alias t='gyeoldaero new --edit'
```

## Contributing

Development is spec-driven: see [`docs/workflow.md`](docs/workflow.md). Start from the
[bible](docs/bible.md) and the [PRD index](docs/prd/README.md).

## License

MIT © 2026 daehanmigukin
