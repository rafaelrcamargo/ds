use clap::{arg, Command};

pub fn args() -> Command {
    Command::new("ds")
        .about("Think \"docker stats\" but with beautiful, real-time charts. 📊")
        .arg_required_else_help(false)
        .arg(arg!(<CONTAINER> ... "The container to show stats for.").required(false))
        .arg(arg!(-c - -compact "Enable a simpler, more compact view."))
        .arg(arg!(-f - -full "Enable a more detailed view."))
}
