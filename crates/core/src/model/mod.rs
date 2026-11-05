//! Core types for the Truth Social API.
//!
//! This module provides Rust bindings for deserializing (and serializing) Truth Social status
//! (post) data and related types: accounts, media attachments, mentions, tags, preview cards,
//! groups, polls, custom emoji, and Truth Social specific extensions such as TV metadata and
//! advertising metrics.
//!
//! String fields use [`Cow`], and borrow from the input when the deserializer allows it (for JSON
//! parsed from a string or byte slice, strings without escapes). Use [`from_str_owned`] for values
//! that must outlive their input. Wayback Machine integration lives in
//! `archivindex-truthsocial-wbm`.

use std::borrow::Cow;

use chrono::{DateTime, NaiveDate, Utc};
use either::Either;
use serde_field_attributes::{integer_str, optional_integer_str, optional_usize};

mod tv_timestamp;

/// A Truth Social status (post).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Status<'a> {
    /// Unique identifier for this status.
    #[serde(with = "integer_str")]
    pub id: u64,

    /// Timestamp when this status was created.
    pub created_at: DateTime<Utc>,

    /// ID of the status this is replying to, if any.
    #[serde(default, with = "optional_integer_str")]
    pub in_reply_to_id: Option<u64>,

    /// ID of a quoted status, if any.
    #[serde(default, with = "optional_integer_str")]
    pub quote_id: Option<u64>,

    /// ID of the account being replied to, if any.
    #[serde(default, with = "optional_integer_str")]
    pub in_reply_to_account_id: Option<u64>,

    /// Whether this status is marked as sensitive.
    pub sensitive: bool,

    /// Content warning text, if any.
    #[serde(borrow)]
    pub spoiler_text: Cow<'a, str>,

    /// Visibility level, or `None` when the field is missing or `null`.
    pub visibility: Option<Visibility>,

    /// ISO 639-1 language code, if detected.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub language: Option<Cow<'a, str>>,

    /// `ActivityPub` URI for this status.
    #[serde(borrow)]
    pub uri: Cow<'a, str>,

    /// Web URL for this status.
    #[serde(borrow)]
    pub url: Cow<'a, str>,

    /// HTML-formatted content of this status.
    #[serde(borrow)]
    pub content: Cow<'a, str>,

    /// Plain-text source of the content. Only `null` has been observed in fixtures;
    /// non-null values are rejected.
    pub text: Option<()>,

    /// The application used to post this status. Only `null` has been observed in fixtures;
    /// non-null values are rejected.
    pub application: Option<()>,

    /// The account that authored this status.
    #[serde(borrow)]
    pub account: Account<'a>,

    /// Media attachments (images, videos, etc.).
    #[serde(borrow)]
    pub media_attachments: Vec<MediaAttachment<'a>>,

    /// Accounts mentioned in this status.
    #[serde(borrow)]
    pub mentions: Vec<Mention<'a>>,

    /// Hashtags used in this status.
    #[serde(borrow)]
    pub tags: Vec<Tag<'a>>,

    /// Preview card for linked content, if any.
    #[serde(borrow)]
    pub card: Option<Card<'a>>,

    /// Group this status was posted to, if any.
    #[serde(borrow)]
    pub group: Option<Group<'a>>,

    /// Quoted status, if this is a quote post.
    pub quote: Option<Box<Self>>,

    /// The status being replied to, when included in the response (a Truth Social extension).
    pub in_reply_to: Option<Box<Self>>,

    /// Reblogged status, if this is a reblog/boost.
    pub reblog: Option<Box<Self>>,

    /// Whether this is a sponsored/promoted status.
    pub sponsored: Option<bool>,

    /// Number of replies to this status.
    ///
    /// `None` if the API withheld the count, which it signals by sending `-1` rather than omitting
    /// the field (e.g. for some nested or reblogged statuses). The same applies to
    /// [`reblogs_count`](Self::reblogs_count) and [`favourites_count`](Self::favourites_count).
    #[serde(with = "optional_usize")]
    pub replies_count: Option<usize>,

    /// Number of times this status has been reblogged (`None` if the API withheld the count).
    #[serde(with = "optional_usize")]
    pub reblogs_count: Option<usize>,

    /// Number of times this status has been favourited (`None` if the API withheld the count).
    #[serde(with = "optional_usize")]
    pub favourites_count: Option<usize>,

    /// Current user's reaction to this status, if any (e.g. `"upvote"`).
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub reaction: Option<Cow<'a, str>>,

    /// Number of upvotes (Truth Social specific).
    pub upvotes_count: Option<u64>,

    /// Number of downvotes (Truth Social specific).
    pub downvotes_count: Option<u64>,

    /// Whether the current user has favourited this status.
    pub favourited: Option<bool>,

    /// Whether the current user has reblogged this status.
    pub reblogged: Option<bool>,

    /// Whether the current user has muted this status.
    pub muted: Option<bool>,

    /// Whether the current user has muted this status's quoted status (Truth Social specific).
    pub quote_muted: Option<bool>,

    /// Whether this status is visible in its group's timeline (Truth Social specific).
    pub group_timeline_visible: Option<bool>,

    /// Whether this status is pinned to the author's profile.
    pub pinned: Option<bool>,

    /// Whether the current user has bookmarked this status.
    pub bookmarked: Option<bool>,

    /// Poll attached to this status, if any.
    #[serde(borrow)]
    pub poll: Option<Poll<'a>>,

    /// Custom emoji used in this status.
    #[serde(borrow)]
    pub emojis: Vec<Emoji<'a>>,

    /// Whether the current user can vote on this status (Truth Social specific).
    pub votable: Option<bool>,

    /// Timestamp when this status was last edited, if ever.
    pub edited_at: Option<DateTime<Utc>>,

    /// Version number of this status (for edit tracking).
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub version: Option<Cow<'a, str>>,

    /// Whether the current user can edit this status.
    pub editable: Option<bool>,

    /// Title of this status (Truth Social specific).
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub title: Option<Cow<'a, str>>,

    /// Tombstone information if this status was deleted or muted.
    pub tombstone: Option<Tombstone>,

    /// Timestamp when this status was deleted, if applicable.
    pub deleted_at: Option<DateTime<Utc>>,

    /// Advertising metrics for sponsored content (Truth Social specific).
    #[serde(borrow)]
    pub metrics: Option<Metrics<'a>>,

    /// TV-specific metadata for live/scheduled TV content (Truth Social specific).
    #[serde(borrow)]
    pub tv: Option<TvMetadata<'a>>,

    /// Announcement embedded in this status (Truth Social specific). Absent in older snapshots; its
    /// non-null shape has not been observed, so it is kept as an opaque value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedded_announcement: Option<serde_json::Value>,

    /// URL of the next status in a paged video feed (Truth Social specific). Absent in older
    /// snapshots.
    #[serde(
        borrow,
        default,
        with = "archivindex_serde::borrowable_str::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_status: Option<Cow<'a, str>>,

    /// Per-request signature token. Observed as `<base64-json>.<base64-other>`, where the JSON
    /// payload decodes to `[viewer_id, status_id, request_timestamp]`. Absent in older snapshots;
    /// kept as an opaque token.
    #[serde(
        borrow,
        default,
        with = "archivindex_serde::borrowable_str::option",
        skip_serializing_if = "Option::is_none"
    )]
    pub signature: Option<Cow<'a, str>>,

    /// Whether this reply is pinned beneath its parent status by that status's author. Absent in
    /// older snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_pinned: Option<bool>,
}

/// The `content` of a snapshot: a single [`Status`], or an array of them (some endpoints, such as
/// paged video feeds, return a list).
#[derive(Clone, Debug, PartialEq)]
pub enum StatusContent<'a> {
    /// A single status object.
    Single(Box<Status<'a>>),
    /// An array of status objects.
    Multiple(Vec<Status<'a>>),
}

impl<'a> StatusContent<'a> {
    /// Borrows each status in the content: one for [`Single`](Self::Single), all for
    /// [`Multiple`](Self::Multiple).
    pub fn statuses(&self) -> impl Iterator<Item = &Status<'a>> {
        match self {
            // `Either` keeps the two arms a single iterator type without allocating for `Single`.
            Self::Single(status) => Either::Left(std::iter::once(status.as_ref())),
            Self::Multiple(statuses) => Either::Right(statuses.iter()),
        }
    }

    /// Consumes the content, yielding each owned status: one for [`Single`](Self::Single), all for
    /// [`Multiple`](Self::Multiple).
    pub fn into_statuses(self) -> impl Iterator<Item = Status<'a>> {
        match self {
            Self::Single(status) => Either::Left(std::iter::once(*status)),
            Self::Multiple(statuses) => Either::Right(statuses.into_iter()),
        }
    }
}

