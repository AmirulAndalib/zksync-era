use std::time::Duration;

use smart_config::{
    de::{FromSecretString, Optional},
    metadata::TimeUnit,
    DescribeConfig, DeserializeConfig,
};
use zksync_basic_types::secrets::{APIKey, SeedPhrase};

#[derive(Clone, Debug, PartialEq, DescribeConfig, DeserializeConfig)]
#[config(tag = "avail_client_type")]
pub enum AvailClientConfig {
    FullClient(AvailDefaultConfig),
    GasRelay(AvailGasRelayConfig),
}

#[derive(Clone, Debug, PartialEq, DescribeConfig, DeserializeConfig)]
pub struct AvailConfig {
    pub bridge_api_url: String,
    #[config(default_t = Duration::from_secs(30))]
    pub timeout: Duration,
    #[config(flatten)]
    pub config: AvailClientConfig,
}

#[derive(Clone, Debug, PartialEq, DescribeConfig, DeserializeConfig)]
pub struct AvailDefaultConfig {
    pub api_node_url: String,
    pub app_id: u32,
    #[config(default_t = 3 * TimeUnit::Minutes)]
    pub dispatch_timeout: Duration,
    #[config(default_t = 5)]
    pub max_blocks_to_look_back: usize,
}

#[derive(Clone, Debug, PartialEq, DescribeConfig, DeserializeConfig)]
pub struct AvailGasRelayConfig {
    pub gas_relay_api_url: String,
    #[config(default_t = 5)]
    pub max_retries: usize,
}

#[derive(Clone, Debug, DescribeConfig, DeserializeConfig)]
pub struct AvailSecrets {
    #[config(with = Optional(FromSecretString))]
    pub seed_phrase: Option<SeedPhrase>,
    #[config(with = Optional(FromSecretString))]
    pub gas_relay_api_key: Option<APIKey>,
}
