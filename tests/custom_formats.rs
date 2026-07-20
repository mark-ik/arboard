//! On-host verification of the fork's additions: a single `set_data` that places
//! several representations at once, each readable back, including an arbitrary
//! custom MIME format via `get_custom`.
//!
//! Ignored by default because it touches the real OS clipboard and needs a
//! display. Run per host with:
//!   cargo test --test custom_formats -- --ignored
//! It restores whatever text was on the clipboard first, so it does not clobber
//! the user's copy buffer.

use std::borrow::Cow;

use arboard::{Clipboard, ClipboardData, CustomItem};

#[test]
#[ignore = "touches the real OS clipboard; run locally with --ignored"]
fn set_data_writes_coexisting_formats_that_all_read_back() {
    let mut clipboard = Clipboard::new().expect("a clipboard on this host");
    let restore = clipboard.get_text().ok();

    let mut data = ClipboardData {
        text: Some(Cow::Borrowed("a loop")),
        html: Some(Cow::Borrowed("<b>a loop</b>")),
        custom: vec![CustomItem {
            media_type: "application/x-arboard-test".to_string(),
            data: vec![1, 2, 3, 4],
        }],
        ..Default::default()
    };
    #[cfg(feature = "image-data")]
    {
        data.image = Some(arboard::ImageData {
            width: 2,
            height: 1,
            bytes: Cow::Owned(vec![255, 0, 0, 255, 0, 255, 0, 255]),
        });
    }

    clipboard.set_data(&data).unwrap();

    // Every representation coexists and reads back from the one write.
    assert_eq!(clipboard.get_text().unwrap(), "a loop");
    assert_eq!(clipboard.get().html().unwrap(), "<b>a loop</b>");
    assert_eq!(
        clipboard.get_custom("application/x-arboard-test").unwrap(),
        vec![1, 2, 3, 4]
    );
    #[cfg(feature = "image-data")]
    {
        let image = clipboard.get_image().unwrap();
        assert_eq!((image.width, image.height), (2, 1));
    }

    // A MIME type that was never written is absent, not a stale hit.
    assert!(clipboard.get_custom("application/x-not-written").is_err());

    match restore {
        Some(text) => clipboard.set_text(text).unwrap(),
        None => clipboard.clear().unwrap(),
    }
}
