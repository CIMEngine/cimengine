use std::{fs, path::Path, time};

use serde_json::json;

use crate::{
    types::{CountryData, ToCollection, ToFeatures},
    utils::{
        diff_countries, get_country, is_match, read_config, rewrite_if_some, rewrite_if_some_option,
    },
};
use wax::Glob;

pub fn build() {
    let config = read_config();

    let total_time = time::Instant::now();

    for processing_item in config.processing {
        println!("--- {} ---", processing_item.output_folder);

        let processed_time = time::Instant::now();

        let out_folder = Path::new(&processing_item.output_folder);

        let tags = processing_item.tags.unwrap_or(vec![]);
        let globs: Vec<Glob> = tags.iter().map(|tag| Glob::new(tag).unwrap()).collect();

        let mut countries: Vec<CountryData> = vec![];

        {
            let dissolved_time = time::Instant::now();

            for country_id in &config.main.layers {
                let country = get_country(country_id.to_owned());

                if is_match(&country.config.tags, &globs) {
                    countries.push(country);
                }
            }

            println!("Dissolved in {:?}", dissolved_time.elapsed());
        }

        let countries: Vec<CountryData> = {
            let diff_time = time::Instant::now();

            let mut countries = diff_countries(countries);

            println!("Diffed in {:?}", diff_time.elapsed());

            countries.iter_mut().for_each(|c| {
                if !processing_item.show_markers.unwrap_or(true) {
                    c.markers = vec![];
                }

                for country_rewrite in processing_item.countries_rewrite.clone().unwrap_or(vec![]) {
                    let tags = country_rewrite.tags.unwrap_or(vec![]);
                    let globs: Vec<Glob> = tags.iter().map(|tag| Glob::new(tag).unwrap()).collect();

                    if is_match(&c.config.tags, &globs) {
                        rewrite_if_some(country_rewrite.properties.name, &mut c.config.name);
                        rewrite_if_some(
                            country_rewrite.properties.description,
                            &mut c.config.description,
                        );
                        rewrite_if_some(
                            country_rewrite.properties.foundation_date,
                            &mut c.config.foundation_date,
                        );
                        rewrite_if_some(country_rewrite.properties.flag, &mut c.config.flag);
                        rewrite_if_some_option(
                            country_rewrite.properties.about,
                            &mut c.config.about,
                        );
                        rewrite_if_some(country_rewrite.properties.fill, &mut c.config.fill);
                        rewrite_if_some(country_rewrite.properties.stroke, &mut c.config.stroke);
                        rewrite_if_some_option(country_rewrite.properties.tags, &mut c.config.tags);
                    }
                }
            });

            countries
        };

        {
            let generated_time = time::Instant::now();
            let countries_json = serde_json::to_string_pretty(&serde_json::Map::from_iter(
                countries
                    .iter()
                    .map(|country| (country.id.clone(), json!(country.config))),
            ))
            .unwrap();

            fs::create_dir_all(out_folder).unwrap();

            fs::write(
                out_folder.join("geo.geojson"),
                countries.to_collection().to_string(),
            )
            .unwrap();
            fs::write(out_folder.join("countries.json"), countries_json).unwrap();

            if let Some(public) = processing_item.public {
                let public = serde_json::to_string(&public).unwrap();
                fs::write(out_folder.join("public.json"), public).unwrap();
            }

            println!("Generated files in {:?}", generated_time.elapsed());
        }

        let processed = format!("{:?}", processed_time.elapsed());

        println!(
            "--- {} {}---\n",
            processed,
            "-".repeat(processing_item.output_folder.len() - processed.len())
        );
    }

    println!("Total time: {:?}", total_time.elapsed());
}
