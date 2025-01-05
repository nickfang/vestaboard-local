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
    name: String,
    region: String,
    country: String,
    lat: f64,
    lon: f64,
    tz_id: String,
    localtime_epoch: i64,
    localtime: String,
}

#[derive(Deserialize, Debug)]
struct Current {
    last_updated_epoch: i64,
    last_updated: String,
    temp_c: f64,
    temp_f: f64,
    is_day: i32,
    condition: Condition,
    wind_kph: f64,
    wind_mph: f64,
    wind_degree: i32,
    wind_dir: String,
    pressure_in: f64,
    pressure_mb: f64,
    precip_in: f64,
    precip_mm: f64,
    humidity: i32,
    cloud: i32,
    feelslike_c: f64,
    feelslike_f: f64,
    windchill_c: f64,
    windchill_f: f64,
    heatindex_c: f64,
    heatindex_f: f64,
    dewpoint_c: f64,
    dewpoint_f: f64,
    vis_km: f64,
    vis_miles: f64,
    uv: f64,
    gust_kph: f64,
    gust_mph: f64,
}

#[derive(Deserialize, Debug)]
struct Condition {
    text: String,
    icon: String,
    code: i32,
}

#[derive(Deserialize, Debug)]
struct Forecast {
    forecastday: Vec<ForecastDay>,
}

#[derive(Deserialize, Debug)]
struct ForecastDay {
    astro: Astro,
    date: String,
    date_epoch: i64,
    day: Day,
    hour: Vec<Hour>,
}

#[derive(Deserialize, Debug)]
struct Astro {
    sunrise: String,
    sunset: String,
    moonrise: String,
    moonset: String,
    moon_phase: String,
    moon_illumination: i32,
    is_moon_up: i32,
    is_sun_up: i32,
}

#[derive(Deserialize, Debug)]
struct Day {
    maxtemp_c: f64,
    maxtemp_f: f64,
    mintemp_c: f64,
    mintemp_f: f64,
    avgtemp_c: f64,
    avgtemp_f: f64,
    maxwind_kph: f64,
    maxwind_mph: f64,
    totalprecip_in: f64,
    totalprecip_mm: f64,
    totalsnow_cm: f64,
    avgvis_km: f64,
    avgvis_miles: f64,
    avghumidity: i32,
    daily_will_it_rain: i32,
    daily_chance_of_rain: i32,
    daily_will_it_snow: i32,
    daily_chance_of_snow: i32,
    condition: Condition,
    uv: f64,
}

#[derive(Deserialize, Debug)]
struct Hour {
    time_epoch: i64,
    time: String,
    temp_c: f64,
    temp_f: f64,
    is_day: i32,
    condition: Condition,
    wind_kph: f64,
    wind_mph: f64,
    wind_degree: i32,
    wind_dir: String,
    pressure_in: f64,
    pressure_mb: f64,
    precip_in: f64,
    precip_mm: f64,
    snow_cm: f64,
    humidity: i32,
    cloud: i32,
    feelslike_c: f64,
    feelslike_f: f64,
    windchill_c: f64,
    windchill_f: f64,
    heatindex_c: f64,
    heatindex_f: f64,
    dewpoint_c: f64,
    dewpoint_f: f64,
    will_it_rain: i32,
    chance_of_rain: i32,
    will_it_snow: i32,
    chance_of_snow: i32,
    vis_km: f64,
    vis_miles: f64,
    gust_kph: f64,
    gust_mph: f64,
    uv: f64,
}

pub async fn get_weather() -> Result<Vec<String>, reqwest::Error> {
    dotenv().ok();
    let weather_api_key = env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY not set");

    let client = Client::new();
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
