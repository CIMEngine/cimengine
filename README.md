# CIMEngine build tools

## Installation

```bash
npm i cimengine-build-tools
```

or

```bash
npm i cimengine-build-tools -g
```

## Usage

### Tools

1. cimengine-init - Initializes the cimengine project
2. cimengine-build - Assembles project files into a single geojson
3. cimengine-country-fix - Transforms the country geojson file (merges polygons and simplifies them)

### cimengine-init

params: name - name of project  
usage: `cimengine-init --name map`

### cimengine-build

params:

1. projectFolder - folder with project generated by cimengine-init (If this parameter is specified, no other parameters are needed)
2. layers - path to layers.yaml
3. properties - path to properties.yaml
4. config - path to config.yaml
5. countries - path to countries folder
6. natureObjects - path to nature folder
7. roads - path to roads folder
8. output - path to output file

usage: `cimengine-build --projectFolder ./map`

### cimengine-country-fix

params:

1. countryPath - path to country file
2. simplify - Simplification accuracy of polygons (optional)

usage: `cimengine-country-fix --countryPath ./map/src/countries/usa.geojson`
