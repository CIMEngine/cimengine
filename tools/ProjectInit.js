#!/usr/bin/env node

const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const args = yargs(hideBin(process.argv)).argv;

const fs = require("fs");
const path = require("path");

let geojsonTemplate = fs.readFileSync(
  path.join(__dirname, "file_templates", "geojson.geojson")
);

let configTemplate = fs.readFileSync(
  path.join(__dirname, "file_templates", "config.yaml")
);

let propertiesTemplate = fs.readFileSync(
  path.join(__dirname, "file_templates", "properties.yaml")
);

let layersTemplate = fs.readFileSync(
  path.join(__dirname, "file_templates", "layers.yaml")
);

let countriesTemplate = fs.readFileSync(
  path.join(__dirname, "file_templates", "countries.json")
);

let roadSizesTemplate = fs.readFileSync(
  path.join(__dirname, "file_templates", "roadSizes.json")
);

let name = args.name || "cime-project";

let currpath = path.join(process.cwd(), name, "src");

fs.mkdirSync(path.join(currpath, "countries"), { recursive: true });
fs.mkdirSync(path.join(currpath, "nature"), { recursive: true });
fs.mkdirSync(path.join(currpath, "roads"), { recursive: true });

fs.writeFileSync(path.join(currpath, "config.yaml"), configTemplate);
fs.writeFileSync(path.join(currpath, "properties.yaml"), propertiesTemplate);
fs.writeFileSync(path.join(currpath, "layers.yaml"), layersTemplate);
fs.writeFileSync(path.join(currpath, "countries.json"), countriesTemplate);

fs.writeFileSync(path.join(currpath, "roads", "sizes.json"), roadSizesTemplate);

let geoTemplateFiles = [
  path.join("roads", "orange.geojson"),
  path.join("roads", "white.geojson"),
  path.join("roads", "yellow.geojson"),
  path.join("nature", "grass.geojson"),
  path.join("nature", "sand.geojson"),
  path.join("nature", "water.geojson"),
];

for (let geoFile of geoTemplateFiles) {
  fs.writeFileSync(path.join(currpath, geoFile), geojsonTemplate);
}

console.log("Done!");