// A hand-written `Deserialize` (rather than `#[serde(untagged)]`) that dispatches on the JSON shape
// via `deserialize_any`: an object is a single status, an array is many. Dispatching on the shape
// also keeps an array from being read as the positional form of a single status, which a derived
// struct would accept. The untagged derive is not an option here: it buffers into an intermediate
// value, through which `Status`'s field-level custom deserializers (e.g. the `integer_str` and
// `last_status_at` helpers) do not round-trip.
impl<'de: 'a, 'a> serde::Deserialize<'de> for StatusContent<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ContentVisitor<'a>(std::marker::PhantomData<&'a ()>);

        impl<'de: 'a, 'a> serde::de::Visitor<'de> for ContentVisitor<'a> {
            type Value = StatusContent<'a>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a status object or an array of statuses")
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                map: M,
            ) -> Result<Self::Value, M::Error> {
                <Status<'a> as serde::Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )
                .map(|status| StatusContent::Single(Box::new(status)))
            }

            fn visit_seq<S: serde::de::SeqAccess<'de>>(
                self,
                seq: S,
            ) -> Result<Self::Value, S::Error> {
                <Vec<Status<'a>> as serde::Deserialize>::deserialize(
                    serde::de::value::SeqAccessDeserializer::new(seq),
                )
                .map(StatusContent::Multiple)
            }
        }

        deserializer.deserialize_any(ContentVisitor(std::marker::PhantomData))
    }
}

/// A status, status list, or account response body.
///
/// Used to validate captures from status and account lookup endpoints. Other response shapes,
/// such as search results and groups, are not represented here.
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseBody<'a> {
    /// A status endpoint's body: a single status or an array of them.
    Statuses(StatusContent<'a>),
    /// An account endpoint's body.
    Account(Box<Account<'a>>),
}

// A hand-written `Deserialize` that dispatches on the JSON shape via `deserialize_any`, the same
// approach `StatusContent` uses: an array is a page of statuses, and an object is either a single
// status or an account. Both object schemas use `deny_unknown_fields`, so they are mutually
// exclusive; the object is buffered once and dispatched on the `username` field, which every
// account carries and no status does. Buffering (rather than streaming straight into `Status`) is
// what lets the account alternative be tried, and routing it through `serde_json::Value` preserves
// the field-level custom deserializers (e.g. `integer_str`) that serde's own untagged buffering
// would break. Strings in a buffered object are owned, so only an array of statuses borrows.
impl<'de: 'a, 'a> serde::Deserialize<'de> for ResponseBody<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ResponseBodyVisitor<'a>(std::marker::PhantomData<&'a ()>);

        impl<'de: 'a, 'a> serde::de::Visitor<'de> for ResponseBodyVisitor<'a> {
            type Value = ResponseBody<'a>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a status object, an array of statuses, or an account object")
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                map: M,
            ) -> Result<Self::Value, M::Error> {
                let value = <serde_json::Value as serde::Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )?;
                // Deserializing from the buffered `Value` raises a `serde_json::Error`, which is
                // remapped onto the driving deserializer's error type.
                if value.get("username").is_some() {
                    <Account<'a> as serde::Deserialize>::deserialize(value)
                        .map(|account| ResponseBody::Account(Box::new(account)))
                        .map_err(serde::de::Error::custom)
                } else {
                    <Status<'a> as serde::Deserialize>::deserialize(value)
                        .map(|status| {
                            ResponseBody::Statuses(StatusContent::Single(Box::new(status)))
                        })
                        .map_err(serde::de::Error::custom)
                }
            }

            fn visit_seq<S: serde::de::SeqAccess<'de>>(
                self,
                seq: S,
            ) -> Result<Self::Value, S::Error> {
                <Vec<Status<'a>> as serde::Deserialize>::deserialize(
                    serde::de::value::SeqAccessDeserializer::new(seq),
                )
                .map(|statuses| ResponseBody::Statuses(StatusContent::Multiple(statuses)))
            }
        }

        deserializer.deserialize_any(ResponseBodyVisitor(std::marker::PhantomData))
    }
}

/// Tombstone information for a deleted or muted status.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tombstone {
    /// Reason the status is unavailable.
    pub reason: TombstoneReason,
}

/// Advertising metrics for sponsored content.
///
/// Present on sponsored statuses to track ad impressions and display information.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics<'a> {
    /// URL to call for tracking impressions.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub impression: Option<Cow<'a, str>>,

    /// When these metrics expire.
    pub expires_at: Option<DateTime<Utc>>,

    /// Reason displayed to user for why this ad is shown.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub reason: Option<Cow<'a, str>>,
}

/// TV-specific metadata for live or scheduled TV content.
///
/// Present on statuses and media attachments related to Truth Social TV.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TvMetadata<'a> {
    /// Channel identifier.
    pub channel_id: u64,

    /// When the TV program starts.
    #[serde(deserialize_with = "tv_timestamp::deserialize")]
    pub start_time: DateTime<Utc>,

    /// When the TV program ends. Unsigned extended years (such as `57564`) are accepted and
    /// serialize with an explicit `+` sign.
    #[serde(deserialize_with = "tv_timestamp::deserialize")]
    pub end_time: DateTime<Utc>,

    /// Name/title of the TV program.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// PLTV timespan, in milliseconds.
    ///
    /// The only value observed across the captured fixtures is `259200000`, exactly 72 hours,
    /// which is what fixes the unit; the field is otherwise undocumented. No negative value has
    /// been observed, despite the signed type.
    pub pltv_timespan: i64,

    /// URL to the program's image/thumbnail.
    #[serde(borrow)]
    pub image_url: Cow<'a, str>,

    /// Description of the TV program.
    #[serde(borrow)]
    pub description: Cow<'a, str>,
}

/// Reason why a status was tombstoned.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TombstoneReason {
    /// The status was deleted by the user or a moderator.
    Deleted,
    /// The status is muted for the current user.
    Muted,
}

/// Visibility level of a status.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Visible to everyone.
    Public,
    /// Visible to followers only.
    Private,
    /// Visible only to mentioned users.
    Direct,
    /// Public but not shown in public timelines.
    Unlisted,
    /// A status posted to a group. Access depends on the group settings.
    Group,
    /// Visible only to the author (self).
    #[serde(rename = "self")]
    Self_,
}

