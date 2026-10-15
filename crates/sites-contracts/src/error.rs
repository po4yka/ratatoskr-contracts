//! [`SitesContractError`]: every way a sites document or message invariant can fail.

/// Every way a cross-field rule of a sites document or message can fail.
///
/// A *parse* error type: it is never serialized onto the wire. Field-level rejection happens in
/// the identifier and string types; only the rules that span a list or several fields surface
/// here.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SitesContractError {
    /// A manifest named no publication, so the site could read nothing.
    #[error("publications must name at least one publication")]
    NoPublication,
    /// A list named the same item twice.
    #[error("{list} names `{item}` twice")]
    Duplicate {
        /// The member that holds the list.
        list: &'static str,
        /// The repeated item.
        item: String,
    },
    /// A list that must be sorted and hold each item once did not.
    #[error("{list} must be in ascending order with each item once, but `{item}` breaks it")]
    Unsorted {
        /// The member that holds the list.
        list: &'static str,
        /// The first item out of order or repeated.
        item: String,
    },
    /// A frontmatter field name was empty, longer than 255 characters or held a control
    /// character.
    #[error("frontmatter field name {key:?} is not a valid field name")]
    FrontmatterKey {
        /// The refused name.
        key: String,
    },
    /// A bundle manifest named an entry document that is not one of its files.
    #[error("entry_document `{path}` is not one of the files of the bundle")]
    EntryDocumentNotListed {
        /// The entry document the manifest named.
        path: String,
    },
    /// A navigation entry named a slug that no page of the release has.
    #[error("navigation names `{slug}`, which is not a page of the release")]
    NavigationSlugNotInRelease {
        /// The slug the entry named.
        slug: String,
    },
}
