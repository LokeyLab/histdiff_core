use histdiff_core::{get_min_max_plate, UserConfig};

#[test]
fn test_minmax() {
    let path = "/Users/dterciano/Desktop/LokeyLabFiles/TargetMol/cellData_examples/10uM/d0a5160e-9544-11ee-ac86-02420a000112_cellbycell.tsv";

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

    let veh_cntrl: Vec<String> = vec!["A1".into(), "P24".into()];

    let config = UserConfig::new(path, id, useless, true, None, None, veh_cntrl, None);

    let res = get_min_max_plate(&config).unwrap();

    println!("num feats: {:?}", res.features.len());
}
