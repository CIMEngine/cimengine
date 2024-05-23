use std::{fs, path::Path};

use geo::{BooleanOps, MultiPolygon};
use geojson::{FeatureCollection, GeoJson};

use crate::types::{
    Config, CountryConfig, CountryData, Territory, ToCountryFeature, ToMultiPolygon, ToSplitGeo,
    UnsplitGeo,
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

    CountryData {
        id: id.clone(),
        config: config.clone(),
        geo: dissolve_territory(geo, id, config),
    }
}

pub fn dissolve_territory(
    geo: FeatureCollection,
    id: String,
    config: CountryConfig,
) -> FeatureCollection {
    let (markers, territories) = geo.split_geo();

    let dissolved = territories
        .iter()
        .fold(MultiPolygon::new(vec![]), |a, b| match b {
            Territory::Polygon(p) => a.union(&p.to_mp()),
            Territory::MultiPolygon(mp) => a.union(mp),
        })
        .to_country_feature(&id, &config);

    (markers, dissolved).unsplit_geo()
}

pub fn hash_hex_color(s: String) -> String {
    let hex_str = format!("{:x}", xxhash_rust::xxh3::xxh3_64(s.as_bytes()));

    format!("#{}", hex_str.chars().take(6).collect::<String>())
}
