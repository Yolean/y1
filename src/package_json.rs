use std::collections::BTreeMap;
use std::fs;

pub fn read_scripts() -> Result<BTreeMap<String, String>, String> {
    let content =
        fs::read_to_string("package.json").map_err(|e| format!("Failed to read package.json: {e}"))?;

    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse package.json: {e}"))?;

    let scripts = match value.get("scripts") {
        Some(serde_json::Value::Object(map)) => map
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect(),
        _ => BTreeMap::new(),
    };

    Ok(scripts)
}
