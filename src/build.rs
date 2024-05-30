use std::{fs, path::Path, time};

use serde_json::json;
use wax::{Glob, Pattern};

use crate::{
    types::{CountryData, ToCollection},
    utils::{diff_countries, get_country, read_config},
};

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

            if tags.len() == 0 {
                for country_id in &config.main.layers {
                    countries.push(get_country(country_id.to_owned()));
                }
            } else {
                for country_id in &config.main.layers {
                    let country = get_country(country_id.to_owned());

                    match &country.config.tags {
                        Some(tags) => {
                            let mut matches = false;
                            for glob in &globs {
                                for tag in tags {
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
                        None => {
                            continue;
                        }
                    }
                }
            }

            println!("Dissolved in {:?}", dissolved_time.elapsed());
        }

        // TODO: Add country_rewrite support

        let countries = {
            let diff_time = time::Instant::now();

            let countries = diff_countries(countries);

            println!("Diffed in {:?}", diff_time.elapsed());

            countries
        };

        // TODO: Add nature support

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
