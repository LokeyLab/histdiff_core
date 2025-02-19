#![allow(unused_parens, unused_imports)]
mod hd;
mod hd_core;
pub use hd::{calculate_scores, HistDiffRes};
pub use hd_core::calculations::get_min_max_plate;
pub use hd_core::histograms::{hist_square_diff, hist_square_diff_deprecated, Hist1D};
pub use hd_core::utils::UserConfig;
// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
