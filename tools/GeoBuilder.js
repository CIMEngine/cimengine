#!/usr/bin/env node

const turf = require("@turf/turf");
const fs = require("fs");
const YAML = require("yaml");
const _ = require("lodash");
const path = require("node:path");

const yargs = require("yargs/yargs");
const { hideBin } = require("yargs/helpers");
const args = yargs(hideBin(process.argv)).argv;

const geofixConf = {
  projectFolder: args.projectFolder,
  layers: args.layers || path.join(args.projectFolder, "src", "layers.yaml"),
  properties:
    args.properties || path.join(args.projectFolder, "src", "properties.yaml"),
  config: args.config || path.join(args.projectFolder, "src", "config.yaml"),
  countries:
    args.countries || path.join(args.projectFolder, "src", "countries"),
  naturesObjects:
    args.naturesObjects || path.join(args.projectFolder, "src", "nature"),
  roads: args.roads || path.join(args.projectFolder, "src", "roads"),
  output: args.output || path.join(args.projectFolder, "geo.geojson"),
};

let layers = YAML.parse(fs.readFileSync(geofixConf.layers, "utf-8"));
let countries_properties = YAML.parse(
  fs.readFileSync(geofixConf.properties, "utf-8")
);
let config = YAML.parse(fs.readFileSync(geofixConf.config, "utf-8"));

let features = [];

for (country of layers) {
  let properties = countries_properties[country];

  let co_features = JSON.parse(
    fs.readFileSync(
      path.join(geofixConf.countries, `${country}.geojson`),
      "utf-8"
    )
  );

  co_features.features = co_features.features.map((val) => {
    if (val?.geometry?.type === "Polygon") {
      val.properties = {};
    } else if (val?.geometry?.type === "MultiPolygon") {
      console.error("Error: MultiPolygons are not allowed!");
      process.exit(1);
    }

    return val;
  });

  fs.writeFileSync(
    path.join(geofixConf.countries, `${country}.geojson`),
    JSON.stringify(co_features, null, "  ")
  );

  co_features = co_features.features;

  features = [
    ...features,
    ...co_features.map((val) => {
      if (val.geometry.type == "Polygon") {
        val.properties = properties;
        val.properties.name = country;
      }

      return val;
    }),
  ];
}

let geo = {
  type: "FeatureCollection",
  features: features.reverse(),
};

geo.features = geo.features.filter((v) => v.properties.name);

console.time("Total");

console.log("Dissolve");
console.time("Dissolve");
let nonPoly = geo.features.filter((v) => !v.geometry.type.endsWith("Polygon"));

nonPoly = nonPoly.map((v) => {
  if (v.properties.type === "landmark") {
    v.properties.type = "landmark-0";
  }

  if (!v.properties.type) {
    v.properties.type = "city";
  }

  return v;
});

let polygons = geo.features.filter((v) => v.geometry.type.endsWith("Polygon"));
let props = {};

for (let feature of polygons) {
  if (props[feature.properties.name]) continue;
  props[feature.properties.name] = {
    stroke: feature.properties.stroke,
    fill: feature.properties.fill,
    type: feature.properties.type,
    tags: feature.properties.tags,
  };
}

let dissolved = turf.dissolve(turf.featureCollection(polygons), {
  propertyName: "name",
});

dissolved.features = dissolved.features.map((v) => {
  v.properties = {
    name: v.properties.name,
    fill: props[v.properties.name].fill,
    stroke: props[v.properties.name].stroke,
    type: props[v.properties.name].type,
    tags: props[v.properties.name].tags,
  };
  return v;
});
geo.features = dissolved.features.concat(nonPoly);
console.timeEnd("Dissolve");
console.log();

console.log("Polygons to MultiPolygons");
console.time("Polygons to MultiPolygons");

polygons = geo.features.filter((v) => v.geometry.type.endsWith("Polygon"));
nonPoly = geo.features.filter((v) => !v.geometry.type.endsWith("Polygon"));

let countries = {};

for (let somePolygon of polygons) {
  if (!countries[somePolygon.properties.name]) {
    countries[somePolygon.properties.name] = {
      type: "Feature",
      properties: somePolygon.properties,
      geometry: {
        type: "MultiPolygon",
        coordinates: [],
      },
    };
  }

  if (somePolygon.geometry.type === "MultiPolygon") {
    countries[somePolygon.properties.name].geometry.coordinates = countries[
      somePolygon.properties.name
    ].concat(somePolygon.geometry.coordinates);
  } else if (somePolygon.geometry.type === "Polygon") {
    countries[somePolygon.properties.name].geometry.coordinates.push(
      somePolygon.geometry.coordinates
    );
  }
}

let multiPolygons = Object.values(countries);

geo.features = multiPolygons.concat(nonPoly);

console.timeEnd("Polygons to MultiPolygons");
console.log();

