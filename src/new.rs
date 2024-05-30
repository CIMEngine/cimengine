use std::{fs, path::Path};
use toml_edit::{value, DocumentMut, Value};

use crate::{
    types::{CountryConfig, NewCommands},
    utils::{hash_hex_color, read_config},
};

pub fn new(cmd: NewCommands) {
    match cmd {
        NewCommands::Country {
            name,
            id,
            description,
            foundation_date,
            flag,
            about,
            fill,
            stroke,
        } => {
            let name = name.unwrap_or_default();
            let description = description.unwrap_or_default();
            let foundation_date = foundation_date.unwrap_or_default();
            let flag = flag.unwrap_or_default();
            let fill = fill.unwrap_or_else(|| hash_hex_color(id.clone() + "_fill"));
            let stroke = stroke.unwrap_or_else(|| hash_hex_color(id.clone() + "_stroke"));

            let country = CountryConfig {
                name: name.clone(),
                description,
                foundation_date,
                flag,
                about,
                fill,
                stroke,
                tags: None,
            };

            let config = fs::read_to_string("config.toml").unwrap();

            // Validate config
            read_config();

            // Get actual config
            let mut config = config.parse::<DocumentMut>().unwrap();

            // Add country to layers
            let layers = config["main"]["layers"].clone().into_value().unwrap();

            let layers = if let Value::Array(mut layers) = layers {
                layers.insert(0, &id);
                layers
            } else {
                panic!("layers is not an array");
            };

            config["main"]["layers"] = value(layers);

            fs::write("config.toml", config.to_string()).unwrap();

            // Add country to countries
            let country_folder = Path::new(".").join("countries").join(&id);
            fs::create_dir_all(&country_folder).unwrap();
            fs::write(
                country_folder.join("country.toml"),
                toml::to_string_pretty(&country).unwrap(),
            )
            .unwrap();
            fs::write(
                country_folder.join("country.geojson"),
                include_str!("./templates/sample.geojson"),
            )
            .unwrap();
        }
    }
}
