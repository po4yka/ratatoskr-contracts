//! [`ProducerName`]: the deployment identity of whatever asserted a fact.

ratatoskr_identifiers::wire_string_newtype! {
    /// The deployable that emitted this record, e.g. `ratatoskr-extractor`.
    ///
    /// A deployment identity, not an instance identity: never a hostname, pod name, region or
    /// build version. Kebab-case, because the service names in `README.md` and
    /// `ARCHITECTURE.md` S5.2 are kebab-case. Every value used inside this repository must be
    /// registered in `contracts.toml [services].known`.
    pub struct ProducerName {
        pattern  = r"^[a-z][a-z0-9-]{1,63}$",
        max_len  = 64,
        examples = ["ratatoskr-extractor", "ratatoskr-x"],
    }
}
