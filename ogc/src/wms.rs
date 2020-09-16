//! Web Mapping Service support, versions 1.1.0 and 1.3.0.
//!
//! Typically one would use `WebMappingService::from_url` to invoke a remote
//! Web Mapping Service endpoint, e.g.:
//! ```
//! use ogc::wms::{Wms, WebMappingService};
//! #[tokio::main]
//! async fn main() -> Result<(), String> {
//!   let url =
//!   "http://giswebservices.massgis.state.ma.us/geoserver/wms?request=GetCapabilities&service=WMS&version=1.3.0".to_string();
//!   let capa = WebMappingService::from_url(url.clone()).unwrap()
//!         .get_capabilities().await.expect("Failure during GetCapabilities call");
//!   assert_eq!(capa.service.name, "WMS");
//!   assert_eq!(capa.service.title, "Massachusetts Data from MassGIS (GeoServer)");
//!   Ok(())
//! }
//! ```
//! ## WMS GetMap Support
//! The supported request parameters are:
//!  * VERSION
//!  * LAYERS
//!  * STYLES
//!  * SRS
//!  * WIDTH
//!  * HEIGHT
//!  * FORMAT
//!
//! e.g.:
//! ```
//! use ogc::wms::{BoundingBox, GetMapParameters, Wms, WebMappingService};
//! use std::fs::File;
//! use std::io::Write;
//! #[tokio::main]
//! async fn main() {
//!   let url = "https://ows.terrestris.de/osm/service?";
//!   let bytes = WebMappingService::from_url(url.to_string()).unwrap().get_map(
//!     GetMapParameters {
//!       layers: vec!["OSM-WMS".to_string()],
//!       srs: "EPSG:4326".to_string(),
//!       bbox: BoundingBox {
//!           srs: "EPSG:4326".to_string(),
//!           minx: -180.0,
//!           miny: -90.0,
//!           maxx: 180.0,
//!           maxy: 90.0,
//!       },
//!       ..GetMapParameters::default()
//!     }).await.unwrap();
//!   assert_ne!(bytes.len(), 0);
//!   let mut file = File::create("/tmp/terrestris-get-map.png").unwrap();
//!   assert!(file.write_all(&bytes).is_ok());
//! }
use anyhow::Context;
use async_trait::async_trait;
use serde_xml_rs::from_reader;
use std::collections::HashSet;
use url::Url;

/// Behaviour for a Web Mapping Service endpoint as per the specification.
#[async_trait]
pub trait Wms {
  /// The GetCapabilities request
  async fn get_capabilities(&mut self) -> anyhow::Result<GetCapabilities>;

  /// Optionally supported by a WMS endpoint
  async fn get_feature_info(&mut self) -> anyhow::Result<GetFeatureInfo> {
    Err(anyhow::Error::msg("Not supported"))
  }

  /// Perform the GetMap request against the configured endpoint
  async fn get_map(&mut self, req: GetMapParameters) -> anyhow::Result<bytes::Bytes>;
}

/// A configurable WMS endpoint
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WebMappingService {
  pub version: String,
  url: Option<Url>,
  raw_xml: Option<String>,
}

impl WebMappingService {
  /// Use the raw XML string as this "endpoint" for service calls
  fn from_string(xml: String) -> Self {
    WebMappingService {
      version: "1.3.0".to_string(),
      url: None,
      raw_xml: Some(xml),
    }
  }

  /// Use the given URL as the endpoint for service calls
  /// The URL should be the base URL for a WMS Service. Request parameters essential for
  /// WMS requests will be replaced accordingly.
  pub fn from_url(url: String) -> anyhow::Result<Self> {
    let mut url = Url::parse(&url)?;
    url
      .query_pairs_mut()
      .append_pair("REQUEST", "GetCapabilities")
      .append_pair("SERVICE", "WMS");
    Ok(WebMappingService {
      version: "1.3.0".to_string(),
      url: Some(url),
      raw_xml: None,
    })
  }
}

#[async_trait]
impl Wms for WebMappingService {
  /// The WMS GetCapabilities request
  async fn get_capabilities(&mut self) -> anyhow::Result<GetCapabilities> {
    match &self.raw_xml {
      None => match reqwest::get(self.url.clone().unwrap()).await?.text().await {
        Ok(xml) => {
          self.raw_xml = Some(xml);
          self.get_capabilities().await
        }
        Err(e) => Err(anyhow::Error::msg(e)),
      },
      Some(xml) => match from_reader(xml.as_bytes()) {
        Ok(w) => Ok(w),
        Err(e) => Err(anyhow::Error::msg(e)),
      },
    }
  }

