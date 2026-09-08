// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Bartek Kus
//
// Relicensed from the Open Agentic Platform (crates/canonical-json,
// AGPL-3.0-or-later) to Apache-2.0 by the sole copyright holder. See NOTICE.

//! Canonical JSON: recursive lex-sort of object keys at the serialization
//! boundary.
//!
//! # Why this crate exists
//!
//! `serde_json`'s `preserve_order` feature flips `serde_json::Map`'s
//! iteration order from lexicographic (BTreeMap-style) to insertion order
//! (IndexMap-style). Under Cargo's resolver 2 the feature unifies
//! monotonically across a dependency graph: any crate enabling
//! `preserve_order` enables it for every dependent that also pulls
//! `serde_json`.
//!
//! This means one crate needing `preserve_order` silently flips key order
//! for every other crate that emits JSON. The fix is **explicit
//! canonicalization at the emission boundary**: callers route through
//! [`canonicalize_value`] (or the [`to_canonical_string`] convenience) so the
//! lex-key-order contract is upheld regardless of `preserve_order`'s state
//! elsewhere in the graph.
//!
//! The principle: **ordering requirements are explicit at the serialization
//! boundary, never implicit via shared-dep feature flags.** This is the
//! substrate that makes every downstream record hash reproducible: two
//! producers serializing the same logical value must produce the same bytes.

use serde_json::Value;

/// Canonicalize a [`Value`] to lexicographically-sorted object keys,
/// recursively. Arrays and scalars pass through unchanged; callers are
/// responsible for any array-ordering invariants.
///
/// This is the load-bearing helper. The recursion descends through every
/// nested object so deeply-nested keys are sorted at every level, not just
/// the top.
///
/// # What "lexicographic" means here
///
/// Keys are ordered by [`String::cmp`], which compares UTF-8 bytes. For
/// well-formed UTF-8 that is exactly Unicode code point order. Two properties
/// follow, and both matter to anyone writing a second implementation of this
/// contract in another language:
///
/// - **It is code point order, not UTF-16 code unit order.** JavaScript's
///   default `Array.prototype.sort` compares UTF-16 code units, where a
///   surrogate pair (0xD800..=0xDBFF) sorts below U+E000..=U+FFFF. So `"😀"`
///   (U+1F600) sorts *after* U+FFFD here and *before* it there. A port must
///   sort by code point, or equivalently by UTF-8 bytes, to agree.
/// - **There is no carve-out for integer-like keys.** `"10"` sorts before
///   `"2"`, at every nesting level. A port that sorts keys and then inserts
///   them into a JavaScript object loses this, because that language
///   enumerates array-index-like keys first in ascending numeric order
///   regardless of insertion order; such a port must emit the serialized text
///   directly rather than round-trip through an object.
///
/// A [`Value`] cannot hold an unpaired surrogate, so the byte comparison is
/// always total here. A port from a language whose strings can hold one must
/// reject it: `serde_json` refuses to parse `\uD800` in either key or value
/// position, so a record carrying one is unverifiable on this side, not merely
/// ordered differently.
pub fn canonicalize_value(v: Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = serde_json::Map::new();
            for (k, v) in entries {
                out.insert(k, canonicalize_value(v));
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(canonicalize_value).collect()),
        other => other,
    }
}

