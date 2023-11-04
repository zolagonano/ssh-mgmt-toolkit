#![feature(exit_status_error)]
pub mod models;
pub mod stats;
pub mod users;

pub mod consts {
    // Use Config file in future
    pub const PASSWD_IV: &[u8] = b"18334ba316694d5e5917ce520420cb1018adfcb2";
    pub const PASSWD_PREFIX: &str = "SSHMGMTKIT_";
    pub const PASSWD_PARAMS: &str = "$6$mENJascSdtQuhrXH";
    pub const JWT_SECRET: &[u8] = b"1bdf1a42-4966-1bd0-6f69-04c34f2eaa29";
    pub const NETHOGS_TRACE_PATH: &str = "/tmp/log";
    pub const DEFAULT_SHELL: &str = "/bin/rbash";
}

pub mod config {
    use config::Config;
    use serde::{Deserialize, Serialize};

    /// Represents information about an SSH management node.
    #[derive(Serialize, Deserialize, Clone)]
    pub struct NodeInfo {
        name: String,
        location: String,
        capacity: Option<u64>,
    }

    /// Represents the configuration file structure for SSH management.
    #[derive(Serialize, Deserialize, Clone)]
    pub struct ConfigFile {
        pub node_info: NodeInfo,
    }

    impl ConfigFile {
        /// Loads the configuration from the specified file path ("/etc/sshmgmt_config.json").
        ///
        /// # Errors
        ///
        /// Returns a `Result` with `ConfigFile` if successful, or a `Box<dyn std::error::Error>`
        /// on failure.
        pub fn load() -> Result<ConfigFile, Box<dyn std::error::Error>> {
            let settings = Config::builder()
                .add_source(config::File::with_name("/etc/sshmgmt_config.json"))
                .build()?;

            Ok(settings.try_deserialize::<ConfigFile>()?)
        }
    }
}
