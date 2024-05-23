use clap::{Parser, Subcommand};
use geo::{Geometry, MultiPolygon};
use geo::{Point, Polygon};
use geojson::{Feature, FeatureCollection, Value};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Parser)]
#[command(name = "cimengine", bin_name = "cimengine")]
#[command(about = "CIMEngine build tools")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Debug, Subcommand)]
#[clap(author, version, about)]
pub enum Commands {
    /// Build project
    Build,
    /// Initialize a new project
    Init {
        #[clap(default_value = "map")]
        name: String,
    },
    /// Fix geospatial file
    Fix {
        /// Path to geospatial file supported by cimengine
        #[clap(short, long)]
        path: String,
    },
    /// Utility for creating countries, roads, etc.
    New {
        #[command(subcommand)]
        cmd: NewCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum NewCommands {
    /// Create new country
    Country {
        id: String,
        #[clap(long)]
        name: Option<String>,
        #[clap(long)]
        description: Option<String>,
        #[clap(long)]
        foundation_date: Option<String>,
        #[clap(long)]
        flag: Option<String>,
        #[clap(long)]
        about: Option<String>,
        #[clap(long)]
        fill: Option<String>,
        #[clap(long)]
        stroke: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub main: MainConfig,
    pub processing: Vec<ProcessingConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MainConfig {
    pub layers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingConfig {
    pub show_markers: Option<bool>,
    pub output_folder: String,

    pub tags: Option<Vec<String>>,
    pub countries_rewrite: Option<CountryRewriteConfig>,
    pub public: Option<PublicConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountryRewriteConfig {
    pub name: Option<String>,
    pub description: Option<String>,
    pub foundation_date: Option<String>,
    pub flag: Option<String>,
    pub about: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicConfig {
    pub name: String,
    pub description: String,
    pub geo: String,
    pub countries: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountryConfig {
    pub name: String,
    pub description: String,
    pub foundation_date: String,
    pub flag: String,
    pub fill: String,
    pub stroke: String,
    pub about: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountryData {
    pub id: String,
    pub config: CountryConfig,
    pub geo: FeatureCollection,
}

pub struct Marker {
    pub coordinates: Point,
    pub title: String,
    pub description: String,
    pub ty: MarkerType,
}

impl ToFeature for Marker {
    fn to_feature(&self) -> geojson::Feature {
        geojson::Feature {
            geometry: Some(geojson::Geometry::new(Value::Point(vec![
                self.coordinates.x(),
                self.coordinates.y(),
            ]))),
            properties: Some(
                serde_json::Map::from_iter([
                    ("title".to_owned(), json!(self.title)),
                    ("description".to_owned(), json!(self.description)),
                    ("marker-type".to_owned(), json!(self.ty.to_str())),
                ])
                .into(),
            ),

            bbox: None,
            id: None,
            foreign_members: None,
        }
    }
}

pub enum MarkerType {
    Capital,
    City,
    Landmark,
}

impl MarkerType {
    pub fn to_str(&self) -> &'static str {
        match self {
            MarkerType::Capital => "capital",
            MarkerType::City => "city",
            MarkerType::Landmark => "landmark-0",
        }
    }
}

pub enum Territory {
    Polygon(Polygon),
    MultiPolygon(MultiPolygon),
}

impl ToTerritory for MultiPolygon {
    fn to_territory(&self) -> Territory {
        Territory::MultiPolygon(self.clone())
    }
}

impl ToCountryFeature for MultiPolygon {
    fn to_country_feature(&self, id: &String, config: &CountryConfig) -> geojson::Feature {
        geojson::Feature {
            geometry: Some(geojson::Geometry::new((self).into())),
            properties: Some(
                serde_json::Map::from_iter([
                    ("id".to_owned(), json!(id)),
                    ("type".to_owned(), json!("country")),
                    ("fill".to_owned(), json!(config.fill)),
                    ("stroke".to_owned(), json!(config.stroke)),
                    ("tags".to_owned(), json!(config.tags)),
                ])
                .into(),
            ),

            bbox: None,
            id: None,
            foreign_members: None,
        }
    }
}

impl ToFeatures for Vec<Marker> {
    fn to_features(&self) -> Vec<geojson::Feature> {
        self.iter().map(|m| m.to_feature()).collect()
    }
}

impl ToCollection for Vec<geojson::Feature> {
    fn to_collection(self) -> geojson::FeatureCollection {
        geojson::FeatureCollection {
            features: self,
            bbox: None,
            foreign_members: None,
        }
    }
}

impl ToSplitGeo for FeatureCollection {
    fn split_geo(&self) -> (Vec<Marker>, Vec<Territory>) {
        let mut markers: Vec<Marker> = vec![];
        let mut territories: Vec<Territory> = vec![];

        self.features.iter().for_each(|f| {
            let properties = f.properties.clone().unwrap();

            let geometry: Geometry = f.geometry.clone().unwrap().try_into().unwrap();

            match geometry {
                Geometry::Point(p) => {
                    let ty = match properties
                        .get("type")
                        .expect("Missing marker type")
                        .to_string()
                        .trim_matches('"')
                    {
                        "capital" | "capital-city" => MarkerType::Capital,
                        "city" => MarkerType::City,
                        "landmark" => MarkerType::Landmark,

                        t => panic!("Invalid marker type: {}", t),
                    };

                    markers.push(Marker {
                        coordinates: p,
                        title: properties
                            .get("title")
                            .expect("Missing marker title")
                            .to_string(),
                        description: properties
                            .get("description")
                            .unwrap_or(&json!(""))
                            .to_string(),
                        ty,
                    })
                }

                Geometry::MultiPolygon(mp) => territories.push(Territory::MultiPolygon(mp)),

                Geometry::Polygon(p) => territories.push(Territory::Polygon(p)),

                _ => panic!("Unexpected geometry type"),
            }
        });

        (markers, territories)
    }
}

impl UnsplitGeo for (Vec<Marker>, Feature) {
    fn unsplit_geo(self) -> FeatureCollection {
        let (markers, territories) = self;

        let mut features: Vec<geojson::Feature> = markers.to_features();
        features.push(territories);

        features.to_collection()
    }
}

impl ToMultiPolygon for Polygon {
    fn to_mp(&self) -> MultiPolygon {
        MultiPolygon::new(vec![self.clone()])
    }
}

pub trait ToFeature {
    fn to_feature(&self) -> geojson::Feature;
}

pub trait ToFeatures {
    fn to_features(&self) -> Vec<geojson::Feature>;
}

pub trait ToCountryFeature {
    fn to_country_feature(&self, id: &String, config: &CountryConfig) -> geojson::Feature;
}

pub trait ToCollection {
    fn to_collection(self) -> geojson::FeatureCollection;
}

pub trait ToSplitGeo {
    fn split_geo(&self) -> (Vec<Marker>, Vec<Territory>);
}

pub trait ToTerritory {
    fn to_territory(&self) -> Territory;
}

pub trait UnsplitGeo {
    fn unsplit_geo(self) -> FeatureCollection;
}

pub trait ToMultiPolygon {
    fn to_mp(&self) -> MultiPolygon;
}
