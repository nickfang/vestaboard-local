use dotenv::dotenv;
use std::env;
use reqwest::Client;
use serde::Deserialize;
use serde_json;

use crate::widgets::widget_utils::{
    full_justify_line,
    center_line,
    split_into_lines,
    center_message,
    WidgetOutput,
};
use crate::errors::VestaboardError;

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

pub async fn get_weather() -> Result<WidgetOutput, VestaboardError> {
    dotenv().ok();
    let weather_api_key = env
        ::var("WEATHER_API_KEY")
        .map_err(|_|
            VestaboardError::config_error("WEATHER_API_KEY", "Environment variable not set")
        )?;

    let client = Client::new();
    #[allow(unused_variables)]
    let url_current =
        format!("https://api.weatherapi.com/v1/current.json?key={}&q=austin", weather_api_key);
    // format!("https://api.weatherapi.com/v1/current.json?key={}", weather_api_key);
    let url_forecast =
        format!("https://api.weatherapi.com/v1/forecast.json?key={}&q=austin&days=3&aqi=no&alerts=no", weather_api_key);

    let response = client
        .get(&url_forecast)
        .send().await
        .map_err(|e| VestaboardError::reqwest_error(e, "requesting weather forecast"))?;

    let status_code = response.status().as_u16();
    let response_text = response
        .text().await
        .map_err(|e| VestaboardError::reqwest_error(e, "reading weather response"))?;

    match status_code {
        200 => {
            // Try to parse the text as JSON
            let json: WeatherResponse = serde_json
                ::from_str(&response_text)
                .map_err(|e| VestaboardError::json_error(e, "parsing weather API response"))?;

            let localtime = json.location.localtime.to_lowercase();
            let temps = format!(
                "W{:>3.1}D B{:>3.1}D R{:>3.1}D",
                json.current.temp_f,
                json.forecast.forecastday[0].day.mintemp_f,
                json.forecast.forecastday[0].day.maxtemp_f
            );
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
            let weather_summary = format!("{} {} {}", condition, rain_chance, rain_amount);
            let pressure_in = format!(" {}", json.current.pressure_in);
            let future_pressure_in =
                json.forecast.forecastday
                    .iter()
                    .take(2)
                    .map(|day| format!("{:>.2}", day.hour[0].pressure_in))
                    .collect::<Vec<String>>()
                    .join(" ") + " ";

            let mut weather_description = Vec::new();
            weather_description.push(center_line(localtime));
            weather_description.push(center_line(temps));

            for line in center_message(split_into_lines(&weather_summary), 3) {
                weather_description.push(center_line(line.to_string()));
            }
            weather_description.push(full_justify_line(pressure_in, future_pressure_in));

            Ok(weather_description)
        }
        400 | 401 | 403 => {
            // Parse error response
            let error: serde_json::Value = serde_json
                ::from_str(&response_text)
                .map_err(|_|
                    VestaboardError::api_error(Some(status_code), "Invalid API response format")
                )?;

            let error_code = error["error"]["code"].as_i64();
            let error_message = error["error"]["message"].as_str().unwrap_or("Unknown error");

            Err(
                VestaboardError::api_error(
                    error_code.map(|c| c as u16),
                    &format!("Weather API error: {}", error_message)
                )
            )
        }
        502 | 504 => {
            Err(
                VestaboardError::api_error(
                    Some(status_code),
                    "Weather service temporarily unavailable"
                )
            )
        }
        _ => {
            Err(
                VestaboardError::api_error(
                    Some(status_code),
                    &format!("Unexpected response status: {}", status_code)
                )
            )
        }
    }
}
