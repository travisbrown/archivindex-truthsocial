//! Regression coverage for statuses extracted from Wayback Machine captures.

use archivindex_truthsocial::model::{
    ResponseBody, Status, StatusContent, TombstoneReason, TvMetadata,
};
use chrono::Datelike;
use serde::de::Deserialize as _;

// Each fixture contains one status from the capture identified by the accompanying digest.
const FIXTURES: &[(&str, &str)] = &[
    (
        "BJKWCYNZWSZFTPHKP2XAFGHYISICNGJ2",
        include_str!("data/statuses/109633016411287107.json"),
    ),
    (
        "EFNJ2HAJ3EA3F73RRM3P2EWWO6DRM3JD",
        include_str!("data/statuses/114433364721798299.json"),
    ),
    (
        "NGII3FVRMT6QOOEXTOF656VUOLUUGYNY",
        include_str!("data/statuses/111330937777141825.json"),
    ),
    (
        "OGZCLGPPPQROGFECY7OVLVLDCMYWNOEA",
        include_str!("data/statuses/111138816391236375.json"),
    ),
    (
        "OS7AO3FIOJ3REZKOKJJPXV5JPY6XSMY2",
        include_str!("data/statuses/111049050553468868.json"),
    ),
    (
        "X3Y3QS2BZEPJIP4KKEVFMY43OBUUYXRJ",
        include_str!("data/statuses/116297711156351192.json"),
    ),
];

#[test]
fn captured_statuses_parse_in_borrowed_and_owned_modes() {
    for &(digest, json) in FIXTURES {
        let status: Status<'_> =
            serde_json::from_str(json).unwrap_or_else(|error| panic!("{digest}: {error}"));
        let ResponseBody::Statuses(StatusContent::Single(buffered)) =
            serde_json::from_str(json).expect("buffered response parses")
        else {
            panic!("{digest}: expected a single status");
        };
        assert_eq!(status, *buffered);
        let serialized = serde_json::to_value(&status).expect("status serializes");
        assert_eq!(Status::deserialize(serialized).unwrap(), status);

        match status.id {
            109_633_016_411_287_107 => {
                assert_eq!(status.in_reply_to.unwrap().account.followers_count, -2);
            }
            114_433_364_721_798_299 => {
                assert_eq!(
                    status.quote.unwrap().tombstone.unwrap().reason,
                    TombstoneReason::Muted
                );
            }
            111_330_937_777_141_825 => {
                let group = status.group.unwrap();
                assert_eq!(group.id, 110_492_799_375_986_293);
                assert_eq!(group.statuses_visibility, None);
                assert_eq!(group.updated_at, None);
            }
            111_138_816_391_236_375 => {
                assert_eq!(status.in_reply_to.unwrap().account.following_count, -1);
            }
            111_049_050_553_468_868 => {
                assert_eq!(status.in_reply_to.unwrap().account.followers_count, -1);
            }
            116_297_711_156_351_192 => {
                let reply = status.in_reply_to.unwrap();
                let tv = reply.tv.unwrap();
                assert_eq!(tv.start_time.year(), 2024);
                assert_eq!(tv.end_time.year(), 57_564);
                assert_eq!(tv.end_time.timestamp(), 1_754_407_547_000);
                assert_eq!(reply.media_attachments[0].tv.as_ref(), Some(&tv));
            }
            id => panic!("unexpected fixture status {id}"),
        }
    }
}

#[test]
fn tv_timestamps_validate_dates_and_accept_signed_years() {
    let status: serde_json::Value = serde_json::from_str(FIXTURES[5].1).unwrap();
    let tv = &status["in_reply_to"]["tv"];
    for field in ["start_time", "end_time"] {
        for (timestamp, valid) in [
            ("2024-08-05T11:25:47.000000Z", true),
            ("57564-12-17T21:43:20.000000Z", true),
            ("+57564-12-17T21:43:20Z", true),
            ("-0001-12-17T21:43:20Z", true),
            ("57564-02-30T21:43:20Z", false),
            ("57564-12-17T21:43:20", false),
            ("999999-12-17T21:43:20Z", false),
            ("invalid", false),
        ] {
            let mut value = tv.clone();
            value[field] = timestamp.into();
            let json = serde_json::to_string(&value).unwrap();
            assert_eq!(
                serde_json::from_str::<TvMetadata<'_>>(&json).is_ok(),
                valid,
                "{field}: {timestamp}"
            );
            assert_eq!(
                TvMetadata::deserialize(value).is_ok(),
                valid,
                "{field}: {timestamp}"
            );
        }
    }
}
