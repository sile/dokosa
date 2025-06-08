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

    pub fn embed(&self, input_texts: &[String]) -> orfail::Result<String> {
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
                "curl command failed: {}",
                stderr
            )));
        }

        let response = String::from_utf8(output.stdout)
            .or_fail_with(|e| format!("Failed to parse curl response as UTF-8: {e}"))?;

        Ok(response)
    }
}
