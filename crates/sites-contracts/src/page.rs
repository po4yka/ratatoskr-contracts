//! The page document `sites/page.v1` and its body tree.

use std::collections::BTreeMap;

use ratatoskr_identifiers::{DigestHex, MediaType, wire_string_newtype};

use crate::error::SitesContractError;
use crate::release::{PageTitle, Slug, strictly_ascending};

wire_string_newtype! {
    /// The name of one frontmatter field of a published page.
    ///
    /// 1 to 255 characters with no C0 or DEL control character. Only the fields on the
    /// publication's allowlist reach a page (workspace spec `site-release-privacy`).
    pub struct FrontmatterKey {
        pattern  = r"^[^\x00-\x1f\x7f]{1,255}$",
        max_len  = 1020,
        examples = ["title", "description"],
    }
}

wire_string_newtype! {
    /// The anchor of a heading or a block inside a page, without the leading `#`.
    ///
    /// 1 to 255 characters with no space, no `#`, no `/` and no control character. A block
    /// anchor starts with `^`, as in Obsidian.
    pub struct Anchor {
        pattern  = r"^[^ #/\x00-\x1f\x7f]{1,255}$",
        max_len  = 1020,
        examples = ["start-here", "^summary"],
    }
}

wire_string_newtype! {
    /// The type of an Obsidian callout, in lower case, such as `note` or `warning`.
    pub struct CalloutKind {
        pattern  = r"^[a-z0-9-]{1,40}$",
        max_len  = 40,
        examples = ["note", "warning"],
    }
}

wire_string_newtype! {
    /// The target of an external link: an `https`, `http` or `mailto` URL.
    ///
    /// A link to a page of the release is a `wikiLink` and an asset is an `embed`, so this type
    /// takes no relative reference. No white space and no control character.
    pub struct LinkUrl {
        pattern  = r"^(?:https?://|mailto:)[^ \x00-\x1f\x7f]+$",
        max_len  = 2048,
        examples = ["https://example.test/about"],
    }
}

wire_string_newtype! {
    /// The source of an external image: an `https` or `http` URL. No white space and no control
    /// character.
    pub struct ImageUrl {
        pattern  = r"^https?://[^ \x00-\x1f\x7f]+$",
        max_len  = 2048,
        examples = ["https://example.test/logo.png"],
    }
}

/// Contract `sites/page.v1`: one page of a release, as the delivery route
/// `/v1/sites/{site}/content/pages/{slug}` serves it.
///
/// The body is here twice, both made from the same filtered tree (workspace design D2): `ast` for a
/// site that renders the structure itself, and `html` for a site that inserts markup. Neither holds
/// a vault path or anything read from a file that the publication does not select. Every other
/// member is refused.
///
/// `Deserialize` is hand-written because the order rules span a list. It parses a private mirror
/// struct and then calls [`Page::validate`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Page {
    /// The slug of the page, unique in its release.
    pub slug: Slug,

    /// The title of the page.
    pub title: PageTitle,

    /// The frontmatter fields on the publication's allowlist, by name, with the values the note
    /// gives them. `publish` and `slug` are never here. Each name is a [`FrontmatterKey`]; the
    /// Rust type checks it, because the TypeScript projection has no form for a key pattern.
    pub frontmatter: BTreeMap<String, serde_json::Value>,

    /// The body as an mdast-compatible tree.
    pub ast: PageRoot,

    /// The body rendered from the same tree and passed through the closed element and attribute
    /// allowlist of the workspace spec `site-release-privacy`. An asset is `assets/<sha256>`.
    pub html: String,

    /// The pages of the release that this page links to or embeds, in ascending order, each once.
    pub links: Vec<Slug>,

    /// The pages of the release that link to or embed this page, in ascending order, each once.
    pub backlinks: Vec<Slug>,

    /// The assets of the release that this page uses, in ascending order of SHA-256, each once.
    pub assets: Vec<AssetRef>,
}

/// One asset that a [`Page`] uses.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct AssetRef {
    /// The lowercase hex SHA-256 of the asset bytes; the asset URL is `assets/<sha256>`.
    pub sha256: DigestHex,

    /// The media type of the asset bytes.
    pub media_type: MediaType,
}

/// The root of a page tree: mdast `root`.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PageRoot {
    /// Always `root`.
    #[serde(rename = "type")]
    pub node_type: PageRootType,

    /// The block nodes of the page, in document order.
    pub children: Vec<PageNode>,
}

