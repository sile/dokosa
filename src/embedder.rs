#[derive(Debug)]
pub struct Embedder {
    openai_api_key: String,
    model: String,
    input_texts: Vec<String>,
}

impl Embedder {
    pub fn new(openai_api_key: String, model: String, input_texts: Vec<String>) -> Self {
        Self {
            openai_api_key,
            model,
            input_texts,
        }
    }
}
