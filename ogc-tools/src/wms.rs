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
use geojson::{Feature, GeoJson, Geometry};
use ogc::wms::{Layer, WebMappingService, Wms};
use prettytable::{cell, row, Table};
use proj::{Proj};
use shapefile::{Reader, Shape};
use std::convert::TryFrom;
use std::io::prelude::*;
use strum_macros::EnumString;

#[derive(Clap)]
#[clap(version = "1.0", author = "Alex Collins <grampz@pm.me>")]
struct Args {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
  /// Convert Shapefiles to GeoJSON
  Convert(ConvertCmd),
  Show(ShowCmd),
}

/// Operations for converting source data to GeoJSON
#[derive(Clap)]
struct ConvertCmd {
  /// The absolute path to the file to read
  #[clap(short, long)]
  file: String,

  /// The name of the source Spatial Reference System to use
  #[clap(long)]
  from_crs: Option<String>,

  /// The name of the source Spatial Reference System to use
  #[clap(long)]
  to_crs: Option<String>,
}

#[derive(Clap)]
struct ShowCmd {
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
    SubCommand::Show(w) => {
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
    SubCommand::Convert(cmd) => {
      let shapes = Reader::from_path(cmd.file.clone()).unwrap().into_iter();
      let (from_crs, to_crs) = (cmd.from_crs.clone(), cmd.to_crs.clone());
      let features = shapes.map(|shape| {
        let sh = shape.expect("Invalid shape");
        if let (Some(fcrs), Some(tcrs)) = (from_crs.clone(), to_crs.clone()) {
          let p = Proj::new_known_crs(&fcrs, &tcrs, None).expect("Could load CRS for conversion");
          geometry_to_feature(convert(p, geo_types::Geometry::try_from(sh).unwrap()))
        } else {
          shape_to_feature(sh)
        }
      });
      let fc = geojson::FeatureCollection {
        bbox: None,
        features: features.collect(),
        foreign_members: None,
      };
      let mut file =
        std::fs::File::create(format!("{}_geo.json", cmd.file)).expect("Couldn't create GeoJSON file");
      file
        .write_all(
          &GeoJson::FeatureCollection(fc)
            .to_string()
            .as_bytes()
            .to_vec(),
        )
        .expect("Couldn't save to GeoJSON file");
      Ok(())
    }
  }
}

fn convert(p: Proj, geom: geo_types::Geometry<f64>) -> geo_types::Geometry<f64> {
  match geom {
      geo_types::Geometry::MultiLineString(ls) => {
        let x: Vec<geo_types::Point<f64>> = ls.into_iter().flat_map(|l| l.into_points()).map(|pt| {
          let cpt = p.convert(pt).unwrap();
          cpt
        }).collect();
        geo_types::MultiLineString::from(x).into()
      }
      geo_types::Geometry::Point(pt) => {
        geo_types::Point::from(p.convert(pt).unwrap()).into()
      }
      geo_types::Geometry::Line(l) => {
        geo_types::Line::new(p.convert(l.start_point()).unwrap(), p.convert(l.end_point()).unwrap()).into()
      }
      geo_types::Geometry::LineString(ls) => {
        let x: Vec<geo_types::Point<f64>> = ls.into_points().into_iter().map(|point| p.convert(point).unwrap()).collect();
        geo_types::LineString::from(x).into()
      }
      // geo_types::Geometry::Polygon(po) => {
      //   let i: Vec<geo_types::LineString<f64>> = convert(p, po.interiors().into_iter().collect()).try_into().unwrap();
      //   // .into_iter()
      //     // .map(|x| convert(p.clone(), geo_types::Geometry::from(*x)))
      //     // .map(|geom| geom.try_into().unwrap())
      //     // .collect();
      //   let translated_pts: Vec<geo_types::Point<f64>> = po.exterior().points_iter()
      //     .map(|epo| convert(p, epo.into()).try_into().unwrap())
      //     .collect();
      //   let g = geo_types::LineString::from(translated_pts);
      //   // let e = convert(p, g);
      //   geo_types::Polygon::new(g.try_into().unwrap(), i).into()
      // }
      geo_types::Geometry::MultiPoint(mp) => {
        geo_types::MultiPoint::from(mp.into_iter().map(|pt| p.convert(pt).unwrap()).collect::<Vec<geo_types::Point<f64>>>()).into()
      }
      // geo_types::Geometry::MultiPolygon(mp) => {
      //   let pts: Vec<geo_types::Polygon<f64>> = mp.into_iter()
      //     .map(|pol| pol.p.convert(p))
      //     .collect();
      //   geo_types::MultiPolygon::from(pts).into()
      // }
      // geo_types::Geometry::GeometryCollection(gc) => {
      //   geo_types::GeometryCollection(gc.into_iter().map(|g| convert(p, g)).collect()).into()
      // }
      geo_types::Geometry::Rect(r) => {
        geo_types::Rect::new(r.min(), r.max()).into()
      }
      // geo_types::Geometry::Triangle(t) => {
        // t.to_lines().into_iter().map(|l| convert(p, *l.into())).into()
        // let ar = t.to_array().into_iter().map(|co| p.convert(*co).unwrap());
        // geo_types::Triangle::from(ar).into()
      // }
      e => panic!(format!("Unsupported: {:?}", e))
  }
}

fn shape_to_feature(shape: Shape) -> Feature {
  let shape_geom: geo_types::Geometry<f64> =
    geo_types::Geometry::try_from(shape).expect("Failed to convert shape");
  geometry_to_feature(shape_geom)
}

fn geometry_to_feature(shape_geom: geo_types::Geometry<f64>) -> Feature {
  let val = geojson::Value::from(&shape_geom);
  Feature {
    bbox: None,
    geometry: Some(Geometry::new(val)),
    id: None,
    properties: None,
    foreign_members: None,
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
