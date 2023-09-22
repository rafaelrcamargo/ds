use clap::{arg, ArgMatches, Command};

pub fn args() -> Command {
    Command::new("ds")
        .about("Think \"docker stats\" but with beautiful, real-time charts. 📊")
        .arg_required_else_help(false)
        .arg(arg!(<CONTAINER> ... "The container to show stats for.").required(false))
        .arg(arg!(-c - -compact "Enable a simpler, more compact view."))
        .arg(arg!(-f - -full "Enable a more detailed view."))
}

pub fn has_arg(args: &ArgMatches, id: &str) -> bool {
    args.get_one::<bool>(id).is_some_and(|x| *x == true)
}
