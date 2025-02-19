use core::f64;
use dashmap::DashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::BufReader,
    usize,
};

use crate::{
    get_min_max_plate, hd_core::utils::clean_well_names, hist_square_diff, Hist1D, UserConfig,
};

use super::HistDiffRes;

#[allow(dead_code)]
pub fn calculate_scores(config: &UserConfig) -> Result<HistDiffRes, Box<dyn Error>> {
    let plate_def = &config.plate_def;

    let min_max = get_min_max_plate(config)?;
    let min_max_vec = min_max.min_max;
    let features = min_max.features;

    let file = File::open(&config.path)?;
    let reader = BufReader::new(file);
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .flexible(true)
        .from_reader(reader);

    let headers = csv_reader.headers()?.clone();
    let headers_len = headers.len();
    let headers_vec: Vec<String> = headers.into_iter().map(|x| x.to_string()).collect();

    let id_col_idx: &Vec<usize> = &config
        .id_cols
        .iter()
        .map(|col| headers.iter().position(|h| h == col))
        .collect::<Option<Vec<_>>>()
        .ok_or("ID Column not found in headers")?;

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

    let histograms: DashMap<String, DashMap<String, Hist1D>> = DashMap::new();

    let start_t = std::time::Instant::now();

    for record in csv_reader.records() {
        let rec = record?;
        if rec.len() != headers_len {
            continue;
        }

        // in case there are multiple id columns merge them else use use the string
        let curr_well = &config
            .id_cols
            .iter()
            .map(|i| {
                rec.get(headers_vec.iter().position(|h| h == i).unwrap())
                    .unwrap()
            })
            .collect::<Vec<&str>>()
            .join("_");

        if !plate_def.contains(&curr_well) {
            continue;
        }

        let feature_values: Vec<(&str, f64)> = feature_idx
            .par_iter()
            .map(|&i| {
                let feat_name = headers_vec[i].as_str();
                let value = rec.get(i).unwrap().parse::<f64>().unwrap_or(f64::NAN);
                return (feat_name, value);
            })
            .collect();

        histograms.entry(curr_well.clone()).or_insert_with(|| {
            let feature_map: DashMap<String, Hist1D> = DashMap::new();
            let _ = &features.par_iter().for_each(|feat| {
                let (_, vals) = min_max_vec.iter().find(|(f, _)| f == feat).unwrap();
                let new_hist = Hist1D::new(config.nbins.clone(), vals.xlow, vals.xhigh);
                feature_map.insert(feat.clone(), new_hist);
            });

            return feature_map;
        });

        let well_histogram = histograms.get_mut(curr_well).unwrap();
        feature_values.par_iter().for_each(|(feat, val)| {
            if let Some(mut hist) = well_histogram.get_mut(*feat) {
                hist.fill(&[*val]);
            }
        });
    }

    let histograms: HashMap<String, HashMap<String, Hist1D>> = histograms
        .into_iter()
        .map(|(well_id, well_hist)| {
            let hists = well_hist.into_iter().collect();
            (well_id, hists)
        })
        .collect();

    if config.verbose {
        println!("Time to read file: {:?}", start_t.elapsed());
    }

    // NOTE: HistDiff calculation process below
    let start_t = std::time::Instant::now();

    let mut hd_scores: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for group in &config.block_def {
        // clean the well names
        let select_wells: HashSet<String> = clean_well_names(group).into_iter().collect();

        let mut hd_group: HashMap<String, HashMap<String, Hist1D>> = histograms
            .iter()
            .filter(|(well, _)| select_wells.contains(*well))
            .map(|(well, well_hist)| (well.clone(), well_hist.clone()))
            .collect();

        let mut cntr_hists: HashMap<String, Hist1D> = HashMap::new();

        for feat in &features {
            let mut sum_hist: Option<Hist1D> = None;

            for well in &config.vehicle_cntrls {
                if let Some(hist) = hd_group.get(well).and_then(|hists| hists.get(feat)) {
                    if let Some(ref mut sums) = sum_hist {
                        sums.add(hist);
                    } else {
                        sum_hist = Some(hist.clone());
                    }
                }
            }

            if let Some(sums) = sum_hist {
                cntr_hists.insert(feat.clone(), sums);
            }
        }

        if config.verbose {
            println!("Adding control sum into HD group");
        }
        hd_group.insert("CNTRL".to_string(), cntr_hists);

        if config.verbose {
            println!("Smoothing and normalizing histograms");
        }
        for histograms in hd_group.values_mut() {
            for feats in histograms.values_mut() {
                feats.smooth(0.25);
                feats.normalize();
            }
        }

        if config.verbose {
            println!("Calculating scores!");
        }

        let per_feature_score: Vec<HashMap<String, HashMap<String, f64>>> = features
            .par_iter()
            .map(|feat| {
                let mut local_scores: HashMap<String, HashMap<String, f64>> = HashMap::new();

                // lets get the exp wells
                let mut exp_wells: Vec<Vec<f64>> = Vec::new();
                let mut well_ids: Vec<String> = Vec::new();

                for (well_id, histogram) in &hd_group {
                    if well_id == "CNTRL" {
                        continue; // skip control row
                    }

                    if let Some(hist) = histogram.get(feat) {
                        exp_wells.push(hist.data().1.to_vec());
                        well_ids.push(well_id.clone());
                    }
                }

                let cntrl_row = hd_group
                    .get("CNTRL")
                    .and_then(|hist| hist.get(feat))
                    .expect("CNTRL row not found!")
                    .data()
                    .1
                    .to_vec();

                let factor = 1.0;
                let score = hist_square_diff(&exp_wells, &cntrl_row, factor)
                    .expect("Unable to calculate HistDiff score");

                for (well_id, hd_value) in well_ids.iter().zip(score.into_iter()) {
                    local_scores
                        .entry(well_id.clone())
                        .or_insert_with(HashMap::new)
                        .insert(feat.clone(), hd_value);
                }

                return local_scores;
            })
            .collect();

        for local_scores in per_feature_score {
            for (well_id, feat_map) in local_scores {
                hd_scores
                    .entry(well_id)
                    .or_insert_with(HashMap::new)
                    .extend(feat_map);
            }
        }
    }

    if config.verbose {
        println!("Wrapping things up!");
        println!("Finished calculations! Time: {:?}", start_t.elapsed());
    }

    let res = HistDiffRes::new(hd_scores);

    return Ok(res);
}
