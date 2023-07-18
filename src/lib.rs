//! This crate is to read the configuration yaml file from the `Spring Cloud Config` service.
//! Then deserialize them to a data object

use {crate::SCCError::SpringCloudConfigServiceError,
     serde::de::DeserializeOwned,
     std::sync::Arc,
     thiserror::Error,
     tokio::sync::Mutex};

#[derive(Debug, Clone, Error)]
pub enum SCCError {
  #[error("spring cloud config service error: {0}")]
  SpringCloudConfigServiceError(String),
}
pub type Result<T> = std::result::Result<T, SCCError>;

struct SCCInner<T: DeserializeOwned + Default + Send + Sync> {
  scc_service:   String,
  label:         String,
  application:   String,
  configuration: Mutex<Arc<T>>,
}

impl<T: DeserializeOwned + Default + Send + Sync> SCCInner<T> {
  fn new(scc_service: &str, label: &str, application: &str) -> Self {
    Self {
      scc_service:   scc_service.to_string(),
      label:         label.to_string(),
      application:   application.to_string(),
      configuration: Mutex::new(Arc::new(Default::default())),
    }
  }

  async fn load(&self) -> Result<()> {
    let json_endpoint = format!("{}/{}-{}.json", self.scc_service, self.application, self.label);
    let resp = reqwest::get(json_endpoint)
      .await
      .map_err(|e| SpringCloudConfigServiceError(e.to_string()))?;
    if !resp.status().is_success() {
      let err = resp
        .text()
        .await
        .map_err(|e| SpringCloudConfigServiceError(e.to_string()))?;
      Err(SpringCloudConfigServiceError(err))
    } else {
      let json_text = resp
        .text()
        .await
        .map_err(|e| SpringCloudConfigServiceError(e.to_string()))?;

      let conf: T =
        serde_json::from_str(json_text.as_str()).map_err(|e| SpringCloudConfigServiceError(e.to_string()))?;
      *self.configuration.lock().await = Arc::new(conf);
      Ok(())
    }
  }

  async fn get_configuration(&self) -> Arc<T> {
    let c: Arc<T> = (*self.configuration.lock().await).clone();
    return c;
  }
}

#[derive(Clone)]
pub struct SCC<T: DeserializeOwned + Default + Send + Sync> {
  inner: Arc<SCCInner<T>>,
}

impl<T: DeserializeOwned + Default + Send + Sync> SCC<T> {
  /// ## Create an `SCC` instance
  ///
  /// - `scc_service`: is the Spring Cloud Config Server root url, with the credential.
  pub fn new(scc_service: &str, label: &str, app: &str) -> SCC<T> {
    Self {
      inner: Arc::new(SCCInner::new(scc_service, label, app)),
    }
  }

  /// ## Load configuration
  pub async fn load(&self) -> Result<()> {
    self.inner.load().await
  }

  /// ## Get a part of the configuration object
  ///
  /// `f`: the function to extract a part out of the configuration object.
  pub async fn get<F, A>(&self, f: F) -> A
  where
    F: FnOnce(&T) -> A, {
    let x = self.inner.get_configuration().await;
    let a = f(x.as_ref());
    a
  }
}
