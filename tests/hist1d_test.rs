use histdiff_core::Hist1D;

#[test]
fn test_hist1d_creation() {
    let nbins = 20;
    let xlow = 0.0;
    let xhigh = 1.0;
    let hist = Hist1D::new(nbins, xlow, xhigh);

    assert_eq!(hist.nbins, nbins);
    assert_eq!(hist.xlow, xlow);
    assert_eq!(hist.xhigh, xhigh);
    assert_eq!(hist.bin_width, (xhigh - xlow) / nbins as f64);
    assert_eq!(hist.bins.len(), nbins);
    assert_eq!(hist.counts.len(), nbins);

    // Check that bin centers are correctly calculated
    let expected_bins: Vec<f64> = (0..nbins)
        .map(|i| xlow + (i as f64 + 0.5) * hist.bin_width)
        .collect();
    assert_eq!(hist.bins, expected_bins);

    // Check that counts are initialized to zero
    assert!(hist.counts.iter().all(|&count| count == 0.0));
}

#[test]
fn test_hist1d_fill() {
    let mut hist = Hist1D::new(5, 0.0, 1.0);
    let data = vec![0.1, 0.2, 0.3, 0.4, 0.5];

    hist.fill(&data);

    // Expected bin counts after filling
    let expected_counts = vec![1.0, 2.0, 2.0, 0.0, 0.0];
    println!("expected: {:?}\noutput: {:?}", expected_counts, hist.counts);
    assert_eq!(hist.counts, expected_counts);
}

#[test]
fn test_hist1d_fill_various_bins() {
    let mut hist = Hist1D::new(5, 0.0, 1.0);
    let data = vec![
        0.0,  // Lower bound
        0.2,  // Bin 1
        0.4,  // Bin 2
        0.6,  // Bin 3
        0.8,  // Bin 4
        1.0,  // Upper bound (should be in last bin)
        -0.1, // Below range
        1.1,  // Above range
    ];

    hist.fill(&data);

    // Expected counts: [2, 1, 1, 1, 2]
    // - 0.0 and -0.1 are in bin 0 (but -0.1 is ignored)
    // - 0.2 in bin 1
    // - 0.4 in bin 2
    // - 0.6 in bin 3
    // - 0.8 and 1.0 in bin 4 (1.0 is included)
    // - 1.1 is ignored
    let expected_counts = vec![1.0, 1.0, 2.0, 0.0, 2.0];
    assert_eq!(hist.counts, expected_counts);
}
