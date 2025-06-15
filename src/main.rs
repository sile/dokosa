fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();

    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    if noargs::cmd("add")
        .doc("Add and index a Git repository for semantic search")
        .take(&mut args)
        .is_present()
    {
        dokosa::subcommand_add::run(args)?;
    } else if noargs::cmd("remove")
        .doc("Remove a Git repository from the index")
        .take(&mut args)
        .is_present()
    {
        dokosa::subcommand_remove::run(args)?;
    } else if noargs::cmd("list")
        .doc("List all indexed repositories")
        .take(&mut args)
        .is_present()
    {
        dokosa::subcommand_list::run(args)?;
    } else if noargs::cmd("sync")
        .doc("Synchronize indexed repositories with their latest commits")
        .take(&mut args)
        .is_present()
    {
        dokosa::subcommand_sync::run(args)?;
    } else if noargs::cmd("search")
        .doc("Search for semantically similar text chunks")
        .take(&mut args)
        .is_present()
    {
        dokosa::subcommand_search::run(args)?;
    } else if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    Ok(())
}
