//! Simple diagnostic reporting for compilers.
//!
//! ```
//! use reporting::{error, note, File, Location, Renderer, Styles};
//!
//! let file = File::new("test.txt", "import stds;");
//! let styles = Styles::styled();
//!
//! print!(
//!     "{}",
//!     Renderer::new(
//!         &styles,
//!         &[
//!             error!("Could not find package `{}`", "stds")
//!                 .location(Location::new(file.clone(), 7)),
//!             note!("Perhaps you meant `std`?")
//!         ]
//!     )
//! );
//! ```

use std::sync::Arc;

use anstyle::{AnsiColor, Color, Reset, Style};
use unicode_width::UnicodeWidthChar;

pub use anstyle;

/// Information about a file.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct File {
    /// The file path, used for identifying the file in diagnostic reports.
    path: String,

    /// The contents of the file.  Used to display snippets in diagnostic reports.
    source: String,
}

impl File {
    /// Creates a new `File` with the given path and source.
    pub fn new(path: impl Into<String>, source: impl Into<String>) -> Arc<Self> {
        Arc::new(Self {
            path: path.into(),
            source: source.into(),
        })
    }

    /// A shorthand method to read a file and create a `File` from its contents.
    pub fn open(path: impl Into<String>) -> Result<Arc<Self>, std::io::Error> {
        let path = path.into();
        let source = std::fs::read_to_string(&path)?;
        Ok(File::new(path, source))
    }

    /// Returns the file's path.
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns the file's source contents.
    #[inline]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the line and column number corresponding to the given offset in the file's source.
    pub fn line_column(&self, offset: usize) -> Option<(usize, usize)> {
        if offset > self.source().len() {
            return None;
        }

        let mut line = 1;
        let mut column = 1;

        for (idx, char) in self.source().char_indices() {
            if idx >= offset {
                break;
            }
            if char == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        Some((line, column))
    }
}

/// A location in a file.
#[derive(Clone, PartialEq, Eq)]
pub struct Location {
    file: Arc<File>,
    offset: usize,
}

impl Location {
    /// Creates a new `Location` with the given file and offset.
    ///
    /// # Panics
    /// Panics if the given offset is out of bounds for the file's source.
    pub fn new(file: Arc<File>, offset: usize) -> Self {
        Self::try_new(file, offset).expect("Offset should not be out of file's bounds")
    }

    /// Attempts to create a `Location` with the given file and offset, returning `None` if the
    /// offset is out of bounds.
    pub fn try_new(file: Arc<File>, offset: usize) -> Option<Self> {
        if offset > file.source().len() {
            None
        } else {
            Some(Location { file, offset })
        }
    }

    /// Returns the file that contains this source location.
    #[inline]
    pub fn file(&self) -> Arc<File> {
        self.file.clone()
    }

    /// Returns the byte offset of this source location within its file.
    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns the line and column number of this source location within its file.
    ///
    /// ```
    /// # use reporting::{Location, File};
    /// let my_file = File::new("test.txt", "hello\nworld");
    ///
    /// let (line1, column1) = Location::new(my_file.clone(), 0).line_column();
    /// let (line2, column2) = Location::new(my_file.clone(), 6).line_column();
    /// assert_eq!((line1, column1), (1, 1));
    /// assert_eq!((line2, column2), (2, 1));
    /// ```
    pub fn line_column(&self) -> (usize, usize) {
        self.file()
            .line_column(self.offset())
            .expect("Offset should not be out of file's bounds")
    }
}

impl std::fmt::Debug for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.file().path(), self.offset())
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (line, column) = self.line_column();
        write!(f, "{}:{}:{}", self.file().path(), line, column)
    }
}

/// The severity of a diagnostic.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Note,
    Warning,
    Error,
    Bug,
}

/// A diagnostic report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub location: Option<Location>,
    pub severity: Severity,
    pub message: String,
}

impl Report {
    /// Creates a new `Report` with the given severity and message.  Defaults with no [Location].
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Self {
            location: None,
            severity,
            message: message.into(),
        }
    }

    /// Creates a [`Severity::Bug`] report with the given message.
    pub fn bug(message: impl Into<String>) -> Self {
        Self::new(Severity::Bug, message)
    }

    /// Creates a [`Severity::Error`] report with the given message.
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(Severity::Error, message)
    }

    /// Creates a [`Severity::Warning`] report with the given message.
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, message)
    }

    /// Creates a [`Severity::Note`] report with the given message.
    pub fn note(message: impl Into<String>) -> Self {
        Self::new(Severity::Note, message)
    }

    /// Adds a location to this diagnostic report.
    pub fn location(mut self, location: impl Into<Option<Location>>) -> Self {
        self.location = location.into();
        self
    }

    /// Creates a renderer for a single diagnostic report.
    pub fn render<'a>(&'a self, styles: &'a Styles) -> Renderer<'a> {
        Renderer::new(styles, std::slice::from_ref(self))
    }
}

