use serde_xml_rs::from_reader;

#[derive(Debug, Serialize, Deserialize)]
pub struct Type {
	#[serde(rename = "type")]
	pub Type: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeType {
	#[serde(rename = "typeType")]
	pub TypeType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Href {
	#[serde(rename = "href")]
	pub Href: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HrefType {
	#[serde(rename = "hrefType")]
	pub HrefType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
	#[serde(rename = "role")]
	pub Role: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleType {
	#[serde(rename = "roleType")]
	pub RoleType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arcrole {
	#[serde(rename = "arcrole")]
	pub Arcrole: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArcroleType {
	#[serde(rename = "arcroleType")]
	pub ArcroleType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
	#[serde(rename = "title")]
	pub Title: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleAttrType {
	#[serde(rename = "titleAttrType")]
	pub TitleAttrType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Show {
	#[serde(rename = "show")]
	pub Show: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShowType {
	#[serde(rename = "showType")]
	pub ShowType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Actuate {
	#[serde(rename = "actuate")]
	pub Actuate: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActuateType {
	#[serde(rename = "actuateType")]
	pub ActuateType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Label {
	#[serde(rename = "label")]
	pub Label: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelType {
	#[serde(rename = "labelType")]
	pub LabelType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct From {
	#[serde(rename = "from")]
	pub From: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FromType {
	#[serde(rename = "fromType")]
	pub FromType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct To {
	#[serde(rename = "to")]
	pub To: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToType {
	#[serde(rename = "toType")]
	pub ToType: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleAttrs {
	#[serde(rename = "xlink:type")]
	pub XlinkType: Vec<char>,
	#[serde(rename = "xlink:href")]
	pub XlinkHref: Vec<char>,
	#[serde(rename = "xlink:role")]
	pub XlinkRole: Vec<char>,
	#[serde(rename = "xlink:arcrole")]
	pub XlinkArcrole: Vec<char>,
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: Vec<char>,
	#[serde(rename = "xlink:show")]
	pub XlinkShow: Vec<char>,
	#[serde(rename = "xlink:actuate")]
	pub XlinkActuate: Vec<char>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimpleModel {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Simple {
	#[serde(rename = "xlink:simpleAttrs")]
	pub XlinkSimpleAttrs: Vec<SimpleAttrs>,
	#[serde(rename = "xlink:simpleModel")]
	pub XlinkSimpleModel: SimpleModel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtendedAttrs {
	#[serde(rename = "xlink:type")]
	pub XlinkType: Vec<char>,
	#[serde(rename = "xlink:role")]
	pub XlinkRole: Vec<char>,
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: Vec<char>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtendedModel {
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: char,
	#[serde(rename = "xlink:resource")]
	pub XlinkResource: ResourceType,
	#[serde(rename = "xlink:locator")]
	pub XlinkLocator: LocatorType,
	#[serde(rename = "xlink:arc")]
	pub XlinkArc: ArcType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extended {
	#[serde(rename = "xlink:extendedAttrs")]
	pub XlinkExtendedAttrs: Vec<ExtendedAttrs>,
	#[serde(rename = "xlink:extendedModel")]
	pub XlinkExtendedModel: Vec<ExtendedModel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleAttrs {
	#[serde(rename = "xlink:type")]
	pub XlinkType: Vec<char>,
	#[serde(rename = "xml:lang")]
	pub XmlLang: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleModel {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TitleEltType {
	#[serde(rename = "xlink:titleAttrs")]
	pub XlinkTitleAttrs: Vec<TitleAttrs>,
	#[serde(rename = "xlink:titleModel")]
	pub XlinkTitleModel: TitleModel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Resource {
	#[serde(rename = "resource")]
	pub Resource: ResourceType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceAttrs {
	#[serde(rename = "xlink:type")]
	pub XlinkType: Vec<char>,
	#[serde(rename = "xlink:role")]
	pub XlinkRole: Vec<char>,
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: Vec<char>,
	#[serde(rename = "xlink:label")]
	pub XlinkLabel: Vec<char>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceModel {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceType {
	#[serde(rename = "xlink:resourceAttrs")]
	pub XlinkResourceAttrs: Vec<ResourceAttrs>,
	#[serde(rename = "xlink:resourceModel")]
	pub XlinkResourceModel: ResourceModel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Locator {
	#[serde(rename = "locator")]
	pub Locator: LocatorType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocatorAttrs {
	#[serde(rename = "xlink:type")]
	pub XlinkType: Vec<char>,
	#[serde(rename = "xlink:href")]
	pub XlinkHref: Vec<char>,
	#[serde(rename = "xlink:role")]
	pub XlinkRole: Vec<char>,
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: Vec<char>,
	#[serde(rename = "xlink:label")]
	pub XlinkLabel: Vec<char>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocatorModel {
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocatorType {
	#[serde(rename = "xlink:locatorAttrs")]
	pub XlinkLocatorAttrs: Vec<LocatorAttrs>,
	#[serde(rename = "xlink:locatorModel")]
	pub XlinkLocatorModel: LocatorModel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Arc {
	#[serde(rename = "arc")]
	pub Arc: ArcType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArcAttrs {
	#[serde(rename = "xlink:type")]
	pub XlinkType: Vec<char>,
	#[serde(rename = "xlink:arcrole")]
	pub XlinkArcrole: Vec<char>,
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: Vec<char>,
	#[serde(rename = "xlink:show")]
	pub XlinkShow: Vec<char>,
	#[serde(rename = "xlink:actuate")]
	pub XlinkActuate: Vec<char>,
	#[serde(rename = "xlink:from")]
	pub XlinkFrom: Vec<char>,
	#[serde(rename = "xlink:to")]
	pub XlinkTo: Vec<char>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArcModel {
	#[serde(rename = "xlink:title")]
	pub XlinkTitle: char,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArcType {
	#[serde(rename = "xlink:arcAttrs")]
	pub XlinkArcAttrs: Vec<ArcAttrs>,
	#[serde(rename = "xlink:arcModel")]
	pub XlinkArcModel: ArcModel,
}
