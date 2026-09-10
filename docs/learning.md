# Learning notes

## Starting again

Plust began as a hands-on exploration of video playback. The code already spans Rust, Swift, Kotlin, and JavaScript. The next step is to rebuild a clear understanding of how it works, starting with the parser.

### First investigation: one MP4 box

Start at [`get_header_data`](../plust-core/src/fmp4.rs), then follow how [`get_atom_range`](../plust-core/src/fmp4.rs) walks the bytes.

Questions to answer with a small, known sample:

- What does each byte in the box header represent?
- Where does the box end, and where does the next one begin?
- What assumptions does the current code make about sizes and available bytes?
- What should happen when a header is incomplete or a size is invalid?

Record the observed bytes, an explanation, and a small experiment before changing the parser. This investigation has not been completed yet.
