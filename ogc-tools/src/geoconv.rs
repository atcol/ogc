extern crate clap;
use clap::Clap;
use colored::*;
use geojson::{Feature, GeoJson, Geometry};
use proj::{Proj};
use shapefile::{Reader, Shape};
use std::convert::TryFrom;
use std::io::prelude::*;

#[derive(Clap)]
#[clap(version = "1.0", author = "Alex Collins <grampz@pm.me>")]
struct Args {
  #[clap(subcommand)]
  subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
  Convert(ConvertCmd),
}

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

//     SubCommand::Convert(cmd) => {
//       let shapes = Reader::from_path(cmd.file.clone()).unwrap().into_iter();
//       let (from_crs, to_crs) = (cmd.from_crs.clone(), cmd.to_crs.clone());
//       let features = shapes.map(|shape| {
//         let sh = shape.expect("Invalid shape");
//         if let (Some(fcrs), Some(tcrs)) = (from_crs.clone(), to_crs.clone()) {
//           let p = Proj::new_known_crs(&fcrs, &tcrs, None).expect("Could load CRS for conversion");
//           geometry_to_feature(convert(p, geo_types::Geometry::try_from(sh).unwrap()))
//         } else {
//           shape_to_feature(sh)
//         }
//       });
//       let fc = geojson::FeatureCollection {
//         bbox: None,
//         features: features.collect(),
//         foreign_members: None,
//       };
//       let mut file =
//         std::fs::File::create(format!("{}_geo.json", cmd.file)).expect("Couldn't create GeoJSON file");
//       file
//         .write_all(
//           &GeoJson::FeatureCollection(fc)
//             .to_string()
//             .as_bytes()
//             .to_vec(),
//         )
//         .expect("Couldn't save to GeoJSON file");
//       Ok(())
//     }
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

// fn shape_to_feature(shape: Shape) -> Feature {
//   // let shape_geom: geo_types::Geometry<f64> =
//   //   geo_types::Geometry::try_from(shape).expect("Failed to convert shape");
//   // geometry_to_feature(shape_geom)
// }

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

pub fn main() {
    print!("TODO");
}