/// The node type of a [`PageRoot`]: always `root`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub enum PageRootType {
    /// mdast `root`.
    Root,
}

/// One node of a page tree. The names and members follow mdast, and three Obsidian node types are
/// added: `callout`, `wikiLink` and `embed`.
///
/// **Closed on purpose**: the release builder emits only these types, and a site that meets
/// another must stop rather than render a guess. The tree has no raw HTML node; raw HTML of a note
/// reaches only the sanitised `html` field of the page. A link or embed whose target the
/// publication does not select is not a node at all: the builder writes the author's text as
/// `text`, or removes the embed (workspace spec `site-release-privacy`).
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
#[non_exhaustive]
pub enum PageNode {
    /// mdast `paragraph`.
    Paragraph {
        /// The inline nodes of the paragraph.
        children: Vec<PageNode>,
    },
    /// mdast `heading`.
    Heading {
        /// The heading level, 1 to 6.
        #[serde(deserialize_with = "heading_depth")]
        #[schemars(range(min = 1, max = 6))]
        depth: u8,
        /// The anchor that a `wikiLink` or an `embed` names to reach this heading.
        anchor: Anchor,
        /// The inline nodes of the heading.
        children: Vec<PageNode>,
    },
    /// mdast `thematicBreak`. A struct variant without members, not a unit variant: serde
    /// ignores `deny_unknown_fields` on an internally tagged unit variant.
    ThematicBreak {},
    /// mdast `blockquote`.
    Blockquote {
        /// The block nodes of the quote.
        children: Vec<PageNode>,
    },
    /// An Obsidian callout, `> [!kind]± title`.
    Callout {
        /// The callout type.
        kind: CalloutKind,
        /// Whether the callout folds, and how it starts. Absent when it does not fold.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fold: Option<CalloutFold>,
        /// The inline nodes of the title. Absent, never empty, when the author wrote none.
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        title: Vec<PageNode>,
        /// The block nodes of the callout.
        children: Vec<PageNode>,
    },
    /// mdast `list`.
    List {
        /// `true` for a numbered list.
        ordered: bool,
        /// The number of the first item of a numbered list. Absent for a list without numbers.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        start: Option<u32>,
        /// `true` when the items are separated by blank lines.
        spread: bool,
        /// The `listItem` nodes of the list.
        children: Vec<PageNode>,
    },
    /// mdast `listItem`.
    ListItem {
        /// The state of a task item. Absent for an item that is not a task.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        checked: Option<bool>,
        /// `true` when the item holds blank lines.
        spread: bool,
        /// The block nodes of the item.
        children: Vec<PageNode>,
    },
    /// mdast `table` (GFM).
    Table {
        /// The alignment of each column; `null` for a column without one.
        align: Vec<Option<TableAlign>>,
        /// The `tableRow` nodes of the table; the first is the header row.
        children: Vec<PageNode>,
    },
    /// mdast `tableRow`.
    TableRow {
        /// The `tableCell` nodes of the row.
        children: Vec<PageNode>,
    },
    /// mdast `tableCell`.
    TableCell {
        /// The inline nodes of the cell.
        children: Vec<PageNode>,
    },
    /// mdast `code`: a fenced or indented code block. A plugin query block, such as Dataview, is a
    /// code block too, and it is never run.
    Code {
        /// The language word after the opening fence. Absent when the author wrote none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lang: Option<String>,
        /// The rest of the line after the language word. Absent when the author wrote none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        meta: Option<String>,
        /// The code, as written. mdast names this member `value`.
        value: String,
    },
    /// mdast `text`. mdast names its member `value`.
    Text {
        /// The text.
        value: String,
    },
    /// mdast `emphasis`.
    Emphasis {
        /// The emphasised inline nodes.
        children: Vec<PageNode>,
    },
    /// mdast `strong`.
    Strong {
        /// The strong inline nodes.
        children: Vec<PageNode>,
    },
    /// mdast `delete` (GFM strikethrough).
    Delete {
        /// The struck-through inline nodes.
        children: Vec<PageNode>,
    },
    /// An Obsidian highlight, `==text==`.
    Highlight {
        /// The highlighted inline nodes.
        children: Vec<PageNode>,
    },
    /// mdast `inlineCode`. mdast names its member `value`.
    InlineCode {
        /// The code, as written.
        value: String,
    },
    /// mdast `break`: a hard line break. A struct variant without members, for the reason
    /// `thematicBreak` gives.
    Break {},
    /// mdast `link` to an external URL.
    Link {
        /// The target.
        url: LinkUrl,
        /// The link title. Absent when the author wrote none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// The inline nodes of the link text.
        children: Vec<PageNode>,
    },
    /// mdast `image` from an external URL.
    Image {
        /// The source.
        url: ImageUrl,
        /// The alternative text. Absent when the author wrote none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
        /// The image title. Absent when the author wrote none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    /// A link to a page of the release, from `[[target]]`, `[[target|alias]]` or a Markdown link to
    /// a note, after the notes resolver resolved it.
    WikiLink {
        /// The page that the link opens.
        slug: Slug,
        /// The heading or block of that page. Absent for a link to the whole page.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<Anchor>,
        /// The link text: the author's alias, or else the target as written.
        children: Vec<PageNode>,
    },
    /// An embed, `![[target]]` or `![alt](path)`, of a page or an asset of the release.
    Embed {
        /// What the embed shows.
        target: EmbedTarget,
    },
}

/// How a folding callout starts.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CalloutFold {
    /// `+`: the callout folds and starts open.
    Open,
    /// `-`: the callout folds and starts closed.
    Closed,
}

