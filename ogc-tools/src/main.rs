extern crate clap;
extern crate prettytable;
extern crate reqwest;
use clap::Clap;
use ogc::wms::{WebMappingService, Wms};
use prettytable::{cell, row, Table};

#[derive(Clap)]
#[clap(version = "1.0", author = "Alex Collins <grampz@pm.me>")]
struct Args {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
  Show(ShowCmd),
}

#[derive(Clap)]
struct ShowCmd {
  /// The URL to read
  #[clap(short, long)]
  url: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
  //execute!(stdout(), EnterAlternateScreen).unwrap();
  let args: Args = Args::parse();
  match args.subcmd {
    SubCommand::Show(w) => {
      match WebMappingService::from_url(w.url.clone())
        .get_capabilities()
        .await
      {
        Ok(capa) => {
          // Create the table
          let mut table = Table::new();

          table.add_row(row!["Name", "SRS"]);
          if let Some(top_layer) = capa.capability.layer {
            for layer in top_layer.layers {
              let srs = if !layer.srs.is_empty() {
                layer.srs
              } else {
                top_layer
                  .srs
                  .get(0)
                  .unwrap_or(&"EPSG:4326".to_string())
                  .to_string()
              };
              table.add_row(row![layer.name, srs]);
            }
            table.printstd();
          } else {
            println!("No layers available");
          }

          Ok(())
        }
        Err(e) => {
          println!("Failed to talk to WMS URL {}: {}", w.url, e);
          Err(format!("{:?}", e))
        }
      }
    }
  }
}
