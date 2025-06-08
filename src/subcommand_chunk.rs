use std::io::Read;

use orfail::OrFail;

use crate::chunker::Chunker;

pub fn run(args: noargs::RawArgs) -> noargs::Result<()> {
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let chunker = Chunker::new();
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).or_fail()?;
    let chunks = chunker.apply(&input);

    println!("{}", nojson::Json(chunks));

    Ok(())
}
