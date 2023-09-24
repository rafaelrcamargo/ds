use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DockerStats {
    #[serde(rename = "BlockIO")]
    pub block_io: String,
    #[serde(rename = "CPUPerc")]
    pub cpu_perc: String,
    #[serde(rename = "ID")]
    pub id: String,
    pub mem_perc: String,
    pub mem_usage: String,
    pub name: String,
    #[serde(rename = "NetIO")]
    pub net_io: String,
}
