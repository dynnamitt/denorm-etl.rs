mod factory;
mod pipeline;
mod plugins;

// Some shared "grunt"
mod common {
    use phf::phf_map;
    use std::env;
    use std::error::Error;
    pub type ResBoxed<T> = Result<T, Box<dyn Error + Sync + Send>>;
    static ENV_DEFS: phf::Map<&'static str, &'static str> = phf_map! {
        "JIRA_ENDPOINT" => "",
        "JIRA_JQL" => "",
        "JIRA_TOKEN" => "",
        "SKIP_PANDOC" => "false",
        "DEBUG_DIR" => "",
        "DEST_BUCKET" => "jira-cleaned-for-inference",
        "DEST_PREFIX" => "",
        "PRODUCER_WAIT" => "1000",
        "PRODUCER_BATCH_SIZE" => "15"
    };

    pub fn get_conf(x: &str) -> String {
        assert!(ENV_DEFS.contains_key(x)); // code err
        env::var(x).unwrap_or(ENV_DEFS.get(x).unwrap().to_string())
    }
}

use common::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ResBoxed<()> {
    factory::create().await
}