/// A Truth Social user account.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Account<'a> {
    /// Unique identifier for this account.
    #[serde(with = "integer_str")]
    pub id: u64,

    /// Username (without domain).
    #[serde(borrow)]
    pub username: Cow<'a, str>,

    /// Account address: `username@domain` for a remote account, or `username` for a local one.
    #[serde(borrow)]
    pub acct: Cow<'a, str>,

    /// Display name shown on profile.
    #[serde(borrow)]
    pub display_name: Cow<'a, str>,

    /// Whether this account requires follow approval.
    pub locked: bool,

    /// Whether this account is a bot.
    pub bot: bool,

    /// Whether this account has opted into discovery features.
    pub discoverable: Option<bool>,

    /// Whether this is a group account.
    pub group: bool,

    /// Timestamp when this account was created.
    pub created_at: DateTime<Utc>,

    /// HTML-formatted bio/description.
    #[serde(borrow)]
    pub note: Cow<'a, str>,

    /// URL to the account's profile page.
    #[serde(borrow)]
    pub url: Cow<'a, str>,

    /// URL to the account's avatar image.
    #[serde(borrow)]
    pub avatar: Cow<'a, str>,

    /// URL to the static version of the avatar.
    #[serde(borrow)]
    pub avatar_static: Cow<'a, str>,

    /// URL to the account's header/banner image.
    #[serde(borrow)]
    pub header: Cow<'a, str>,

    /// URL to the static version of the header.
    #[serde(borrow)]
    pub header_static: Cow<'a, str>,

    /// Number of followers (occasionally negative for unknown reasons).
    pub followers_count: i64,

    /// Number of accounts this account follows (occasionally negative for unknown reasons).
    pub following_count: i64,

    /// Number of statuses posted (occasionally negative for unknown reasons).
    pub statuses_count: i64,

    /// Timestamp of the most recent status, if any.
    ///
    /// Truth Social returns this inconsistently as either a date (`"2023-05-23"`, [`Either::Left`])
    /// or a full RFC 3339 datetime (`"2023-05-23T22:31:45.360Z"`, [`Either::Right`]); both forms
    /// are preserved.
    #[serde(default, with = "either::serde_untagged_optional")]
    pub last_status_at: Option<Either<NaiveDate, DateTime<Utc>>>,

    /// Whether this account is verified (Truth Social specific), or `None` when unavailable.
    pub verified: Option<bool>,

    /// Location string (Truth Social specific).
    #[serde(borrow)]
    pub location: Cow<'a, str>,

    /// Website URL (Truth Social specific).
    #[serde(borrow)]
    pub website: Cow<'a, str>,

    /// Whether unauthenticated users can view this account (Truth Social specific).
    pub unauth_visibility: Option<bool>,

    /// Whether this account has completed chat onboarding (Truth Social specific).
    pub chats_onboarded: Option<bool>,

    /// Whether this account has completed feeds onboarding (Truth Social specific).
    pub feeds_onboarded: Option<bool>,

    /// Whether this account has completed bookmarks onboarding (Truth Social specific).
    pub bookmarks_onboarded: Option<bool>,

    /// Whether this account has completed group-reactions onboarding (Truth Social specific).
    pub group_reactions_onboarded: Option<bool>,

    /// Whether this account accepts direct messages (Truth Social specific).
    pub accepting_messages: Option<bool>,

    /// Whether this account only receives mentions from accounts it follows (Truth Social
    /// specific).
    pub receive_only_follow_mentions: Option<bool>,

    /// Whether this account has accepted the status-edit prompt (Truth Social specific).
    pub accepted_status_edit_prompt: Option<bool>,

    /// Whether to show non-member group statuses (Truth Social specific).
    pub show_nonmember_group_statuses: Option<bool>,

    /// Custom emoji used by this account.
    #[serde(borrow)]
    pub emojis: Vec<Emoji<'a>>,

    /// Profile metadata fields.
    #[serde(borrow)]
    pub fields: Vec<Field<'a>>,

    /// Whether this account has completed TV onboarding (Truth Social specific).
    pub tv_onboarded: Option<bool>,

    /// Whether this is a TV account (Truth Social specific).
    pub tv_account: Option<bool>,

    /// Whether this account has a premium subscription (Truth Social specific).
    pub premium: Option<bool>,

    /// Whether this account is suspended.
    pub suspended: Option<bool>,

    /// Pleroma-specific account data (legacy compatibility field).
    pub pleroma: Option<Pleroma>,

    /// The account this one has moved to. Only `null` has been observed in fixtures;
    /// non-null values are rejected.
    pub moved: Option<()>,
}

/// Pleroma-specific account data.
///
/// This is a legacy compatibility field from Mastodon/Pleroma federation.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pleroma {
    /// Whether this account accepts chat messages.
    pub accepts_chat_messages: Option<bool>,
}

/// A media attachment (image, video, audio, etc.).
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct MediaAttachment<'a> {
    /// Unique identifier for this attachment.
    #[serde(with = "integer_str")]
    pub id: u64,

    /// Type of media.
    #[serde(rename = "type")]
    pub media_type: MediaType,

    /// URL to the full-size media.
    #[serde(borrow)]
    pub url: Cow<'a, str>,

    /// URL to the preview/thumbnail.
    #[serde(borrow)]
    pub preview_url: Cow<'a, str>,

    /// External video ID for embedded videos, if any.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub external_video_id: Option<Cow<'a, str>>,

    /// URL to the remote original, if federated.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub remote_url: Option<Cow<'a, str>>,

    /// URL to the remote preview, if federated.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub preview_remote_url: Option<Cow<'a, str>>,

    /// Short URL for the attachment, if any.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub text_url: Option<Cow<'a, str>>,

    /// Metadata about the media dimensions and processing, or `None` when the attachment carries
    /// none (as an `unknown` or not-yet-processed attachment can).
    #[serde(borrow)]
    pub meta: Option<MediaMeta<'a>>,

    /// Alt text description.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub description: Option<Cow<'a, str>>,

    /// Blurhash placeholder string, if available.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub blurhash: Option<Cow<'a, str>>,

    /// Processing status of this attachment.
    pub processing: Option<ProcessingStatus>,

    /// TV-specific metadata for live/scheduled TV content.
    #[serde(borrow)]
    pub tv: Option<TvMetadata<'a>>,
}

/// Type of media attachment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    /// Unsupported or unrecognized file type.
    Unknown,
    /// Static image.
    Image,
    /// Looping, silent video used for animated GIFs.
    Gifv,
    /// Video file.
    Video,
    /// Audio file.
    Audio,
    /// Truth Social TV stream.
    Tv,
}

/// Processing status of a media attachment.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProcessingStatus {
    /// Queued for processing but not yet started.
    Queued,
    /// Processing is in progress.
    Processing,
    /// Processing is complete.
    Complete,
    /// Processing failed.
    Failed,
}

/// Metadata about media dimensions.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct MediaMeta<'a> {
    /// Original size metadata.
    #[serde(borrow)]
    pub original: Option<ImageMeta<'a>>,

    /// Small/thumbnail size metadata.
    #[serde(borrow)]
    pub small: Option<ImageMeta<'a>>,

    /// Color information extracted from the media.
    #[serde(borrow)]
    pub colors: Option<MediaColors<'a>>,
}

/// Color information extracted from media.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct MediaColors<'a> {
    /// Accent color as hex string (e.g., "#c2aca1").
    #[serde(borrow)]
    pub accent: Cow<'a, str>,

    /// Background color as hex string.
    #[serde(borrow)]
    pub background: Cow<'a, str>,

    /// Foreground color as hex string.
    #[serde(borrow)]
    pub foreground: Cow<'a, str>,
}

/// Dimension metadata for an image or video.
///
/// For images, contains aspect ratio, dimensions, and size string. For videos, contains bitrate,
/// duration, frame rate, and dimensions.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ImageMeta<'a> {
    /// Aspect ratio (width / height). Present for images.
    pub aspect: Option<f64>,

    /// Height in pixels.
    pub height: Option<u64>,

    /// Size as a `WxH` string. Present for images.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub size: Option<Cow<'a, str>>,

    /// Width in pixels.
    pub width: Option<u64>,

    /// Bitrate in bits per second. Present for videos.
    pub bitrate: Option<u64>,

    /// Duration in seconds. Present for videos.
    pub duration: Option<f64>,

    /// Frame rate as a fraction string (e.g., "30000/1001"). Present for videos.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub frame_rate: Option<Cow<'a, str>>,
}

/// ID type for mentions, which can be a normal account ID or a sentinel value.
///
/// The Truth Social API uses `-99` as a sentinel value for domain-level mentions (e.g., mentioning
/// `@truthsocial.com` rather than a specific user account).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MentionId {
    /// A normal account ID.
    Account(u64),
    /// A domain-level mention (serialized as "-99").
    Domain,
}

impl<'de> serde::Deserialize<'de> for MentionId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // The ID comes as a string (e.g., "12345" or "-99"). Deserialize through `Cow` rather than
        // `&str`: a borrowed `&str` only works with input the deserializer can borrow from, so a
        // plain `&str` here would fail under owned or streaming deserializers such as
        // `serde_json::from_reader` or `from_value`. `Cow` accepts both borrowed and owned input.
        let s = Cow::<'de, str>::deserialize(deserializer)?;

        if s == "-99" {
            Ok(Self::Domain)
        } else {
            s.parse::<u64>()
                .map(Self::Account)
                .map_err(serde::de::Error::custom)
        }
    }
}

impl serde::Serialize for MentionId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Account(id) => serializer.serialize_str(&id.to_string()),
            Self::Domain => serializer.serialize_str("-99"),
        }
    }
}

/// A mention of an account or domain in a status.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mention<'a> {
    /// Account ID of the mentioned user, or a sentinel value for domain mentions.
    pub id: MentionId,

    /// Username of the mentioned user.
    #[serde(borrow)]
    pub username: Cow<'a, str>,

    /// URL to the mentioned user's profile.
    #[serde(borrow)]
    pub url: Cow<'a, str>,

    /// Account address (`username` or `username@domain`).
    #[serde(borrow)]
    pub acct: Cow<'a, str>,
}

