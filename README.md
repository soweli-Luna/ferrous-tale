# FerrousTale

 A simple slide based interactive story game engine written entirely in Rust.

## Building from source

 These instructions assume you are already familiar with building from Rust source.

 FerrousTale supports packaging story assets statically into the binary, in case thats desired for easier distribution. This is the default mode FerrousTale will compile in, and the story should be placed in `assets/story/`.

 To build in portable mode, pass `--features portable` to cargo.
