use std::{fs, path::Path};

use geojson::{Feature, FeatureCollection};
use serde_json::json;
use wax::{Glob, Pattern};

use crate::{
    types::CountryData,
    utils::{get_country, read_config},
};

pub fn build() {
    let config = read_config();

    for processing_item in config.processing {
        let mut features: Vec<Feature> = vec![];

        let out_folder = Path::new(&processing_item.output_folder);

        let mut countries: Vec<CountryData> = vec![];

        let tags = processing_item.tags.unwrap_or(vec!["*".to_string()]);
        let globs: Vec<Glob> = tags.iter().map(|tag| Glob::new(tag).unwrap()).collect();

        for country_id in &config.main.layers {
            let mut country = get_country(country_id.to_owned());

            let mut matches = false;
            for glob in &globs {
                for tag in &country.config.tags.clone().unwrap_or(vec![]) {
                    if glob.is_match(tag.as_str()) {
                        matches = true;
                    }
                }
            }

            if !matches {
                continue;
            }

            features.append(&mut country.geo.features);
            countries.push(country);
        }

        // TODO: Country diff

        // TODO: Add country_rewrite support
        // TODO: let countries = vec![rewrite_info];

        // TODO: Add nature support

        let feature_collection = FeatureCollection {
            bbox: None,
            features,
            foreign_members: None,
        }
        .to_string();

        let countries = serde_json::to_string_pretty(&serde_json::Map::from_iter(
            countries
                .iter()
                .map(|country| (country.id.clone(), json!(country.config))),
        ))
        .unwrap();

        fs::create_dir_all(out_folder).unwrap();

        fs::write(out_folder.join("geo.geojson"), feature_collection).unwrap();
        fs::write(out_folder.join("countries.json"), countries).unwrap();

        if let Some(public) = processing_item.public {
            let public = serde_json::to_string(&public).unwrap();
            fs::write(out_folder.join("public.json"), public).unwrap();
        }
    }
}
