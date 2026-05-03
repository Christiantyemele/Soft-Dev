pub const LOGO_LINES: &[&str] = &[
    "",
    "",
    "                    O P E N F L O W S",
    "",
    "            Autonomous AI Development Orchestration",
    "",
];

pub const TAGLINE: &str = "";

pub fn version_string() -> String {
    let git_hash = std::env::var("GIT_HASH")
        .unwrap_or_else(|_| "dev".to_string());
    let short_hash = if git_hash.len() >= 7 {
        &git_hash[..7]
    } else {
        &git_hash
    };
    format!("v{} · {}", env!("CARGO_PKG_VERSION"), short_hash)
}

pub fn get_logo_lines() -> Vec<String> {
    LOGO_LINES.iter().map(|s| s.to_string()).collect()
}
