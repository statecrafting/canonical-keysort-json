# Conformance vectors

Language-neutral test vectors for the canonical form defined in
[spec 000 section 4](../specs/000-canonical-keysort-json-bootstrap/spec.md).
Governed by [spec 002](../specs/002-conformance-vectors/spec.md), which is the
authority for everything below.

`canonical-form.json` arrives when spec 002 is built. Until then this directory
holds only this description.

## Format (`canonical-keysort-json/vectors/v1`)

One JSON object: `format`, `contract`, and a `vectors` array. Each vector has:

| key | meaning |
|---|---|
| `id` | unique, kebab-case, stable |
| `rule` | the spec 000 section 4 rule it exercises, e.g. `4.1` |
| `normative` | `false` for vectors outside the section 4 guarantee (number forms); a port may skip them |
| `input` | the input as JSON **text** in a string, so your own parser and ingestion path run on it |
| `expected` | the canonical output as a string (accepting vectors) |
| `expected_utf8_hex` | the canonical output's UTF-8 bytes in lowercase hex; this is the authority |
| `reject` | `true` when a conforming implementation must refuse the input |

## Using it from a port

Pin a release tag, not `main`. For each vector, feed `input` through the same
path your producer uses, and compare the bytes you emit against
`expected_utf8_hex`. For a `reject` vector, assert that you refuse it and emit
nothing. Refuse a `format` value you do not recognize.

## Changing it

The file is append-only. Changing or removing the expected output of a
normative vector changes the canonical form, which breaks every downstream
hash. That requires a spec 000 change in the same pull request and a major
version.
