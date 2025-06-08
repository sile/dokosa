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
        todo!()
    }
}
