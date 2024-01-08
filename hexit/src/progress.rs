use indicatif::{ProgressBar, ProgressStyle};

pub fn bar(header: String, length: u64) -> ProgressBar {
    let pb = ProgressBar::hidden();
    pb.set_prefix(header);
    pb.set_length(length);
    pb.set_style(
        ProgressStyle::with_template("{prefix}...\n[{wide_bar:.cyan/blue}]")
            .expect("incorrect progress bar format string")
            .progress_chars("#>-"),
    );
    pb
}
