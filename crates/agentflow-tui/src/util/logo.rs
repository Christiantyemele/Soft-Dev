pub const LOGO: &str = r#"

  +----------------------------------------------------------+
  |                                                          |
  |   OOOOO   PPPP    EEEEE  N   N  FFFFF  L   OOOOO  W   W  |
  |  O     O  P   P   E      NN  N  F      L  O     O W   W  |
  |  O     O  PPPP    EEEE   N N N  FFFF   L  O     O W W W  |
  |  O     O  P       E      N  NN  F      L  O     O WW WW  |
  |   OOOOO   P       EEEEE  N   N  F      LLL OOOOO  W   W  |
  |                                                          |
  +----------------------------------------------------------+
"#;

pub const TAGLINE: &str = "Autonomous AI Development Team Orchestration";

pub fn version_string() -> String {
    let git_hash = std::env::var("GIT_HASH")
        .unwrap_or_else(|_| "unknown".to_string());
    let short_hash = if git_hash.len() >= 7 {
        &git_hash[..7]
    } else {
        &git_hash
    };
    format!(
        "OpenFlow {} ({})",
        env!("CARGO_PKG_VERSION"),
        short_hash
    )
}

pub fn get_logo_lines() -> Vec<String> {
    LOGO.lines().map(|s| s.to_string()).collect()
}
