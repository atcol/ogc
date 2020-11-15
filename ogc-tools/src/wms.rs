extern crate clap;
extern crate colored;
extern crate prettytable;
extern crate reqwest;
extern crate strum;
extern crate strum_macros;
use anyhow::*;
use clap::Clap;
use colored::*;
use epsg::references::get_crs;
use ogc::wms::{Layer, WebMappingService, Wms};
use prettytable::{cell, row, Table};
use strum_macros::EnumString;

#[derive(Clap)]
#[clap(version = "1.0", author = "Alex Collins <grampz@pm.me>")]
struct Args {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
  GetCapabilities(GetCapabilities),
}

/// Operations for converting source data to GeoJSON
#[derive(Clap)]
struct GetCapabilities {
  /// The URL to read
  #[clap(short, long)]
  url: String,

  /// What data to select from the endpoint
  #[clap(short, long)]
  select: Option<Selection>,
}

/// What to show
#[derive(Clap, EnumString, PartialEq)]
enum Selection {
  All,
  Layers,
  CRS,
}

#[tokio::main]
async fn main() -> Result<(), String> {
  let args: Args = Args::parse();
  match args.subcmd {
    SubCommand::GetCapabilities(w) => {
      let mut wms = WebMappingService::from_url(w.url.clone()).unwrap();
      match wms.get_capabilities().await {
        Ok(capa) => {
          if let Some(top_layer) = capa.capability.layer {
            //FIXME extract to printing strategy for more complex "picks" of data
            let selection = w.select.unwrap_or(Selection::All);
            if selection == Selection::All || selection == Selection::CRS {
              println!("{}", "Shared CRS".underline());
              for shared_crs in &top_layer.crs() {
                println!(
                  "{} - {}",
                  shared_crs.bold(),
                  get_crs(shared_crs)
                    .map(|x| x.coord_ref_sys_name)
                    .unwrap_or("")
                );
              }
            }
            if selection == Selection::All || selection == Selection::Layers {
              print_table(&top_layer);
            }
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
    },
  }
}

fn print_table(top_layer: &Layer) {
  print!("{}", top_layer.name.bold().underline());
  let mut table = Table::new();
  table.add_row(row!["Name".bold(), "Abstract".bold(), "Keywords".bold()]);
  let mut abstr;
  let mut keywords;
  for layer in &top_layer.layers {
    // let l_crs = layer.crs();
    // let crs_list = l_crs.symmetric_difference(&top_layer_crs);
    // let mut crs_str = String::new();
    // for crs in crs_list {
    //   crs_str.push_str(",");
    //   crs_str.push_str(crs.as_str());
    // }
    // crs_str = crs_str.chars().skip(1).collect::<String>();
    abstr = layer.abstr.clone();
    abstr.truncate(30);
    
    keywords = layer.keyword_list.keyword.join(",");
    keywords.truncate(30);
    table.add_row(row![layer.name, format!("{}...", abstr), keywords]);

    //TODO recurse layers
    if layer.layers.len() > 0 {
      let layers = &layer.layers;
      layers.into_iter().for_each(print_table);
    }
  }
  table.printstd();
}
