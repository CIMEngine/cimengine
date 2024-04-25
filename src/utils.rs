use std::{collections::HashMap, fs, path::Path};

use geo::{BooleanOps, Geometry, MultiPolygon};
use geojson::{Feature, FeatureCollection, GeoJson};
use serde_json::json;

use crate::types::{Config, CountryConfig, CountryData, Marker, MarkerType, Territory};

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
    let mut markers: Vec<Marker> = vec![];
    let mut territories: Vec<Territory> = vec![];

    geo.features.iter().for_each(|f| {
        let properties = f.properties.clone().unwrap();

        let geometry: Geometry = f.geometry.clone().unwrap().try_into().unwrap();

        match geometry {
            Geometry::Point(p) => {
                let ty = match properties
                    .get("type")
                    .expect("Missing marker type")
                    .to_string()
                    .trim_matches('"')
                {
                    "capital" | "capital-city" => MarkerType::Capital,
                    "city" => MarkerType::City,
                    "landmark" => MarkerType::Landmark,

                    t => panic!("Invalid marker type: {}", t),
                };

                markers.push(Marker {
                    coordinates: p,
                    title: properties
                        .get("title")
                        .expect("Missing marker title")
                        .to_string(),
                    description: properties
                        .get("description")
                        .unwrap_or(&json!(""))
                        .to_string(),
                    ty,
                })
            }

            Geometry::MultiPolygon(mp) => territories.push(Territory::MultiPolygon(mp)),

            Geometry::Polygon(p) => territories.push(Territory::Polygon(p)),

            _ => panic!("Unexpected geometry type"),
        }
    });

    let territories = territories.iter();

    let dissolved = territories.fold(MultiPolygon::new(vec![]), |a, b| match b {
        Territory::Polygon(p) => a.union(&MultiPolygon::new(vec![p.clone()])),
        Territory::MultiPolygon(mp) => a.union(mp),
    });

    // combine markers and dissolved
    let mut features: Vec<Feature> = markers.iter().map(|m| m.to_feature()).collect();

    features.push(geojson::Feature {
        geometry: Some(geojson::Geometry::new((&dissolved).into())),
        properties: Some(
            serde_json::Map::from_iter([
                ("id".to_owned(), json!(id)),
                ("type".to_owned(), json!("country")),
                ("fill".to_owned(), json!(config.fill)),
                ("stroke".to_owned(), json!(config.stroke)),
                ("tags".to_owned(), json!(config.tags)),
            ])
            .into(),
        ),

        bbox: None,
        id: None,
        foreign_members: None,
    });

    FeatureCollection {
        features,
        bbox: None,
        foreign_members: None,
    }
}

pub fn hash_hex_color(s: String) -> String {
    let hex_str = format!("{:x}", xxhash_rust::xxh3::xxh3_64(s.as_bytes()));

    format!("#{:6}", hex_str.chars().take(6).collect::<String>())
}
