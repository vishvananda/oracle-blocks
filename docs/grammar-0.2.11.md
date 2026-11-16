# Oracle Grammar 0.2.11 adoption

The editor pins published `oracle-ir-grammar` 0.2.11 and
`oracle-ir-contract` 0.2.3, with transitive `oracle-ir` 0.2.3. The wire schema
remains 191. There are no local crate overrides or new AST definitions.

The optional Oracle panel and its Wasm worker use the new grammar. The existing
checksum-pinned 33,529-record catalog remains the same frozen MTGish population;
upgrading the parser does not replace it with new Scryfall data or select an
ambiguous native reading. The standalone build validates every catalog record.

The preceding CI repair, commit `0375015`, fixes a browser-test race rather than
changing target semantics. Blockly defers dropdown events and rebuilds their
owning trigger. A valid previous program is not evidence that the new edit has
finished. The test now waits for the nested target's actual `Ability` kind and
successful native validation before checking enclosing/sibling scope isolation.
That commit's GitHub run passed:
https://github.com/vishvananda/oracle-blocks/actions/runs/43805386915 .

Validation after dependency adoption:

- `cargo test --locked --workspace --features oracle-text`: all five tests pass.
- `python3 -m unittest discover -s tools`: the catalog fixture test passes.
- `web/build.sh`: release Wasm and the catalog build successfully; all 33,529
  catalog records validate, with 33,481 searchable cards.
- `npm --prefix web run test:all -- "$PWD/build/index.html"`: all 29 browser
  suites pass against the newly built Grammar 0.2.11 Wasm, including actual
  Oracle parse, explicit reading application, edited verified print, refusal,
  exact integer transport and nested target editing.

The HTML, catalog, Wasm and build caches are ignored artifacts, not Git inputs.