/// A hashtag used in a status.
///
/// The [`trending`](Self::trending) fields appear inline in the JSON object. Either the required
/// [`TagTrending`] fields are all present and not null, or every trending field is absent. As in
/// the rest of the model, unknown keys are rejected.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "RawTag<'a>", bound(deserialize = "'de: 'a"))]
pub struct Tag<'a> {
    /// The hashtag text (without the # symbol).
    pub name: Cow<'a, str>,

    /// URL to view statuses with this hashtag.
    pub url: Option<Cow<'a, str>>,

    /// Hashtag ID, present as an integer string in the tags of a group from the group endpoints.
    /// The trending view carries the ID as a number in [`TagTrending::id`] instead.
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "optional_integer_str::serialize"
    )]
    pub id: Option<u64>,

    /// Per-day usage history buckets, present in the tag-search and trending views (Mastodon
    /// standard) and absent from the lightweight tags inlined in a status. Skipped when absent so
    /// an inline tag still serializes to just `{name, url}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<TagHistory<'a>>>,

    /// Recent daily usage counts, newest first (Truth Social specific). Present in the tag-search
    /// view alongside [`history`](Self::history).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_history: Option<Vec<u64>>,

    /// Number of statuses that recently used this hashtag (Truth Social specific). Present in the
    /// tag-search view.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_statuses_count: Option<u64>,

    /// Extended trending/moderation metadata, present only in the trending/admin tag view and
    /// absent from the lightweight tags inlined in a status or a group.
    #[serde(flatten)]
    pub trending: Option<TagTrending>,
}

/// The JSON form of a [`Tag`], with each [`TagTrending`] field inline and optional.
///
/// [`Tag`] deserializes through this type because flattening [`TagTrending`] would accept unknown
/// keys, and would turn any malformed trending field into a silently absent block.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTag<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    url: Option<Cow<'a, str>>,
    #[serde(borrow)]
    history: Option<Vec<TagHistory<'a>>>,
    recent_history: Option<Vec<u64>>,
    recent_statuses_count: Option<u64>,
    /// A number in the trending view, and an integer string in a group's tags.
    #[serde(default, with = "either::serde_untagged_optional")]
    id: Option<Either<u64, Cow<'a, str>>>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    last_status_at: Option<DateTime<Utc>>,
    usable: Option<bool>,
    trendable: Option<bool>,
    listable: Option<bool>,
    reviewed_at: Option<DateTime<Utc>>,
    requested_review_at: Option<DateTime<Utc>>,
    max_score: Option<serde_json::Number>,
    max_score_at: Option<DateTime<Utc>>,
}

impl<'a> TryFrom<RawTag<'a>> for Tag<'a> {
    type Error = &'static str;

    fn try_from(raw: RawTag<'a>) -> Result<Self, Self::Error> {
        let no_optional_trending = raw.reviewed_at.is_none()
            && raw.requested_review_at.is_none()
            && raw.max_score.is_none()
            && raw.max_score_at.is_none();

        let (id, trending) = match (
            raw.id,
            raw.created_at,
            raw.updated_at,
            raw.last_status_at,
            raw.usable,
            raw.trendable,
            raw.listable,
        ) {
            (
                Some(Either::Left(id)),
                Some(created_at),
                Some(updated_at),
                Some(last_status_at),
                Some(usable),
                Some(trendable),
                Some(listable),
            ) => Ok((
                None,
                Some(TagTrending {
                    id,
                    created_at,
                    updated_at,
                    last_status_at,
                    usable,
                    trendable,
                    listable,
                    reviewed_at: raw.reviewed_at,
                    requested_review_at: raw.requested_review_at,
                    max_score: raw.max_score,
                    max_score_at: raw.max_score_at,
                }),
            )),
            (None, None, None, None, None, None, None) if no_optional_trending => Ok((None, None)),
            (Some(Either::Right(id)), None, None, None, None, None, None)
                if no_optional_trending =>
            {
                id.parse()
                    .map(|id| (Some(id), None))
                    .map_err(|_| "a tag's string id must be an unsigned integer")
            }
            _ => Err(
                "a tag's trending fields must be all present and non-null with a numeric id, or all \
                 absent",
            ),
        }?;

        Ok(Self {
            name: raw.name,
            url: raw.url,
            id,
            history: raw.history,
            recent_history: raw.recent_history,
            recent_statuses_count: raw.recent_statuses_count,
            trending,
        })
    }
}

/// The extended trending/moderation fields of a [`Tag`].
///
/// These fields appear only in the trending/admin view, so they are grouped behind the single
/// `Option` in [`Tag::trending`]. The non-`Option` fields are always present in that view, and the
/// `Option` fields may be absent or null. A [`Tag`] serializes these fields inline.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TagTrending {
    /// Numeric hashtag ID.
    pub id: u64,

    /// When the hashtag was first seen.
    pub created_at: DateTime<Utc>,

    /// When the hashtag's metadata was last updated.
    pub updated_at: DateTime<Utc>,

    /// When the hashtag was most recently used.
    pub last_status_at: DateTime<Utc>,

    /// Whether the hashtag may be used.
    pub usable: bool,

    /// Whether the hashtag is allowed to trend.
    pub trendable: bool,

    /// Whether the hashtag is listed publicly.
    pub listable: bool,

    /// When the hashtag was reviewed by a moderator, if ever.
    pub reviewed_at: Option<DateTime<Utc>>,

    /// When review of the hashtag was requested, if ever.
    pub requested_review_at: Option<DateTime<Utc>>,

    /// The hashtag's peak trending score, if scored (a number; `serde_json::Number` keeps
    /// [`TagTrending`] `Eq`/`Hash`).
    pub max_score: Option<serde_json::Number>,

    /// When the hashtag reached its peak trending score, if ever.
    pub max_score_at: Option<DateTime<Utc>>,
}

/// A single day's usage bucket in a [`Tag`]'s [`history`](Tag::history).
///
/// The API sends `uses` and `accounts` as decimal strings (Mastodon convention), and `days_ago` as
/// a plain number.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct TagHistory<'a> {
    /// Start of the day this bucket covers, as a Unix timestamp (seconds) string.
    #[serde(borrow)]
    pub day: Cow<'a, str>,

    /// Number of uses of the hashtag that day.
    #[serde(with = "integer_str")]
    pub uses: u64,

    /// Number of distinct accounts that used the hashtag that day.
    #[serde(with = "integer_str")]
    pub accounts: u64,

    /// How many days before the response this bucket is (`0` is the current day). Truth Social
    /// specific.
    pub days_ago: u64,
}

/// A preview card for linked content.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Card<'a> {
    /// Unique identifier for this card, if any.
    pub id: Option<u64>,

    /// URL of the linked resource.
    #[serde(borrow)]
    pub url: Cow<'a, str>,

    /// Title of the linked resource.
    #[serde(borrow)]
    pub title: Cow<'a, str>,

    /// Description of the linked resource.
    #[serde(borrow)]
    pub description: Cow<'a, str>,

    /// Type of preview card.
    #[serde(rename = "type")]
    pub card_type: CardType,

    /// Author name, if available.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub author_name: Option<Cow<'a, str>>,

    /// Author URL, if available.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub author_url: Option<Cow<'a, str>>,

    /// Provider name (e.g., `YouTube`).
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub provider_name: Option<Cow<'a, str>>,

    /// Provider URL.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub provider_url: Option<Cow<'a, str>>,

    /// HTML for embedding, if available.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub html: Option<Cow<'a, str>>,

    /// Suggested width for embed.
    pub width: Option<u64>,

    /// Suggested height for embed.
    pub height: Option<u64>,

    /// Preview image URL.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub image: Option<Cow<'a, str>>,

    /// Blurhash for preview image.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub blurhash: Option<Cow<'a, str>>,

    /// URL for embedding, if different from main URL.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub embed_url: Option<Cow<'a, str>>,

    /// Associated links, if any.
    pub links: Option<serde_json::Value>,

    /// Associated group, if any.
    #[serde(rename = "group")]
    pub card_group: Option<serde_json::Value>,
}

/// Type of preview card.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CardType {
    /// Generic link preview.
    Link,
    /// Photo preview.
    Photo,
    /// Video preview.
    Video,
    /// Rich embed (e.g., `YouTube`).
    Rich,
}