console.log("Difference");
console.time("Difference");
for (let g = 0; g < geo.features.length; g++) {
  for (let i = 0; i < geo.features.length; i++) {
    try {
      if (
        geo.features[g] === geo.features[i] ||
        geo.features[i]?.properties.type === "sand" ||
        geo.features[i]?.properties.type === "water" ||
        geo.features[i]?.properties.type === "grass" ||
        geo.features[g]?.properties.type === "sand" ||
        geo.features[g]?.properties.type === "water" ||
        geo.features[g]?.properties.type === "grass"
      ) {
        continue;
      }

      if (
        (geo.features[g]?.geometry.type === "Polygon" ||
          geo.features[g]?.geometry.type === "MultiPolygon") &&
        (geo.features[i]?.geometry.type === "Polygon" ||
          geo.features[i]?.geometry.type === "MultiPolygon")
      ) {
        let p1 =
          geo.features[g]?.geometry.type === "MultiPolygon"
            ? turf.multiPolygon(
                geo.features[g].geometry.coordinates,
                geo.features[g].properties
              )
            : turf.polygon(
                geo.features[g].geometry.coordinates,
                geo.features[g].properties
              );
        let p2 =
          geo.features[i]?.geometry.type === "MultiPolygon"
            ? turf.multiPolygon(
                geo.features[i].geometry.coordinates,
                geo.features[i].properties
              )
            : turf.polygon(
                geo.features[i].geometry.coordinates,
                geo.features[i].properties
              );

        let diff = turf.difference(p1, p2);
        geo.features[g] = diff ? diff : geo.features[g];
      } else continue;
    } catch (e) {
      console.log("Error, skip \n", e, "\n");
    }
  }
}
console.timeEnd("Difference");
console.log();

console.log("Add Map Components");
console.time("Add Map Components");

let sand = JSON.parse(
  fs.readFileSync(path.join(geofixConf.naturesObjects, "sand.geojson"), "utf-8")
).features;

let water = JSON.parse(
  fs.readFileSync(
    path.join(geofixConf.naturesObjects, "water.geojson"),
    "utf-8"
  )
).features;

let grass = JSON.parse(
  fs.readFileSync(
    path.join(geofixConf.naturesObjects, "grass.geojson"),
    "utf-8"
  )
).features;

let white_road = JSON.parse(
  fs.readFileSync(path.join(geofixConf.roads, "white.geojson"), "utf-8")
).features;

let orange_road = JSON.parse(
  fs.readFileSync(path.join(geofixConf.roads, "orange.geojson"), "utf-8")
).features;

let yellow_road = JSON.parse(
  fs.readFileSync(path.join(geofixConf.roads, "yellow.geojson"), "utf-8")
).features;

let road_sizes = JSON.parse(
  fs.readFileSync(path.join(geofixConf.roads, "sizes.json"), "utf-8")
);

let map_comps = [
  ...water.map((val) => {
    val.properties.type = "water";
    val.properties.fill = "#75cff0";
    val.properties.stroke = "#75cff0";
    val.properties["fill-opacity"] = 1;
    return val;
  }),
  ...sand.map((val) => {
    val.properties.type = "sand";
    val.properties.fill = "#efe9e1";
    val.properties.stroke = "#efe9e1";
    val.properties["fill-opacity"] = 1;
    return val;
  }),
  ...grass.map((val) => {
    val.properties.type = "grass";
    val.properties.fill = "#d1e6be";
    val.properties.stroke = "#d1e6be";
    val.properties["fill-opacity"] = 1;
    return val;
  }),
  ...white_road.map((val) => {
    let total = turf.buffer(
      val,
      road_sizes[val.properties.type] || road_sizes["middle"]
    );
    total.properties.type = "white_road";

    val.properties.fill = "#fff";
    val.properties.stroke = "#fff";
    val.properties["fill-opacity"] = 1;

    return total;
  }),
  ...yellow_road.map((val) => {
    let total = turf.buffer(
      val,
      road_sizes[val.properties.type] || road_sizes["middle"]
    );
    total.properties.type = "yellow_road";
    val.properties.fill = "#ffc107";
    val.properties.stroke = "#ffc107";
    val.properties["fill-opacity"] = 1;
    return total;
  }),
  ...orange_road.map((val) => {
    let total = turf.buffer(
      val,
      road_sizes[val.properties.type] || road_sizes["big"]
    );
    total.properties.type = "orange_road";
    val.properties.fill = "#fd7e14";
    val.properties.stroke = "#fd7e14";
    val.properties["fill-opacity"] = 1;
    return total;
  }),
];

props = {};

for (let feature of map_comps) {
  if (props[feature.properties.type]) continue;
  props[feature.properties.type] = {
    stroke: feature.properties.stroke,
    fill: feature.properties.fill,
    type: feature.properties.type,
    tags: feature.properties.tags,
    "fill-opacity": feature.properties["fill-opacity"],
  };
}

map_comps = turf.dissolve(turf.featureCollection(map_comps), {
  propertyName: "type",
});

map_comps.features = map_comps.features.map((v) => {
  v.properties = {
    fill: props[v.properties.type].fill,
    stroke: props[v.properties.type].stroke,
    type: props[v.properties.type].type,
    tags: props[v.properties.type].tags,
    "fill-opacity": props[v.properties.type]["fill-opacity"],
  };
  return v;
});

geo.features = [...map_comps.features, ...geo.features];
console.timeEnd("Add Map Components");
console.log();

if (config?.tags) {
  console.log("Filter countries by tags");
  console.time("Filter countries by tags");

  geo.features = geo.features.filter((val) => {
    if (_.intersection(config.tags, val.properties.tags).length === 0)
      return false;
    // else if (config?.cities == false && val.geometry.type === "Point")
    //   return false;
    else return true;
  });

  console.timeEnd("Filter countries by tag");
  console.log();
}

if (config?.reProperty) {
  console.log("replace Properties");
  console.time("replace Properties");

  geo.features = geo.features.map((val) => {
    val.properties = config.reProperty;
    return val;
  });

  console.timeEnd("replace Properties");
  console.log();
}

console.log("Set new ids");
console.time("Set new ids");

let id = 0;
geo.features = geo.features.map((val) => {
  val.id = id++;
  return val;
});

console.timeEnd("Set new ids");

fs.writeFileSync(geofixConf.output, JSON.stringify(geo, null, "  "));
console.timeEnd("Total");
