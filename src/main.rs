extern crate rustc_serialize;
extern crate chrono;
extern crate sxd_document;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use rustc_serialize::json::Json;
use chrono::*;
use sxd_document::Package;
use sxd_document::writer::format_document;

fn help() {
    println!("usage: rust-msband-to-gpx infile.json [outfile.gpx]");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        help();
        return;
    }

    // Process the arguments
    let input = args[1].to_owned();
    let output;
    if args.len() > 2 {
        let path = Path::new(&args[2]);
        output = path.file_stem().unwrap().to_str().unwrap();
    }
    else {
        let path = Path::new(&input);
        output = path.file_stem().unwrap().to_str().unwrap();
    }

    // Read the JSON file
    let mut file = File::open(&input).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let json = Json::from_str(&contents).unwrap();
    let object = json.as_object().unwrap();

    // Try to find the "Runs" array in the JSON
    if let Some(run_array) = object.get("Runs").and_then(|x| x.as_array()) {

        let mut run_number = 1;

        // Loop over each run
        for run in run_array.iter().map(|x| x.as_object().unwrap()) {

            println!("Run #{}", run_number);

            // Collect the distance and time array from the PaceData
            // This array will later help us calculate the time each WayPoint occurs
            let mut distance_and_time_array = Vec::<(f64, i64)>::new();
            if let Some(pace_data_array) = run.get("PaceData").and_then(|x| x.as_array()) {
                for pace_data in pace_data_array.iter().map(|x| x.as_object().unwrap()) {
                    match (pace_data.get("ScaledDistance").and_then(|x| x.as_f64()),
                        pace_data.get("ElapsedSeconds").and_then(|x| x.as_i64())) {

                        (Some(distance), Some(time)) => distance_and_time_array.push((distance, time)),
                        _ => {}
                    }
                }
            }

            if distance_and_time_array.len() == 0 {
                run_number += 1;
                continue;
            }
            let mut last_distance_and_time = distance_and_time_array.remove(0);

            // Process the start date from the MS timestamp
            let start_date_seconds = run.get("StartDate")
                .and_then(|x| x.as_string().unwrap_or("").split(&['(', ')'][..]).nth(1))
                .unwrap()
                .parse::<i64>()
                .ok()
                .unwrap() / 1_000;
            let naive_start_date = NaiveDateTime::from_timestamp(start_date_seconds, 0);
            let start_date = DateTime::<UTC>::from_utc(naive_start_date, UTC);

            // Construct GPX document
            let gpx_package = Package::new();
            let gpx_document = gpx_package.as_document();
            let gpx_element = gpx_document.create_element("gpx");
            gpx_element.set_attribute_value("creator", "convert-to-gpx");
            gpx_element.set_attribute_value("version", "1.1");
            gpx_element.set_attribute_value("xmlns", "http://www.topografix.com/GPX/1/1");
            gpx_element.set_attribute_value("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance");
            gpx_element.set_attribute_value("xsi:schemaLocation", "http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd http://www.garmin.com/xmlschemas/GpxExtensions/v3 http://www.garmin.com/xmlschemas/GpxExtensionsv3.xsd http://www.garmin.com/xmlschemas/TrackPointExtension/v1 http://www.garmin.com/xmlschemas/TrackPointExtensionv1.xsd");
            gpx_element.set_attribute_value("xmlns:gpxtpx", "http://www.garmin.com/xmlschemas/TrackPointExtension/v1");
            gpx_element.set_attribute_value("xmlns:gpxx", "http://www.garmin.com/xmlschemas/GpxExtensions/v3");
            gpx_document.root().append_child(gpx_element);

            let metadata_element = gpx_document.create_element("metadata");
            let time_element = gpx_document.create_element("time");
            time_element.append_child(gpx_document.create_text(&start_date.format("%Y-%m-%dT%H:%M:%SZ").to_string()));
            metadata_element.append_child(time_element);
            gpx_element.append_child(metadata_element);

            let track_element = gpx_document.create_element("trk");
            let track_segment_element = gpx_document.create_element("trkseg");
            track_element.append_child(track_segment_element);
            gpx_element.append_child(track_element);

            if let Some(map_point_array) = run.get("MapPoints").and_then(|x| x.as_array()) {
                for map_point in map_point_array.iter().map(|x| x.as_object().unwrap()) {
                    if let Some(location) = map_point.get("Location").and_then(|x| x.as_object()) {
                        match (location.get("Latitude").and_then(|x| x.as_f64()),
                            location.get("Longitude").and_then(|x| x.as_f64()),
                            map_point.get("TotalDistance").and_then(|x| x.as_f64()),
                            map_point.get("HeartRate").and_then(|x| x.as_i64())) {

                            (Some(latitude), Some(longitude), Some(distance), heart_rate) => {
                                while distance_and_time_array.len() > 0 && distance > distance_and_time_array[0].0 {
                                    last_distance_and_time = distance_and_time_array.remove(0);
                                }

                                // Try to extrapolate the elapsed seconds from the distance and time from the PaceData
                                let time;
                                if distance_and_time_array.len() > 0 {
                                    let scale = (distance - last_distance_and_time.0) / (distance_and_time_array[0].0 - last_distance_and_time.0);
                                    time = scale * (distance_and_time_array[0].1 - last_distance_and_time.1) as f64 + last_distance_and_time.1 as f64;
                                }
                                else {
                                    time = last_distance_and_time.1 as f64;
                                }

                                // Create the GPX elements for the track point
                                let trackpoint_element = gpx_document.create_element("trkpt");
                                trackpoint_element.set_attribute_value("lat", &format!("{}", latitude / 10_000_000.0));
                                trackpoint_element.set_attribute_value("lon", &format!("{}", longitude / 10_000_000.0));
                                track_segment_element.append_child(trackpoint_element);

                                let waypoint_date = start_date + Duration::seconds(time as i64);
                                let time_element = gpx_document.create_element("time");
                                time_element.append_child(gpx_document.create_text(&waypoint_date.format("%Y-%m-%dT%H:%M:%SZ").to_string()));
                                trackpoint_element.append_child(time_element);

                                if let Some(heart_rate) = heart_rate {
                                    let extensions_element = gpx_document.create_element("extensions");
                                    let trackpoint_extension_element = gpx_document.create_element("gpxtpx:TrackPointExtension");
                                    let heartrate_element = gpx_document.create_element("gpxtpx:hr");
                                    heartrate_element.append_child(gpx_document.create_text(&format!("{}", heart_rate)));
                                    trackpoint_extension_element.append_child(heartrate_element);
                                    extensions_element.append_child(trackpoint_extension_element);
                                    trackpoint_element.append_child(extensions_element);
                                }
                            },
                            _ => {}
                        }
                    }
                }

                // Output the XML file
                let output_filename = String::from(output) + &format!("-{}.gpx", run_number);
                let mut buffer = File::create(output_filename).ok().expect("There was a problem opening the GPX file.");
                format_document(&gpx_document, &mut buffer).ok().expect("There was a problem outputting the XML.");
            }
            else {
                println!("Could not find any MapPoints! Skipping this run...");
            }

            run_number += 1;
        }
    }
}
