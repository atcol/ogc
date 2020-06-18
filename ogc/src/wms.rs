use serde_xml_rs::from_reader;

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Service {
  #[serde(rename = "Abstract", default)]
  pub abstr: String,
  #[serde(rename = "Name", default)]
  pub name: String,
  #[serde(rename = "Title", default)]
  pub title: String,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Capability {
  #[serde(rename = "Layer", default)]
  pub layer: Layers,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Layers {
  #[serde(rename = "Abstract", default)]
  pub abstr: String,
  #[serde(rename = "Layer", default)]
  pub layers: Vec<Layer>,
  #[serde(rename = "Name", default)]
  pub name: String,
  #[serde(rename = "SRS", default)]
  pub srs: Vec<String>,
  #[serde(rename = "Title", default)]
  pub title: String,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Layer {
  #[serde(rename = "Abstract", default)]
  pub abstr: String,
  #[serde(rename = "LatLonBoundingBox", default)]
  pub ll_bbox: Option<LatLonBoundingBox>,
  #[serde(rename = "BoundingBox", default)]
  pub bbox: Option<BoundingBox>,
  #[serde(rename = "Name", default)]
  pub name: String,
  #[serde(rename = "SRS", default)]
  pub srs: String,
  #[serde(rename = "Title", default)]
  pub title: String,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LatLonBoundingBox {
  pub minx: f32,
  pub miny: f32,
  pub maxx: f32,
  pub maxy: f32,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct BoundingBox {
  pub minx: f32,
  pub miny: f32,
  pub maxx: f32,
  pub maxy: f32,

  #[serde(rename = "SRS", default)]
  pub srs: String,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ScaleHint {
  pub min: f32,
  pub max: f32,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Wms {
  #[serde(rename = "Service", default)]
  pub service: Service,
  #[serde(rename = "Capability", default)]
  pub capability: Capability,
}

pub fn from_string(xml: String) -> anyhow::Result<Wms> {
  match from_reader(xml.as_bytes()) {
    Err(e) => Err(anyhow::Error::msg(e)),
    Ok(w) => Ok(w),
  }
}

use proptest::prelude::*;

proptest! {
 #[test]
  fn test_invalid_safe(a in ".*") {
      prop_assert!(from_string(a).is_err());
  }
}

#[cfg(test)]
mod tests {
  use crate::wms::{from_string, Service, Wms};
  use std::fs::read_to_string;

  #[test]
  fn test_basic_parse() {
    let xml = read_to_string("./examples/WMS-1.1.1.xml").unwrap();
    let wms_opt = from_string(xml);
    println!("Service {:?}", wms_opt);
    assert!(wms_opt.is_ok());

    let wms = wms_opt.unwrap();
    assert_eq!(wms.service.name, "OGC:WMS");

    assert_eq!(
      wms.service.title,
      "Massachusetts Data from MassGIS (GeoServer)"
    );

    assert_eq!(
      wms.service.abstr,
      "Statewide Massachusetts data served by MassGIS via GeoServer."
    );

    assert_eq!(wms.capability.layers.len(), 1);
    for layers in wms.capability.layers.iter() {
      assert_eq!(layers.layers.len(), 1090);
    }
  }
}
