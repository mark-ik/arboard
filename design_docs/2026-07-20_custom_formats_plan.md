# arboard fork: custom formats and multi-format writes

Fork of 1Password/arboard 3.6.1 (`mark-ik/arboard`, branch `custom-formats`).
Adds what genet's clipboard capability (P3) needs and stock arboard lacks:
arbitrary MIME formats, and holding several representations at once.

## Status (2026-07-20)

All four platforms implemented. Windows (`b8c1e11`) was written and verified
on-host by the Windows author; macOS (`f40763b`) and Linux X11 + Wayland
(`fac2660`) were implemented on their own hosts through the platform hand-off
note, each following the Windows reference and the same `custom_formats` on-host
test. Verify each with `cargo test --test custom_formats -- --ignored` on that
host.

## Why

arboard writes one representation per `set` (each empties the clipboard), and it
hardcodes text / html / image / file-list. The OS clipboards underneath are all
MIME-typed (Win32 registered formats, X11 targets, Wayland mime types, macOS
`NSPasteboard` UTIs), so exposing arbitrary types and a single multi-format write
is additive, not a rewrite. Upstream's 2021 `any-format` branch proved the
concept (`CustomItem`, `set_custom`, `get_all`) but predates the `platform/`
layout and cannot be merged; it is a design reference only.

## API (additive; nothing existing changes)

```rust
/// One arbitrary representation: a MIME type and its bytes.
pub struct CustomItem { pub media_type: String, pub data: Vec<u8> }

/// A multi-representation payload, written to the clipboard in one session so
/// the representations coexist (unlike the one-at-a-time `set_*` methods).
#[derive(Default)]
pub struct ClipboardData<'a> {
    pub text: Option<Cow<'a, str>>,
    pub html: Option<Cow<'a, str>>,
    pub image: Option<ImageData<'a>>,   // cfg image-data
    pub custom: Vec<CustomItem>,        // registered MIME formats
}

impl Clipboard {
    /// Place every representation in `data` on the clipboard at once.
    pub fn set_data(&mut self, data: &ClipboardData) -> Result<(), Error>;
    /// Read one arbitrary representation by MIME type.
    pub fn get_custom(&mut self, media_type: &str) -> Result<Vec<u8>, Error>;
}
```

`get_all` (enumerate every present format) is a later refinement; a consumer that
knows which MIME types it wants uses `get_custom`, which is enough for
Hocket-to-Hocket and audio paste.

## Per-platform implementation

Each platform's `Get`/`Set` gains `custom` / `data`, following the existing
methods in the same file.

- **Windows** (`platform/windows.rs`): `set_data` opens once, `empty()` once,
  then writes each representation with `set_without_clear` (text as
  `CF_UNICODETEXT`, html as the registered "HTML Format", image via the existing
  `add_png_file` + `add_cf_dibv5`, each custom via `register_format(mime)` +
  `set_without_clear`). `get_custom` = `register_format` + `is_format_avail` +
  `raw::get_vec`. Verifiable on Windows; done first.
- **X11** (`platform/linux/x11.rs`): the selection owner already answers any
  requested target; add the custom targets to the owned set and serve their
  bytes. Natural fit.
- **Wayland** (`platform/linux/wayland.rs`): a data source advertises multiple
  mime types; add the custom ones.
- **macOS** (`platform/osx.rs`): `NSPasteboard` `declareTypes` + `setData:forType:`
  with the MIME string as a UTI; read via `dataForType:`.

## Verification (across the three hosts)

An ignored `#[test]` round-trips a `set_data` of text + html + image + a custom
`application/x-arboard-test` format, reads each back (`get_text` / `get().html()`
/ `get_image` / `get_custom`), and restores the prior clipboard. Run per host:

- Windows: verified here.
- macOS (iMac), Fedora (Wayland), Mint (X11): `cargo test --features image-data
  -- --ignored` on each, once its `Set::data` / `Get::custom` land.

Until a platform's methods land, it returns an explicit `Error::unknown("custom
formats not yet implemented on this platform")` rather than a silent no-op or a
panic, so the crate builds everywhere and the gap is loud.

## Interop note

A custom format named after a MIME type is app-to-app: two apps that agree on
`application/x-hocket-loop` round-trip losslessly. Universal DAW audio paste
wants the platform-native audio format (`CF_WAVE`, etc.) alongside; that is a
follow-on, not this fork's first cut.

## Downstream

genet-clipboard's `SystemClipboard::write` maps its `ClipboardItem` to
`ClipboardData` and calls `set_data` (real simultaneous text+image+custom);
`read` keeps probing text/html/image/files and adds `get_custom` for known
custom types. Then Hocket's audio interchange lands on top.
