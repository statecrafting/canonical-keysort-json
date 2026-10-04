// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Bartek Kus
//
// Relicensed from the Open Agentic Platform (crates/canonical-json,
// AGPL-3.0-or-later) to Apache-2.0 by the sole copyright holder. See NOTICE.

//! Deterministic JSON key-sort: recursive lex-sort of object keys at the
//! serialization boundary.
//!
//! This sorts object keys and nothing else. It is not Canonical JSON in the
//! gibson042 sense, nor RFC 8785 (JCS): it neither normalizes numbers nor
//! escapes non-ASCII, so its bytes differ from both for some inputs.
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

    // --- the test build must be the adversarial one ---

    #[test]
    fn test_build_runs_with_preserve_order() {
        // Every test above is vacuous without `preserve_order`: a BTreeMap
        // already iterates in sorted order, so an identity
        // `canonicalize_value` would pass them all. The dev-dependency in
        // Cargo.toml turns the feature on; this fails if that ever stops.
        let mut m = serde_json::Map::new();
        m.insert("z".into(), json!(1));
        m.insert("a".into(), json!(2));
        let keys: Vec<&str> = m.keys().map(String::as_str).collect();
        assert_eq!(
            keys,
            ["z", "a"],
            "serde_json/preserve_order is off in the test build"
        );
    }

    #[test]
    fn to_canonical_string_matches_canonicalize_then_serialize() {
        let v = json!({ "b": 1, "a": 2 });
        assert_eq!(
            to_canonical_string(&v),
            serde_json::to_string(&canonicalize_value(v.clone())).unwrap()
        );
    }
    // --- property: byte stability over generated values ---

    mod properties {
        use super::*;
        use proptest::prelude::*;

        /// Keys drawn to hit the orderings that matter: ASCII, integer-like,
        /// the empty key, BMP vs astral (the UTF-16 trap), U+2028, and a
        /// short arbitrary tail.
        fn key() -> impl Strategy<Value = String> {
            prop_oneof![
                "[a-z]{0,3}",
                "[0-9]{1,3}",
                Just("\u{FFFD}".to_string()),
                Just("\u{1F600}".to_string()),
                Just("\u{E000}".to_string()),
                Just("\u{2028}".to_string()),
                Just("é".to_string()),
                any::<String>().prop_map(|s| s.chars().take(4).collect()),
            ]
        }

        fn value() -> impl Strategy<Value = Value> {
            let leaf = prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::Bool),
                any::<i64>().prop_map(Value::from),
                key().prop_map(Value::String),
            ];
            leaf.prop_recursive(4, 48, 6, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
                    prop::collection::vec((key(), inner), 0..6)
                        .prop_map(|kvs| Value::Object(kvs.into_iter().collect())),
                ]
            })
        }

        /// Rebuild `v` with every object's insertion order shuffled by
        /// `seed`. Under `preserve_order` this changes what a naive
        /// serializer emits while leaving the logical value equal.
        fn reorder(v: &Value, seed: &mut u64) -> Value {
            match v {
                Value::Object(map) => {
                    let mut entries: Vec<(&String, &Value)> = map.iter().collect();
                    for i in (1..entries.len()).rev() {
                        *seed ^= *seed << 13;
                        *seed ^= *seed >> 7;
                        *seed ^= *seed << 17;
                        entries.swap(i, (*seed % (i as u64 + 1)) as usize);
                    }
                    Value::Object(
                        entries
                            .into_iter()
                            .map(|(k, v)| (k.clone(), reorder(v, seed)))
                            .collect(),
                    )
                }
                Value::Array(arr) => Value::Array(arr.iter().map(|e| reorder(e, seed)).collect()),
                other => other.clone(),
            }
        }

        /// Every object in `v` iterates in strictly ascending `String::cmp`
        /// order: the canonical form of spec 000 section 4 rule 1.
        fn keys_strictly_sorted(v: &Value) -> bool {
            match v {
                Value::Object(map) => {
                    map.keys().zip(map.keys().skip(1)).all(|(a, b)| a < b)
                        && map.values().all(keys_strictly_sorted)
                }
                Value::Array(arr) => arr.iter().all(keys_strictly_sorted),
                _ => true,
            }
        }

        proptest! {
            #![proptest_config(ProptestConfig {
                cases: 512,
                failure_persistence: None,
                ..ProptestConfig::default()
            })]

            #[test]
            fn byte_stable_across_key_insertion_order(v in value(), seed in 1u64..) {
                let mut s = seed;
                let shuffled = reorder(&v, &mut s);
                prop_assert_eq!(to_canonical_string(&v), to_canonical_string(&shuffled));
            }

            #[test]
            fn output_is_sorted_lossless_and_idempotent(v in value()) {
                let out = to_canonical_string(&v);
                let reparsed: Value = serde_json::from_str(&out).unwrap();
                prop_assert!(keys_strictly_sorted(&reparsed));
                prop_assert_eq!(&reparsed, &v);
                prop_assert_eq!(to_canonical_string(&reparsed), out);
            }
        }
    }
}
