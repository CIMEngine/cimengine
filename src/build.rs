use std::{fs, path::Path};

use geojson::{Feature, FeatureCollection};
use serde_json::json;
use wax::{Glob, Pattern};

use crate::{
    types::{CountryData, ToCollection, ToFeature, ToFeatures},
    utils::{diff_countries, get_country, read_config},
};

pub fn build() {
    let config = read_config();

    for processing_item in config.processing {
        let features: Vec<Feature> = vec![];

        let out_folder = Path::new(&processing_item.output_folder);

        let tags = processing_item.tags.unwrap_or(vec!["*".to_string()]);
        let globs: Vec<Glob> = tags.iter().map(|tag| Glob::new(tag).unwrap()).collect();

        let mut countries: Vec<CountryData> = vec![];

        for country_id in &config.main.layers {
            let country = get_country(country_id.to_owned());

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

            countries.push(country);
        }

        // TODO: Add country_rewrite support

        let countries = diff_countries(countries);

        // TODO: Add nature support

        let countries_json = serde_json::to_string_pretty(&serde_json::Map::from_iter(
            countries
                .iter()
                .map(|country| (country.id.clone(), json!(country.config))),
        ))
        .unwrap();

        let feature_collection = countries.to_collection().to_string();

        fs::create_dir_all(out_folder).unwrap();

        fs::write(out_folder.join("geo.geojson"), feature_collection).unwrap();
        fs::write(out_folder.join("countries.json"), countries_json).unwrap();

        if let Some(public) = processing_item.public {
            let public = serde_json::to_string(&public).unwrap();
            fs::write(out_folder.join("public.json"), public).unwrap();
        }
    }
}
