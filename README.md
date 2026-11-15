# Oracle Blocks

A Blockly editor for the published Oracle AST. Search a card, inspect its blocks,
and build new cards by dragging actions, selectors, values and abilities into
typed sockets. Blocks ↔ AST is the primary interface. Rust in WebAssembly checks
scope, targets, result references, schema and complete IR before export.

The optional **Oracle text** panel uses published `oracle-ir-grammar` to parse
wording into AST and generate wording from the current AST. Parsing can refuse
or return several readings; each reading is shown before explicit application.
Printing succeeds only when Grammar verifies its wording against the current
meaning. Unsupported projections show a refusal; the app never presents old
wording as the output of edited blocks. This is a rules authoring tool, not a
Magic rules engine.

## Build and run

Requirements: stable Rust, Python 3, Node.js for browser tests, the
`wasm32-unknown-unknown` target and `wasm-bindgen-cli` **0.2.127**.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.127 --locked
web/build.sh
python3 -m http.server --directory build 8080
```

Open http://localhost:8080 or the generated `build/index.html` directly. The
standalone HTML embeds Blockly, the Rust Wasm validator, optional Grammar and a
compressed card catalog; the editor makes no runtime network requests. Grammar
loads lazily in the validation worker. The browser keeps integer fields exactly,
including values outside JavaScript's safe-number range.

The default build fetches a checksum-pinned catalog release using
`catalog.lock.json`. This initial 33,529-record catalog preserves the frozen
MTGish-produced AST at the documented Source hash, including multi-face cards.
It is not a new Scryfall snapshot or an implicit selection from ambiguous grammar
readings. The AST/contract crates remain the schema authority. The catalog is a
GitHub release asset, not a crate or a large checked-in data file.

Set `ORACLE_BLOCKS_CATALOG=/path/to/cards.jsonl` to build with another owned
UnifiedCard JSONL export (or version-1 portable records with a `card` member).
Every record is validated without silently discarding unknown fields. Set
`ORACLE_BLOCKS_BUILD` to change intermediates; pass an output HTML path to
`web/build.sh` to change the final destination. A partial face export must not be
presented as a complete multi-face card; catalog producers own that assembly.

## Tests

```sh
cargo test --locked --workspace --features oracle-text
python3 -m unittest discover -s tools
npm --prefix web ci
npx --prefix web playwright install chromium
npm --prefix web run test:all -- "$PWD/build/index.html"
```

Browser tests exercise the real embedded Wasm, block editing, search, typed
selectors, round trips, migrations, named bindings, unsafe integer boundaries,
optional controls, and Oracle parse/edit/verified print/refusal. CI builds the
standalone HTML and retains it as an artifact after these checks.

## Ownership

* [`oracle-grammar`](https://github.com/vishvananda/oracle-grammar) owns AST,
  validation, canonicalization, migrations and Oracle parse/print. This repo pins
  published `oracle-ir-contract` 0.2.1 and `oracle-ir-grammar` 0.2.1 (schema 191).
* [`mtgish`](https://github.com/vishvananda/mtgish) owns MTGish → AST lowering and
  comparison with released Grammar data.
* [`Cantrip tooling`](https://github.com/vishvananda/cantrip-rs/tree/tools/oracle-codegen-20261115/tools)
  owns AST → Cantrip generation and card lanes.

There are no local path dependencies on either consumer. The catalog builder
reflects published Rust declarations; the UI adds presentation conveniences
without creating a second AST schema. No backend generator or campaign state
belongs here. See [editor design notes](web/README.md) for the UI details.

Extracted from MTGish `5a0546450b7ad3ba07b583dfb749051b7b763a9b`, preserving
its MIT license and Blockly's Apache-2.0 notice under `web/vendor`.
