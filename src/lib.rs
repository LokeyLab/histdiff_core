#![allow(unused_parens, unused_imports)]
mod hd;
mod hd_core;
pub use hd::{calculate_scores, HistDiffRes};
pub use hd_core::calculations::get_min_max_plate;
pub use hd_core::histograms::{hist_square_diff, hist_square_diff_deprecated, Hist1D};
pub use hd_core::utils::UserConfig;
