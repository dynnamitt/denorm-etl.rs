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
    pub mod core_helper {
        use nix::sched::sched_setaffinity;
        use nix::sched::CpuSet;
        use nix::unistd::{sysconf, Pid, SysconfVar};

        pub fn get_num_cpus() -> usize {
            match sysconf(SysconfVar::_NPROCESSORS_ONLN) {
                Ok(Some(n)) => n as usize,
                _ => 1, // fallback
            }
        }

        pub fn set_affinity(pid: i32) -> nix::Result<()> {
            let target_cores = if get_num_cpus() >= 4 {
                vec![2] // pin to cores 1 and 2
            } else {
                vec![0] // fallback to core 0
            };
            let mut cpuset = CpuSet::new();
            for &core in &target_cores {
                cpuset.set(core)?;
            }
            sched_setaffinity(Pid::from_raw(pid), &cpuset)
        }
    }
}

use common::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> ResBoxed<()> {
    let cpus = common::core_helper::get_num_cpus(); // debug
    println!("UNIX OS/SYS cpus:{}", cpus);
    factory::create().await
}
