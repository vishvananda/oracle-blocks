//! Build static demo inputs from the actual Rust model and checked-in corpus.
use oracle_ir_contract::{
    ir_schema::{Fields, Schema, Ty, TypeDef},
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use oracle_ir_contract::declarations;
const SOURCE: &str = declarations::SOURCE_VOCABULARY;

fn ty(t: &Ty) -> Value {
    match t {
        Ty::Named(n) => json!({"named":n}),
        Ty::Vec(t) => json!({"vec":ty(t)}),
        Ty::Option(t) => json!({"option":ty(t)}),
        Ty::Tuple(t) => json!({"tuple":t.iter().map(ty).collect::<Vec<_>>()}),
        Ty::Opaque => panic!("The demo needs a typed descriptor for every reachable field"),
    }
}
fn fields(f: &Fields) -> Value {
    match f {
        Fields::Unit => json!({"kind":"unit"}),
        Fields::Tuple(t) => json!({"kind":"tuple","items":t.iter().map(ty).collect::<Vec<_>>()}),
        Fields::Named(t) => {
            json!({"kind":"record","fields":t.iter().map(|(n,t)|json!({"name":n,"ty":ty(t)})).collect::<Vec<_>>()})
        }
    }
}
fn docs(attrs: &[syn::Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|a| {
            if a.path().is_ident("doc") {
                if let syn::Meta::NameValue(n) = &a.meta {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &n.value
                    {
                        return Some(s.value().trim().to_string());
                    }
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn schema() -> Value {
    // Source is read first so the normalized Card/Ability/etc. definitions win.
    let mut schema = Schema::from_sources(&declarations::authoring_texts()).unwrap();
    let mut annotations = BTreeMap::new();
    // Mechanic is macro-generated in ir.rs; use its compiled registry rather
    // than maintaining a second list or guessing at macro expansion.
    let mechanics = oracle_ir_contract::ir::MECHANICS
        .iter()
        .map(|(_, mechanic)| {
            let wire = serde_json::to_value(mechanic)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned();
            annotations.insert(
                ("Mechanic".to_string(), wire.clone()),
                (wire.clone(), String::new()),
            );
            (wire, Fields::Unit)
        })
        .collect();
    schema.types.insert(
        "Mechanic".into(),
        TypeDef::Enum {
            adjacent: false,
            variants: mechanics,
        },
    );
    for (is_source, source) in std::iter::once((true, SOURCE))
        .chain(declarations::SOURCES.iter().map(|source| (false, source.text))) {
        for item in syn::parse_file(source).unwrap().items {
            if let syn::Item::Enum(e) = item {
                if is_source
                    && ![
                        "CardType",
                        "CreatureType",
                        "SubType",
                        "SuperType",
                        "LandType",
                        "ArtifactType",
                    ]
                    .contains(&e.ident.to_string().as_str())
                {
                    continue;
                }
                for v in e.variants {
                    let mut wire = v.ident.to_string();
                    for a in &v.attrs {
                        if a.path().is_ident("serde") {
                            a.parse_nested_meta(|m| {
                                if m.path.is_ident("rename") {
                                    wire = m.value()?.parse::<syn::LitStr>()?.value();
                                }
                                Ok(())
                            })
                            .unwrap();
                        }
                    }
                    annotations.insert(
                        (e.ident.to_string(), v.ident.to_string()),
                        (wire, docs(&v.attrs)),
                    );
                }
            }
        }
    }
    fn names(t: &Ty, out: &mut Vec<String>) {
        match t {
            Ty::Named(n) => out.push(n.clone()),
            Ty::Vec(t) | Ty::Option(t) => names(t, out),
            Ty::Tuple(t) => t.iter().for_each(|t| names(t, out)),
            Ty::Opaque => panic!("opaque reachable field"),
        }
    }
    fn nested(f: &Fields, out: &mut Vec<String>) {
        match f {
            Fields::Tuple(t) => t.iter().for_each(|t| names(t, out)),
            Fields::Named(t) => t.iter().for_each(|(_, t)| names(t, out)),
            Fields::Unit => (),
        }
    }
    let mut pending = vec!["UnifiedCard".to_string()];
    let mut seen = BTreeSet::new();
    let mut types = BTreeMap::new();
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(def) = schema.types.get(&name) else {
            assert!(
                [
                    "String", "bool", "u8", "u16", "u32", "u64", "usize", "i8", "i16", "i32",
                    "i64", "isize"
                ]
                .contains(&name.as_str()),
                "unknown type {name}"
            );
            continue;
        };
        let value = match def {
            TypeDef::Struct(f) => {
                nested(f, &mut pending);
                json!({"kind":"struct","fields":fields(f)})
            }
            TypeDef::Enum { adjacent, variants } => {
                let variants: Vec<_> = variants
                    .iter()
                    .map(|(variant, f)| {
                        nested(f, &mut pending);
                        let (wire, doc) = annotations
                            .get(&(name.clone(), variant.clone()))
                            .unwrap_or_else(|| panic!("missing annotation for {name}::{variant}"));
                        json!({"name":variant,"wire":wire,"doc":doc,"fields":fields(f)})
                    })
                    .collect();
                json!({"kind":"enum","adjacent":adjacent,"variants":variants})
            }
        };
        types.insert(name, value);
    }
    json!({"version":oracle_ir_contract::ir::SCHEMA_VERSION,"types":types,"authoring":oracle_ir_contract::human::editor_metadata()})
}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: oracle-blocks-catalog <owned-ir.jsonl> <output-directory>".into());
    }
    let output = Path::new(&args[1]);
    std::fs::create_dir_all(output)?;
    std::fs::write(output.join("schema.json"), serde_json::to_vec_pretty(&schema())?)?;
    let mut data = String::new();
    use std::io::BufRead;
    for (index, line) in std::io::BufReader::new(std::fs::File::open(&args[0])?).lines().enumerate() {
        let value = oracle_ir_contract::json_input::value(&line?)?;
        let value = value.get("card").unwrap_or(&value);
        let card: oracle_ir_contract::ir::UnifiedCard = serde_json::from_value(value.clone())?;
        if serde_json::to_value(&card)? != *value { return Err(format!("row {} has unknown or missing fields", index + 1).into()); }
        oracle_ir_contract::validate::unified(&card)?;
        data.push_str(&serde_json::to_string(&card)?);
        data.push('\n');
    }
    if data.is_empty() { return Err("empty IR catalog".into()); }
    std::fs::write(output.join("cards.jsonl"), data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schema_tracks_source_vocabulary_and_serde_renames() {
        let s = schema();
        assert!(s["types"]["CardType"]["variants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["wire"] == "Creature"));
        assert!(s["types"]["Operation"]["variants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["name"] == "DraftChangeZone" && v["wire"] == "ChangeZone"));
        assert_eq!(s["types"]["UnifiedCard"]["fields"]["kind"], "record");
    }
}
