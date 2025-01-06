use dotenv::dotenv;
use std::env;
use reqwest::Client;
use serde::Deserialize;

// reference: https://www.weatherapi.com/api-explorer.aspx#forecast

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current: Current,
    location: Location,
    forecast: Forecast,
}

#[derive(Deserialize, Debug)]
struct Location {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    region: String,
    #[allow(dead_code)]
    country: String,
    #[allow(dead_code)]
    lat: f64,
    #[allow(dead_code)]
    lon: f64,
    #[allow(dead_code)]
    tz_id: String,
    #[allow(dead_code)]
    localtime_epoch: i64,
    localtime: String,
}

#[derive(Deserialize, Debug)]
struct Current {
    #[allow(dead_code)]
    last_updated_epoch: i64,
    #[allow(dead_code)]
    last_updated: String,
    #[allow(dead_code)]
    temp_c: f64,
    temp_f: f64,
    #[allow(dead_code)]
    is_day: i32,
    condition: Condition,
    #[allow(dead_code)]
    wind_kph: f64,
    #[allow(dead_code)]
    wind_mph: f64,
    #[allow(dead_code)]
    wind_degree: i32,
    #[allow(dead_code)]
    wind_dir: String,
    pressure_in: f64,
    #[allow(dead_code)]
    pressure_mb: f64,
    #[allow(dead_code)]
    precip_in: f64,
    #[allow(dead_code)]
    precip_mm: f64,
    #[allow(dead_code)]
    humidity: i32,
    #[allow(dead_code)]
    cloud: i32,
    #[allow(dead_code)]
    feelslike_c: f64,
    #[allow(dead_code)]
    feelslike_f: f64,
    #[allow(dead_code)]
    windchill_c: f64,
    #[allow(dead_code)]
    windchill_f: f64,
    #[allow(dead_code)]
    heatindex_c: f64,
    #[allow(dead_code)]
    heatindex_f: f64,
    #[allow(dead_code)]
    dewpoint_c: f64,
    #[allow(dead_code)]
    dewpoint_f: f64,
    #[allow(dead_code)]
    vis_km: f64,
    #[allow(dead_code)]
    vis_miles: f64,
    #[allow(dead_code)]
    uv: f64,
    #[allow(dead_code)]
    gust_kph: f64,
    #[allow(dead_code)]
    gust_mph: f64,
}

#[derive(Deserialize, Debug)]
struct Condition {
    text: String,
    #[allow(dead_code)]
    icon: String,
    #[allow(dead_code)]
    code: i32,
}

#[derive(Deserialize, Debug)]
struct Forecast {
    forecastday: Vec<ForecastDay>,
}

#[derive(Deserialize, Debug)]
struct ForecastDay {
    #[allow(dead_code)]
    astro: Astro,
    #[allow(dead_code)]
    date: String,
    #[allow(dead_code)]
    date_epoch: i64,
    day: Day,
    #[allow(dead_code)]
    hour: Vec<Hour>,
}

#[derive(Deserialize, Debug)]
struct Astro {
    #[allow(dead_code)]
    sunrise: String,
    #[allow(dead_code)]
    sunset: String,
    #[allow(dead_code)]
    moonrise: String,
    #[allow(dead_code)]
    moonset: String,
    #[allow(dead_code)]
    moon_phase: String,
    #[allow(dead_code)]
    moon_illumination: i32,
    #[allow(dead_code)]
    is_moon_up: i32,
    #[allow(dead_code)]
    is_sun_up: i32,
}

#[derive(Deserialize, Debug)]
struct Day {
    #[allow(dead_code)]
    maxtemp_c: f64,
    maxtemp_f: f64,
    #[allow(dead_code)]
    mintemp_c: f64,
    mintemp_f: f64,
    #[allow(dead_code)]
    avgtemp_c: f64,
    #[allow(dead_code)]
    avgtemp_f: f64,
    #[allow(dead_code)]
    maxwind_kph: f64,
    #[allow(dead_code)]
    maxwind_mph: f64,
    totalprecip_in: f64,
    #[allow(dead_code)]
    totalprecip_mm: f64,
    #[allow(dead_code)]
    totalsnow_cm: f64,
    #[allow(dead_code)]
    avgvis_km: f64,
    #[allow(dead_code)]
    avgvis_miles: f64,
    #[allow(dead_code)]
    avghumidity: i32,
    #[allow(dead_code)]
    daily_will_it_rain: i32,
    #[allow(dead_code)]
    daily_chance_of_rain: i32,
    #[allow(dead_code)]
    daily_will_it_snow: i32,
    #[allow(dead_code)]
    daily_chance_of_snow: i32,
    #[allow(dead_code)]
    condition: Condition,
    #[allow(dead_code)]
    uv: f64,
}

