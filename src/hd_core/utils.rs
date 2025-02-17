use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct UserConfig {
    pub path: PathBuf,
    pub id_cols: Vec<String>,
    pub useless_cols: Option<Vec<String>>,
    pub verbose: bool,
    pub block_def: Vec<Vec<String>>, // used for defining block analysis
    pub plate_def: Vec<String>,      // defines the avaliable wells used for calculation
    // wells could be all 384
    pub vehicle_cntrls: Vec<String>,
    pub nbins: usize,
}

impl UserConfig {
    pub fn new<P: AsRef<Path>>(
        path: P,
        id_cols: Vec<String>,
        useless_cols: Option<Vec<String>>,
        verbose: bool,
        block_def: Option<Vec<Vec<String>>>,
        plate_def: Option<Vec<String>>,
        vehicle_cntrls: Vec<String>,
        nbins: Option<usize>,
    ) -> Self {
        let plate_def = match plate_def {
            Some(def) => def,
            None => plate_definition(),
        };

        let nbins = match nbins {
            Some(bins) => bins,
            None => 20 as usize,
        };

        let block_def = match block_def {
            Some(mut def) => {
                let mut undefined_blocks: HashSet<String> = HashSet::new();
                for block in &def {
                    let cleaned = clean_well_names(&block);
                    undefined_blocks.extend(cleaned);
                }

                let undefined_blocks: HashSet<String> = plate_def
                    .clone()
                    .into_iter()
                    .filter(|well| !undefined_blocks.contains(well))
                    .collect();

                def.push(undefined_blocks.into_iter().collect());
                def
            }
            None => vec![plate_def.clone()],
        };

        return Self {
            path: path.as_ref().to_path_buf(),
            id_cols,
            useless_cols,
            verbose,
            vehicle_cntrls,
            plate_def,
            nbins,
            block_def,
        };
    }
}

pub fn plate_definition() -> Vec<String> {
    const WELL_384_LETTERS: std::ops::RangeInclusive<u8> = ('A' as u8)..=('P' as u8);
    const WELL_384_NUMBERS: std::ops::RangeInclusive<i32> = (1..=24);

    let mut res: Vec<String> = Vec::new();

    for letter in WELL_384_LETTERS {
        for num in WELL_384_NUMBERS {
            let format_string = format!("{}{}", letter as char, num);
            res.push(format_string);
        }
    }

    return res;
}

/// Makes the wells into a standard format
///
/// Turns "A01" into A1
/// but "P24" is unaffected.
/// This just removes the prefix digit of the well.
pub fn clean_well_names(well_names: &[String]) -> Vec<String> {
    well_names
        .iter()
        .map(|name| {
            if name.len() >= 2 {
                let letter = &name[0..1];
                let number_str = &name[1..];

                match number_str.parse::<u32>() {
                    Ok(number) => format!("{}{}", letter, number),
                    Err(_) => name.clone(),
                }
            } else {
                name.clone()
            }
        })
        .collect()
}
