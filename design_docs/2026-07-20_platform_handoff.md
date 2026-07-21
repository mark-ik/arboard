# Hand-off: implement custom clipboard formats on macOS / X11 / Wayland

> **DONE 2026-07-20.** macOS (`f40763b`) and Linux X11 + Wayland (`fac2660`) were
> implemented on their hosts via this note. Kept for the record and as the model
> for future per-host hand-offs.

You are picking this up on a Mac or Linux host. Your job is to implement, for
**this** platform, the two clipboard additions this fork introduces, then verify
them on the real OS clipboard here (which the Windows author could not do). This
note is self-contained; you do not need the originating conversation.

## What this fork is

`mark-ik/arboard`, branch `custom-formats`, forks 1Password/arboard 3.6.1. It
adds, additively (nothing existing changes), what genet's shared clipboard
capability needs and stock arboard lacks: arbitrary MIME formats, and holding
several representations at once.

New public API (`src/lib.rs`, `src/common.rs`):

```rust
pub struct CustomItem { pub media_type: String, pub data: Vec<u8> }   // one MIME -> bytes
pub struct ClipboardData<'a> { text, html, image, custom: Vec<CustomItem> }
impl Clipboard {
    pub fn set_data(&mut self, data: &ClipboardData) -> Result<()>;   // all reps, one session
    pub fn get_custom(&mut self, media_type: &str) -> Result<Vec<u8>>;
}
```

These dispatch to per-platform `Get`/`Set` methods `fn custom(self, media_type)`
and `fn data(self, &ClipboardData)`.

## What is done, and what you are doing

- **Windows is implemented and verified on-host** (`src/platform/windows.rs`,
  `Set::data` + `Get::custom`). Read it first: it is the reference shape. It
  opens the clipboard once, empties once, then writes each representation with a
  non-clearing set (`register_format(mime)` + `set_without_clear`) so they
  coexist, and reads a custom format by registering its name and reading its
  bytes.
- **macOS, X11, and Wayland currently return an explicit "not yet implemented on
  this platform" error** so the crate builds everywhere. Your task is to replace
  the stub for **this** platform with a real implementation, following the
  existing `text` / `html` / `image` / `file_list` methods in the same file.

The stubs to replace:

- **macOS** — `src/platform/osx.rs`: `Get::custom` and `Set::data`. Use
  `NSPasteboard`: `declareTypes` with each representation's MIME string as a
  pasteboard type, `setData:forType:` per representation for `set_data`; read via
  `dataForType:` for `get_custom`. The existing `text` / `image` / `file_list`
  methods in this file show the pasteboard access pattern.
- **X11** — `src/platform/linux/x11.rs` (plus the dispatch in
  `src/platform/linux/mod.rs`, where the stub lives now). The selection owner
  already answers requests for any TARGET; extend the owned set with the custom
  target atoms and serve their bytes for `set_data`, and request a target atom
  for `get_custom`. Follow the existing set/get in `x11.rs`.
- **Wayland** — `src/platform/linux/wayland.rs`, behind the
  `wayland-data-control` feature. A data source advertises multiple mime types;
  add the custom ones. Note: without this feature, arboard on Wayland falls back
  to X11 via XWayland, so implementing the X11 path already covers a Wayland
  laptop for a first pass.

Design notes and the full per-platform plan:
`design_docs/2026-07-20_custom_formats_plan.md`.

## How to verify (do this on this host)

```sh
cargo test --test custom_formats -- --ignored
```

`tests/custom_formats.rs` writes one `set_data` of text + html + image + a custom
`application/x-arboard-test` format, then reads each back
(`get_text` / `get().html()` / `get_image` / `get_custom`), and asserts an
unwritten MIME type is absent. It restores whatever text was on the clipboard
first, so it will not clobber your copy buffer. It is `#[ignore]`d because it
needs a real display and clipboard. Green here means this platform is done.

## Dependencies

- **Rust** toolchain (stable is fine; the crate builds on the arboard MSRV).
- **A running display / clipboard** to run the ignored test (an X server on
  Linux; Wayland works via XWayland by default).
- **X11 build**: arboard's X11 backend is `x11rb` (largely pure-Rust); a normal
  desktop has what it needs. If a build complains about xcb, install the distro's
  xcb dev package (`libxcb1-dev` on Debian/Mint, `libxcb-devel` on Fedora).
- **Native Wayland** (only if you implement `wayland.rs`): build with
  `--features wayland-data-control`, which pulls `wl-clipboard-rs` and wants the
  Wayland client libraries present.
- **macOS**: no extra system packages; the pasteboard bindings are `objc2`.

## When done

Commit and push to `origin custom-formats` (follow the repo's `CLAUDE.md`
conventions for commit messages). If you implement more than one platform, one
commit per platform is fine. Leave the other platforms' stubs untouched.

## Not your job (handled on the Windows side)

Pointing genet-clipboard's `SystemClipboard` at this fork (real multi-format
writes + `Mime::Custom`) and wiring Hocket's audio interchange are Windows-
verifiable and are being done separately. You are only implementing and verifying
this host's platform clipboard code.
