use approx::assert_relative_eq;
use histdiff_core::{hist_square_diff, hist_square_diff_deprecated};
use ndarray::{arr1, arr2};

#[test]
fn test_hd_sq_dff() {
    let exp = vec![
        vec![2.0, 3.0, 4.0],
        vec![1.0, 5.0, 7.0],
        vec![3.0, 6.0, 9.0],
    ];
    let ctrl = vec![2.0, 4.0, 6.0];
    let factor = 1.0;

    // Compute expected using ndarray
    let exp_nd = arr2(&[[2.0, 3.0, 4.0], [1.0, 5.0, 7.0], [3.0, 6.0, 9.0]]).reversed_axes();
    let ctrl_nd = arr1(&[2.0, 4.0, 6.0]);
    let expected = hist_square_diff_deprecated(&exp_nd, &ctrl_nd, factor).unwrap();

    // Compute actual result using Vec version
    let result = hist_square_diff(&exp, &ctrl, factor).unwrap();

    // println!("{:?} {:?}", expected, result);
    // Compare results
    for (r, e) in result.iter().zip(expected.iter()) {
        assert_relative_eq!(r, e, epsilon = 1e-6);
    }
}

#[test]
fn test_hd_sq_dff_20bins() {
    // 3 rows, each row has 20 columns:
    // Row 1: 1..=20
    // Row 2: 21..=40
    // Row 3: 41..=60
    let exp = vec![
        (1..=20).map(|x| x as f64).collect::<Vec<f64>>(),
        (21..=40).map(|x| x as f64).collect::<Vec<f64>>(),
        (41..=60).map(|x| x as f64).collect::<Vec<f64>>(),
    ];

    // Control vector has length 20 (matching the number of columns).
    let ctrl = (1..=20).map(|x| x as f64).collect::<Vec<f64>>();

    // Some factor
    let factor = 1.0;

    // Build the same data in ndarray for the "deprecated" or reference function.
    use ndarray::{arr1, arr2};
    let exp_nd = arr2(&[
        [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0, 18.0, 19.0, 20.0,
        ],
        [
            21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0, 32.0, 33.0, 34.0,
            35.0, 36.0, 37.0, 38.0, 39.0, 40.0,
        ],
        [
            41.0, 42.0, 43.0, 44.0, 45.0, 46.0, 47.0, 48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0,
            55.0, 56.0, 57.0, 58.0, 59.0, 60.0,
        ],
    ])
    .reversed_axes()
    .to_owned();
    let ctrl_nd = arr1(&[
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        17.0, 18.0, 19.0, 20.0,
    ]);

    // "Deprecated" reference function for comparison
    let expected =
        hist_square_diff_deprecated(&exp_nd, &ctrl_nd, factor).expect("Reference function failed");
    println!("EXPECTED: {:?}", expected);

    // Now compute actual result with your Vec-based function
    // (which should require ctrl.len() == num_cols == 20)
    let result = hist_square_diff(&exp, &ctrl, factor).expect("hist_square_diff failed");

    // Compare element by element
    for (r, e) in result.iter().zip(expected.iter()) {
        assert_relative_eq!(r, e, epsilon = 1e-6);
    }
}

#[test]
fn test_hd_sq_dff_factor() {
    let exp = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 4.0, 6.0],
        vec![3.0, 6.0, 9.0],
    ];
    let ctrl = vec![1.0, 2.0, 3.0];
    let factor = 2.0;
    // Compute expected using ndarray
    let exp_nd = arr2(&[[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [3.0, 6.0, 9.0]]).reversed_axes();
    let ctrl_nd = arr1(&[1.0, 2.0, 3.0]);
    let expected = hist_square_diff_deprecated(&exp_nd, &ctrl_nd, factor).unwrap();

    // Compute actual result using Vec version
    let result = hist_square_diff(&exp, &ctrl, factor).unwrap();

    // Compare results
    for (r, e) in result.iter().zip(expected.iter()) {
        assert_relative_eq!(r, e, epsilon = 1e-6);
    }
}

// #[test]
// fn test_hd_sq_dff_mismatched_shapes() {
//     let exp = vec![vec![1.0, 2.0, 3.0]];
//     let ctrl = vec![1.0, 2.0]; // Incorrect shape
//     let factor = 1.0;
//
//     let result = hist_square_diff(&exp, &ctrl, factor);
//     assert!(result.is_err());
// }
