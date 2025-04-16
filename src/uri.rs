use std::{hash::Hash, ops::Deref, str::FromStr};

use serde::{Deserialize, Serialize, de::Error};
extern crate alloc;
use fluent_uri::encoding::Split;
use fluent_uri::encoding::encoder::Path as UriPath;
use fluent_uri::{component::Scheme, error::BuildError};

mod control_chars {
    use percent_encoding::AsciiSet;
    /// https://url.spec.whatwg.org/#fragment-percent-encode-set
    const FRAGMENT: &AsciiSet = &percent_encoding::CONTROLS
        .add(b' ')
        .add(b'"')
        .add(b'<')
        .add(b'>')
        .add(b'`');

    /// https://url.spec.whatwg.org/#path-percent-encode-set
    const PATH: &AsciiSet = &FRAGMENT.add(b'#').add(b'?').add(b'{').add(b'}');
    pub(crate) const PATH_SEGMENT: &AsciiSet = &PATH.add(b'/').add(b'%');

    pub(crate) const SPECIAL_PATH_SEGMENT: &AsciiSet = &PATH_SEGMENT.add(b'\\');
}

/// The error describe what happened during converting [std::path::Path] to [Uri] or converting
/// [Uri] to [std::path::Path]
#[derive(Debug, thiserror::Error)]
pub enum UriPathError {
    #[error("No Segments")]
    NoSegments,
    #[error("HostError")]
    HostError,
    #[error("build error")]
    BuildError(#[from] BuildError),
    #[error("not absolute path")]
    NotAbsolutePath,
    #[error("Path illegal")]
    IllegalPath,
}

const SCHEME_FILE: &Scheme = Scheme::new_or_panic("file");

/// Newtype struct around `fluent_uri::Uri<String>` with serialization implementations that use `as_str()` and 'from_str()' respectively.
#[derive(Debug, Clone)]
pub struct Uri(fluent_uri::Uri<String>);

impl Serialize for Uri {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Uri {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        fluent_uri::Uri::<String>::parse(string)
            .map(Uri)
            .map_err(|err| Error::custom(err.to_string()))
    }
}

impl Ord for Uri {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Uri {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl FromStr for Uri {
    type Err = fluent_uri::error::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TOUCH-UP:
        // Use upstream `FromStr` implementation if and when
        // https://github.com/yescallop/fluent-uri-rs/pull/10
        // gets merged.
        // fluent_uri::Uri::from_str(s).map(Self)
        fluent_uri::Uri::parse(s).map(|uri| Self(uri.to_owned()))
    }
}

impl Deref for Uri {
    type Target = fluent_uri::Uri<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/*
    TOUCH-UP: `PartialEq`, `Eq` and `Hash` could all be derived
    if and when the respective implementations get merged upstream:
    https://github.com/yescallop/fluent-uri-rs/pull/9
*/
impl PartialEq for Uri {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Uri {}

impl Hash for Uri {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state)
    }
}

impl Uri {
    /// Assuming the URL is in the `file` scheme or similar,
    /// convert its path to an absolute `std::path::Path`.
    ///
    /// **Note:** This does not actually check the URL’s `scheme`,
    /// and may give nonsensical results for other schemes.
    /// It is the user’s responsibility to check the URL’s scheme before calling this.
    ///
    /// ```
    /// # use lsp_types_f::Uri;
    /// # let url = Uri::parse("file:///etc/passwd").unwrap();
    /// let path = url.to_file_path();
    /// ```
    ///
    /// # Errors
    /// Returns `Err` if the host is neither empty nor `"localhost"` (except on Windows, where
    /// `file:` URLs may have a non-local host),
    /// or if `Path::new_opt()` returns `None`.
    /// (That is, if the percent-decoded path contains a NUL byte or,
    /// for a Windows path, is not UTF-8.)
    ///
    /// This method is only available if the `std` Cargo feature is enabled.
    ///
    pub fn to_file_path(&self) -> Result<std::path::PathBuf, UriPathError> {
        let segments = self.path();
        let segments = segments
            .segments_if_absolute()
            .ok_or(UriPathError::NoSegments)?;
        let host: Option<&str> = match self.authority().map(|authority| authority.host()) {
            None | Some("localhost") => None,
            Some(host_data) if self.scheme().as_str() == "file" => Some(host_data),
            Some(_) => return Err(UriPathError::NoSegments),
        };
        file_url_segments_to_pathbuf(host, segments)
    }

