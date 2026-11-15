//! Optional text projection over the same owned IR used by Blockly.
//! The grammar loads lazily in the browser worker; refusal and ambiguity survive.
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn oracle_status() -> String {
    json!({"available":cfg!(feature="oracle-text"),"grammar":"0.2.1"}).to_string()
}

#[cfg(feature = "oracle-text")]
mod grammar {
    use super::*;
    use oracle_ir_contract::{ir::UnifiedCard, validate};
    use oracle_ir_grammar::engine::{CardReading, Engine};
    use std::cell::OnceCell;

    thread_local! {
        static ENGINE: OnceCell<Result<Engine, String>> = const { OnceCell::new() };
    }

    fn with_engine(f: impl FnOnce(&Engine) -> Result<Value, String>) -> Result<Value, String> {
        ENGINE.with(|engine| match engine.get_or_init(Engine::load) {
            Ok(engine) => f(engine),
            Err(error) => Err(error.clone()),
        })
    }

    fn request(input: &str) -> Result<(Value, UnifiedCard), String> {
        let value = oracle_ir_contract::json_input::value(input)?;
        let card = super::super::check(value["ir"].as_str().ok_or("IR must be a JSON string")?)?;
        Ok((value, card))
    }

    pub(super) fn parse(input: &str) -> Result<Value, String> {
        let (request, card) = request(input)?;
        let index = match request.get("face") {
            None => 0,
            Some(value) => usize::try_from(value.as_u64().ok_or("face must be a nonnegative integer")?)
                .map_err(|_| "face index is out of range")?,
        };
        let face = card.faces.get(index).ok_or("unknown face")?;
        let text = request["text"].as_str().ok_or("missing Oracle text")?;
        if text.len() > 65_536 { return Err("Oracle text exceeds 64 KiB".into()); }
        with_engine(|engine| {
            let parsed = match engine.parse_card(&face.name, text, &face.characteristics) {
                Ok(parsed) => parsed,
                Err(error) => return Ok(json!({"valid":false,"status":"refused",
                    "error":format!("{error:?}"),"readings":[]})),
            };
            let mut readings = Vec::new();
            for reading in parsed.readings {
                let mut candidate = card.clone();
                candidate.faces[index].abilities = serde_json::from_value(reading.abilities)
                    .map_err(|error| error.to_string())?;
                validate::unified(&candidate)?;
                readings.push(json!({"ir":serde_json::to_string(&candidate).map_err(|e|e.to_string())?,
                    "notes":reading.notes}));
            }
            if readings.is_empty() { return Err("grammar returned no complete reading".into()); }
            Ok(json!({"valid":true,
                "status":if readings.len()==1 {"unique"} else {"ambiguous"},
                "readings":readings}))
        })
    }

    pub(super) fn print(input: &str) -> Result<Value, String> {
        let (request, card) = request(input)?;
        with_engine(|engine| {
            let mut texts = Vec::new();
            for (index, face) in card.faces.iter().enumerate() {
                let notes = request["notes"].get(index).cloned().unwrap_or(json!([]));
                let reading = CardReading {
                    abilities: serde_json::to_value(&face.abilities).map_err(|e|e.to_string())?,
                    notes: serde_json::from_value(notes).map_err(|e|e.to_string())?,
                };
                // This API verifies generated wording by parsing it back against
                // the current operands. Unsupported printing is a real refusal.
                let text = engine.print_card_keyed(&face.name, &reading, &face.characteristics, &[])?
                    .ok_or_else(|| format!("No verified Oracle wording for {}", face.name))?;
                texts.push(json!({"name":face.name,"text":text}));
            }
            Ok(json!({"valid":true,"faces":texts,"verified":true}))
        })
    }
}

#[cfg(not(feature = "oracle-text"))]
fn unavailable() -> Value {
    json!({"valid":false,"status":"unavailable","error":"Build with the oracle-text feature."})
}

#[wasm_bindgen]
pub fn oracle_to_ir(input: &str) -> String {
    #[cfg(feature = "oracle-text")]
    return grammar::parse(input).unwrap_or_else(|e| json!({"valid":false,"status":"refused","error":e})).to_string();
    #[cfg(not(feature = "oracle-text"))]
    { let _ = input; unavailable().to_string() }
}

#[wasm_bindgen]
pub fn ir_to_oracle(input: &str) -> String {
    #[cfg(feature = "oracle-text")]
    return grammar::print(input).unwrap_or_else(|e| json!({"valid":false,"status":"refused","error":e})).to_string();
    #[cfg(not(feature = "oracle-text"))]
    { let _ = input; unavailable().to_string() }
}

#[cfg(all(test, feature = "oracle-text"))]
mod tests {
    use super::*;
    fn input() -> Value {
        json!({"schema_version":191,"layout":"Card","data":[],"faces":[{
            "schema_version":191,"name":"Test Bolt","characteristics":{
                "supertypes":[],"card_types":["Instant"],"subtypes":[],"mana_cost":[],
                "color_indicator":[],"power_toughness":null,"loyalty":null,"defense":null},"abilities":[]}]})
    }
    #[test]
    fn grammar_prints_edited_current_ir_and_refuses_unparsed_text() {
        std::thread::Builder::new().stack_size(64<<20).spawn(|| {
            let request = json!({"ir":input().to_string(),"text":"Test Bolt deals 3 damage to any target."});
            let result: Value = serde_json::from_str(&oracle_to_ir(&request.to_string())).unwrap();
            assert_eq!(result["valid"],true,"{result}");
            let reading = &result["readings"][0];
            let mut card: Value = serde_json::from_str(reading["ir"].as_str().unwrap()).unwrap();
            card["faces"][0]["abilities"][0]["args"]["body"]["instructions"][0]
                ["args"]["operation"]["args"]["amount"]["args"] = json!(7);
            let printed: Value = serde_json::from_str(&ir_to_oracle(&json!({"ir":card.to_string(),"notes":[reading["notes"]]}).to_string())).unwrap();
            assert_eq!(printed["valid"],true,"{printed}");
            assert!(printed["faces"][0]["text"].as_str().unwrap().contains('7'));
            let bad: Value = serde_json::from_str(&oracle_to_ir(&json!({"ir":input().to_string(),"text":"This is not valid card rules gibberish."}).to_string())).unwrap();
            assert_eq!(bad["valid"],false);
        }).unwrap().join().unwrap();
    }
}
