use clap::Parser;

#[derive(Parser)]
struct Args {
    initial_message: Option<String>,
    #[arg(long)]
    config: Option<String>,
    #[arg(long)]
    save: Option<String>,
    #[arg(long)]
    endpoint: Option<String>,
    #[arg(long)]
    token: Option<String>,
    #[arg(long)]
    model: Option<String>,
}

fn main() {
    let args = Args::parse();
    llmaap::boot(
        args.config,
        args.save.as_ref(),
        args.endpoint,
        args.token,
        args.model,
        args.initial_message.as_ref(),
    );
}
