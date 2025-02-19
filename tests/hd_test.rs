use histdiff_core::{calculate_scores, HistDiffRes, UserConfig};

#[test]
fn test_hd() {
    // global thread pool
    // rayon::ThreadPoolBuilder::new()
    //     .num_threads(15)
    //     .build_global()
    //     .unwrap();

    let path = "/Users/dterciano/Desktop/LokeyLabFiles/TargetMol/cellData_examples/10uM/d0a5160e-9544-11ee-ac86-02420a000112_cellbycell.tsv";
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

    let veh_cntrl: Vec<String> = vec!["A1".into(), "P24".into()];

    let config = UserConfig::new(path, id, useless, true, None, None, veh_cntrl, None);

    let mut res = calculate_scores(&config).expect("Unable to get results");
    let df = res.dataframe_scores.clone().unwrap();
    res.to_csv("/Users/dterciano/Desktop/test.csv");

    println!("{:?} {:?} {:?}", df, df.shape(), res.raw_scores.len());
}
