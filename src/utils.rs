use std::{fs, marker, path::Path};

use geo::{BooleanOps, MultiPolygon};
use geojson::{Feature, FeatureCollection, GeoJson};

use crate::types::{
    Config, CountryConfig, CountryData, Marker, Territory, ToCollection, ToCountryFeature,
    ToFeature, ToFeatures, ToMultiPolygon, ToSplitGeo, UnsplitGeo,
};

pub fn read_config() -> Config {
    let c = toml::from_str::<Config>(&fs::read_to_string("config.toml").unwrap());

    match c {
        Ok(c) => c,
        Err(err) => panic!("Invalid config: {}", err),
    }
}

pub fn get_country(id: String) -> CountryData {
    let country_folder = Path::new(".").join("countries").join(&id);

    let config = toml::from_str::<CountryConfig>(
        &fs::read_to_string(country_folder.join("country.toml")).unwrap(),
    );

    let config = match config {
        Ok(c) => c,
        Err(err) => panic!("Invalid config: {}", err),
    };

    let geo_str = fs::read_to_string(country_folder.join("country.geojson")).unwrap();
    let geo: GeoJson = geo_str.parse().unwrap();

    let geo = match geo {
        GeoJson::FeatureCollection(coll) => coll,
        _ => panic!("Invalid geojson, expected FeatureCollection"),
    };

    let (markers, territories) = geo.split_geo();
    let geo = dissolve_territories(territories);

    CountryData {
        id: id.clone(),
        config: config.clone(),
        land: geo,
        markers,
    }
}

pub fn dissolve_territories(territories: Vec<Territory>) -> MultiPolygon {
    let dissolved = territories
        .iter()
        .fold(MultiPolygon::new(vec![]), |a, b| match b {
            Territory::Polygon(p) => a.union(&p.to_mp()),
            Territory::MultiPolygon(mp) => a.union(mp),
        });

    dissolved
}

pub fn diff_countries(countries: Vec<CountryData>) -> Vec<CountryData> {
    let mut countries = countries;

    for i in 0..countries.len() {
        for j in 0..countries.len() {
            if i == j {
                continue;
            }

            countries[i].land = countries[i].land.difference(&countries[j].land);
        }
    }

    countries
}

pub fn hash_hex_color(s: String) -> String {
    let hex_str = format!("{:x}", xxhash_rust::xxh3::xxh3_64(s.as_bytes()));

    format!("#{}", hex_str.chars().take(6).collect::<String>())
}