/// The alignment of a table column.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum TableAlign {
    /// `:--`.
    Left,
    /// `:-:`.
    Center,
    /// `--:`.
    Right,
}

/// What an [`PageNode::Embed`] shows.
#[derive(
    Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
#[non_exhaustive]
pub enum EmbedTarget {
    /// A page of the release, or a heading or block of it.
    Page {
        /// The embedded page.
        slug: Slug,
        /// The heading or block of that page. Absent for the whole page.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<Anchor>,
    },
    /// An asset of the release, named by its SHA-256 and never by its vault path.
    Asset {
        /// The lowercase hex SHA-256 of the asset bytes; the asset URL is `assets/<sha256>`.
        sha256: DigestHex,
        /// The media type of the asset bytes.
        media_type: MediaType,
        /// The alternative text that the author wrote in the published page. Absent when the
        /// author wrote none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        alt: Option<String>,
    },
}

/// Refuses a heading level outside 1 to 6.
fn heading_depth<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let depth = <u8 as serde::Deserialize>::deserialize(deserializer)?;
    if (1..=6).contains(&depth) {
        Ok(depth)
    } else {
        Err(serde::de::Error::custom(format!(
            "heading depth must be 1 to 6, found {depth}"
        )))
    }
}

impl Page {
    /// Checks the order rules. `Deserialize` calls this; the release builder calls it before it
    /// stores a page.
    ///
    /// # Errors
    ///
    /// [`SitesContractError::Unsorted`] and [`SitesContractError::FrontmatterKey`].
    pub fn validate(&self) -> Result<(), SitesContractError> {
        if let Some(key) = self
            .frontmatter
            .keys()
            .find(|key| FrontmatterKey::parse(key).is_err())
        {
            return Err(SitesContractError::FrontmatterKey { key: key.clone() });
        }
        strictly_ascending("links", self.links.iter().map(Slug::as_str))?;
        strictly_ascending("backlinks", self.backlinks.iter().map(Slug::as_str))?;
        strictly_ascending(
            "assets",
            self.assets.iter().map(|asset| asset.sha256.as_str()),
        )
    }
}

/// The wire mirror of [`Page`], parsed before the order rules run.
///
/// **Drift guard:** a field added to [`Page`] and not to this mirror would be refused as unknown
/// on parse, and the page fixtures fail at once. Do not add a field to one of these structs alone.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PageWire {
    slug: Slug,
    title: PageTitle,
    frontmatter: BTreeMap<String, serde_json::Value>,
    ast: PageRoot,
    html: String,
    links: Vec<Slug>,
    backlinks: Vec<Slug>,
    assets: Vec<AssetRef>,
}

impl<'de> serde::Deserialize<'de> for Page {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = PageWire::deserialize(deserializer)?;
        let page = Self {
            slug: wire.slug,
            title: wire.title,
            frontmatter: wire.frontmatter,
            ast: wire.ast,
            html: wire.html,
            links: wire.links,
            backlinks: wire.backlinks,
            assets: wire.assets,
        };
        page.validate().map_err(serde::de::Error::custom)?;
        Ok(page)
    }
}
