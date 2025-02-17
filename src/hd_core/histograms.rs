#![allow(unused_imports, dead_code)]
use core::f64;
// use dashmap::DashMap;
use ndarray::{Array1, Array2, Axis};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::usize;

use super::calculations::{exponential_smoothing, normalize};
use super::utils::UserConfig;

#[derive(Clone, Debug)]
pub struct Hist1D {
    pub nbins: usize,
    pub xlow: f64,
    pub xhigh: f64,
    pub bin_width: f64,
    pub bins: Vec<f64>,
    pub counts: Vec<f64>,
}

impl Hist1D {
    pub fn new(nbins: usize, xlow: f64, xhigh: f64) -> Self {
        let bin_width = (xhigh - xlow) / nbins as f64;
        let bins = (0..nbins)
            .map(|i| xlow + (i as f64 + 0.5) * bin_width)
            .collect();
        let counts = vec![0 as f64; nbins];
        return Hist1D {
            nbins,
            xlow,
            xhigh,
            bin_width,
            bins,
            counts,
        };
    }

    pub fn fill(&mut self, data: &[f64]) {
        for &value in data {
            if value >= self.xlow && value < self.xhigh {
                let bin_index = ((value - self.xlow) / self.bin_width) as usize;
                self.counts[bin_index] += 1.0;
            } else if value == self.xhigh {
                // upper bound must be in last bin
                self.counts[self.nbins - 1] += 1.0;
            }

            // any values out of range get ignored
        }
    }

    /// Returns (bins [0], counts [1])
    pub fn data(&self) -> (&[f64], &[f64]) {
        (&self.bins, &self.counts)
    }

    pub fn smooth(&mut self, alpha: f64) {
        self.counts = exponential_smoothing(&self.counts, alpha);
    }

    pub fn normalize(&mut self) {
        self.counts = normalize(&self.counts)
    }

    pub fn add(&mut self, other: &Hist1D) {
        assert_eq!(self.nbins, other.nbins);
        assert_eq!(self.xlow, other.xlow);
        assert_eq!(self.xhigh, self.xhigh);

        for (c1, c2) in self.counts.iter_mut().zip(other.counts.iter()) {
            *c1 += c2;
        }
    }
}

// deprecated below use native structures
pub fn hist_square_diff_deprecated(
    exp: &Array2<f64>,
    ctrl: &Array1<f64>,
    factor: f64,
) -> Result<Array1<f64>, Box<dyn Error>> {
    // shape check
    if exp.shape()[0] != ctrl.len() {
        return Err("Input arrays must have the same shape".into());
    }

    //ctrl mean proxy
    let ctrl_indices = Array1::from_iter(1..=ctrl.shape()[0]).mapv(|x| x as f64);
    let ctrl_mean_proxy = (ctrl * &ctrl_indices).sum();

    // exp mean proxy
    let exp_indices = ctrl_indices.clone().insert_axis(Axis(1));
    let exp_mean_proxy = (exp * &exp_indices).sum_axis(Axis(0));

    //determine negative scores
    let neg_score = exp_mean_proxy.mapv(|e| if ctrl_mean_proxy > e { -1.0 } else { 1.0 });

    // compute differential
    let exp_scaled = exp.mapv(|x| x * factor);
    let ctrl_expanded = ctrl.clone().insert_axis(Axis(1));
    let diff = &ctrl_expanded - &exp_scaled;

    // square the diffs
    let square_diff = diff.mapv(|x| x.powi(2));

    // sum along axis=1
    let sum_diff = square_diff.sum_axis(Axis(0));

    // multiply negative score
    let result = sum_diff * &neg_score;

    return Ok(result);
}

pub fn hist_square_diff(
    exp: &Vec<Vec<f64>>,
    ctrl: &Vec<f64>,
    factor: f64,
) -> Result<Vec<f64>, Box<dyn Error>> {
    let num_rows = exp.len();
    let num_cols = exp.get(0).map(|row| row.len()).unwrap_or(0);

    if num_rows == 0 || num_cols == 0 || ctrl.len() != num_rows {
        return Err("Input vectors  must have matching shapes".into());
    }

    let ctrl_indices: Vec<f64> = (1..=num_rows).map(|x| x as f64).collect();

    let ctrl_mean_proxy: f64 = ctrl.iter().zip(&ctrl_indices).map(|(c, i)| c * i).sum();
    let exp_mean_proxy: Vec<f64> = (0..num_cols)
        .map(|j| {
            exp.iter()
                .zip(&ctrl_indices)
                .map(|(row, i)| row[j] * i)
                .sum()
        })
        .collect();

    // determine when and where to change value signs
    let neg_scores: Vec<f64> = exp_mean_proxy
        .iter()
        .map(|&e| if ctrl_mean_proxy > e { -1.0 } else { 1.0 })
        .collect();

    // square the differences
    let mut sum_diff: Vec<f64> = vec![0.0; num_cols];
    for i in 0..num_rows {
        for j in 0..num_cols {
            let diff = ctrl[i] - exp[i][j] * factor;
            sum_diff[j] += diff * diff;
        }
    }

    // asjust sign values based on the negative scores
    let result: Vec<f64> = sum_diff
        .iter()
        .zip(&neg_scores)
        .map(|(sd, ns)| sd * ns)
        .collect();

    return Ok(result);
}

