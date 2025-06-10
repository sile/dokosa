fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();

    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    if noargs::cmd("embed").take(&mut args).is_present() {
        saguru::subcommand_embed::run(args)?;
    } else if noargs::cmd("chunk").take(&mut args).is_present() {
        saguru::subcommand_chunk::run(args)?;
    } else if noargs::cmd("index").take(&mut args).is_present() {
        saguru::subcommand_index::run(args)?;
    } else if noargs::cmd("search").take(&mut args).is_present() {
        saguru::subcommand_search::run(args)?;
    } else if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    Ok(())
}
