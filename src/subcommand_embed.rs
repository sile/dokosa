use std::io::Read;

use orfail::OrFail;

use crate::embedder::Embedder;

pub fn run(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let api_key: String = noargs::opt("openai-api-key")
        .ty("STRING")
        .example("YOUR_API_KEY")
        .env("OPENAI_API_KEY")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let model: String = noargs::opt("model")
        .ty("STRING")
        .default("text-embedding-3-small")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).or_fail()?;

    let embedder = Embedder::new(api_key, model);
    let embeddings = embedder.embed(&[input]).or_fail()?;
    println!("{}", nojson::Json(&embeddings));

    Ok(())
}
