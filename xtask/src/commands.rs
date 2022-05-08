use clap::{Args, Parser};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub enum Command {
    #[clap(about = "Run unit tests")]
    Test(TestArgs),

    #[clap(about = "Download and build instruction set documentation")]
    BuildDocs(BuildDocArgs),
}

#[derive(Args)]
pub struct TestArgs {
    #[clap(long)]
    #[clap(help = "Generate coverage report")]
    pub coverage: bool,
}

#[derive(Args)]
pub struct BuildDocArgs {
    #[clap(long)]
    #[clap(
        help = "Downloads and builds the documentation for the AArch64 instruction set from arm.com"
    )]
    pub aarch64: bool,
}

pub(crate) fn get_args() -> Command {
    Command::parse()
}
