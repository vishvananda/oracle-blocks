# Oracle Blocks

Keep this repository focused on blocks ↔ the published Oracle AST. The optional
Oracle text path uses `oracle-ir-grammar`; do not add a second text grammar or AST
schema. Preserve refusal, ambiguity and exact integers across the Wasm boundary.
Do not auto-select an ambiguous reading or print stale wording for edited IR.

Run Rust/fixture tests and relevant browser tests before committing. For a release
build, run all browser scripts through `npm --prefix web run test:all -- HTML`.
No catalog corpus, target cache, node_modules, credentials or built HTML in Git.
Publish catalog assets separately and pin their checksum in catalog.lock.json.
Push each commit to origin immediately. No Cantrip engine or card-lane code here.
