use clap::Parser;
use serde_json::Value;
use std::{collections::HashMap, env};

#[derive(Parser)]
#[command(name = "weather_app")]
#[command(about = "Simple weather app")]
struct Args {
    // Define a single string argument
    #[arg(short, long, help = "City to check current weather")]
    city: String,
}

async fn get_reqeust(url: &str) -> Option<String> {
    let response = reqwest::get(url).await.expect("msg");
    if response.status().is_success() {
        let body = response.text().await.expect("msg");
        return Some(body);
    }
    println!("Failed to fetch data. Status: {}", response.status());
    None
}
async fn get_location_keys(city: &str, api_key: &str) -> HashMap<String, String> {
    let url = format!(
        "http://dataservice.accuweather.com/locations/v1/cities/search?q={}&apikey={}",
        city, api_key
    );
    // Send the GET request
    let mut result = HashMap::new();

    if let Some(response) = get_reqeust(&url).await {
        let json_value: Vec<Value> = serde_json::from_str(&response).expect("msg");
        for item in json_value {
            if let Some(key) = item.get("Key") {
                if let Some(key_str) = key.as_str() {
                    if let Some(administrative_area) = item.get("AdministrativeArea") {
                        if let Some(english_name) = administrative_area.get("EnglishName") {
                            if let Some(english_name_str) = english_name.as_str() {
                                result.insert(
                                    key_str.to_owned(),
                                    format!("{}, {}", city, english_name_str),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    // Check if the response status is success
    result
}

async fn get_weather(locations: HashMap<String, String>, api_key: &str) {
    for (city_key, name) in locations {
        let url = format!(
            " http://dataservice.accuweather.com/currentconditions/v1/{}?apikey={}",
            city_key, api_key
        );
        if let Some(response) = get_reqeust(&url).await {
            let json_value: Vec<Value> = serde_json::from_str(&response).expect("msg");
            for item in json_value {
                if let Some(temperature) = item.get("Temperature") {
                    if let Some(metric_temp) = temperature.get("Metric") {
                        if let Some(celcias_temp) =
                            metric_temp.get("Value").and_then(|v| v.as_f64())
                        {
                            if let Some(weather_text) = item.get("WeatherText") {
                                println!(
                                    "{} ,temp: {} C, {}",
                                    name,
                                    celcias_temp,
                                    weather_text.as_str().unwrap()
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let api_key = env::var("API_KEY").expect("API KEY");
    let locations = get_location_keys(&args.city, &api_key).await;
    get_weather(locations, &api_key).await;
}
