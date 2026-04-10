use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

pub fn load_settings(path: &Path) -> io::Result<(HashMap<String, String>, serde_json::Value)> {
    if !path.exists() {
        return Ok((HashMap::new(), serde_json::Value::Object(serde_json::Map::new())));
    }

    let content = fs::read_to_string(path)?;
    let mut root: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut values = HashMap::new();

    if let Some(env_obj) = root.get("env").and_then(|v| v.as_object()) {
        for (key, val) in env_obj {
            if let Some(s) = val.as_str() {
                values.insert(key.clone(), s.to_string());
            } else if let Some(b) = val.as_bool() {
                values.insert(key.clone(), if b { "1".to_string() } else { "0".to_string() });
            } else if let Some(n) = val.as_i64() {
                values.insert(key.clone(), n.to_string());
            }
        }
    }

    // Remove env from the root so we can preserve everything else
    if let Some(obj) = root.as_object_mut() {
        obj.remove("env");
    }

    Ok((values, root))
}

pub fn save_settings(
    path: &Path,
    values: &HashMap<String, String>,
    other_settings: &serde_json::Value,
) -> io::Result<()> {
    let mut root = other_settings.clone();

    // Build the env object from current values
    let mut env_map = serde_json::Map::new();
    for (key, val) in values {
        if !val.is_empty() {
            env_map.insert(key.clone(), serde_json::Value::String(val.clone()));
        }
    }

    // Only write env key if there are values
    if let Some(obj) = root.as_object_mut() {
        if !env_map.is_empty() {
            obj.insert("env".to_string(), serde_json::Value::Object(env_map));
        }
    }

    // Ensure .claude directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(&root)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    fs::write(path, json + "\n")
}
