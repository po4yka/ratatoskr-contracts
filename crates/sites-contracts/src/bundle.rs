//! The bundle manifest `ratatoskr-bundle.json` (`sites/bundle-manifest.v1`) and its vocabulary.

use ratatoskr_identifiers::{DigestHex, wire_string_newtype};

use crate::error::SitesContractError;
use crate::manifest::SiteId;
use crate::release::{asset_size, strictly_ascending};

wire_string_newtype! {
    /// The path of one file of a site bundle, relative to the bundle root.
    ///
    /// Segments separated by `/`, at most 1024 UTF-8 bytes, no leading or trailing `/`, no empty,
    /// `.` or `..` segment, no backslash and no C0 or DEL control character, so a path never names
    /// a file outside the bundle. The first segment is never `_r`, which the gateway reserves for
    /// forwarding to Edge; `.ratatoskr/`, the reserved path for SDK pages, is accepted (workspace
    /// spec `site-bundles`). The path is in Unicode NFC; this contract does not check NFC, and the
    /// producer normalises every path.
    ///
    /// The length bound is 1024 UTF-8 bytes, because a file system limits a path in bytes. The Rust
    /// type checks bytes. The `maxLength` of this schema is the same number, but JSON Schema counts
    /// characters, so the schema accepts a path of at most 1024 characters whose UTF-8 form is
    /// longer than 1024 bytes, and the Rust type refuses it. A producer checks the byte length.
    pub struct BundlePath {
        pattern  = concat!(
            "^(?:",
            // A first segment that starts with neither `.` nor `_`.
            r"[^._/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*",
            // `.` and a character other than `.`.
            r"|\.[^./\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*",
            // `..` and at least one more character.
            r"|\.\.[^/\\\x00-\x1f\x7f]+",
            // `_` alone, `_` and a character other than `r`, or `_r` and more characters.
            r"|_|_[^r/\\\x00-\x1f\x7f][^/\\\x00-\x1f\x7f]*|_r[^/\\\x00-\x1f\x7f]+",
            ")",
            // Every later segment: not empty, not `.` and not `..`.
            r"(?:/(?:[^./\\\x00-\x1f\x7f]|\.[^./\\\x00-\x1f\x7f]|\.\.[^/\\\x00-\x1f\x7f])",
            r"[^/\\\x00-\x1f\x7f]*)*$",
        ),
        max_len  = 1024,
        examples = ["index.html", "assets/index-4f2a9c.js", ".ratatoskr/sign-in/index.html"],
    }
}

wire_string_newtype! {
    /// The GitHub repository that built the bundle, as `owner/name`.
    ///
    /// The owner has 1 to 39 ASCII letters, digits and `-` and starts with a letter or digit; the
    /// name has 1 to 100 ASCII letters, digits, `.`, `_` and `-`.
    pub struct RepositoryName {
        pattern  = r"^[A-Za-z0-9][A-Za-z0-9-]{0,38}/[A-Za-z0-9._-]{1,100}$",
        max_len  = 140,
        examples = ["owner/docs-site"],
    }
}

wire_string_newtype! {
    /// A full Git commit id: 40 lowercase hexadecimal characters.
    pub struct CommitSha {
        pattern  = r"^[0-9a-f]{40}$",
        max_len  = 40,
        examples = ["3f6c2a9e8b1d4c7f0a5e2b9d6c3f8a1e4b7d0c2f"],
    }
}

/// Contract `sites/bundle-manifest.v1`: the file `ratatoskr-bundle.json` at the root of a site
/// bundle.
///
/// It names the site, the repository and the commit that built the bundle, the entry document,
/// whether the gateway answers an unknown path with the entry document, and every file of the
/// bundle with its size, SHA-256 and `hashed` flag. The archive holds no file that the manifest
/// does not list (workspace spec `site-bundles`). Every other member is refused.
///
/// `Deserialize` is hand-written because two rules span the file list: the files are sorted by
/// path with each path once, and the entry document is one of them. It parses a private mirror
/// struct and then calls [`BundleManifest::validate`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BundleManifest {
    /// The site that the bundle is for. A deploy to another site is refused.
    pub site_id: SiteId,

    /// The repository that built the bundle.
    pub repository: RepositoryName,

    /// The commit that the bundle was built from.
    pub commit: CommitSha,

    /// The file that the gateway serves for `/`, and for an unknown path when `entry_fallback` is
    /// on.
    pub entry_document: BundlePath,

    /// `true` when the gateway answers a path that the bundle does not hold with the entry
    /// document, as a single-page application needs. Never for a path under `/.ratatoskr/`.
    pub entry_fallback: bool,

    /// Every file of the bundle, in ascending order of path, each path once.
    pub files: Vec<BundleFile>,
}

/// One file of a [`BundleManifest`].
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct BundleFile {
    /// The path of the file in the bundle.
    pub path: BundlePath,

    /// The length of the file bytes, at most 2^53 - 1.
    #[serde(deserialize_with = "asset_size")]
    #[schemars(range(max = 9_007_199_254_740_991_u64))]
    pub size_bytes: u64,

    /// The lowercase hex SHA-256 of the file bytes; the gateway sends it as the `ETag`.
    pub sha256: DigestHex,

    /// `true` when the file name carries a hash of its content, so the gateway marks it
    /// `immutable` and may serve it from the previous deploy.
    pub hashed: bool,
}

impl BundleManifest {
    /// Checks the rules that span the file list. `Deserialize` calls this; the bundle workflow
    /// calls it before it writes a manifest.
    ///
    /// # Errors
    ///
    /// [`SitesContractError::Unsorted`] and [`SitesContractError::EntryDocumentNotListed`].
    pub fn validate(&self) -> Result<(), SitesContractError> {
        strictly_ascending("files", self.files.iter().map(|file| file.path.as_str()))?;
        if self
            .files
            .iter()
            .any(|file| file.path == self.entry_document)
        {
            Ok(())
        } else {
            Err(SitesContractError::EntryDocumentNotListed {
                path: self.entry_document.as_str().to_owned(),
            })
        }
    }
}

/// The wire mirror of [`BundleManifest`], parsed before the list rules run.
///
/// **Drift guard:** a field added to [`BundleManifest`] and not to this mirror would be refused as
/// unknown on parse, and the manifest fixtures fail at once. Do not add a field to one of these
/// structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleManifestWire {
    site_id: SiteId,
    repository: RepositoryName,
    commit: CommitSha,
    entry_document: BundlePath,
    entry_fallback: bool,
    files: Vec<BundleFile>,
}

impl<'de> serde::Deserialize<'de> for BundleManifest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = BundleManifestWire::deserialize(deserializer)?;
        let manifest = Self {
            site_id: wire.site_id,
            repository: wire.repository,
            commit: wire.commit,
            entry_document: wire.entry_document,
            entry_fallback: wire.entry_fallback,
            files: wire.files,
        };
        manifest.validate().map_err(serde::de::Error::custom)?;
        Ok(manifest)
    }
}
