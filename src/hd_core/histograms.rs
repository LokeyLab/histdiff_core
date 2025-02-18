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
    let exp = transpose_2d_vec(exp);

    let num_rows = exp.len();
    let num_cols = exp.get(0).map(|row| row.len()).unwrap_or(0);

    // if num_rows == 0 || num_cols == 0 || ctrl.len() != num_rows {
    //     // println!("{:?} {:?} {:?}", num_rows, num_cols, ctrl.len());
    //     return Err("Input vectors  must have matching shapes".into());
    // }

    let ctrl_indices: Vec<f64> = (1..=num_rows).map(|x| x as f64).collect();

    let ctrl_mean_proxy: f64 = ctrl.iter().zip(&ctrl_indices).map(|(c, i)| c * i).sum();
    let exp_mean_proxy: Vec<f64> = (0..num_cols)
        .map(|i| {
            let col = exp
                .iter()
                .zip(&ctrl_indices)
                .map(|(j, cntrl_idx)| j[i] * cntrl_idx)
                .collect::<Vec<f64>>();
            col.iter().sum()
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

fn transpose_2d_vec(matrix: &Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let n_rows = matrix.len();
    if n_rows == 0 {
        return Vec::new();
    }
    let n_cols = matrix[0].len();

    let mut transpose = vec![vec![0.0; n_rows]; n_cols];

    for i in 0..n_rows {
        for j in 0..n_cols {
            transpose[j][i] = matrix[i][j];
        }
    }

    return transpose;
}
