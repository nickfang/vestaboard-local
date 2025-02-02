use dotenv::dotenv;
use std::env;
use reqwest::Client;
use serde::Deserialize;

use crate::widgets::widget_utils::{ format_error, full_justify_line, center_line, WidgetOutput };

// reference: https://www.weatherapi.com/api-explorer.aspx#forecast

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current: Current,
    location: Location,
    forecast: Forecast,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
struct ForecastDay {
    astro: Astro,
    date: String,
    date_epoch: i64,
    day: Day,
    hour: Vec<Hour>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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

pub async fn get_weather() -> WidgetOutput {
    dotenv().ok();
    let weather_api_key = env::var("WEATHER_API_KEY").expect("WEATHER_API_KEY not set");

    let client = Client::new();
    #[allow(unused_variables)]
    let url_current =
        format!("https://api.weatherapi.com/v1/current.json?key={}&q=austin", weather_api_key);
    let url_forecast =
        format!("https://api.weatherapi.com/v1/forecast.json?key={}&q=austin&days=3&aqi=no&alerts=no", weather_api_key);
    let res = client.get(&url_forecast).send().await;
    match res {
        Ok(response) => {
            match response.json::<WeatherResponse>().await {
                Ok(json) => {
                    let localtime = json.location.localtime.to_lowercase();
                    let temp_f = format!("W {}D", json.current.temp_f);
                    let min_temp_f = format!("B {}D", json.forecast.forecastday[0].day.mintemp_f);
                    let max_temp_f = format!("R {}D", json.forecast.forecastday[0].day.maxtemp_f);
                    let condition = json.current.condition.text.replace("\"", "").to_lowercase();
                    let chance_precip = json.forecast.forecastday[0].day.daily_chance_of_rain;
                    let totalprecip_in = json.forecast.forecastday[0].day.totalprecip_in;
                    let rain_chance = if chance_precip > 0 {
                        format!("w/ {}% chance", chance_precip)
                    } else {
                        "".to_string()
                    };
                    let rain_amount = if totalprecip_in > 0.0 {
                        format!("{}\" of rain", totalprecip_in)
                    } else {
                        "".to_string()
                    };
                    let pressure_in = format!("{}", json.current.pressure_in);
                    let future_pressure_in = json.forecast.forecastday
                        .iter()
                        .map(|day| day.hour[0].pressure_in.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");

                    let mut weather_description = Vec::new();
                    weather_description.push(center_line(localtime));
                    weather_description.push(full_justify_line(temp_f, condition));
                    weather_description.push(full_justify_line(min_temp_f, rain_chance));
                    weather_description.push(full_justify_line(max_temp_f, rain_amount));
                    weather_description.push("pressure:".to_string());
                    weather_description.push(full_justify_line(pressure_in, future_pressure_in));
                    weather_description
                }
                Err(e) => {
                    let error = format!("Failed to parse JSON: {:?}", e);
                    eprintln!("{}", error);
                    format_error("error parsing weather data.")
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to retrieve weather data: {:?}", e);
            format_error("error retrieving weather data.")
        }
    }
}
