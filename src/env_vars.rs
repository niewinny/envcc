use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

const DOCS_URL: &str = "https://code.claude.com/docs/en/env-vars";

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvVarType {
    Bool,
    String,
    Int,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EnvVarDef {
    pub name: std::string::String,
    pub var_type: EnvVarType,
    pub description: std::string::String,
}

#[derive(Serialize, Deserialize)]
struct VarCache {
    content_hash: std::string::String,
    fetched_at: std::string::String,
    vars: Vec<EnvVarDef>,
}

fn cache_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache").join("envcc").join("vars.json")
}

fn load_cache() -> Option<VarCache> {
    let path = cache_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_cache(cache: &VarCache) -> io::Result<()> {
    let path = cache_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cache)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(&path, json)
}

fn simple_hash(s: &str) -> String {
    // Simple hash for change detection - not cryptographic
    let mut h: u64 = 5381;
    for b in s.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    format!("{:016x}", h)
}

fn infer_type(name: &str, description: &str) -> EnvVarType {
    let name_upper = name.to_uppercase();
    let desc_lower = description.to_lowercase();

    // Bool patterns
    if name_upper.starts_with("DISABLE_")
        || name_upper.contains("_DISABLE_")
        || name_upper.contains("_ENABLE_")
        || name_upper.starts_with("ENABLE_")
        || name_upper.contains("_SKIP_")
        || name_upper.contains("_USE_")
        || name_upper.contains("_FORCE_")
        || name_upper.contains("_KEEP_")
        || name_upper.contains("_MAINTAIN_")
        || name_upper.ends_with("_MODE")
        || name_upper == "CCR_FORCE_BUNDLE"
        || desc_lower.starts_with("enable ")
        || desc_lower.starts_with("disable ")
        || desc_lower.starts_with("opt out")
        || desc_lower.starts_with("skip ")
        || desc_lower.starts_with("force ")
        || desc_lower.starts_with("hide ")
        || desc_lower.contains("(default: true)")
        || desc_lower.contains("(default: false)")
    {
        return EnvVarType::Bool;
    }

    // Int patterns
    if name_upper.ends_with("_MS")
        || name_upper.ends_with("_TIMEOUT_MS")
        || name_upper.ends_with("_TOKENS")
        || name_upper.ends_with("_LENGTH")
        || name_upper.ends_with("_SECONDS")
        || name_upper.ends_with("_RETRIES")
        || name_upper.ends_with("_CONCURRENCY")
        || name_upper.ends_with("_SPEED")
        || name_upper.ends_with("_OVERRIDE")
        || name_upper.ends_with("_WINDOW")
        || name_upper.ends_with("_DELAY")
        || desc_lower.contains("timeout")
        || desc_lower.contains("millisecond")
        || desc_lower.contains("(default: ")
            && desc_lower
                .split("(default: ")
                .nth(1)
                .map(|s| s.trim_end_matches(')').chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false)
    {
        return EnvVarType::Int;
    }

    EnvVarType::String
}

fn parse_html(html: &str) -> Vec<EnvVarDef> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let row_sel = Selector::parse("tr").unwrap();
    let cell_sel = Selector::parse("td").unwrap();
    let code_sel = Selector::parse("code").unwrap();

    let mut vars = Vec::new();

    for row in document.select(&row_sel) {
        let cells: Vec<_> = row.select(&cell_sel).collect();
        if cells.len() < 2 {
            continue;
        }

        // First cell: variable name (usually in <code> tag)
        let name = cells[0]
            .select(&code_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_else(|| cells[0].text().collect::<String>())
            .trim()
            .to_string();

        // Skip if doesn't look like an env var
        if name.is_empty() || !name.chars().next().unwrap_or(' ').is_ascii_uppercase() {
            continue;
        }

        // Second cell: description
        let description = cells[1].text().collect::<String>().trim().to_string();

        let var_type = infer_type(&name, &description);

        vars.push(EnvVarDef {
            name,
            var_type,
            description,
        });
    }

    // Deduplicate by name
    vars.sort_by(|a, b| a.name.cmp(&b.name));
    vars.dedup_by(|a, b| a.name == b.name);

    vars
}

pub struct FetchResult {
    pub vars: Vec<EnvVarDef>,
    pub changed: bool,
    pub from_cache: bool,
}

pub fn fetch_vars() -> Result<FetchResult, String> {
    let cached = load_cache();

    // Try fetching from web
    match ureq::get(DOCS_URL).call() {
        Ok(response) => {
            let html = response
                .into_string()
                .map_err(|e| format!("Failed to read response: {}", e))?;

            let vars = parse_html(&html);

            if vars.is_empty() {
                // Parse failed, fall back to cache
                if let Some(cache) = cached {
                    return Ok(FetchResult {
                        vars: cache.vars,
                        changed: false,
                        from_cache: true,
                    });
                }
                return Err("No env vars found on page and no cache available".to_string());
            }

            let new_hash = simple_hash(&serde_json::to_string(&vars).unwrap_or_default());

            let changed = cached
                .as_ref()
                .map(|c| c.content_hash != new_hash)
                .unwrap_or(true);

            // Save to cache
            let cache = VarCache {
                content_hash: new_hash,
                fetched_at: chrono_now(),
                vars: vars.clone(),
            };
            let _ = save_cache(&cache);

            Ok(FetchResult {
                vars,
                changed,
                from_cache: false,
            })
        }
        Err(e) => {
            // Network error, try cache
            if let Some(cache) = cached {
                Ok(FetchResult {
                    vars: cache.vars,
                    changed: false,
                    from_cache: true,
                })
            } else {
                Err(format!("Failed to fetch: {} (no cache available)", e))
            }
        }
    }
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}
