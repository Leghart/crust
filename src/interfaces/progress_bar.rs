use indicatif;

/// Wrapper to indicatif::ProgressBar.
/// TODO?: add customization
pub struct ProgressBar {
    pb: indicatif::ProgressBar,
}

impl ProgressBar {
    /// Creates a new progress bar with configured styles.
    pub fn new(size: u64) -> Self {
        let pb = indicatif::ProgressBar::new(size);
        pb.set_style(
            indicatif::ProgressStyle::with_template(
                "[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )
            .unwrap()
            .with_key(
                "eta",
                |state: &indicatif::ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                },
            )
            .progress_chars("=>-"),
        );
        Self { pb }
    }

    /// Increments a value of progress bar with a passed value.
    pub fn inc(&self, value: usize) {
        self.pb.inc(value as u64)
    }

    /// Finishes the progress bar and leaves the bar filled.
    pub fn finish(&self) {
        self.pb.finish();
    }
}
