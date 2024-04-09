use chrono::SecondsFormat::Secs;
use chrono::{Duration, TimeZone, Utc};
use chrono_tz::Europe::London;
use reqwest::Method;
use serde::Deserialize;

fn main() {
    let period_from: String;
    let period_to: String;

    {
        let tomorrow = London.from_utc_datetime(&(Utc::now() + Duration::days(1)).naive_utc());

        period_from = London
            .from_local_datetime(&tomorrow.date_naive().and_hms_opt(0, 0, 0).unwrap())
            .unwrap()
            .to_rfc3339_opts(Secs, true);

        period_to = London
            .from_local_datetime(
                &(*&tomorrow + Duration::days(1))
                    .date_naive()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            )
            .unwrap()
            .to_rfc3339_opts(Secs, true);
    }

    let request = reqwest::blocking::Client::new()
        .request(Method::GET, "https://api.octopus.energy/v1/products/AGILE-18-02-21/electricity-tariffs/E-1R-AGILE-18-02-21-C/standard-unit-rates/")
        .query(&[("period_from", period_from), ("period_to", period_to)])
        .send()
        .unwrap()
        .json::<StandardUnitRates>()
        .unwrap();

    let mut rates = vec![];

    for i in 1..(request.results.len() - 1) {
        let valid_from = &request.results[i].valid_from;
        let valid_to = &request.results[i + 1].valid_from;

        let value_exc_vat = request.results[i].value_exc_vat + request.results[i + 1].value_exc_vat;

        rates.push((format!("{} - {}", valid_from, valid_to), value_exc_vat));
    }

    println!("{:?}", rates)
}

#[derive(Deserialize)]
struct StandardUnitRates {
    results: Vec<StandardUnitRate>,
}

#[derive(Deserialize)]
struct StandardUnitRate {
    value_exc_vat: f64,
    valid_from: String,
    valid_to: String,
}
