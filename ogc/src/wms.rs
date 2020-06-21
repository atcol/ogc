/// Web Mapping Service support, v1.3.0.
use async_trait::async_trait;
use serde_xml_rs::from_reader;
use std::io::{Error, ErrorKind};
use std::fs::read_to_string;
use std::path::PathBuf;

#[async_trait]
trait Wms {
  async fn get_capabilities(&self, source: String) -> Result<GetCapabilities, std::io::Error>;
  async fn get_feature_info(&self, source: String) -> Result<GetFeatureInfo, std::io::Error> {
    Err(Error::new(ErrorKind::Other, "Not supported"))
  }
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct GetFeatureInfo {}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Service {
  #[serde(rename = "Abstract", default)]
  pub abstr: String,
  #[serde(rename = "Name", default)]
  pub name: String,
  #[serde(rename = "Title", default)]
  pub title: String,
  pub maxWidth: Option<u32>,
  pub MaxHeight: Option<u32>,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Capability {
  #[serde(rename = "Layer", default)]
  pub layer: Option<LayerList>,
}

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LayerList {
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
  /// The LatLonBoundingBox element
  #[serde(rename = "LatLonBoundingBox", default)]
  pub ll_bbox: Option<LatLonBoundingBox>,
  #[serde(rename = "BoundingBox", default)]
  pub bbox: Vec<BoundingBox>,
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
pub struct GetCapabilities {
  #[serde(rename = "Service", default)]
  pub service: Service,
  #[serde(rename = "Capability", default)]
  pub capability: Capability,
}

/// Read the current WMS GetCapabilities XML String and parse it to a `GetCapabilities` instance
pub fn get_capabilities_string(xml: String) -> Result<GetCapabilities, std::io::Error> {
  match from_reader(xml.as_bytes()) {
    Ok(w) => Ok(w),
    Err(e) => Err(Error::new(ErrorKind::InvalidData, e))
  }
}

/// Read the file at `p` and parse as a WMS GetCapabilities response
pub fn get_capabilities_path(p: PathBuf) -> Result<GetCapabilities, std::io::Error> {
  match p.into_os_string().to_str() {
    Some(path) => read_to_string(path)
        .and_then(|xml_str| get_capabilities_string(xml_str))
        .or(Err(Error::new(ErrorKind::InvalidData, "Failed to parse as GetCapabilities"))),
    None => Err(Error::new(ErrorKind::InvalidInput, "Could not convert to path")),
  }
}

use proptest::prelude::*;

proptest! {
  #[test]
  fn test_invalid_safe(a in ".*") {
      prop_assert!(get_capabilities_string(a).is_err());
  }
}

#[cfg(test)]
mod tests {
  use crate::wms::{get_capabilities_string, GetCapabilities, Service};
  use std::fs::read_to_string;
  use std::io::{Error, ErrorKind};

  struct ParseExpectation {
    service_name:  String,
    service_title: String,
    service_abstr: String,
    inner_layers_len: usize,
    skip_llbbox: bool,
    skip_layer_srs: bool,
    skip_layer_bbox: bool,
    skip_layer_list_name: bool,
  }

  fn verify_parse(wms_opt: Result<GetCapabilities, Error>, exp: ParseExpectation) {
    assert!(wms_opt.is_ok());

    let wms = wms_opt.unwrap();
    assert_eq!(wms.service.name, exp.service_name);

    assert_eq!(
      wms.service.title,
      exp.service_title
    );

    assert_eq!(
      wms.service.abstr,
      exp.service_abstr,
    );

    assert!(wms.capability.layer.is_some());
    let layer_list = wms.capability.layer.unwrap();
    assert_eq!(layer_list.layers.len(), exp.inner_layers_len);
    for layer in layer_list.layers.iter() {
      if !exp.skip_llbbox {
          assert!(layer.ll_bbox.is_some());
      }
      if !exp.skip_layer_bbox {
        assert!(!layer.bbox.is_empty());
      }
      if !exp.skip_layer_list_name {
          assert!(!layer.name.is_empty());
      }
      if !exp.skip_layer_srs {
          assert!(!layer.srs.is_empty());
      }
      assert!(!layer.title.is_empty());
    }
  }

  #[test]
  fn test_basic_parse_v1_1_1() {
    let xml = read_to_string("./examples/WMS-1.1.1.xml").unwrap();
    let wms_opt = get_capabilities_string(xml);
    println!("{:?}", wms_opt);
    verify_parse(wms_opt, ParseExpectation {
        service_name: "OGC:WMS".to_string(),
        service_title: "Massachusetts Data from MassGIS (GeoServer)".to_string(),
        service_abstr: "Statewide Massachusetts data served by MassGIS via GeoServer.".to_string(),
        inner_layers_len: 1090,
        skip_llbbox: false,
        skip_layer_srs: false,
        skip_layer_bbox: false,
        skip_layer_list_name: false,
    });
  }

  #[test]
  fn test_basic_parse_v1_3_0() {
    let xml = read_to_string("./examples/WMS-1.3.0.xml").unwrap();
    let wms_opt = get_capabilities_string(xml);
    println!("{:?}", wms_opt);
    verify_parse(wms_opt, ParseExpectation {
        service_name: "WMS".to_string(),
        service_title: "Acme Corp. Map Server".to_string(),
        service_abstr: "Map Server maintained by Acme Corporation.  Contact: webmaster@wmt.acme.com.  High-quality maps showing roadrunner nests and possible ambush locations.".to_string(),
        inner_layers_len: 4,
        skip_llbbox: true,
        skip_layer_srs: true,
        skip_layer_bbox: true,
        skip_layer_list_name: true,
    });
  }
}