/// A group on Truth Social.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Group<'a> {
    /// Unique identifier for this group.
    #[serde(with = "integer_str")]
    pub id: u64,

    /// Display name of the group.
    #[serde(borrow)]
    pub display_name: Cow<'a, str>,

    /// URL slug for the group.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub slug: Option<Cow<'a, str>>,

    /// Whether this is a private group.
    pub locked: Option<bool>,

    /// Whether this group is discoverable in search.
    pub discoverable: Option<bool>,

    /// Group avatar URL.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub avatar: Option<Cow<'a, str>>,

    /// Static version of the group avatar URL.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub avatar_static: Option<Cow<'a, str>>,

    /// Group header/banner URL.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub header: Option<Cow<'a, str>>,

    /// Static version of the group header URL.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub header_static: Option<Cow<'a, str>>,

    /// Group description (HTML formatted).
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub note: Option<Cow<'a, str>>,

    /// Group visibility setting.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub group_visibility: Option<Cow<'a, str>>,

    /// Visibility setting for group statuses. Only `null` has been observed in fixtures;
    /// non-null values are rejected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statuses_visibility: Option<()>,

    /// When this group was created.
    pub created_at: Option<DateTime<Utc>>,

    /// When this group was last updated, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,

    /// When this group was deleted, if it has been.
    pub deleted_at: Option<DateTime<Utc>>,

    /// Whether membership is required to view content.
    pub membership_required: Option<bool>,

    /// Number of members in the group.
    pub members_count: Option<u64>,

    /// Tags associated with this group.
    #[serde(borrow)]
    pub tags: Option<Vec<Tag<'a>>>,

    /// URL to the group's page.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub url: Option<Cow<'a, str>>,

    /// Source content for the group (raw text).
    #[serde(borrow)]
    pub source: Option<GroupSource<'a>>,

    /// Owner of the group.
    pub owner: Option<GroupOwner>,

    /// Event category name, if this group is an event (Truth Social specific). Absent in older
    /// snapshots.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub event_category_name: Option<Cow<'a, str>>,

    /// Event subcategory name, if this group is an event (Truth Social specific). Absent in older
    /// snapshots.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub event_subcategory_name: Option<Cow<'a, str>>,

    /// Search-relevance rank, present only when this group is returned by `GET /api/v2/search`
    /// (`type=groups`) and absent when the group is embedded in a status.
    #[serde(default)]
    pub position: Option<u64>,

    /// Membership details from group search. Only `null` has been observed in fixtures;
    /// non-null values are rejected.
    #[serde(default)]
    pub member_details: Option<()>,

    /// A small preview sample of member avatars (Truth Social specific), present in the
    /// single-group lookup response (`GET /api/v1/groups/:id`) and absent when the group is
    /// embedded in a status or returned by search.
    #[serde(borrow, default)]
    pub member_avatars: Option<Vec<GroupMemberAvatar<'a>>>,
}

/// A member preview in a [`Group`]'s [`member_avatars`](Group::member_avatars): the lightweight
/// account fields needed to render an avatar stack.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupMemberAvatar<'a> {
    /// Account ID of the member.
    #[serde(with = "integer_str")]
    pub id: u64,

    /// Member's handle (without a leading `@`).
    #[serde(borrow)]
    pub username: Cow<'a, str>,

    /// Member's avatar URL.
    #[serde(borrow)]
    pub avatar: Cow<'a, str>,

    /// Static version of the member's avatar URL.
    #[serde(borrow)]
    pub avatar_static: Cow<'a, str>,
}

/// Source content for a group.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupSource<'a> {
    /// Raw note/description text (not HTML).
    #[serde(borrow)]
    pub note: Cow<'a, str>,
}

/// Owner information for a group.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupOwner {
    /// Account ID of the group owner.
    #[serde(with = "integer_str")]
    pub id: u64,
}

/// A member's role within a group, as reported by `GET /api/v1/groups/:id/memberships`.
///
/// The wire form is lowercase; note that a regular member serializes as `"user"` (not `"member"`),
/// matching the endpoint's `role` query parameter.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum GroupMembershipRole {
    /// The group's owner.
    Owner,
    /// A group administrator.
    Admin,
    /// A regular member (wire value `"user"`).
    User,
}

/// A single entry from `GET /api/v1/groups/:id/memberships`.
///
/// Pairs a member's [`account`](Self::account) with their [`role`](Self::role) in the group. The
/// endpoint returns a bare array of these, so no wrapper type is needed.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupMembership<'a> {
    /// The member's role in the group.
    pub role: GroupMembershipRole,

    /// Membership ID (distinct from the account ID), string-encoded on the wire.
    #[serde(with = "integer_str")]
    pub id: u64,

    /// The member's full account.
    #[serde(borrow)]
    pub account: Account<'a>,
}

/// A poll attached to a status.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Poll<'a> {
    /// Unique identifier for this poll.
    #[serde(with = "integer_str")]
    pub id: u64,

    /// When the poll expires.
    pub expires_at: Option<DateTime<Utc>>,

    /// Whether the poll has expired.
    pub expired: bool,

    /// Whether multiple choices are allowed.
    pub multiple: bool,

    /// Total number of votes cast.
    pub votes_count: u64,

    /// Total number of unique voters.
    pub voters_count: Option<u64>,

    /// Whether the current user has voted.
    pub voted: Option<bool>,

    /// Indices of options the current user voted for.
    pub own_votes: Option<Vec<u64>>,

    /// The poll options.
    #[serde(borrow)]
    pub options: Vec<PollOption<'a>>,

    /// Custom emoji used in poll options.
    #[serde(borrow)]
    pub emojis: Vec<Emoji<'a>>,
}

/// An option in a poll.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PollOption<'a> {
    /// The option text.
    #[serde(borrow)]
    pub title: Cow<'a, str>,

    /// Number of votes for this option.
    pub votes_count: Option<u64>,
}

/// A custom emoji.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Emoji<'a> {
    /// Shortcode for the emoji (without colons).
    #[serde(borrow)]
    pub shortcode: Cow<'a, str>,

    /// URL to the emoji image.
    #[serde(borrow)]
    pub url: Cow<'a, str>,

    /// URL to the static version of the emoji.
    #[serde(borrow)]
    pub static_url: Cow<'a, str>,

    /// Whether this emoji is visible in the picker.
    pub visible_in_picker: bool,

    /// Category for organizing emoji.
    #[serde(borrow, default, with = "archivindex_serde::borrowable_str::option")]
    pub category: Option<Cow<'a, str>>,
}

/// A profile metadata field.
#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct Field<'a> {
    /// Field label/name.
    #[serde(borrow)]
    pub name: Cow<'a, str>,

    /// Field value (may contain HTML links).
    #[serde(borrow)]
    pub value: Cow<'a, str>,

    /// When this field was verified, if applicable.
    pub verified_at: Option<DateTime<Utc>>,
}

