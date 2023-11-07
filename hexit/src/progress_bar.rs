use indicatif::{ProgressBar, ProgressStyle};

/// Returns a progress bar object for the given parquet file and name.
pub fn make_progress_bar(prefix: String, total_size: u64) -> ProgressBar {
    #[allow(clippy::cast_sign_loss)]
    let pb = ProgressBar::new(total_size);
    pb.set_prefix(prefix);
    pb.set_style(
        ProgressStyle::with_template("{prefix}...\n[{wide_bar:.cyan/blue}]")
            .expect("incorrect progress bar format string")
            .progress_chars("#>-"),
    );
    pb
}
