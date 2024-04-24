use std::{fs, path::Path};
use toml_edit::{value, DocumentMut, Value};

use crate::types::{Config, Country, NewCommands};

pub fn new(cmd: NewCommands) {
    match cmd {
        NewCommands::Country {
            name,
            id,
            description,
            foundation_date,
            flag,
            about,
        } => {
            let name = name.unwrap_or_default();
            let description = description.unwrap_or_default();
            let foundation_date = foundation_date.unwrap_or_default();
            let flag = flag.unwrap_or_default();
            let about = about;

            let country = Country {
                name: name.clone(),
                description,
                foundation_date,
                flag,
                about,
                tags: Some(vec![]),
            };

            let config = fs::read_to_string("config.toml").unwrap();

            // Validate config
            let c = toml::from_str::<Config>(&config);

            match c {
                Ok(c) => c,
                Err(err) => {
                    panic!("Invalid config: {}", err);
                }
            };

            // Get actual config
            let mut config = config.parse::<DocumentMut>().unwrap();

            // Add country to layers
            let layers = config["country"]["layers"].clone().into_value().unwrap();

            let layers = if let Value::Array(mut layers) = layers {
                layers.push(&id);
                layers
            } else {
                panic!("layers is not an array");
            };

            config["country"]["layers"] = value(layers);

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
