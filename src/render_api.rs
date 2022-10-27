lazy_static::lazy_static! {
  static ref CONFIG: crate::config::Config = {
    crate::config::Config::load()
  };
  static ref REQWEST_CLIENT: reqwest::Client = {
    reqwest::Client::new()
  };
}

// https://github.com/rust-lang/rfcs/pull/2584
// nested struct declaration is not supported at the moment
// making schema typing into a naming disaster
// so we use serde_json::Value for convenience
pub async fn list_owners() -> serde_json::Value {
  let res = REQWEST_CLIENT
    .get("https://api.render.com/v1/owners")
    .header("authorization", format!("Bearer {}", CONFIG.api_key))
    .send()
    .await
    .unwrap();

  res.json().await.unwrap()
}

pub async fn list_services() -> serde_json::Value {
  let res = REQWEST_CLIENT
    .get("https://api.render.com/v1/services")
    .query(&[("type", "web_service"), ("ownerId", &CONFIG.owner_id)])
    .header("authorization", format!("Bearer {}", CONFIG.api_key))
    .send()
    .await
    .unwrap();

  res.json().await.unwrap()
}

pub async fn delete_service(service_id: &str) {
  REQWEST_CLIENT
    .delete(format!("https://api.render.com/v1/services/{service_id}"))
    .header("authorization", format!("Bearer {}", CONFIG.api_key))
    .send()
    .await
    .unwrap();
}

pub async fn list_env_vars(service_id: &str) -> serde_json::Value {
  let res = REQWEST_CLIENT
    .get(format!(
      "https://api.render.com/v1/services/{service_id}/env-vars"
    ))
    .header("authorization", format!("Bearer {}", CONFIG.api_key))
    .send()
    .await
    .unwrap();

  res.json().await.unwrap()
}