#[derive(Deserialize, Debug)]
struct Hour {
    #[allow(dead_code)]
    time_epoch: i64,
    #[allow(dead_code)]
    time: String,
    #[allow(dead_code)]
    temp_c: f64,
    #[allow(dead_code)]
    temp_f: f64,
    #[allow(dead_code)]
    is_day: i32,
    #[allow(dead_code)]
    condition: Condition,
    #[allow(dead_code)]
    wind_kph: f64,
    #[allow(dead_code)]
    wind_mph: f64,
    #[allow(dead_code)]
    wind_degree: i32,
    #[allow(dead_code)]
    wind_dir: String,
    #[allow(dead_code)]
    pressure_in: f64,
    #[allow(dead_code)]
    pressure_mb: f64,
    #[allow(dead_code)]
    precip_in: f64,
    #[allow(dead_code)]
    precip_mm: f64,
    #[allow(dead_code)]
    snow_cm: f64,
    #[allow(dead_code)]
    humidity: i32,
    #[allow(dead_code)]
    cloud: i32,
    #[allow(dead_code)]
    feelslike_c: f64,
    #[allow(dead_code)]
    feelslike_f: f64,
    #[allow(dead_code)]
    windchill_c: f64,
    #[allow(dead_code)]
    windchill_f: f64,
    #[allow(dead_code)]
    heatindex_c: f64,
    #[allow(dead_code)]
    heatindex_f: f64,
    #[allow(dead_code)]
    dewpoint_c: f64,
    #[allow(dead_code)]
    dewpoint_f: f64,
    #[allow(dead_code)]
    will_it_rain: i32,
    #[allow(dead_code)]
    chance_of_rain: i32,
    #[allow(dead_code)]
    will_it_snow: i32,
    #[allow(dead_code)]
    chance_of_snow: i32,
    #[allow(dead_code)]
    vis_km: f64,
    #[allow(dead_code)]
    vis_miles: f64,
    #[allow(dead_code)]
    gust_kph: f64,
    #[allow(dead_code)]
    gust_mph: f64,
    #[allow(dead_code)]
    uv: f64,
}

pub async fn get_weather() -> Result<Vec<String>, reqwest::Error> {
    dotenv().ok();
    let weather_api_key = env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY not set");

    let client = Client::new();
    #[allow(unused_variables)]
    let url_current =
        format!("https://api.weatherapi.com/v1/current.json?key={}&q=austin", weather_api_key);
    let url_forecast =
        format!("https://api.weatherapi.com/v1/forecast.json?key={}&q=austin&days=1&aqi=no&alerts=no", weather_api_key);
    let res = client.get(&url_forecast).send().await;

    match res {
        Ok(response) => {
            match response.json::<WeatherResponse>().await {
                Ok(json) => {
                    let mut weather_description = Vec::new();
                    weather_description.push(format!("{}", json.location.localtime.to_lowercase()));
                    weather_description.push(
                        format!(
                            "{}° l:{}° h:{}°",
                            json.current.temp_f,
                            json.forecast.forecastday[0].day.mintemp_f,
                            json.forecast.forecastday[0].day.maxtemp_f
                        )
                    );
                    weather_description.push(
                        format!(
                            "{:?}",
                            json.current.condition.text.replace("\"", "").to_lowercase()
                        )
                    );
                    weather_description.push(
                        format!(
                            "with {}in of rain",
                            json.forecast.forecastday[0].day.totalprecip_in
                        )
                    );
                    weather_description.push(format!("pressure: {}", json.current.pressure_in));
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
