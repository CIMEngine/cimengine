use std::{fs, path::Path};

pub fn init(name: String) {
    let config = include_str!("./templates/config.toml");
    let country_config = include_str!("./templates/country.toml");
    let geojson = include_str!("./templates/sample.geojson");

    let root_folder = Path::new(&name);
    let country_folder = Path::new(&name).join("countries").join("sample_country_id");
    let nature_folder = Path::new(&name).join("nature");

    fs::create_dir_all(&country_folder).unwrap();
    fs::create_dir_all(&nature_folder).unwrap();

    fs::write(root_folder.join("config.toml"), config).unwrap();

    fs::write(country_folder.join("country.toml"), country_config).unwrap();
    fs::write(country_folder.join("country.geojson"), geojson).unwrap();

    fs::write(nature_folder.join("water.geojson"), geojson).unwrap();
    fs::write(nature_folder.join("sand.geojson"), geojson).unwrap();
    fs::write(nature_folder.join("grass.geojson"), geojson).unwrap();
}