/// Deserialize JSON into a value that owns all of its strings, such as a `Status<'static>`.
///
/// [`serde_json::from_str`] lends strings from its input, which ties the result to `json`. This
/// reads `json` as a stream instead, so every string is copied.
///
/// # Errors
///
/// Returns a [`serde_json::Error`] if `json` is not valid JSON for `T`, or has trailing content.
pub fn from_str_owned<T: serde::Deserialize<'static>>(json: &str) -> serde_json::Result<T> {
    let mut deserializer = serde_json::Deserializer::from_reader(json.as_bytes());
    let value = T::deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::unreadable_literal,
        reason = "status IDs are long opaque integers; digit separators do not aid readability"
    )]

    use serde::de::Deserialize as _;

    use super::*;

    /// Decode a fixture as UTF-8 JSON, decompressing it when gzip magic bytes are present.
    fn decode_snapshot(bytes: &[u8]) -> String {
        use std::io::Read as _;
        if bytes.starts_with(&[0x1f, 0x8b]) {
            let mut text = String::new();
            flate2::read::GzDecoder::new(bytes)
                .read_to_string(&mut text)
                .expect("gzip fixture decompresses to text");
            text
        } else {
            String::from_utf8(bytes.to_vec()).expect("fixture is UTF-8 JSON")
        }
    }

    /// `last_status_at` accepts a plain date ([`Either::Left`]), a full RFC 3339 datetime
    /// ([`Either::Right`]), and `null` (Truth Social returns all three), preserving each form and
    /// round-tripping back to the same text.
    #[test]
    fn last_status_at_accepts_date_or_datetime() {
        #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
        struct Wrapper {
            #[serde(default, with = "either::serde_untagged_optional")]
            value: Option<Either<NaiveDate, DateTime<Utc>>>,
        }

        let date = NaiveDate::from_ymd_opt(2023, 5, 23).unwrap();
        let datetime = "2023-05-23T22:31:45.360Z".parse::<DateTime<Utc>>().unwrap();
        let cases = [
            (r#"{"value":"2023-05-23"}"#, Some(Either::Left(date))),
            (
                r#"{"value":"2023-05-23T22:31:45.360Z"}"#,
                Some(Either::Right(datetime)),
            ),
            (r#"{"value":null}"#, None),
        ];
        for (json, expected) in cases {
            let parsed: Wrapper = serde_json::from_str(json).expect("deserializes");
            assert_eq!(parsed.value, expected, "for {json}");
            assert_eq!(
                serde_json::to_string(&parsed).unwrap(),
                json,
                "round-trip {json}"
            );
        }

        assert!(serde_json::from_str::<Wrapper>(r#"{"value":"not a date"}"#).is_err());
    }

    /// A lightweight status tag (`{name, url}`) carries no [`TagTrending`]; a full trending-view
    /// tag flattens its extra fields into one. Both round-trip; the absent block emits no keys.
    #[test]
    fn tag_trending_flattens_optionally() {
        let lightweight: Tag<'_> =
            serde_json::from_str(r#"{"name":"Truth","url":"https://truthsocial.com/tags/Truth"}"#)
                .expect("lightweight tag");
        assert!(lightweight.trending.is_none());
        assert_eq!(
            serde_json::to_value(&lightweight).unwrap(),
            serde_json::json!({"name": "Truth", "url": "https://truthsocial.com/tags/Truth"}),
        );

        let full_json = r#"{"name":"music","url":null,"id":227,"created_at":"2022-02-16T22:38:42.721Z","updated_at":"2023-02-17T15:08:07.243Z","last_status_at":"2023-02-17T15:08:06.953Z","usable":true,"trendable":true,"listable":true,"reviewed_at":null,"requested_review_at":null,"max_score":null,"max_score_at":null}"#;
        let full: Tag<'_> = serde_json::from_str(full_json).expect("full tag");
        let trending = full.trending.as_ref().expect("trending present");
        assert_eq!(trending.id, 227);
        assert!(trending.usable && trending.trendable && trending.listable);
        assert!(trending.max_score.is_none());
        assert_eq!(
            serde_json::to_value(&full).unwrap(),
            serde_json::from_str::<serde_json::Value>(full_json).unwrap(),
        );
    }

    /// The group endpoints send each tag's ID as an integer string, without a URL or trending
    /// block. That ID was silently dropped while the trending block was flattened.
    #[test]
    fn tag_reads_group_id() {
        let json = r#"{"name":"Truth","id":"30"}"#;
        let tag: Tag<'_> = serde_json::from_str(json).expect("group tag");
        assert_eq!(tag.id, Some(30));
        assert!(tag.trending.is_none());
        assert_eq!(
            serde_json::to_value(&tag).unwrap(),
            serde_json::json!({"name": "Truth", "url": null, "id": "30"}),
        );
    }

    /// A malformed or partial trending block is an error, not a tag without trending data. When
    /// the block was flattened, each of these parsed as a tag with `trending: None`, so the schema
    /// check stayed green while the block's shape changed.
    #[test]
    fn tag_rejects_malformed_trending() {
        let full = serde_json::json!({"name":"music","url":null,"id":227,"created_at":"2022-02-16T22:38:42.721Z","updated_at":"2023-02-17T15:08:07.243Z","last_status_at":"2023-02-17T15:08:06.953Z","usable":true,"trendable":true,"listable":true,"reviewed_at":null,"requested_review_at":null,"max_score":null,"max_score_at":null});
        Tag::deserialize(full.clone()).expect("full tag");

        let lightweight = serde_json::json!({"name": "Truth", "url": null});
        for (key, value, base) in [
            ("last_status_at", Some(serde_json::Value::Null), &full),
            ("id", Some(serde_json::json!("1")), &full),
            ("usable", None, &full),
            ("usable", Some(serde_json::json!("yes")), &full),
            ("unknown", Some(serde_json::json!(1)), &full),
            ("unknown", Some(serde_json::json!(1)), &lightweight),
            ("max_score", Some(serde_json::json!(1)), &lightweight),
            ("id", Some(serde_json::json!(30)), &lightweight),
            ("id", Some(serde_json::json!("x")), &lightweight),
        ] {
            let mut tag = base.clone();
            let object = tag.as_object_mut().unwrap();
            match &value {
                Some(value) => object.insert(key.to_owned(), value.clone()),
                None => object.remove(key),
            };
            let result = Tag::deserialize(tag);
            assert!(result.is_err(), "{key} = {value:?}: {result:?}");
        }
    }

    /// A reblog (boost): the outer status carries a `reblog` with its own account and card.
    #[test]
    fn test_deserialize_reblog() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAA7WWHWJB6AP4HLEMQBVJQP6QSDQV4Q"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 107962811324118694);
        assert_eq!(status.visibility, Some(Visibility::Public));
        assert_eq!(status.account.username, "DineshDSouza");

        let reblog = status.reblog.as_ref().expect("Expected reblog");
        assert_eq!(reblog.id, 107957941707714512);
        assert_eq!(reblog.account.username, "DineshDSouza");
        assert_eq!(reblog.account.verified, Some(true));

        // Card on the reblogged status.
        let card = reblog.card.as_ref().expect("Expected card");
        assert_eq!(card.card_type, CardType::Link);
        assert_eq!(card.provider_name.as_deref(), Some("dinesh.locals.com"));
    }

    /// A video attachment with `original` and `small` metadata, a premium account, and
    /// `version` and `editable` status fields.
    #[test]
    fn test_deserialize_video_metadata() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AASQSDZXOZ2C7Z2UMTZTIJ63I33KY6A3"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 116616050796377967);
        assert_eq!(status.account.username, "OAN");
        assert_eq!(status.account.premium, Some(true));

        // Newer Status fields.
        assert_eq!(status.version.as_deref(), Some("1"));
        assert_eq!(status.editable, Some(false));
        assert!(status.edited_at.is_none());

        assert_eq!(status.media_attachments.len(), 1);
        let attachment = &status.media_attachments[0];
        assert_eq!(attachment.media_type, MediaType::Video);
        assert_eq!(attachment.processing, Some(ProcessingStatus::Complete));
        assert!(attachment.blurhash.is_some());

        let meta = attachment.meta.as_ref().expect("Expected media metadata");
        let original = meta.original.as_ref().expect("Expected original metadata");
        assert_eq!(original.bitrate, Some(4323756));
        assert!(original.duration.is_some());
        assert!(original.frame_rate.is_some());

        let small = meta
            .small
            .as_ref()
            .expect("Expected small thumbnail metadata");
        assert_eq!(small.width, Some(1067));
        assert_eq!(small.height, Some(600));
    }

    /// Deserialization from an owned JSON value must agree with deserialization from text.
    /// This catches fields that incorrectly require borrowed strings.
    #[test]
    fn status_deserializes_from_owned_value() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AASQSDZXOZ2C7Z2UMTZTIJ63I33KY6A3"
        ));
        let value: serde_json::Value = serde_json::from_str(&json).expect("snapshot is valid JSON");
        let from_value =
            Status::deserialize(value).expect("Status must deserialize from an owned Value tree");

        // Both deserialization paths must preserve the same field values.
        let from_str: Status<'_> = serde_json::from_str(&json).expect("borrowed parse");
        assert_eq!(
            serde_json::to_value(&from_value).unwrap(),
            serde_json::to_value(&from_str).unwrap(),
        );
    }

    /// Parsing from a string borrows each string without escapes, at every level: a status's own
    /// required and optional fields, its nested account, and its tags (which deserialize through
    /// `RawTag`). Borrowing is easy to lose silently, since an owned `Cow` compares equal.
    #[test]
    fn status_strings_borrow_from_the_input() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAAJOS3WOYRYLASLVLDYFDU7EE44SHR2"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("status parses");

        assert!(matches!(status.uri, Cow::Borrowed(_)));
        assert!(matches!(status.language, Some(Cow::Borrowed("en"))));
        assert!(matches!(status.account.username, Cow::Borrowed(_)));
        assert!(matches!(status.tags[0].name, Cow::Borrowed("AMC")));
    }

    /// An array of statuses streams straight into `Status`, so it borrows, but an object body is
    /// buffered through `serde_json::Value` to choose between a status and an account, so it owns.
    #[test]
    fn response_body_borrows_only_from_status_arrays() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAAJOS3WOYRYLASLVLDYFDU7EE44SHR2"
        ));
        let array = format!("[{json}]");

        let ResponseBody::Statuses(StatusContent::Multiple(statuses)) =
            serde_json::from_str(&array).expect("array parses")
        else {
            panic!("an array is a list of statuses");
        };
        assert!(matches!(statuses[0].uri, Cow::Borrowed(_)));

        let ResponseBody::Statuses(StatusContent::Single(status)) =
            serde_json::from_str(&json).expect("object parses")
        else {
            panic!("an object without a username is a status");
        };
        assert!(matches!(status.uri, Cow::Owned(_)));
    }

    /// `from_str_owned` copies every string, so its result outlives the input, and like
    /// `serde_json::from_str` it rejects trailing content.
    #[test]
    fn from_str_owned_detaches_from_the_input() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAAJOS3WOYRYLASLVLDYFDU7EE44SHR2"
        ));
        assert!(from_str_owned::<Status<'static>>(&format!("{json} []")).is_err());

        let status = from_str_owned::<Status<'static>>(&json).expect("status parses");
        drop(json);

        assert!(matches!(status.uri, Cow::Owned(_)));
        assert!(matches!(status.language, Some(Cow::Owned(_))));
    }

    /// A JSON `null` visibility deserializes to `None`, as needed for older captures.
    #[test]
    fn status_accepts_null_visibility() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAA7WWHWJB6AP4HLEMQBVJQP6QSDQV4Q"
        ));
        let mut value: serde_json::Value = serde_json::from_str(&json).expect("snapshot JSON");
        // Force the top-level visibility to null, mirroring the real captures that triggered this.
        value["visibility"] = serde_json::Value::Null;

        let status =
            Status::deserialize(value).expect("a null visibility must deserialize, not error");
        assert_eq!(status.visibility, None);
    }

    /// A status whose account carries the `pleroma` extension and whose media is a video backed by
    /// a `CardType::Video` link card (older capture: no blurhash, no processing).
    #[test]
    fn test_deserialize_pleroma_and_video_card() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AFOLBGJSXEG7R5OK2IFC24A7NDRITY7E"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 109967811792793526);
        assert_eq!(status.account.username, "realDonaldTrump");
        assert!(status.account.pleroma.is_some());

        assert_eq!(status.media_attachments.len(), 1);
        let attachment = &status.media_attachments[0];
        assert_eq!(attachment.media_type, MediaType::Video);
        assert!(attachment.blurhash.is_none());

        let meta = attachment.meta.as_ref().expect("Expected media metadata");
        let original = meta.original.as_ref().expect("Expected original metadata");
        assert_eq!(original.bitrate, Some(1276287));
        assert!(original.duration.is_some());
        assert!(original.frame_rate.is_some());

        let card = status.card.as_ref().expect("Expected card");
        assert_eq!(card.card_type, CardType::Video);
        assert!(card.id.is_none()); // Card ID can be null.
        assert_eq!(card.provider_name.as_deref(), Some("Rumble.com"));
    }

    /// A quote post: `quote_id` plus the embedded `quote` status (which itself has a video card).
    #[test]
    fn test_deserialize_quote() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAINYHXV73DIWMOPHE7372JKHHSPJBAV"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 110375104524907659);
        assert_eq!(status.account.username, "ChrisSalcedoShow");
        assert_eq!(status.quote_id, Some(110374797872162426));

        let quote = status.quote.as_ref().expect("Expected quote");
        assert_eq!(quote.id, 110374797872162426);
        assert_eq!(quote.account.username, "DevinNunes");
        assert_eq!(quote.visibility, Some(Visibility::Public));

        let card = quote.card.as_ref().expect("Expected card on quote");
        assert_eq!(card.card_type, CardType::Video);
    }

    /// A status with a suspended account in reply (tombstoned, deleted).
    #[test]
    fn test_deserialize_suspended_account() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/LR2C2SGUH5OHC7IZCTO7R54W3MQTHOKO"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        // Verify top-level status
        assert_eq!(status.id, 107838474127241768);
        assert_eq!(status.account.username, "trumpcash47");
        assert_eq!(status.account.verified, Some(true));

        // Verify `in_reply_to` has a suspended account
        let reply = status.in_reply_to.as_ref().expect("Expected in_reply_to");
        assert_eq!(reply.id, 107838404966019561);
        assert_eq!(reply.account.username, "samanthamarika");
        assert_eq!(reply.account.suspended, Some(true));

        // Verify the reply has a tombstone (deleted)
        let tombstone = reply.tombstone.as_ref().expect("Expected tombstone");
        assert_eq!(tombstone.reason, TombstoneReason::Deleted);
        assert!(reply.deleted_at.is_some());
    }

    /// A reply to a deleted status: the `in_reply_to` is tombstoned with placeholder content, while
    /// the top-level status is intact.
    #[test]
    fn test_deserialize_deleted_reply() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/ADDTCWFH37EYIWPVHC7LUXQ6NND5YAZM"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 110651508324557686);
        assert_eq!(status.account.username, "BrianCates");
        assert!(status.tombstone.is_none()); // Top-level is not deleted.

        let reply = status.in_reply_to.as_ref().expect("Expected in_reply_to");
        assert_eq!(reply.id, 110651505508533166);
        assert_eq!(reply.account.username, "BrianCates");

        let tombstone = reply.tombstone.as_ref().expect("Expected tombstone");
        assert_eq!(tombstone.reason, TombstoneReason::Deleted);
        assert!(reply.deleted_at.is_some());
        assert_eq!(reply.content, "Unavailable.");
    }

    /// An older-format capture: newer optional status/account fields are absent and so deserialize
    /// to `None`.
    #[test]
    fn test_deserialize_older_format() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/ACIOAULA7PEKDUZPYMJHCW3OSVWW6OKY"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 108742189401812078);
        assert_eq!(status.account.username, "realDonaldTrump");

        // Newer status fields are absent in this older capture.
        assert!(status.sponsored.is_none());
        assert!(status.upvotes_count.is_none());
        assert!(status.downvotes_count.is_none());
        assert!(status.votable.is_none());

        // Newer account fields are likewise absent.
        assert!(status.account.unauth_visibility.is_none());
        assert!(status.account.tv_onboarded.is_none());
    }

    /// A sponsored status: `sponsored`, `metrics`, and `Visibility::Unlisted`.
    #[test]
    fn test_deserialize_sponsored_with_metrics() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AJ5CMWDL6TXYHJOJ4E56WMOXDJSTPNJQ"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 110941119092973173);
        assert_eq!(status.account.username, "epochtimes");
        assert_eq!(status.visibility, Some(Visibility::Unlisted));
        assert_eq!(status.sponsored, Some(true));

        let metrics = status.metrics.as_ref().expect("Expected metrics");
        assert!(metrics.impression.is_some());
        assert!(metrics.expires_at.is_some());
    }

    /// A group post: `Visibility::Group` plus the embedded `group` with its extended metadata,
    /// source, and owner.
    #[test]
    fn test_deserialize_group_post() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/ACEWLCRAJMERJRZS4UMDE36SPTSJ2UVP"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 110617267048515904);
        assert_eq!(status.account.username, "elenochle");
        assert_eq!(status.visibility, Some(Visibility::Group));

        let group = status.group.as_ref().expect("Expected group");
        assert_eq!(group.id, 110353664589615240);
        assert_eq!(group.display_name, "News Blast Links");
        assert_eq!(group.locked, Some(false));
        assert_eq!(group.discoverable, Some(true));
        assert_eq!(group.membership_required, Some(true));
        assert_eq!(group.members_count, Some(3106));

        assert!(group.avatar_static.is_some());
        assert!(group.header_static.is_some());
        assert!(group.source.is_some());

        let owner = group.owner.as_ref().expect("Expected group owner");
        assert_eq!(owner.id, 108318222345080429);
    }

    /// A TV status: `status.tv` channel metadata, a `tv_account`, and a `MediaType::Tv` attachment
    /// carrying its own TV metadata.
    #[test]
    fn test_deserialize_tv_status() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAGCDRPMZ65KGPCTAGSZZMJCDHKKR3HA"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 115806398111692077);
        assert_eq!(status.account.username, "NewsMax");
        assert_eq!(status.account.tv_account, Some(true));

        let tv = status.tv.as_ref().expect("Expected tv metadata on status");
        assert_eq!(tv.channel_id, 5);
        assert_eq!(tv.name, "Newsfront S1:E4 – Marxism in America");
        assert_eq!(tv.pltv_timespan, 259200000);

        assert_eq!(status.media_attachments.len(), 1);
        let attachment = &status.media_attachments[0];
        assert_eq!(attachment.media_type, MediaType::Tv);
        assert_eq!(attachment.processing, Some(ProcessingStatus::Complete));
        assert!(attachment.tv.is_some());
    }

    /// A two-level reply chain: `in_reply_to` nested inside `in_reply_to`.
    #[test]
    fn test_deserialize_reply_chain() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AARDSO2IWOC37RSCMF66KUVLXHOS2OI6"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 115396476953854230);
        assert_eq!(status.account.username, "redneck");

        let reply1 = status.in_reply_to.as_ref().expect("Expected in_reply_to");
        assert_eq!(reply1.id, 115396468279137711);

        let reply2 = reply1
            .in_reply_to
            .as_ref()
            .expect("Expected nested in_reply_to");
        assert_eq!(reply2.id, 115396458435977605);
        assert_eq!(reply2.visibility, Some(Visibility::Public));
    }

    /// A status with many mentions (each an account reference) and an image attachment.
    #[test]
    fn test_deserialize_many_mentions() {
        let json = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AANYHQREUL55MLX5HNJ2LEEZZXXL7NDY"
        ));
        let status: Status<'_> = serde_json::from_str(&json).expect("Failed to deserialize status");

        assert_eq!(status.id, 108407676531874529);
        assert_eq!(status.account.username, "RatDog2020");

        assert_eq!(status.mentions.len(), 15);
        // Each mention resolves to an account reference.
        assert!(matches!(status.mentions[0].id, MentionId::Account(_)));

        assert_eq!(status.media_attachments.len(), 1);
        let attachment = &status.media_attachments[0];
        assert_eq!(attachment.media_type, MediaType::Image);
        assert_eq!(attachment.processing, Some(ProcessingStatus::Complete));
        assert!(attachment.blurhash.is_some());
    }

    /// The `Visibility` variants that do not appear in the curated snapshot corpus are covered by
    /// parsing their wire representations directly.
    #[test]
    fn test_visibility_wire_variants() {
        assert_eq!(
            serde_json::from_str::<Visibility>("\"self\"").unwrap(),
            Visibility::Self_
        );
        assert_eq!(
            serde_json::from_str::<Visibility>("\"private\"").unwrap(),
            Visibility::Private
        );
        assert_eq!(
            serde_json::from_str::<Visibility>("\"direct\"").unwrap(),
            Visibility::Direct
        );
    }

    /// Build synthetic account JSON with the fields required by [`Account`].
    /// Optional fields are omitted and default to `None`.
    fn synthetic_account_json(id: u64, username: &str) -> String {
        format!(
            r#"{{"id":"{id}","username":"{username}","acct":"{username}",
                "display_name":"Synthetic {username}","locked":false,"bot":false,"group":false,
                "created_at":"2024-01-01T00:00:00.000Z","note":"<p></p>",
                "url":"https://truthsocial.com/@{username}","avatar":"https://example.test/a.png",
                "avatar_static":"https://example.test/a.png","header":"https://example.test/h.png",
                "header_static":"https://example.test/h.png","followers_count":0,
                "following_count":0,"statuses_count":0,"verified":false,"location":"","website":"",
                "emojis":[],"fields":[]}}"#
        )
    }

    /// A `GET /api/v1/groups/:id/memberships` payload is a bare array of [`GroupMembership`]; each
    /// entry decodes its role, its string-encoded membership ID (distinct from the account ID), and
    /// the nested [`Account`]. Parsing and re-parsing the serialized form yields an equal value.
    #[test]
    fn group_memberships_parse_role_id_and_account() {
        let json = format!(
            r#"[{{"role":"owner","id":"1001","account":{owner}}},
                {{"role":"admin","id":"1002","account":{admin}}},
                {{"role":"user","id":"1003","account":{member}}}]"#,
            owner = synthetic_account_json(11, "placeholder_owner"),
            admin = synthetic_account_json(12, "placeholder_admin"),
            member = synthetic_account_json(13, "placeholder_member"),
        );

        let members = serde_json::from_str::<Vec<GroupMembership<'_>>>(&json)
            .expect("memberships deserialize");

        assert_eq!(
            members.iter().map(|m| m.role).collect::<Vec<_>>(),
            [
                GroupMembershipRole::Owner,
                GroupMembershipRole::Admin,
                GroupMembershipRole::User,
            ]
        );
        // Membership IDs are distinct from the members' account IDs.
        assert_eq!(
            members.iter().map(|m| m.id).collect::<Vec<_>>(),
            [1001, 1002, 1003]
        );
        assert_eq!(
            members.iter().map(|m| m.account.id).collect::<Vec<_>>(),
            [11, 12, 13]
        );
        assert_eq!(members[2].account.username, "placeholder_member");

        // Semantic round-trip: `Account` re-emits its optional fields as `null`, so the bytes
        // differ, but reparsing the serialized form must reproduce the same value.
        let serialized = serde_json::to_string(&members).unwrap();
        let reparsed = serde_json::from_str::<Vec<GroupMembership<'_>>>(&serialized)
            .expect("serialized memberships reparse");
        assert_eq!(reparsed, members);
    }

    /// [`ResponseBody`] accepts single statuses, status arrays, and accounts, and rejects
    /// objects that match neither schema.
    #[test]
    fn response_body_dispatches_status_array_and_account() {
        // A single status object.
        let single = decode_snapshot(include_bytes!(
            "../../tests/data/wbm/snapshots/AAA7WWHWJB6AP4HLEMQBVJQP6QSDQV4Q"
        ));
        let ResponseBody::Statuses(StatusContent::Single(status)) =
            serde_json::from_str(&single).expect("single status is a response body")
        else {
            panic!("expected a single-status response body");
        };
        assert_eq!(status.id, 107962811324118694);

        // An array of statuses reuses the same object twice.
        let value: serde_json::Value = serde_json::from_str(&single).unwrap();
        let array = serde_json::to_string(&serde_json::json!([value, value])).unwrap();
        let ResponseBody::Statuses(StatusContent::Multiple(statuses)) =
            serde_json::from_str(&array).expect("status array is a response body")
        else {
            panic!("expected a multi-status response body");
        };
        assert_eq!(statuses.len(), 2);

        // An account object dispatches on its `username` field.
        let account_json = synthetic_account_json(42, "placeholder_account");
        let ResponseBody::Account(account) =
            serde_json::from_str(&account_json).expect("account is a response body")
        else {
            panic!("expected an account response body");
        };
        assert_eq!(account.id, 42);
        assert_eq!(account.username, "placeholder_account");

        // An object that is neither a status nor an account is rejected.
        assert!(serde_json::from_str::<ResponseBody<'_>>(r#"{"unexpected":true}"#).is_err());
    }

    /// Membership roles round-trip as lowercase strings, with `"user"` for regular members.
    /// Unknown roles and extra fields are rejected.
    #[test]
    fn group_membership_role_wire_and_strictness() {
        for (wire, role) in [
            ("owner", GroupMembershipRole::Owner),
            ("admin", GroupMembershipRole::Admin),
            ("user", GroupMembershipRole::User),
        ] {
            let quoted = format!("\"{wire}\"");
            assert_eq!(
                serde_json::from_str::<GroupMembershipRole>(&quoted).unwrap(),
                role
            );
            assert_eq!(serde_json::to_string(&role).unwrap(), quoted);
        }

        // "member" is the UI vocabulary, not the wire value, so it must not parse.
        assert!(serde_json::from_str::<GroupMembershipRole>("\"member\"").is_err());

        // `deny_unknown_fields` rejects any key beyond {role,id,account}, even with a valid
        // account.
        let account = synthetic_account_json(1, "placeholder");
        let valid = format!(r#"{{"role":"user","id":"1","account":{account}}}"#);
        assert!(serde_json::from_str::<GroupMembership<'_>>(&valid).is_ok());
        let extra = format!(r#"{{"role":"user","id":"1","account":{account},"unexpected":true}}"#);
        assert!(serde_json::from_str::<GroupMembership<'_>>(&extra).is_err());
    }

    /// Mention IDs deserialize equally from text, a reader, and an owned JSON value.
    #[test]
    fn mention_id_parses_in_owned_and_borrowed_modes() {
        let expected = MentionId::Account(12345);
        assert_eq!(
            serde_json::from_str::<MentionId>("\"12345\"").unwrap(),
            expected
        );
        assert_eq!(
            serde_json::from_reader::<_, MentionId>(b"\"12345\"".as_slice()).unwrap(),
            expected
        );
        assert_eq!(
            serde_json::from_value::<MentionId>(serde_json::json!("12345")).unwrap(),
            expected
        );
        // The domain sentinel resolves the same way in a non-borrowing deserializer.
        assert_eq!(
            serde_json::from_reader::<_, MentionId>(b"\"-99\"".as_slice()).unwrap(),
            MentionId::Domain
        );
    }
}
