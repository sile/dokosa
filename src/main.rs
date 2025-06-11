fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();

    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // $ add REPOSITORT_PATH
    // $ remove REPOSITORY_PATH
    // $ sync
    // $ list
    // $ search QUERY

    if noargs::cmd("add").take(&mut args).is_present() {
        dokosa::subcommand_add::run(args)?;
    } else if noargs::cmd("embed").take(&mut args).is_present() {
        dokosa::subcommand_embed::run(args)?;
    } else if noargs::cmd("chunk").take(&mut args).is_present() {
        dokosa::subcommand_chunk::run(args)?;
    } else if noargs::cmd("index").take(&mut args).is_present() {
        dokosa::subcommand_index::run(args)?;
    } else if noargs::cmd("search").take(&mut args).is_present() {
        dokosa::subcommand_search::run(args)?;
    } else if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    Ok(())
}