  async fn get_map(&mut self, req: GetMapParameters) -> anyhow::Result<bytes::Bytes> {
    let mut url = self.url.clone().unwrap();
    url
      .query_pairs_mut()
      .clear()
      .append_pair("REQUEST", "GetMap")
      .append_pair("VERSION", &req.version)
      .append_pair("LAYERS", &req.layers_to_csv())
      .append_pair("STYLES", &req.styles_to_csv())
      .append_pair("SRS", &req.srs)
      .append_pair("CRS", &req.srs)
      .append_pair("BBOX", &req.bbox.to_str())
      .append_pair("WIDTH", &req.width.to_string())
      .append_pair("HEIGHT", &req.height.to_string())
      .append_pair("FORMAT", &req.format)
      .append_pair(
        "TRANSPARENT",
        &req.transparent.unwrap_or(true).to_string().to_uppercase(),
      )
      .append_pair("BG_COLOR", &req.bg_color.unwrap_or("".to_string()))
      .append_pair("EXCEPTIONS", &req.exceptions.unwrap_or("".to_string()))
      .append_pair("TIME", &req.time.unwrap_or("".to_string()))
      .append_pair("ELEVATION", &req.elevation.unwrap_or("".to_string()));

    let resp = reqwest::get(url).await?;
    match resp.status() {
      reqwest::StatusCode::OK => {
        if let Some(ct_type) = resp.headers().get("Content-Type") {
          if ct_type.to_str().unwrap().starts_with("image") {
            resp
              .bytes()
              .await
              .ok()
              .context("Failed to stream bytes for GetMap response")
          } else if ct_type.to_str().unwrap().contains("/xml") {
            Err(anyhow::Error::msg(format!(
              "Exception response for GetMap: {:?}",
              resp.text().await?
            )))
          } else {
            Err(anyhow::Error::msg(format!(
              "Unsupported content type: {:?}",
              ct_type
            )))
          }
        } else {
          // Best guess...
          resp
            .bytes()
            .await
            .ok()
            .context("Failed to stream bytes for GetMap response")
        }
      }
      _ => {
        let excep_xml = resp.text().await.ok().context("Couldn't stream text")?;
        Err(anyhow::Error::msg(excep_xml))
      }
    }
  }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct GetFeatureInfo {}

/// General service metadata
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
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

/// The root element
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Capability {
  #[serde(rename = "Layer", default)]
  pub layer: Option<Layer>,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
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
  #[serde(rename = "CRS", default)]
  crs: HashSet<String>,
  #[serde(rename = "SRS", default)]
  srs: HashSet<String>, // 1.1.0 compat
  #[serde(rename = "Title", default)]
  pub title: String,
  #[serde(rename = "Layer", default)]
  pub layers: Vec<Layer>,
}

impl Layer {
  pub fn crs(&self) -> HashSet<String> {
    let mut combined_crs = HashSet::new();
    combined_crs.extend(self.crs.clone());
    combined_crs.extend(self.srs.clone());
    combined_crs
  }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct LatLonBoundingBox {
  pub minx: f32,
  pub miny: f32,
  pub maxx: f32,
  pub maxy: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct BoundingBox {
  pub minx: f32,
  pub miny: f32,
  pub maxx: f32,
  pub maxy: f32,

  #[serde(rename = "SRS", default)]
  pub srs: String,
}

impl BoundingBox {
  /// Yield minx,miny,maxx,maxy as-per the usual formatting of a bounding box
  fn to_str(&self) -> String {
    format!("{},{},{},{}", self.minx, self.miny, self.maxx, self.maxy)
  }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ScaleHint {
  pub min: f32,
  pub max: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct GetCapabilities {
  #[serde(rename = "Service", default)]
  pub service: Service,
  #[serde(rename = "Capability", default)]
  pub capability: Capability,
}

/// The parameters for a GetMap service request, as per [the WMS test data spec](http://cite.opengeospatial.org/OGCTestData/wms/1.1.1/spec/wms1.1.1.html#wmsops.getmap).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct GetMapParameters {
  /// Request version.  
  pub version: String,
  /// Comma-separated list of one or more map layers. Optional if SLD parameter is present.
  pub layers: Vec<String>,
  /// Comma-separated list of one rendering style per requested layer. Optional if SLD parameter is present.
  pub styles: Vec<String>,
  /// namespace:identifier - Spatial Reference System.
  pub srs: String,
  /// minx,miny,maxx,maxy R Bounding box corners (lower left, upper right) in SRS units.
  pub bbox: BoundingBox,
  /// Width in pixels of map picture.  
  pub width: u16,
  /// Height in pixels of map picture.
  pub height: u16,
  /// Output format of map.
  pub format: String,
  /// Background transparency of map (default=FALSE).
  pub transparent: Option<bool>,
  /// Red-green-blue color value for the background color (default=0xFFFFFF).
  pub bg_color: Option<String>,
  /// The format in which exceptions are to be reported by the WMS (default=SE_XML).
  pub exceptions: Option<String>,
  /// Time value of layer desired.
  pub time: Option<String>,
  /// Elevation of layer desired.
  pub elevation: Option<String>,
}

impl GetMapParameters {
  fn layers_to_csv(&self) -> String {
    if self.layers.len() > 1 {
      self.layers.join(",")
    } else {
      self.layers[0].clone()
    }
  }

  fn styles_to_csv(&self) -> String {
    match self.styles.len() {
      0 => "".to_string(),
      1 => self.styles.join(","),
      _ => self.styles[0].clone(),
    }
  }
}

impl Default for GetMapParameters {
  fn default() -> Self {
    GetMapParameters {
      version: "1.3.0".to_string(),
      layers: Vec::new(),
      styles: Vec::new(),
      srs: "CRS:84".to_string(),
      bbox: BoundingBox::default(),
      width: 250,
      height: 250,
      format: "image/png".to_string(),
      transparent: None,
      bg_color: None,
      exceptions: None,
      time: None,
      elevation: None,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::wms::{BoundingBox, GetCapabilities, GetMapParameters, WebMappingService, Wms};
  use std::fs::read_to_string;
  use std::fs::File;
  use std::io::Write;

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
  async fn test_get_map() {
    //<Name>ROADS_RIVERS</Name>
    //<CRS>EPSG:26986</CRS> <!-- An additional CRS for this layer -->
    //<BoundingBox CRS="CRS:84"
    //minx="-71.63" miny="41.75" maxx="-70.78" maxy="42.90" resx="0.01" resy="0.01"/>
    let params = GetMapParameters {
      layers: vec!["OSM-WMS".to_string()],
      srs: "EPSG:4326".to_string(),
      bbox: BoundingBox {
        srs: "EPSG:4326".to_string(),
        minx: -180.0,
        miny: -90.0,
        maxx: 180.0,
        maxy: 90.0,
      },
      ..GetMapParameters::default()
    };
    let url =
      "http://ows.mundialis.de/services/service?SERVICE=WMS&REQUEST=GetCapabilities&VERSION=1.3.0"
        .to_string();
    let get_map_res = WebMappingService::from_url(url)
      .unwrap()
      .get_map(params)
      .await;
    match get_map_res {
      Ok(bytes) => {
        assert_ne!(bytes.len(), 0);
        let mut file = File::create("/tmp/test-get-map.png").unwrap();
        assert!(file.write_all(&bytes).is_ok());
      }
      Err(e) => {
        panic!(e);
      }
    }
  }

  #[tokio::test]
  async fn test_basic_parse_v1_1_1() {
    let xml = read_to_string("./examples/WMS-1.1.1.xml").unwrap();
    let mut wms_opt = WebMappingService::from_string(xml);
    let get_capa = wms_opt.get_capabilities().await.unwrap();
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
    let top_layer = get_capa.capability.clone().layer.unwrap();
    assert_eq!(top_layer.title, "Acme Corp. Map Server");
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

  #[tokio::test]
  async fn test_get_crs_layer() {
    let xml = read_to_string("./examples/WMS-1.3.0.xml").unwrap();
    let mut wms_opt = WebMappingService::from_string(xml);
    let gc = wms_opt.get_capabilities().await.unwrap();
    let layer_list = gc.capability.layer.unwrap();
    let mut ct = 1;
    for layer in layer_list.layers.iter() {
      ct += layer.crs().len();
      if layer.layers.len() > 0 {
        for i_layer in layer.layers.iter() {
          println!("Layer is {:?} {:?}", i_layer.title, i_layer.crs());
          ct += i_layer.crs().len();
        }
      }
    }
    assert_eq!(ct, 3);
  }

  #[test]
  fn test_boundingbox() {
    let bbox = BoundingBox {
      srs: "EPSG:26986".to_string(),
      minx: -71.63,
      miny: 41.75,
      maxx: -70.78,
      maxy: 42.90,
    };
    assert_eq!("-71.63,41.75,-70.78,42.9", bbox.to_str());
  }
}
