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
}
