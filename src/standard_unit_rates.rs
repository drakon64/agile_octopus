use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct StandardUnitRates {
    pub(crate) results: Vec<StandardUnitRate>,
}

#[derive(Deserialize)]
pub(crate) struct StandardUnitRate {
    pub(crate) value_exc_vat: f64,
    pub(crate) valid_from: String,
    pub(crate) valid_to: String,
}