/// The styles used to render [Report]s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Styles {
    pub location: Style,
    pub bug: Style,
    pub error: Style,
    pub warning: Style,
    pub note: Style,
    pub colon: Style,
    pub message: Style,
    pub snippet: Style,
    pub cursor: Style,
}

impl Styles {
    /// Creates a set of [Styles] with the given styles.
    pub const fn styled() -> Self {
        Self {
            location: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightWhite))),
            bug: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))),
            error: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightRed))),
            warning: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow))),
            note: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightCyan))),
            colon: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack))),
            message: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightWhite))),
            snippet: Style::new(),
            cursor: Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen))),
        }
    }

    /// Creates a new set of [Styles] which outputs plain text.  Useful for platforms which don't
    /// support ANSI escape codes.
    pub const fn plain() -> Self {
        Self {
            location: Style::new(),
            bug: Style::new(),
            error: Style::new(),
            warning: Style::new(),
            note: Style::new(),
            colon: Style::new(),
            message: Style::new(),
            snippet: Style::new(),
            cursor: Style::new(),
        }
    }
}

/// A renderer for diagnostic reports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Renderer<'a> {
    styles: &'a Styles,
    reports: &'a [Report],
}

impl<'a> Renderer<'a> {
    /// Creates a new [Renderer] with the given styles and reports.
    pub const fn new(styles: &'a Styles, reports: &'a [Report]) -> Self {
        Self { styles, reports }
    }
}

impl<'a> std::fmt::Display for Renderer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for report in self.reports {
            // Print location information, if any.
            let line_column = if let Some(location) = &report.location {
                let (line, column) = location.file().line_column(location.offset()).unwrap();
                write!(
                    f,
                    "{}{}:{}:{}:{} ",
                    &self.styles.location,
                    location.file.path(),
                    line,
                    column,
                    Reset
                )?;
                Some((location, line, column))
            } else {
                None
            };

            write!(f, "{}", Reset)?;

            // Print severity label.
            match report.severity {
                Severity::Bug => write!(f, "{}bug", &self.styles.bug)?,
                Severity::Error => write!(f, "{}error", &self.styles.error)?,
                Severity::Warning => write!(f, "{}warning", &self.styles.warning)?,
                Severity::Note => write!(f, "{}note", &self.styles.note)?,
            }

            // Print colon and message.
            write!(f, "{}", Reset)?;
            write!(f, "{}: ", &self.styles.colon)?;
            write!(f, "{}", Reset)?;
            write!(f, "{}{}", &self.styles.message, &report.message)?;

            // Print snippet, if applicable.
            if let Some((location, line, column)) = line_column {
                let line = location.file.source().lines().nth(line - 1).unwrap();

                writeln!(f, "{}", Reset)?;
                writeln!(f, "{}{}", &self.styles.snippet, &line,)?;

                // Calculate cursor offset
                let mut offset = 0;
                let cursor_width = line
                    .chars()
                    .enumerate()
                    .find(|(idx, char)| {
                        if *idx == column - 1 {
                            true
                        } else {
                            offset += char.width().unwrap_or(1);
                            false
                        }
                    })
                    .unwrap()
                    .1
                    .width()
                    .unwrap_or(1);

                // Write cursor
                write!(f, "{}", Reset)?;
                write!(f, "{: <offset$}", "")?;
                write!(
                    f,
                    "{}{:^<cursor_width$}",
                    &self.styles.cursor,
                    // &severity_style,
                    ""
                )?;
            };
            writeln!(f, "{}", Reset)?;
        }

        Ok(())
    }
}

/// [format] macro which creates a [`Severity::Bug`] report.
#[macro_export]
macro_rules! bug {
    ($($t:tt)*) => {{
        $crate::Report::bug(format!($($t)*))
    }};
}

/// [format] macro which creates a [`Severity::Error`] report.
#[macro_export]
macro_rules! error {
    ($($t:tt)*) => {{
        $crate::Report::error(format!($($t)*))
    }};
}

#[macro_export]
/// [format] macro which creates a [`Severity::Warning`] report.
macro_rules! warning {
    ($($t:tt)*) => {{
        $crate::Report::warning(format!($($t)*))
    }};
}

/// [format] macro which creates a [`Severity::Note`] report.
#[macro_export]
macro_rules! note {
    ($($t:tt)*) => {{
        $crate::Report::note(format!($($t)*))
    }};
}
