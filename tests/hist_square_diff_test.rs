use approx::assert_relative_eq;
use histdiff_core::{hist_square_diff, hist_square_diff_deprecated};
use ndarray::{arr1, arr2, Array1, Array2};

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
    let exp_nd = arr2(&[[2.0, 3.0, 4.0], [1.0, 5.0, 7.0], [3.0, 6.0, 9.0]]);
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
fn test_hd_sq_dff_factor() {
    let exp = vec![
        vec![1.0, 2.0, 3.0],
        vec![2.0, 4.0, 6.0],
        vec![3.0, 6.0, 9.0],
    ];
    let ctrl = vec![1.0, 2.0, 3.0];
    let factor = 2.0;
    // Compute expected using ndarray
    let exp_nd = arr2(&[[1.0, 2.0, 3.0], [2.0, 4.0, 6.0], [3.0, 6.0, 9.0]]);
    let ctrl_nd = arr1(&[1.0, 2.0, 3.0]);
    let expected = hist_square_diff_deprecated(&exp_nd, &ctrl_nd, factor).unwrap();

    // Compute actual result using Vec version
    let result = hist_square_diff(&exp, &ctrl, factor).unwrap();

    // Compare results
    for (r, e) in result.iter().zip(expected.iter()) {
        assert_relative_eq!(r, e, epsilon = 1e-6);
    }
}

#[test]
fn test_hd_sq_dff_mismatched_shapes() {
    let exp = vec![vec![1.0, 2.0, 3.0]];
    let ctrl = vec![1.0, 2.0]; // Incorrect shape
    let factor = 1.0;

    let result = hist_square_diff(&exp, &ctrl, factor);
    assert!(result.is_err());
}
