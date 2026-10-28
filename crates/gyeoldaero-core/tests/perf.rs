//! PRD-001 AC10: loading 5,000 notes takes under a second.
//!
//! Ignored by default; run in release:
//! `cargo test --release -p gyeoldaero-core --test perf -- --ignored --nocapture`

mod common;

use std::time::{Duration, Instant};

use common::{TempNotebook, all_notes, show};

// PRD-001 AC10
#[test]
#[ignore = "performance check; run with --release -- --ignored"]
fn loads_5000_notes_in_under_a_second() {
    let nb = TempNotebook::new();
    // 50 topics × 100 notes = 5,000 notes plus 50 topics and the root.
    for t in 0..50 {
        let topic = format!("{t:03}-topic-{t}");
        nb.note(
            &topic,
            &format!("---\ntitle: Topic {t}\ntags: [perf]\n---\nTopic {t}. See [[topic-0]].\n"),
        );
        for n in 0..100 {
            nb.note(
                &format!("{topic}/note-{n}"),
                &format!(
                    "---\ntitle: Note {t}.{n}\n---\n# Note {n}\n\nSome text with `code` and a \
                     link to [[../note-0]] and [[topic-{t}/note-{n}|myself]].\n\n```\n[[not-a-link]]\n```\n"
                ),
            );
        }
    }

    let start = Instant::now();
    let (notebook, diags) = nb.load();
    let elapsed = start.elapsed();

    eprintln!("loaded {} notes in {elapsed:?}", all_notes(&notebook).len());
    assert!(
        diags.is_empty(),
        "{:?}",
        &show(&diags)[..diags.len().min(5)]
    );
    assert_eq!(all_notes(&notebook).len(), 5_051);
    assert!(
        elapsed < Duration::from_secs(1),
        "loading 5,000 notes took {elapsed:?}"
    );
}
