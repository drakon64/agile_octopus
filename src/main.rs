mod standard_unit_rates;

use crate::standard_unit_rates::StandardUnitRates;
use chrono::SecondsFormat::Secs;
use chrono::{DateTime, Duration, TimeZone, Utc};
use chrono_tz::Europe::London;
use chrono_tz::Tz;
use reqwest::Method;

fn main() {
    let tomorrow = get_tomorrow();

    println!("{:?}", get_cheapest_rate(get_rates(tomorrow.0, tomorrow.1)))
}

fn get_tomorrow() -> (DateTime<Tz>, DateTime<Tz>) {
    let tomorrow = London.from_utc_datetime(&(Utc::now() + Duration::days(1)).naive_utc());

    let period_from = London
        .from_local_datetime(&tomorrow.date_naive().and_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    let period_to = London
        .from_local_datetime(
            &(*&tomorrow + Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .unwrap();

    (period_from, period_to)
}

fn get_rates(period_from: DateTime<Tz>, period_to: DateTime<Tz>) -> Vec<(String, f64)> {
    let mut request = reqwest::blocking::Client::new()
        .request(Method::GET, "https://api.octopus.energy/v1/products/AGILE-18-02-21/electricity-tariffs/E-1R-AGILE-18-02-21-C/standard-unit-rates/")
        .query(&[("period_from", period_from.to_rfc3339_opts(Secs, true)), ("period_to", period_to.to_rfc3339_opts(Secs, true))])
        .send()
        .unwrap()
        .json::<StandardUnitRates>()
        .unwrap();

    request.results.reverse();

    let mut rates = vec![];

    for i in 1..(request.results.len() - 1) {
        let valid_from = &request.results[i].valid_from;
        let valid_to = &request.results[i + 1].valid_to;

        let value_exc_vat = request.results[i].value_exc_vat + request.results[i + 1].value_exc_vat;

        rates.push((format!("{} - {}", valid_from, valid_to), value_exc_vat));
    }

    rates
}

fn get_cheapest_rate(rates: Vec<(String, f64)>) -> (String, f64) {
    let mut cheapest: Option<&(String, f64)> = None;
    for i in rates.iter() {
        if cheapest == None || cheapest.unwrap().1 > i.1 {
            cheapest = Option::from(i)
        }
    }

    cheapest.unwrap().clone()
}

#[cfg(test)]
mod tests {
    use crate::{get_cheapest_rate, get_rates};
    use chrono::TimeZone;
    use chrono_tz::Europe::London;

    #[test]
    fn test_cheapest_rate() {
        let period_from = London.with_ymd_and_hms(2020, 2, 12, 0, 0, 0).unwrap();
        let period_to = London.with_ymd_and_hms(2020, 2, 13, 0, 0, 0).unwrap();

        let rates = get_rates(period_from, period_to);
        let cheapest_rate = get_cheapest_rate(rates);

        assert_eq!(
            cheapest_rate,
            (
                "2020-02-12T03:00:00Z - 2020-02-12T04:00:00Z".to_string(),
                4.5
            )
        )
    }
}
