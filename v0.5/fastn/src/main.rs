#[tokio::main]
async fn main() {
    let command = fastn::commands::parse();
    let mut config = fastn_core::Config::read(Default::default()).await.unwrap();
    // read config here and pass to everyone?
    // do common build stuff here
    match command {
        fastn::commands::Cli::Serve(input) => input.run(config).await,
        fastn::commands::Cli::Render(input) => input.run(&mut config).await,
        fastn::commands::Cli::Build(input) => input.run(config).await,
        fastn::commands::Cli::Static { .. } => {}
        fastn::commands::Cli::Test { .. } => {}
        fastn::commands::Cli::Fmt(_) => {}
        fastn::commands::Cli::Upload { .. } => {}
        fastn::commands::Cli::Clone(_) => {}
    };
}