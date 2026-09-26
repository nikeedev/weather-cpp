use reqwest::header::{CONTENT_TYPE, USER_AGENT};
use serde_json::Value;
use std::{env, fs};
use chrono::{DateTime, Local, Utc};
use std::process::exit;

fn direction<'a>(degrees: f64) -> &'a str {
    match degrees {
        0.0..22.5 => "↑",
        22.6..67.5 => "↗",
        67.6..112.5 => "→",
        112.6..157.5 => "↘",
        157.6..202.5 => "↓",
        202.6..247.5 => "↙",
        247.6..292.5 => "←",
        292.6..337.5 => "↖",
        337.5..360.0 => "↑",
        _ => "NaN"
    }
} 




fn get_value_arg<'a>(from: &'static str, v: &'a[String]) -> Option<&'a String> {
    let index = v.iter().position(|r| r == format!("--{from}").as_str()).unwrap();

    return v.get(index + 1_usize);
    // println!("Value for {:?} is not defined!", from);
    // exit(1);
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let args: Vec<String> = env::args().collect();

    // just view help because user asked, exit then
    if args.contains(&"help".to_string()) || args.contains(&"--help".to_string()) {
        println!("usage: ./weather-rs [--short] [--lat <lat>] [--long <long>] [--name <name>] [--place <callname>]");
        println!("\t--short: outputs the weather in a short format");
        println!("\t--lat <int>, --long <int>: change the coordiantes for the weather (Norway only)");
        println!("\t--name <string>: give a name to the place if you use the coordinates");
        println!("\t--place <string>: retrieves the place from the given call name from `places.json` file");
        println!("help, --help: show this message");

        exit(0);
    }

    // let user_info = reqwest::get("https://api.techniknews.net/ipgeo/")
    //     .await?
    //     .text()
    //     .await?;

    // let user_info: Value = serde_json::from_str(&user_info).unwrap();

    // let lat = user_info["lat"].as_f64().unwrap();
    // let long = user_info["lon"].as_f64().unwrap();
    
    // Nesheim
    let mut lat = 59.46279;
    let mut long = 5.57334;
    let mut place = String::from("Nesheim");
   
    let mut got_coords = false;

    if args.contains(&"--lat".to_string()) {
        let lat_temp = get_value_arg("lat", &args);
        if lat_temp.is_none() {
            println!("Value for \"lat\" was not defined!");
            exit(0);
        }
        lat = lat_temp.unwrap().parse::<f64>().unwrap();
    }
   
    if args.contains(&"--long".to_string()) {
        let long_temp = get_value_arg("long", &args);
        if long_temp.is_none() {
            println!("Value for \"long\" was not defined!");
            exit(0);
        }
        long = long_temp.unwrap().parse::<f64>().unwrap();
        got_coords = true;
    }
        
    if got_coords {
        place = format!("{}, {}", lat, long);
    }

    if args.contains(&"--name".to_string()) {
        let name_temp = get_value_arg("name", &args);
        if name_temp.is_none() {
             println!("Value for \"name\" was not defined!");
             exit(0);
        }
        place = name_temp.unwrap().to_string();
    }
 
    if args.contains(&"--place".to_string()) {
        let place_temp = get_value_arg("place", &args);
        if place_temp.is_none() {
             println!("Value for \"place\" was not defined!");
             exit(0);
        }

        // get location of 
        let file_path = match env::current_exe() {
            Ok(path) => {
                let cloned = path.clone();
                let cloned = cloned.parent().unwrap().to_str().unwrap().to_string();
                cloned
            },
            Err(e) => {
                eprintln!("Failed to get executable path: {}", e);
                String::new()
            }   
        };
        
        let file_path = file_path.replace("\\", "/");

        let contents = fs::read_to_string(format!("{}/places.json", file_path))
            .expect("Should have been able to read the file");
        
        let parsed: Value = serde_json::from_str(&contents.as_str()).unwrap();
        
        place = parsed[place_temp.unwrap()]["name"].to_string();
        lat = parsed[place_temp.unwrap()]["lat"].as_f64().unwrap();
        long = parsed[place_temp.unwrap()]["long"].as_f64().unwrap();
    }
 

    // println!("Getting weather for following location: {}, {}", lat, long);
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let locast = client
        .get(format!(
            "https://api.met.no/weatherapi/locationforecast/2.0/complete?lat={lat}&lon={long}&altitude=76"
        ))
        .header(CONTENT_TYPE, "application/json")
        .header(USER_AGENT, "weather 1.0/nikeedev")
        .send()
        .await?
        .text()
        .await?;

    let locast: Value = serde_json::from_str(&locast.as_str()).unwrap();

    let current_weather = locast["properties"]["timeseries"][0].clone();
    
    // println!("{}", current_weather["time"]);
    let weather_time = current_weather["time"].clone()
        .as_str()
        .and_then(|time_str| time_str.parse::<DateTime<Utc>>().ok())
        .map(|utc_time| utc_time.with_timezone(&Local).format("%d.%m.%Y %H:%M").to_string())
        .unwrap_or_else(|| "Invalid time".to_string());
    
    let current_weather = locast["properties"]["timeseries"][0]["data"].clone();

    
    
    match args.contains(&"--short".to_string()) {
        true => {
            println!("Weather at {}", place);
            let now = current_weather["instant"]["details"].clone();
            println!("Temperature 🌡️: {}°C", now["air_temperature"]);
            println!("Wind 🌬️ : \n\tDirection {} ({}°) \n\tWind speed: {} m/s", direction(now["wind_from_direction"].as_f64().unwrap()), now["wind_from_direction"].as_f64().unwrap(), now["wind_speed"]);
           println!("UV level (at clear sky) ☀️: {}", now["ultraviolet_index_clear_sky"].as_f64().unwrap().floor());
        },

        false => {
            println!("Weather at {}", place);
            let now = current_weather["instant"]["details"].clone();
            println!("Now ({}):", weather_time);
            println!("\tTemperature 🌡️: {}°C", now["air_temperature"]);
            println!("\tWind 🌬️ : \n\t\tDirection {} ({}°) \n\t\tWind speed: {} m/s", direction(now["wind_from_direction"].as_f64().unwrap()), now["wind_from_direction"].as_f64().unwrap(), now["wind_speed"]);
            println!("\tUV level (at clear sky) ☀️: {}", now["ultraviolet_index_clear_sky"].as_f64().unwrap().floor());
            
            let next_hour = current_weather["next_1_hours"]["details"].clone();
            println!("\nWeather next hour:");
            println!("\tPrecipitation probability: {}%", next_hour["probability_of_precipitation"]);
            println!("\tPrecipitation amount: {} mm", next_hour["precipitation_amount"]);
            println!("\tPrecipitation amount (min/max): {} mm / {} mm", next_hour["precipitation_amount_min"], next_hour["precipitation_amount_max"]);
            let borrow = current_weather["next_1_hours"]["summary"].clone();
            let next_hour = borrow["symbol_code"].as_str().unwrap();
            println!("\tWeather summary: {}", weather_description(next_hour));

            let six_hour = current_weather["next_6_hours"]["details"].clone();
            println!("\nWeather for the next 6 hours:");
            println!("\tTemperature (min/max): {} / {} °C", six_hour["air_temperature_min"], six_hour["air_temperature_max"]);
            println!("\tPrecipitation probability: {}%", six_hour["probability_of_precipitation"]);
            println!("\tPrecipitation amount: {} mm", six_hour["precipitation_amount"]);
            println!("\tPrecipitation amount (min/max): {} mm / {} mm", six_hour["precipitation_amount_min"], six_hour["precipitation_amount_max"]);
            let borrow = current_weather["next_6_hours"]["summary"].clone();
            let six_hour = borrow["symbol_code"].as_str().unwrap();
            println!("\tWeather summary: {}", weather_description(six_hour));
            
            let twelvehour = current_weather["next_12_hours"]["details"].clone();
            println!("\nWeather for the next 12 hours:");
            println!("\tPrecipitation probability: {}%", twelvehour["probability_of_precipitation"]);
            let borrow = current_weather["next_12_hours"]["summary"].clone();
            let twelvehour = borrow["symbol_code"].as_str().unwrap();
            println!("\tWeather summary: {}", weather_description(twelvehour));
 
        }
        
    }
    
     

    Ok(())
}
