use lemma_proto::lemma::v1::ProviderKind;
use std::time::Duration;

#[derive(serde::Deserialize)]
struct ModelsList {
    data: Vec<IdOnly>,
}

#[derive(serde::Deserialize)]
struct IdOnly {
    id: String,
}

#[derive(serde::Deserialize)]
struct GeminiModels {
    models: Vec<NameOnly>,
}

#[derive(serde::Deserialize)]
struct NameOnly {
    name: String,
}

// 拉取供应商模型列表；models_path 留空用默认 /models
pub async fn fetch_models(
    kind: ProviderKind,
    base_url: &str,
    api_key: &str,
    models_path: &str,
) -> Result<Vec<String>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let path = if models_path.is_empty() {
        "/models"
    } else {
        models_path
    };
    let url = format!("{}{}", base_url.trim_end_matches('/'), path);

    match kind {
        ProviderKind::PROVIDER_KIND_OPENAI => {
            let list: ModelsList = client
                .get(&url)
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .json()
                .await
                .map_err(|e| e.to_string())?;
            Ok(list.data.into_iter().map(|m| m.id).collect())
        }
        ProviderKind::PROVIDER_KIND_ANTHROPIC => {
            let list: ModelsList = client
                .get(&url)
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .json()
                .await
                .map_err(|e| e.to_string())?;
            Ok(list.data.into_iter().map(|m| m.id).collect())
        }
        ProviderKind::PROVIDER_KIND_GEMINI => {
            let list: GeminiModels = client
                .get(&url)
                .header("x-goog-api-key", api_key)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .json()
                .await
                .map_err(|e| e.to_string())?;
            Ok(list
                .models
                .into_iter()
                .map(|m| {
                    m.name
                        .strip_prefix("models/")
                        .unwrap_or(&m.name)
                        .to_string()
                })
                .collect())
        }
        _ => Err("unsupported provider kind".into()),
    }
}
