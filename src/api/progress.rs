//! Progress bar support for track fetching operations (requires `progress` feature).

use indicatif::{ProgressBar, ProgressStyle};

/// Create a progress callback that renders a terminal progress bar.
///
/// The returned closure accepts `(fetched, total)` and updates a [`ProgressBar`]
/// accordingly. Pass the result to any builder's `on_progress()` method, or use
/// the convenience `with_progress()` method instead.
///
/// # Example
/// ```no_run
/// # use lastfm_client::{LastFmClient, prelude::*};
/// # async fn example(client: LastFmClient) -> Result<(), Box<dyn std::error::Error>> {
/// let tracks = client
///     .recent_tracks("username")
///     .with_progress()
///     .fetch()
///     .await?;
/// # Ok(())
/// # }
/// ```
pub(in crate::api) fn make_progress_callback() -> impl Fn(u32, u32) + Send + Sync + 'static {
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)",
        )
        .unwrap_or_else(|_| ProgressStyle::default_bar()),
    );

    move |fetched, total| {
        if fetched == 0 {
            pb.set_length(u64::from(total));
        }
        pb.set_position(u64::from(fetched));
        if fetched >= total {
            pb.finish_with_message("done");
        }
    }
}
