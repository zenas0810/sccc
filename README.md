## Spring Cloud Config Reader
This is a simple crate to read configurations from a Spring Cloud Config server.

### Usage

```rust
use {sccc::{Result,
            SCC},
     serde::Deserialize};

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PUSConf {
  pus: PUSApp,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct PUSApp {
  supported_profile_items: Vec<String>,
}

#[tokio::test]
async fn test_scc() -> Result<()> {
  let scc: SCC<PUSConf> = SCC::new("http://<username>:<password>@config.test2pay.com", "dev", "gp232_pus");
  let _ = scc.load().await?;
  let pus_conf = scc.get(|x| x.pus.clone()).await;
  println!("{:?}", pus_conf);
  Ok(())
}

```