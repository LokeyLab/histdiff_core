#![allow(unused_parens)]

use core::f64;
use dashmap::DashMap;
use log::{info, trace};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::usize;

use super::utils::UserConfig;

/// exponential smoothing function
pub fn exponential_smoothing(x: &[f64], alpha: f64) -> Vec<f64> {
    let n = x.len();
    let mut smoothing: Vec<f64> = Vec::with_capacity(n);

    for i in (0..n) {
        let x_i = x[i];
        let s_i = if i == 0 {
            if n > 1 {
                x_i + alpha * (x[i + 1] - x_i)
            } else {
                x_i
            }
        } else if (i == (n - 1)) {
            alpha * (x[i - 1] - x_i) + x_i
        } else {
            alpha * (x[i - 1] - x_i) + x_i + alpha * (x[i + 1] - x_i)
        };

        smoothing.push(s_i);
    }

    return smoothing;
}

/// normalization function
pub fn normalize(x: &[f64]) -> Vec<f64> {
    let sum: f64 = x.iter().sum();
    if sum == 0.0 {
        return vec![0.0; x.len()];
    } else {
        return x.iter().map(|&e| e / sum).collect();
    }
}

/// Holds the min max value results
#[derive(Debug, Clone)]
pub struct MinMax {
    pub xlow: f64,
    pub xhigh: f64,
}

/// Holds the min max values for the entire dataset
#[derive(Debug)]
pub struct MinMaxPlateResult {
    pub min_max: Vec<(String, MinMax)>,
    pub features: Vec<String>,
    pub problemativ_features: Option<Vec<String>>,
}

/// retrieves the min max values for a given dataset
pub fn get_min_max_plate(config: &UserConfig) -> Result<MinMaxPlateResult, Box<dyn Error>> {
    if config.verbose {
        info!("Starting Min Max Process for all specified features.");
    }

    let file = File::open(config.path.clone())?;
    let reader = BufReader::new(file);

    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(true)
        .from_reader(reader);

    let headers = csv_reader.headers()?.clone();
    let headers_len = headers.len();
    let headers_vec: Vec<String> = headers.iter().map(|s| s.to_string()).collect();

    // get indices for id columns
    let id_col_idx: Vec<usize> = config
        .id_cols
        .clone()
        .iter()
        .map(|i| headers_vec.iter().position(|h| i == h))
        .collect::<Option<Vec<usize>>>()
        .ok_or("id_col_idx: ID col(s) not found!")?;

    // get indices for useless feats
    let useless_idx: Option<Vec<usize>> = config.useless_cols.as_ref().map(|cols| {
        cols.iter()
            .filter_map(|i| headers_vec.iter().position(|h| h == i))
            .collect()
    });

    let feature_idx: Vec<usize> = (0..headers_len)
        .filter(|i| {
            !id_col_idx.contains(i)
                && useless_idx
                    .as_ref()
                    .map_or(true, |useless| !useless.contains(i))
        })
        .collect();

    let mut feats: Vec<String> = feature_idx
        .iter()
        .map(|&x| headers_vec[x].clone())
        .collect();
    let xlow: DashMap<String, f64> = DashMap::new();
    let xhigh: DashMap<String, f64> = DashMap::new();

    for feat in &feats {
        xlow.insert(feat.clone(), f64::NAN);
        xhigh.insert(feat.clone(), f64::NAN);
    }

    // NOTE: Read Start Time
    let start_t = std::time::Instant::now();
    if config.verbose {
        info!("Beginning to read file for MIN_MAX");
    }

    for res in csv_reader.records() {
        let record = res?;
        if record.len() != headers_len {
            continue;
        }

        feature_idx.par_iter().for_each(|&i| {
            let feat = &headers_vec[i];
            let field = &record[i];

            if let Ok(val) = field.parse::<f64>() {
                if val.is_finite() {
                    // min field
                    xlow.entry(feat.to_string()).and_modify(|e| {
                        if e.is_nan() {
                            *e = val;
                        } else {
                            *e = e.min(val);
                        }
                    });

                    // max field
                    xhigh.entry(feat.to_string()).and_modify(|e| {
                        if e.is_nan() {
                            *e = val;
                        } else {
                            *e = e.max(val);
                        }
                    });
                }
            }

            // nan is skipped
        });
    }

    // NOTE: End of start time
    if config.verbose {
        info!("End of reading MIN_MAX. Time: {:?}", start_t.elapsed());
    }

    // NOTE: Start of Adjustment and Exporting
    let start_t = std::time::Instant::now();
    if config.verbose {
        info!("Starting MIN_MAX calculations and adjustments.");
    }

    adjust_min_max(&xlow, &xhigh, &feats);

    let xlow: HashMap<String, f64> = xlow.into_iter().collect();
    let xhigh: HashMap<String, f64> = xhigh.into_iter().collect();

    // drop and find problematic features
    let mut problematic_features: HashSet<String> = HashSet::new();
    for feat in &feats {
        let low = *xlow.get(feat).unwrap();
        let high = *xlow.get(feat).unwrap();
        if low.is_nan() && high.is_nan() {
            problematic_features.insert(feat.clone());
        }
    }

    let mut min_max_vec: Vec<(String, MinMax)> = Vec::new();
    for feat in &feats {
        if !problematic_features.contains(feat) {
            let low = xlow.get(feat).unwrap();
            let high = xhigh.get(feat).unwrap();

            min_max_vec.push((
                feat.clone(),
                MinMax {
                    xlow: *low,
                    xhigh: *high,
                },
            ));
        }
    }

    //remove problematic_features
    feats.retain(|feat| !problematic_features.contains(feat));

    let problematic_features_vec = if !problematic_features.is_empty() {
        let prob_feats_list: Vec<String> = problematic_features.into_iter().collect();
        Some(prob_feats_list)
    } else {
        None
    };

    // NOTE: End of start time
    if config.verbose {
        info!("End of processing. Time: {:?}", start_t.elapsed());
    }

    if config.verbose {
        if let Some(ref prob_vec) = problematic_features_vec {
            info!("len bad features: {}", prob_vec.len());
        }
        info!("len of good feats: {}", feats.len())
    }

    let res = MinMaxPlateResult {
        min_max: min_max_vec,
        features: feats,
        problemativ_features: problematic_features_vec,
    };

    return Ok(res);
}

/// Adjusts the min max values
fn adjust_min_max(xlow: &DashMap<String, f64>, xhigh: &DashMap<String, f64>, feats: &[String]) {
    feats.par_iter().for_each(|feat| {
        let low = xlow.get(feat).map(|v| *v).unwrap_or(f64::NAN);
        let high = xhigh.get(feat).map(|v| *v).unwrap_or(f64::NAN);
        if low.is_nan() || high.is_nan() {
            return;
        } else if low == high {
            let adjust_high = if low != 0.0 {
                low + low + 0.5
            } else {
                low + 1.0
            };

            xhigh.insert(feat.clone(), adjust_high);
        }
    });
}
