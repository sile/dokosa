use orfail::OrFail;

#[derive(Debug)]
pub struct Embedder {
    openai_api_key: String,
    model: String,
}

impl Embedder {
    pub fn new(openai_api_key: String, model: String) -> Self {
        Self {
            openai_api_key,
            model,
        }
    }

    pub fn embed(&self, input_texts: &[String]) -> orfail::Result<Vec<Embedding>> {
        let content = nojson::json(|f| {
            f.object(|f| {
                f.member("model", &self.model)?;
                f.member("input", input_texts)
            })
        })
        .to_string();

        let mut cmd = std::process::Command::new("curl");
        cmd.arg("https://api.openai.com/v1/embeddings")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", self.openai_api_key))
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-d")
            .arg(&content)
            .arg("--silent")
            .arg("--show-error");

        let output = cmd.output().or_fail()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(orfail::Failure::new(format!(
                "curl command failed: {stderr}"
            )));
        }

        let response = String::from_utf8(output.stdout)
            .or_fail_with(|e| format!("Failed to parse curl response as UTF-8: {e}"))?;
        let response = nojson::RawJson::parse(&response).or_fail()?;
        let data = response
            .value()
            .to_member("data")
            .or_fail()?
            .required()
            .or_fail_with(|e| {
                format!("Unexpected embeddings API response: {e}\n\nJSON:\n{response}\n")
            })?;

        let mut embeddings = vec![Embedding::default(); input_texts.len()];
        for object in data.to_array().or_fail()? {
            let index = object.to_member("index").or_fail()?.required().or_fail()?;
            let embedding = object
                .to_member("embedding")
                .or_fail()?
                .required()
                .or_fail()?;
            let i = usize::try_from(index).or_fail()?;
            (i < embeddings.len()).or_fail()?;

            embeddings[i] = Embedding::try_from(embedding).or_fail()?;
        }

        Ok(embeddings)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Embedding(pub Vec<f64>);

impl nojson::DisplayJson for Embedding {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.value(&self.0)
    }
}

impl<'text> TryFrom<nojson::RawJsonValue<'text, '_>> for Embedding {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, '_>) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}