// pub fn get_min_max_plate<P: AsRef<Path>>(
//     file_path: P,
//     id_cols: &[String],
//     verbose: bool,
//     prob_out: Option<&str>,
// ) -> Result<MinMaxPlateResult, Box<dyn Error>> {
//     let file = File::open(file_path)?;
//     let reader = BufReader::new(file);
//
//     let mut csv_reader = csv::ReaderBuilder::new()
//         .delimiter(b'\t')
//         .has_headers(true)
//         .from_reader(reader);
//
//     let headers = csv_reader.headers()?.clone();
//     let headers_vec = headers.iter().map(|s| s.to_string()).collect::<Vec<_>>();
//
//     let id_col_indices: Vec<usize> = id_cols
//         .iter()
//         .map(|col| headers.iter().position(|h| h == col))
//         .collect::<Option<Vec<_>>>()
//         .ok_or("ID column not foind in headers")?;
//
//     let feature_indices: Vec<usize> = (0..headers.len())
//         .filter(|i| !id_col_indices.contains(i))
//         .collect();
//
//     // let mut xlow: HashMap<String, f64> = HashMap::new();
//     // let mut xhigh: HashMap<String, f64> = HashMap::new();
//     let xlow: DashMap<String, f64> = DashMap::new();
//     let xhigh: DashMap<String, f64> = DashMap::new();
//     let mut feats: Vec<String> = Vec::new();
//
//     feats = feature_indices
//         .iter()
//         .map(|&x| headers[x].to_string())
//         .collect();
//
//     // initialize xlow and xhigh
//     for feat in &feats {
//         xlow.insert(feat.clone(), f64::NAN);
//         xhigh.insert(feat.clone(), f64::NAN);
//     }
//
//     for result in csv_reader.records() {
//         let record = result?;
//
//         // println!("{:?}", record);
//         feature_indices.par_iter().for_each(|&i| {
//             // for &i in &feature_indices {
//             let feat = &headers[i];
//             let field = &record[i];
//             if let Ok(value) = field.parse::<f64>() {
//                 if value.is_finite() {
//                     //xlow
//                     xlow.entry(feat.to_string()).and_modify(|e| {
//                         if e.is_nan() {
//                             *e = value;
//                         } else {
//                             *e = e.min(value);
//                         }
//                     });
//
//                     //xhigh
//                     xhigh.entry(feat.to_string()).and_modify(|e| {
//                         if e.is_nan() {
//                             *e = value;
//                         } else {
//                             *e = e.max(value);
//                         }
//                     });
//                 };
//             }
//
//             // skip nans
//         });
//
//         // skip other gibberish
//     }
//
//     let xlow: HashMap<String, f64> = xlow.into_iter().collect();
//     let mut xhigh: HashMap<String, f64> = xhigh.into_iter().collect();
//
//     // adjust the xhigh when xhigh == xlow
//     for feat in &feats {
//         let low = *xlow.get(feat).unwrap_or(&f64::NAN);
//         let high = *xhigh.get(feat).unwrap_or(&f64::NAN);
//         if low.is_nan() || high.is_nan() {
//             continue;
//             // problematic_features.insert(feat.clone());
//         } else if low == high {
//             let adjusted_high = if low != 0.0 {
//                 low + low * 0.5
//             } else {
//                 low + 1.0
//             };
//
//             xhigh.insert(feat.clone(), adjusted_high);
//         }
//     }
//
//     // get problematic features
//     let mut problematic_features: HashSet<String> = HashSet::new();
//     for feat in &feats {
//         let low = *xlow.get(feat).unwrap();
//         let high = *xhigh.get(feat).unwrap();
//         if low.is_nan() && high.is_nan() {
//             problematic_features.insert(feat.clone());
//         }
//     }
//
//     let mut min_max_vec: Vec<(String, MinMax)> = Vec::new();
//
//     for feat in &feats {
//         if problematic_features.contains(feat) {
//             continue;
//         }
//         let low = xlow.get(feat).unwrap();
//         let high = xhigh.get(feat).unwrap();
//         min_max_vec.push((
//             feat.clone(),
//             MinMax {
//                 xlow: *low,
//                 xhigh: *high,
//             },
//         ))
//     }
//
//     //remove problematic features
//     feats.retain(|feat| !problematic_features.contains(feat));
//
//     // outputting problemativ features
//     let problematic_features_vec = if !problematic_features.is_empty() {
//         let problematic_features_list = problematic_features.into_iter().collect::<Vec<_>>();
//         if verbose {
//             eprintln!(
//                 "MinMax: No values have been found in the following features: {}",
//                 problematic_features_list.join(" | ")
//             );
//         }
//         if let Some(prob_path_out) = prob_out {
//             //let's write this out to a file'
//             use std::fs::File;
//             use std::io::Write;
//
//             let mut file = File::create(format!("{}_problematicFeats.csv", prob_path_out))?;
//             for feat in &problematic_features_list {
//                 writeln!(file, "{},noValues", feat)?;
//             }
//         }
//         Some(problematic_features_list)
//     } else {
//         None
//     };
//
//     if verbose {
//         if let Some(ref prob_vec) = problematic_features_vec {
//             eprintln!("len of bad feats: {}", prob_vec.len())
//         }
//         eprintln!("length of good feats: {}", feats.len());
//     }
//
//     return Ok(MinMaxPlateResult {
//         min_max: min_max_vec,
//         features: feats,
//         problemativ_features: problematic_features_vec,
//     });
// }
