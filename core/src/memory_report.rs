//! Live-memory accounting for loaded SWFs.
//!
//! Ruffle keeps one [`MovieLibrary`] per `SwfMovie` (see
//! [`crate::library::Library`]). A movie's decoded characters stay resident for
//! exactly as long as its library is listed there. When content is unloaded but
//! its memory is never returned, the movie is still in this report.
//!
//! This module walks that state so a leak can be measured instead of guessed
//! at: run it once per interval while switching zones, and a movie that should
//! be gone shows up as a row that never disappears.

use std::fmt::Write as _;

use crate::context::UpdateContext;

/// What a single still-resident movie is keeping alive.
#[derive(Debug, Clone)]
pub struct MovieMemory {
    pub url: String,
    /// Size of the movie's decompressed SWF data.
    pub swf_bytes: usize,
    pub characters: usize,
    pub bitmaps: usize,
    /// Bitmaps that have been uploaded to the render backend, and so are also
    /// holding GPU memory. Ruffle never releases these handles on its own.
    pub uploaded_bitmaps: usize,
    /// Bytes of still-compressed bitmap data held by the library.
    pub bitmap_source_bytes: usize,
    /// Bytes those bitmaps occupy once decoded to RGBA, whether or not they
    /// have been uploaded yet. This is the dominant cost for AQW-style content.
    pub bitmap_decoded_bytes: usize,
    pub sounds: usize,
    pub fonts: usize,
    /// Whether the movie's library still points at an AVM2 `ApplicationDomain`.
    pub has_domain: bool,
    /// Whether the display object this movie was loaded into is still
    /// reachable. Content that is gone but still listed here has not been
    /// swept yet.
    pub content_alive: bool,
}

/// A whole-player snapshot, cheap enough to take every frame.
#[derive(Debug, Clone, Default)]
pub struct MemoryReport {
    pub movies: Vec<MovieMemory>,
    pub swf_bytes: usize,
    pub bitmap_source_bytes: usize,
    pub bitmap_decoded_bytes: usize,
    pub characters: usize,
    /// Loaders still registered with the `LoadManager`. Should return to zero
    /// once every load settles; a number that only grows is itself a leak.
    pub pending_loaders: usize,
    /// `registerClassAlias` entries. These are strong roots with no eviction,
    /// so each one pins a class, its translation unit and its movie forever.
    pub class_aliases: usize,
    pub gc_allocation: usize,
    pub gc_objects: usize,
}

impl MemoryReport {
    pub fn capture(context: &mut UpdateContext<'_>) -> Self {
        let mut report = MemoryReport {
            pending_loaders: context.load_manager.len(),
            class_aliases: context.avm2.class_alias_count(),
            gc_allocation: context.gc_context.metrics().total_allocation(),
            gc_objects: context.gc_context.metrics().total_gc_count(),
            ..Default::default()
        };

        let movies: Vec<_> = context.library.known_movies().collect();
        for movie in movies {
            let Some(library) = context.library.library_for_movie(movie.clone()) else {
                continue;
            };
            let usage = library.memory_usage();
            let content_alive = library.has_live_content(context.gc());

            report.swf_bytes += movie.uncompressed_len().max(0) as usize;
            report.bitmap_source_bytes += usage.bitmap_source_bytes;
            report.bitmap_decoded_bytes += usage.bitmap_decoded_bytes;
            report.characters += usage.characters;

            report.movies.push(MovieMemory {
                url: movie.url().to_owned(),
                swf_bytes: movie.uncompressed_len().max(0) as usize,
                characters: usage.characters,
                bitmaps: usage.bitmaps,
                uploaded_bitmaps: usage.uploaded_bitmaps,
                bitmap_source_bytes: usage.bitmap_source_bytes,
                bitmap_decoded_bytes: usage.bitmap_decoded_bytes,
                sounds: usage.sounds,
                fonts: usage.fonts,
                has_domain: usage.has_domain,
                content_alive,
            });
        }

        report
            .movies
            .sort_by(|a, b| b.bitmap_decoded_bytes.cmp(&a.bitmap_decoded_bytes));
        report
    }

    /// One CSV row, for logging a time series across a zone-change run.
    pub fn csv_header() -> &'static str {
        "elapsed_s,movies,characters,swf_bytes,bitmap_source_bytes,bitmap_decoded_bytes,pending_loaders,class_aliases,gc_allocation,gc_objects"
    }

    pub fn to_csv_row(&self, elapsed_s: f64) -> String {
        format!(
            "{:.1},{},{},{},{},{},{},{},{},{}",
            elapsed_s,
            self.movies.len(),
            self.characters,
            self.swf_bytes,
            self.bitmap_source_bytes,
            self.bitmap_decoded_bytes,
            self.pending_loaders,
            self.class_aliases,
            self.gc_allocation,
            self.gc_objects,
        )
    }

    /// The heaviest movies still resident, for a human reading the log.
    pub fn top_movies(&self, count: usize) -> String {
        let mut out = String::new();
        for movie in self.movies.iter().take(count) {
            let _ = write!(
                out,
                "\n    {}  {:>9} KiB decoded  {:>5} chars  {}",
                if movie.content_alive { "live" } else { "dead" },
                movie.bitmap_decoded_bytes / 1024,
                movie.characters,
                movie.url,
            );
        }
        out
    }
}

/// Per-library totals, gathered inside `library.rs` where the fields live.
#[derive(Debug, Clone, Default)]
pub struct LibraryMemoryUsage {
    pub characters: usize,
    pub bitmaps: usize,
    pub uploaded_bitmaps: usize,
    pub bitmap_source_bytes: usize,
    pub bitmap_decoded_bytes: usize,
    pub sounds: usize,
    pub fonts: usize,
    pub has_domain: bool,
}
