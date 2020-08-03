
# Requirements: gdal, topojson, imagemagick.

.PHONY: all clean

# Eventually, you will want to disable this so intermediate files are removed.
# Many of them are larger than 1 GB.
.SECONDARY:

# Download .zip files.
zip/tl_2015_us_county.zip:
	@mkdir -p $(dir $@)
	@curl -sS -o $@.download 'ftp://ftp2.census.gov/geo/tiger/TIGER2015/COUNTY/$(notdir $@)'
	@mv $@.download $@
zip/ne_10m_populated_places.zip:
	@mkdir -p $(dir $@)
	@curl -sS -o $@.download 'http://naciscdn.org/naturalearth/10m/cultural/ne_10m_populated_places.zip'
	@mv $@.download $@
zip/srtm_%.zip:
	@# 90-meter SRTM tiles
	@mkdir -p $(dir $@)
	@curl -sS -o $@.download 'http://srtm.csi.cgiar.org/SRT-ZIP/SRTM_V41/SRTM_Data_GeoTiff/$(notdir $@)'
	@mv $@.download $@

# Unzip
shp/tl_2015_us_county.shp: zip/tl_2015_us_county.zip
	@mkdir -p $(dir $@)
	@rm -rf tmp && mkdir tmp
	@unzip -q -o -d tmp $<
	@cp tmp/* $(dir $@)
	@rm -rf tmp
shp/ne_10m_populated_places.shp: zip/ne_10m_populated_places.zip
	@mkdir -p $(dir $@)
	@rm -rf tmp && mkdir tmp
	@unzip -q -o -d tmp $<
	@cp tmp/* $(dir $@)
	@rm -rf tmp
tif/srtm_%.tif: zip/srtm_%.zip
	@mkdir -p $(dir $@)
	@rm -rf tmp && mkdir tmp
	@unzip -q -o -d tmp $<
	@cp tmp/* $(dir $@)
	@rm -rf tmp

# Extract Texas from U.S.
shp/texas-counties.shp: shp/tl_2015_us_county.shp
	@mkdir -p $(dir $@)
	@ogr2ogr \
		-f 'ESRI Shapefile' \
		-t_srs "EPSG:4326" \
		$@ $< \
		-dialect sqlite \
		-sql "SELECT Geometry, STATEFP, COUNTYFP, NAME \
			FROM tl_2015_us_county \
			WHERE STATEFP = '48'"
# Reduce international cities to only include Texas.
shp/texas-cities.shp: shp/ne_10m_populated_places.shp
	@mkdir -p $(dir $@)
	@ogr2ogr \
		-f 'ESRI Shapefile' \
		$@ $< \
		-dialect sqlite \
		-sql "SELECT Geometry, ADM0NAME, ADM1NAME, NAME, SCALERANK, LABELRANK, NATSCALE \
			FROM 'ne_10m_populated_places' \
			WHERE ADM1NAME = 'Texas' AND \
				  SCALERANK <= 7"

# Convert SHP to GeoJSON.
geojson/texas-counties.json: shp/texas-counties.shp
	@mkdir -p $(dir $@)
	@ogr2ogr \
		-f 'GeoJSON' \
		$@ $<
texas-cities.json: shp/texas-cities.shp
	@mkdir -p $(dir $@)
	@ogr2ogr \
		-f 'GeoJSON' \
		$@ $<

# Convert GeoJSON to TopoJSON.
topojson/texas-counties.json: geojson/texas-counties.json
	@mkdir -p $(dir $@)
	@topojson \
		--no-quantization \
		--properties \
		-o $@ \
		-- $<

# Simplify TopoJSON.
texas-counties.json: topojson/texas-counties.json
	@mkdir -p $(dir $@)
	@topojson \
		--properties \
		--spherical \
		-q 1e8 \
		-s 1e-10 \
		-o $@ \
		-- $<

# Merge topographic tiles.
# Use http://dwtkns.com/srtm/ to find which tiles are needed.
tif/texas-merged-90m.tif: \
	tif/srtm_15_05.tif \
	tif/srtm_16_05.tif \
	tif/srtm_17_05.tif \
	tif/srtm_18_05.tif \
	tif/srtm_15_06.tif \
	tif/srtm_16_06.tif \
	tif/srtm_17_06.tif \
	tif/srtm_18_06.tif \
	tif/srtm_15_07.tif \
	tif/srtm_16_07.tif \
	tif/srtm_17_07.tif \
	tif/srtm_18_07.tif

	@mkdir -p $(dir $@)
	@gdal_merge.py \
		-o $@ \
		-init "255" \
		tif/srtm_*.tif

# Convert to Mercator.
tif/texas-reprojected.tif: tif/texas-merged-90m.tif
	@# Comes as WGS 84 (EPSG:4326). Want Mercator (EPSG:3857) for D3 projection.
	@mkdir -p $(dir $@)
	@gdalwarp \
		-co "TFW=YES" \
		-s_srs "EPSG:4326" \
		-t_srs "EPSG:3857" \
		$< \
		$@

# Crop raster to shape of Texas.
tif/texas-cropped.tif: tif/texas-reprojected.tif shp/texas-counties.shp
	@mkdir -p $(dir $@)
	@gdalwarp \
		-cutline shp/texas-counties.shp \
		-crop_to_cutline \
		-dstalpha \
		tif/texas-reprojected.tif \
		$@

# Shade and color.
tif/texas-color-crop.tif: tif/texas-cropped.tif
	@rm -rf tmp && mkdir -p tmp
	@gdaldem \
		hillshade \
		$< tmp/hillshade.tmp.tif \
		-z 5 \
		-az 315 \
		-alt 60 \
		-compute_edges
	@gdal_calc.py \
		-A tmp/hillshade.tmp.tif \
		--outfile=$@ \
		--calc="255*(A>220) + A*(A<=220)"
	@gdal_calc.py \
		-A tmp/hillshade.tmp.tif \
		--outfile=tmp/opacity_crop.tmp.tif \
		--calc="1*(A>220) + (256-A)*(A<=220)"
	@rm -rf tmp

# Convert to .png.
texas-raster.png: tif/texas-color-crop.tif
	@convert \
		-resize x670 \
		$< $@

all: texas-counties.json \
	texas-cities.json \
	texas-raster.png

clean:
	@rm -rf geojson
	@rm -rf hgt
	@rm -rf shp
	@rm -rf tif
	@rm -rf topojson
