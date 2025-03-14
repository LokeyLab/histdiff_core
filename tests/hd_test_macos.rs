use std::sync::Once;

use env_logger;
use histdiff_core::{calculate_scores, UserConfig};
use log;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Trace)
            .is_test(true)
            .try_init()
            .ok();
    });
}

#[test]
fn test_hd() {
    init_logger();
    log::info!("Beginning MACOS test.");
    // global thread pool
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(15)
    //     .build_global()
    //     .unwrap();

    // let path = "/home/derfelt/LokeyLabFiles/TargetMol/cellData_examples/cellbycellTSV/bfbe6900-005a-11ee-9416-02420a00012a_cellbycell.tsv";
    let path = "/Users/dterciano/Desktop/LokeyLabFiles/TargetMol/cellData_examples/10uM/2bffddea-8a23-11ee-ac86-02420a000112_cellbycell copy 2.tsv";
    //let path = "/Users/dterciano/Desktop/LokeyLabFiles/TargetMol/cellData_examples/10uM/d0a5160e-9544-11ee-ac86-02420a000112_cellbycell.tsv";

    // let path = "/home/derfelt/LokeyLabFiles/TargetMol/cellData_examples/10uM/d0a5160e-9544-11ee-ac86-02420a000112_cellbycell.tsv";
    let id: Vec<String> = vec!["WellName".into()];
    let useless: Option<Vec<String>> = {
        let vec = vec![
            "ScreenName",
            "ScreenID",
            "PlateName",
            "PlateID",
            "MeasurementDate",
            "MeasurementID",
            "Row",
            "Column",
            "Timepoint",
        ]
        .iter()
        .map(|x| x.to_string())
        .collect();
        Some(vec)
    };

    // let useless = None;

    // WARNING: HD does not like leading 0 in name i.e. F8 = good F08 = not good

    // let veh_cntrl: Vec<String> = vec!["A1".into(), "P24".into()];
    let veh_cntrl: Vec<String> = vec!["F8".into()];

    let config = UserConfig::new(path, id, useless, true, None, None, veh_cntrl, None);

    let mut res = calculate_scores(&config).expect("Unable to get results");
    let df = res.dataframe_scores.clone().unwrap();
    // res.to_csv("/Users/dterciano/Desktop/test.csv");

    println!("{:?} {:?} {:?}", df, df.shape(), res.raw_scores.len());
}
