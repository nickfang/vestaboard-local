use dotenv::dotenv;
use std::env;
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current: Current,
    location: Location,
}

#[derive(Deserialize, Debug)]
struct Location {
    country: String,
    lat: f64,
    localtime_epoch: i64,
    localtime: String,
    lon: f64,
    name: String,
    region: String,
    tz_id: String,
}

#[derive(Deserialize, Debug)]
struct Current {
    cloud: i32,
    condition: Condition,
    dewpoint_c: f64,
    dewpoint_f: f64,
    feelslike_c: f64,
    feelslike_f: f64,
    gust_kph: f64,
    gust_mph: f64,
    heatindex_c: f64,
    heatindex_f: f64,
    humidity: i32,
    is_day: i32,
    last_updated_epoch: i64,
    last_updated: String,
    precip_in: f64,
    precip_mm: f64,
    pressure_in: f64,
    pressure_mb: f64,
    temp_c: f64,
    temp_f: f64,
    uv: f64,
    vis_km: f64,
    vis_miles: f64,
    wind_degree: i32,
    wind_dir: String,
    wind_kph: f64,
    wind_mph: f64,
    windchill_c: f64,
    windchill_f: f64,
}

#[derive(Deserialize, Debug)]
struct Condition {
    text: String,
    icon: String,
    code: i32,
}

pub async fn get_weather() -> Result<Vec<String>, reqwest::Error> {
    dotenv().ok();
    let weather_api_key = env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY not set");
    println!("Weather API Key: {}", weather_api_key);
    let client = Client::new();
    let url =
        format!("https://api.weatherapi.com/v1/current.json?key={}&q=austin", weather_api_key);
    println!("URL: {}", url);
    // return Ok(());
    let res = client.get(&url).send().await;

    match res {
        Ok(response) => {
            match response.json::<WeatherResponse>().await {
                Ok(json) => {
                    println!("Response: {:?}", json);
                    let mut weather_description = Vec::new();
                    weather_description.push(format!("{}", json.location.localtime.to_lowercase()));
                    weather_description.push(format!("temp:      {}", json.current.temp_f));
                    weather_description.push(
                        format!("condition: {:?}", json.current.condition.text.to_lowercase())
                    );
                    weather_description.push(format!("precip:    {}", json.current.precip_in));
                    weather_description.push(format!("pressure:  {}", json.current.pressure_in));
                    println!("{:?}", weather_description);
                    Ok(weather_description)
                }
                Err(e) => {
                    println!("Failed to parse JSON: {:?}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
            Err(e)
        }
    }
}
