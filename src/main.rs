extern crate clap;
use clap::Clap;


#[derive(Clap)]
#[clap(version = "1.0", author = "Alex Collins <grampz@pm.me>")]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Wms(WmsCmd),
}

#[derive(Clap)]
struct WmsCmd {
    /// The URL to read
    #[clap(short, long)]
    url: String, 
}

async fn get(url: String) -> Result<String, reqwest::Error> {
    reqwest::get(&url)
            .await?
            .text()
            .await
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();
    match args.subcmd {
        SubCommand::Wms(w) => {
            match get(w.url).await {
                Ok(xml) => println!("Command {:?}", ogc::wms::from_string(xml)),
                Err(e)  => println!("Bad URL? {:?}", e),
            }
        },
    }
}
