use core::f64;
use polars::prelude::*;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

mod histdiff;
pub use histdiff::calculate_scores;

pub struct HistDiffRes {
    pub raw_scores: HashMap<String, HashMap<String, f64>>,
    pub dataframe_scores: Option<DataFrame>,
}

impl HistDiffRes {
    pub fn new(scores: HashMap<String, HashMap<String, f64>>) -> Self {
        let df = to_df(&scores).expect("Can't convert to dataframe");
        Self {
            raw_scores: scores,
            dataframe_scores: Some(df),
        }
    }
}

fn to_df(raw_out: &HashMap<String, HashMap<String, f64>>) -> Result<DataFrame, PolarsError> {
    let mut row_keys: Vec<&String> = raw_out.keys().collect();
    row_keys.sort();

    // col keys
    let mut col_set = HashSet::new();
    for inner_map in raw_out.values() {
        for col_key in inner_map.keys() {
            col_set.insert(col_key.as_str());
        }
    }
    let mut col_keys: Vec<&str> = col_set.into_iter().collect();
    col_keys.sort();

    // make a series for each col
    let mut series_list = Vec::with_capacity(col_keys.len() + 1);

    let row_labels: Vec<String> = row_keys.iter().map(|k| (*k).clone()).collect();

    let row_series = Series::new("row_key".into(), row_labels);
    series_list.push(row_series.into());

    for &col_key in &col_keys {
        let mut col_data = Vec::with_capacity(row_keys.len());
        for &row_key in &row_keys {
            let inner_map = &raw_out[row_key];
            let val = inner_map.get(col_key).cloned().unwrap_or(f64::NAN);
            col_data.push(val);
        }
        let series = Series::new(col_key.into(), col_data);
        series_list.push(series.into());
    }

    return DataFrame::new(series_list);
}