    /// Convert a file name as `std::path::Path` into an URL in the `file` scheme.
    ///
    /// This returns `Err` if the given path is not absolute or,
    /// on Windows, if the prefix is not a disk prefix (e.g. `C:`) or a UNC prefix (`\\`).
    ///
    /// # Examples
    ///
    /// On Unix-like platforms:
    ///
    /// ```
    /// # if cfg!(unix) {
    /// # use lsp_types_f::Uri;
    ///
    /// let uri = Uri::from_file_path("/tmp/foo.txt").unwrap();
    /// assert_eq!(uri.as_str(), "file:///tmp/foo.txt");
    ///
    /// let uri = Uri::from_file_path("../foo.txt");
    /// assert!(uri.is_err());
    ///
    /// let uri = Uri::from_file_path("https://google.com/");
    /// assert!(uri.is_err());
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Will return error when the path is illegal
    pub fn from_file_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, UriPathError> {
        use control_chars::*;
        use fluent_uri::Uri as FUri;
        use fluent_uri::component::Authority;
        use fluent_uri::encoding::EStr;
        use percent_encoding::percent_encode;
        use std::os::unix::ffi::OsStrExt;
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(UriPathError::NotAbsolutePath);
        }
        let mut serialization = "".to_owned();
        let mut empty = true;
        for component in path.components().skip(1) {
            empty = false;
            serialization.push('/');
            #[cfg(not(target_os = "wasi"))]
            serialization.extend(percent_encode(
                component.as_os_str().as_bytes(),
                SPECIAL_PATH_SEGMENT,
            ));

            #[cfg(target_os = "wasi")]
            serialization.extend(percent_encode(
                component.as_os_str().to_string_lossy().as_bytes(),
                SPECIAL_PATH_SEGMENT,
            ));
        }
        if empty {
            serialization.push('/');
        }
        let path = EStr::new(&serialization).ok_or(UriPathError::IllegalPath)?;
        Ok(Self(
            FUri::builder()
                .scheme(SCHEME_FILE)
                .authority(Authority::EMPTY)
                .path(path)
                .build()?,
        ))
    }
    pub fn parse(path: &str) -> Result<Self, fluent_uri::error::ParseError> {
        use fluent_uri::Uri as FUri;
        let uri = FUri::parse(path)?;
        Ok(Self(uri.into()))
    }
}

#[cfg(all(any(unix, target_os = "redox", target_os = "wasi", target_os = "hermit")))]
fn file_url_segments_to_pathbuf(
    host: Option<&str>,

    segments: Split<'_, UriPath>,
) -> Result<std::path::PathBuf, UriPathError> {
    use alloc::vec::Vec;

    use percent_encoding::percent_decode;

    #[cfg(not(target_os = "wasi"))]
    use std::ffi::OsStr;

    #[cfg(target_os = "hermit")]
    use std::os::hermit::ffi::OsStrExt;

    #[cfg(any(unix, target_os = "redox"))]
    use std::os::unix::prelude::OsStrExt;

    use std::path::PathBuf;

    if host.is_some_and(|host| !host.is_empty()) {
        return Err(UriPathError::HostError);
    }

    let mut bytes = if cfg!(target_os = "redox") {
        b"file:".to_vec()
    } else {
        Vec::new()
    };

    for segment in segments {
        bytes.push(b'/');

        bytes.extend(percent_decode(segment.as_str().as_bytes()));
    }

    // A windows drive letter must end with a slash.

    if bytes.len() > 2
        && bytes[bytes.len() - 2].is_ascii_alphabetic()
        && matches!(bytes[bytes.len() - 1], b':' | b'|')
    {
        bytes.push(b'/');
    }

    #[cfg(not(target_os = "wasi"))]
    let path = PathBuf::from(OsStr::from_bytes(&bytes));

    #[cfg(target_os = "wasi")]
    let path = String::from_utf8(bytes)
        .map(|path| PathBuf::from(path))
        .map_err(|_| ())?;

    debug_assert!(
        path.is_absolute(),
        "to_file_path() failed to produce an absolute Path"
    );

    Ok(path)
}

#[cfg(windows)]
fn file_url_segments_to_pathbuf(
    host: Option<&str>,
    segments: Split<'_, Path>,
) -> Result<std::path::PathBuf, UriPathError> {
    file_url_segments_to_pathbuf_windows(host, segments)
}

/// https://url.spec.whatwg.org/#ascii-alpha
#[allow(unused)]
#[inline]
fn ascii_alpha(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

// Build this unconditionally to alleviate https://github.com/servo/rust-url/issues/102
#[cfg_attr(not(windows), allow(dead_code))]
fn file_url_segments_to_pathbuf_windows(
    host: Option<&str>,
    mut segments: Split<'_, UriPath>,
) -> Result<std::path::PathBuf, UriPathError> {
    use percent_encoding::percent_decode;
    use std::path::PathBuf;

    let mut string = if let Some(host) = host {
        r"\\".to_owned() + host
    } else {
        let first = segments.next().ok_or(UriPathError::HostError)?.as_str();

        match first.len() {
            2 => {
                if !first.starts_with(ascii_alpha) || first.as_bytes()[1] != b':' {
                    return Err(UriPathError::HostError);
                }

                first.to_owned()
            }

            4 => {
                if !first.starts_with(ascii_alpha) {
                    return Err(UriPathError::HostError);
                }

                let bytes = first.as_bytes();

                if bytes[1] != b'%' || bytes[2] != b'3' || (bytes[3] != b'a' && bytes[3] != b'A') {
                    return Err(UriPathError::HostError);
                }

                first[0..1].to_owned() + ":"
            }

            _ => return Err(UriPathError::HostError),
        }
    };

    for segment in segments.map(|seg| seg.as_str()) {
        string.push('\\');

        // Currently non-unicode windows paths cannot be represented

        match String::from_utf8(percent_decode(segment.as_bytes()).collect()) {
            Ok(s) => string.push_str(&s),

            Err(..) => return Err(UriPathError::HostError),
        }
    }

    let path = PathBuf::from(string);

    debug_assert!(
        path.is_absolute(),
        "to_file_path() failed to produce an absolute Path"
    );

    Ok(path)
}

#[cfg(test)]
mod tests {
    #[cfg(not(windows))]
    pub use std::fs::canonicalize as strict_canonicalize;

    /// On Windows, rewrites the wide path prefix `\\?\C:` to `C:`  
    /// Source: https://stackoverflow.com/a/70970317
    #[inline]
    #[cfg(windows)]
    fn strict_canonicalize<P: AsRef<Path>>(path: P) -> std::io::Result<PathBuf> {
        use std::io;

        fn impl_(path: PathBuf) -> std::io::Result<PathBuf> {
            let head = path
                .components()
                .next()
                .ok_or(io::Error::new(io::ErrorKind::Other, "empty path"))?;
            let disk_;
            let head = if let std::path::Component::Prefix(prefix) = head {
                if let std::path::Prefix::VerbatimDisk(disk) = prefix.kind() {
                    disk_ = format!("{}:", disk as char);
                    Path::new(&disk_).components().next().ok_or(io::Error::new(
                        io::ErrorKind::Other,
                        "failed to parse disk component",
                    ))?
                } else {
                    head
                }
            } else {
                head
            };
            Ok(std::iter::once(head)
                .chain(path.components().skip(1))
                .collect())
        }

        let canon = std::fs::canonicalize(path)?;
        impl_(canon)
    }
    use super::Uri;
    use std::path::Path;

    #[test]
    #[cfg(windows)]
    fn test_idempotent_canonicalization() {
        let lhs = strict_canonicalize(Path::new(".")).unwrap();
        let rhs = strict_canonicalize(&lhs).unwrap();
        assert_eq!(lhs, rhs);
    }

    #[test]
    fn test_path_roundtrip_conversion() {
        let src = strict_canonicalize(Path::new(".")).unwrap();
        let conv = Uri::from_file_path(&src).unwrap();
        let roundtrip = conv.to_file_path().unwrap();
        assert_eq!(src, roundtrip, "conv={conv:?}",);
        let url = Uri::from_file_path("/tmp/foo.txt").unwrap();

        assert_eq!(url.as_str(), "file:///tmp/foo.txt");
    }
    #[test]
    #[cfg(windows)]
    fn test_windows_uri_roundtrip_conversion() {
        use std::str::FromStr;

        let uri = Uri::from_str("file:///C:/Windows").unwrap();
        let path = uri.to_file_path().unwrap();
        assert_eq!(&path, Path::new("C:/Windows"), "uri={uri:?}");

        let conv = Uri::from_file_path(&path).unwrap();

        assert_eq!(
            uri,
            conv,
            "path={path:?} left={} right={}",
            uri.as_str(),
            conv.as_str()
        );
    }
}
