//! Folder names: order prefix, slug, comparison key and sibling ordering.

use std::cmp::Ordering;

use unicode_normalization::UnicodeNormalization;

/// Splits a folder name into its order prefix and slug.
///
/// The prefix is 1–4 ASCII digits followed by `-` (`020-caches` → `(Some(20), "caches")`).
/// A name that is only a prefix (`010-`) has no prefix: the whole name is the slug.
pub fn split_prefix(dir_name: &str) -> (Option<u32>, &str) {
    let digits = dir_name.bytes().take_while(u8::is_ascii_digit).count();
    if (1..=4).contains(&digits) {
        if let Some(slug) = dir_name[digits..].strip_prefix('-') {
            if !slug.is_empty() {
                // At most four ASCII digits always fit in a u32.
                if let Ok(order) = dir_name[..digits].parse::<u32>() {
                    return (Some(order), slug);
                }
            }
        }
    }
    (None, dir_name)
}

/// The key slugs and link segments are compared by: NFC, then lowercase.
pub fn key(s: &str) -> String {
    s.nfc().collect::<String>().to_lowercase()
}

/// Natural sort: runs of ASCII digits compare as numbers, everything else as text.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut a, mut b) = (a, b);
    loop {
        match (a.is_empty(), b.is_empty()) {
            (true, true) => return Ordering::Equal,
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            (false, false) => {}
        }
        let (ra, rest_a) = next_run(a);
        let (rb, rest_b) = next_run(b);
        let ord = match (is_digits(ra), is_digits(rb)) {
            (true, true) => {
                let (ta, tb) = (ra.trim_start_matches('0'), rb.trim_start_matches('0'));
                ta.len().cmp(&tb.len()).then_with(|| ta.cmp(tb))
            }
            _ => ra.cmp(rb),
        };
        if ord != Ordering::Equal {
            return ord;
        }
        a = rest_a;
        b = rest_b;
    }
}

fn is_digits(run: &str) -> bool {
    run.bytes().next().is_some_and(|b| b.is_ascii_digit())
}

/// The leading run of digits or of non-digits, and the rest.
fn next_run(s: &str) -> (&str, &str) {
    let digit = is_digits(s);
    let end = s
        .char_indices()
        .find(|(_, c)| c.is_ascii_digit() != digit)
        .map_or(s.len(), |(i, _)| i);
    s.split_at(end)
}

/// Sibling order: prefixed notes first by number, then unprefixed notes by natural sort.
/// Ties break by slug key, then by folder name so the order is always deterministic.
pub fn sibling_cmp(
    (order_a, slug_a, dir_a): (Option<u32>, &str, &str),
    (order_b, slug_b, dir_b): (Option<u32>, &str, &str),
) -> Ordering {
    let by_order = match (order_a, order_b) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    };
    by_order
        .then_with(|| natural_cmp(&key(slug_a), &key(slug_b)))
        .then_with(|| dir_a.cmp(dir_b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_only_is_not_a_prefix() {
        assert_eq!(split_prefix("010-"), (None, "010-"));
        assert_eq!(split_prefix("-x"), (None, "-x"));
    }

    #[test]
    fn natural_sort_handles_leading_zeros_and_mixed_runs() {
        assert_eq!(natural_cmp("note-02", "note-10"), Ordering::Less);
        assert_eq!(natural_cmp("a", "a1"), Ordering::Less);
        assert_eq!(natural_cmp("v1-10", "v1-9"), Ordering::Greater);
    }
}