/// Convenience: canonicalize `v` then serialize to a compact string. Every
/// hashing consumer immediately serializes the canonical form, so this is the
/// call they actually want. The returned string is the byte sequence a
/// consumer feeds to its hash function.
///
/// Takes `&Value` and clones internally so the caller keeps ownership.
pub fn to_canonical_string(v: &Value) -> String {
    serde_json::to_string(&canonicalize_value(v.clone())).expect("serde_json::Value re-serializes")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sorts_top_level_keys_lexicographically() {
        let input = json!({ "z": 1, "a": 2, "m": 3 });
        let canon = canonicalize_value(input);
        let serialized = serde_json::to_string(&canon).unwrap();
        assert_eq!(serialized, r#"{"a":2,"m":3,"z":1}"#);
    }

    #[test]
    fn sorts_nested_object_keys_recursively() {
        let input = json!({
            "outer": { "z": 1, "a": 2 },
            "alpha": { "y": 3, "b": 4 }
        });
        let canon = canonicalize_value(input);
        let serialized = serde_json::to_string(&canon).unwrap();
        assert_eq!(
            serialized,
            r#"{"alpha":{"b":4,"y":3},"outer":{"a":2,"z":1}}"#
        );
    }

    #[test]
    fn descends_into_arrays_of_objects() {
        let input = json!([
            { "z": 1, "a": 2 },
            { "y": 3, "b": 4 }
        ]);
        let canon = canonicalize_value(input);
        let serialized = serde_json::to_string(&canon).unwrap();
        assert_eq!(serialized, r#"[{"a":2,"z":1},{"b":4,"y":3}]"#);
    }

    #[test]
    fn passes_through_scalars_unchanged() {
        assert_eq!(canonicalize_value(json!(null)), json!(null));
        assert_eq!(canonicalize_value(json!(42)), json!(42));
        assert_eq!(canonicalize_value(json!(true)), json!(true));
        assert_eq!(canonicalize_value(json!("hello")), json!("hello"));
    }

    #[test]
    fn idempotent_on_already_canonical_input() {
        let input = json!({ "a": 1, "b": { "c": 2, "d": 3 } });
        let once = canonicalize_value(input.clone());
        let twice = canonicalize_value(once.clone());
        assert_eq!(
            serde_json::to_string(&once).unwrap(),
            serde_json::to_string(&twice).unwrap()
        );
    }

    #[test]
    fn handles_deeply_nested_arrays_and_objects() {
        let input = json!({
            "z": [
                { "deep": { "z": 1, "a": 2 } },
                { "shallow": [3, 1, 2] }
            ],
            "a": "leaf"
        });
        let canon = canonicalize_value(input);
        let serialized = serde_json::to_string(&canon).unwrap();
        // Outer object: a before z; deep object: a before z; shallow array
        // preserves insertion order (canonicalize_value does NOT sort array
        // elements; array ordering is the caller's responsibility).
        assert_eq!(
            serialized,
            r#"{"a":"leaf","z":[{"deep":{"a":2,"z":1}},{"shallow":[3,1,2]}]}"#
        );
    }

    // --- determinism gate (the byte-stability property CI must guard) ---

    #[test]
    fn to_canonical_string_is_byte_stable_across_key_insertion_order() {
        // Two logically-equal values built with different key insertion order
        // must produce identical canonical bytes. This is the property the
        // whole ecosystem's record hashing rests on.
        let a = json!({ "z": 1, "a": { "n": 2, "b": 3 }, "m": [ { "y": 4, "x": 5 } ] });
        let b = json!({ "a": { "b": 3, "n": 2 }, "m": [ { "x": 5, "y": 4 } ], "z": 1 });
        assert_eq!(to_canonical_string(&a), to_canonical_string(&b));
        assert_eq!(
            to_canonical_string(&a),
            r#"{"a":{"b":3,"n":2},"m":[{"x":5,"y":4}],"z":1}"#
        );
    }

    // The two orderings a cross-language port gets wrong. Both are pinned
    // here so a second implementation has a fixture that can actually fail;
    // the byte-stability gate above cannot catch either, because every key in
    // it is a single ASCII letter.

    #[test]
    fn sorts_by_code_point_not_utf16_code_unit() {
        // U+FFFD is one UTF-16 code unit (0xFFFD); U+1F600 is a surrogate
        // pair starting 0xD83D. UTF-16 order puts the pair first, code point
        // order puts it last. This crate is code point order.
        let input = json!({ "\u{1F600}": 1, "\u{FFFD}": 2 });
        assert_eq!(
            to_canonical_string(&input),
            "{\"\u{FFFD}\":2,\"\u{1F600}\":1}"
        );
    }

    #[test]
    fn integer_like_keys_sort_lexicographically_not_numerically() {
        // "10" before "2" before "9". A port that rebuilds a JavaScript
        // object here emits numeric order instead.
        let input = json!({ "10": 1, "2": 2, "9": 3, "a": 4 });
        assert_eq!(to_canonical_string(&input), r#"{"10":1,"2":2,"9":3,"a":4}"#);
    }

    #[test]
    fn to_canonical_string_matches_canonicalize_then_serialize() {
        let v = json!({ "b": 1, "a": 2 });
        assert_eq!(
            to_canonical_string(&v),
            serde_json::to_string(&canonicalize_value(v.clone())).unwrap()
        );
    }
}
