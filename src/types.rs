use clap::{Parser, Subcommand};
use geo::{BoundingRect, Geometry, MultiPolygon, Point, Polygon};
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

    pub tags: Option<ProcessingTagsConfig>,
    pub countries_rewrite: Option<CountryRewriteConfig>,
    pub public: Option<PublicConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingTagsConfig {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
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

impl Marker {
    pub fn to_feature(&self) -> geojson::Feature {
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

impl Territory {
    pub fn to_feature(&self) -> geojson::Feature {
        match self {
            Territory::Polygon(p) => geojson::Feature {
                geometry: Some(geojson::Geometry::new(p.into())),
                properties: None,

                bbox: None,
                id: None,
                foreign_members: None,
            },
            Territory::MultiPolygon(mp) => geojson::Feature {
                geometry: Some(geojson::Geometry::new(mp.into())),
                properties: None,

                bbox: None,
                id: None,
                foreign_members: None,
            },
        }
    }
}
