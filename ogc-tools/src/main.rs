extern crate clap;
extern crate prettytable;
extern crate reqwest;
use anyhow::*;
use clap::Clap;
use ogc::wms::{Layer, WebMappingService, Wms};
use prettytable::{cell, row, table, Table};

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
  let args: Args = Args::parse();
  match args.subcmd {
    SubCommand::Show(w) => {
      let mut wms = WebMappingService::from_url(w.url.clone()).unwrap();
      match wms.get_capabilities().await {
        Ok(capa) => {
          if let Some(top_layer) = capa.capability.layer {
            print_layers(top_layer);
          } else {
            println!("No layers available");
          }

          Ok(())
        }
        Err(e) => {
          println!("Failed to talk to WMS {}: {}", w.url, e);
          Err(format!("{:?}", e))
        }
      }
    }
  }
}

fn print_layers(top_layer: Layer) {
  let top_layer_crs = top_layer.crs();
  let mut top_table = table!(["Shared CRS"]);
  for g_crs in top_layer.crs() {
    top_table.add_row(row![g_crs]);
  }

  top_table.printstd();

  let mut table = Table::new();
  table.add_row(row!["Name", "CRS"]);
  for layer in top_layer.layers {
    let l_crs = layer.crs();
    let crs_list = l_crs.difference(&top_layer_crs);

    let mut crs_str = String::new();
    for crs in crs_list {
      crs_str.push_str(",");
      crs_str.push_str(crs.as_str());
    }
    crs_str = crs_str.chars().skip(1).collect::<String>();
    table.add_row(row![layer.name, crs_str]);

    //TODO recurse layers
  }
  table.printstd();
}
