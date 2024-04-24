use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

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
        #[clap(short, long)]
        name: Option<String>,
        #[clap(short, long)]
        description: Option<String>,
        #[clap(short, long)]
        foundation_date: Option<String>,
        #[clap(long)]
        flag: Option<String>,
        #[clap(short, long)]
        about: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    country: CountryConfig,
    processing: Vec<ProcessingConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountryConfig {
    layers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingConfig {
    generate_colors: Option<bool>,
    show_markers: Option<bool>,
    output_folder: String,
    countries_file: String,
    geo_file: String,

    tags: Option<ProcessingTagsConfig>,
    countries_rewrite: Option<CountryRewriteConfig>,
    public: Option<PublicConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessingTagsConfig {
    include: Option<Vec<String>>,
    exclude: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CountryRewriteConfig {
    name: Option<String>,
    description: Option<String>,
    foundation_date: Option<String>,
    flag: Option<String>,
    about: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicConfig {
    name: String,
    geo: String,
    countries: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Country {
    pub name: String,
    pub description: String,
    pub foundation_date: String,
    pub flag: String,
    pub about: Option<String>,
    pub tags: Option<Vec<String>>,
}
