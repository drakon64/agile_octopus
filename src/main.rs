use aws_lambda_events::eventbridge::EventBridgeEvent;
use aws_sdk_sns::types::MessageAttributeValue;
use chrono::SecondsFormat::Secs;
use chrono::{DateTime, Duration, Local, NaiveTime, TimeZone, Utc};
use chrono_tz::Europe::London;
use chrono_tz::Tz;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use reqwest::Method;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Deserialize)]
struct StandardUnitRates {
    results: Vec<StandardUnitRate>,
}

#[derive(Deserialize)]
struct StandardUnitRate {
    value_exc_vat: f64,
    valid_from: DateTime<Local>,
    valid_to: DateTime<Local>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}

async fn function_handler(_event: LambdaEvent<EventBridgeEvent<String>>) -> Result<(), Error> {
    let tomorrow = get_tomorrow();
    let cheapest_rate = get_cheapest_rate(get_rates(tomorrow.0, tomorrow.1).await);

    let valid_from = cheapest_rate.0.format("%-I:%M %p").to_string();
    let valid_to = cheapest_rate.1.format("%-I:%M %p").to_string();

    println!("{}", format_message(&valid_from, &valid_to));

    send_sms(&valid_from, &valid_to).await;

    Ok(())
}

fn get_tomorrow() -> (DateTime<Tz>, DateTime<Tz>) {
    let tomorrow = London.from_utc_datetime(&(Utc::now() + Duration::days(1)).naive_utc());

    let period_from = London
        .from_local_datetime(&tomorrow.date_naive().and_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    let period_to = London
        .from_local_datetime(
            &(tomorrow + Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .unwrap();

    (period_from, period_to)
}

async fn get_rates(
    period_from: DateTime<Tz>,
    period_to: DateTime<Tz>,
) -> Vec<(DateTime<Tz>, DateTime<Tz>, f64)> {
    let mut request = reqwest::Client::new()
        .request(Method::GET, "https://api.octopus.energy/v1/products/AGILE-18-02-21/electricity-tariffs/E-1R-AGILE-18-02-21-C/standard-unit-rates/")
        .query(&[("period_from", period_from.to_rfc3339_opts(Secs, true)), ("period_to", period_to.to_rfc3339_opts(Secs, true))])
        .send()
        .await
        .unwrap()
        .json::<StandardUnitRates>()
        .await
        .unwrap();

    request.results.reverse();

    let mut rates = vec![];

    for i in 1..(request.results.len() - 1) {
        let valid_from = London.from_utc_datetime(&request.results[i].valid_from.naive_utc());
        let valid_to = London.from_utc_datetime(&request.results[i + 1].valid_to.naive_utc());

        let value_exc_vat = request.results[i].value_exc_vat + request.results[i + 1].value_exc_vat;

        rates.push((valid_from.to_owned(), valid_to.to_owned(), value_exc_vat));
    }

    rates
}

fn get_cheapest_rate(rates: Vec<(DateTime<Tz>, DateTime<Tz>, f64)>) -> (NaiveTime, NaiveTime) {
    let mut cheapest: Option<&(DateTime<Tz>, DateTime<Tz>, f64)> = None;
    for i in rates.iter() {
        if Option::is_none(&cheapest) || cheapest.unwrap().2 > i.2 {
            cheapest = Option::from(i)
        }
    }

    (
        cheapest.unwrap().0.naive_local().time(),
        cheapest.unwrap().1.naive_local().time(),
    )
}

async fn send_sms(valid_from: &String, valid_to: &String) {
    aws_sdk_sns::Client::new(&aws_config::load_from_env().await)
        .publish()
        .phone_number(env::var("PHONE_NUMBER").unwrap())
        .set_message_attributes(Some(HashMap::from([(
            "AWS.SNS.SMS.SMSType".into(),
            MessageAttributeValue::builder()
                .set_data_type(Some("String".into()))
                .set_string_value(Some("Promotional".into()))
                .build()
                .unwrap(),
        )])))
        .message(format_message(valid_from, valid_to))
        .send()
        .await
        .unwrap();
}

fn format_message(valid_from: &String, valid_to: &String) -> String {
    format!(
        "The cheapest hour for the Agile Octopus tariff tomorrow is between {} and {}.",
        valid_from, valid_to
    )
}

#[cfg(test)]
mod tests {
    use crate::{get_cheapest_rate, get_rates};
    use chrono::{NaiveTime, TimeZone};
    use chrono_tz::Europe::London;

    #[tokio::test(flavor = "current_thread")]
    async fn test_historic_cheapest_rate() {
        let period_from = London.with_ymd_and_hms(2020, 2, 12, 0, 0, 0).unwrap();
        let period_to = London.with_ymd_and_hms(2020, 2, 13, 0, 0, 0).unwrap();

        let rates = get_rates(period_from, period_to).await;
        let cheapest_rate = get_cheapest_rate(rates);

        assert_eq!(
            cheapest_rate,
            (
                NaiveTime::from_hms_opt(3, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(4, 0, 0).unwrap(),
            )
        )
    }
}
