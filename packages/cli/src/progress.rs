use std::borrow::Cow;

use indicatif::{ProgressBar, ProgressStyle};

pub fn create_spinner(msg: Option<impl Into<Cow<'static, str>>>) -> ProgressBar {
    let mut progress = ProgressBar::new_spinner().with_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg:.bright.blue}")
            .unwrap()
            .tick_chars("⠁⠉⠙⠚⠒⠂⠂⠒⠲⠴⠤⠄⠄⠤⠠⠠⠤⠦⠖⠒⠐⠐⠒⠓⠋⠉✓"),
    );

    if let Some(msg) = msg {
        progress = progress.with_message(msg);
    }

    progress
}

pub fn create_bar(len: usize, msg: impl Into<Cow<'static, str>>) -> ProgressBar {
    ProgressBar::new(len as u64).with_message(msg).with_style(
        ProgressStyle::with_template(
            "{human_pos:.green} / {human_len:.red} {msg:15} {wide_bar:.cyan/blue}",
        )
        .unwrap()
        .progress_chars("##-"),
    )
}
