/// Web Mapping Service support, v1.3.0.
use async_trait::async_trait;
use serde_xml_rs::from_reader;

/// Generic behaviour for a Web Mapping Service endpoint
#[async_trait]
pub trait Wms {
  /// The GetCapabilities request
  async fn get_capabilities(&mut self) -> anyhow::Result<GetCapabilities>;

  /// Optionally supported by a WMS endpoint
  async fn get_feature_info(&mut self) -> anyhow::Result<GetFeatureInfo> {
    Err(anyhow::Error::msg("Not supported"))
  }
}

/// A configurable WMS endpoint
#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct WebMappingService {
  pub version: String,
  url: Option<String>,
  raw_xml: Option<String>,
}

impl WebMappingService {

  // Use the raw XML string as this "endpoint" for service calls
  fn from_string(xml: String) -> Self {
    WebMappingService {
      version: "1.3.0".to_string(),
      url: None,
      raw_xml: Some(xml),
    }
  }

  // Use the given URL as the endpoint for service calls
  fn from_url(url: String) -> Self {
    WebMappingService {
      version: "1.3.0".to_string(),
      url: Some(url),
      raw_xml: None,
    }
  }
}

#[async_trait]
impl Wms for WebMappingService {

  /// The WMS GetCapabilities request
  async fn get_capabilities(&mut self) -> anyhow::Result<GetCapabilities> {
    match &self.raw_xml {
        None => {
            match reqwest::get(self.url.as_ref().unwrap()).await?.text().await {
                Ok(xml) => {
                    self.raw_xml = Some(xml);
                    self.get_capabilities().await
                },
                Err(e) => Err(anyhow::Error::msg(e))

            }
        },
        Some(xml) => {
            match from_reader(xml.as_bytes()) {
                Ok(w) => Ok(w),
                Err(e) => Err(anyhow::Error::msg(e)),
            }
        },
    }
  }

  ///// Read the file at `p` and parse as a WMS GetCapabilities response
  //async fn get_capabilities_path(p: PathBuf) -> Result<GetCapabilities, std::io::Error> {
  //  match p.into_os_string().to_str() {
  //    Some(path) => read_to_string(path)
  //      .and_then(|xml_str| self.from_string(xml_str))
  //      .and_then(|wms| wms.get_capabilities())
  //      .or(Err(Error::new(
  //        ErrorKind::InvalidData,
  //        "Failed to parse as GetCapabilities",
  //      ))),
  //    None => Err(Error::new(
  //      ErrorKind::InvalidInput,
  //      "Could not convert to path",
  //    )),
  //  }
  //}
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
  #[serde(rename = "MaxWidth", default)]
  pub max_width: Option<u32>,
  #[serde(rename = "MaxHeight", default)]
  pub max_height: Option<u32>,
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

#[cfg(test)]
mod tests {
  use crate::wms::{Wms, WebMappingService, GetCapabilities, Service};
  use std::fs::read_to_string;

  struct ParseExpectation {
    service_name: String,
    service_title: String,
    service_abstr: String,
    inner_layers_len: usize,
    skip_llbbox: bool,
    skip_layer_srs: bool,
    skip_layer_bbox: bool,
    skip_layer_list_name: bool,
  }

  fn verify_parse(wms: GetCapabilities, exp: ParseExpectation) {
    assert_eq!(wms.service.name, exp.service_name);

    assert_eq!(wms.service.title, exp.service_title);

    assert_eq!(wms.service.abstr, exp.service_abstr,);

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

  #[tokio::test]
  async fn test_basic_parse_v1_1_1() {
    let xml = read_to_string("./examples/WMS-1.1.1.xml").unwrap();
    let mut wms_opt = WebMappingService::from_string(xml);
    let get_capa = wms_opt.get_capabilities().await.unwrap();
    println!("{:?}", wms_opt);
    verify_parse(
      get_capa,
      ParseExpectation {
        service_name: "OGC:WMS".to_string(),
        service_title: "Massachusetts Data from MassGIS (GeoServer)".to_string(),
        service_abstr: "Statewide Massachusetts data served by MassGIS via GeoServer.".to_string(),
        inner_layers_len: 1090,
        skip_llbbox: false,
        skip_layer_srs: false,
        skip_layer_bbox: false,
        skip_layer_list_name: false,
      },
    );
  }

  #[tokio::test]
  async fn test_basic_parse_v1_3_0() {
    let xml = read_to_string("./examples/WMS-1.3.0.xml").unwrap();
    let mut wms_opt = WebMappingService::from_string(xml);
    let get_capa = wms_opt.get_capabilities().await.unwrap();
    println!("{:?}", wms_opt);
    verify_parse(get_capa, ParseExpectation {
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
