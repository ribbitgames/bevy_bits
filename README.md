# Introduction
Repo for the development of bevy bits, which are micro games developped with the [Bevy game engine](https://bevyengine.org/) for the Ribbit project.

# Beginner setup
- Install [Visual Studio Code](https://code.visualstudio.com/download)
- Install [Rust](https://www.rust-lang.org/tools/install)
- Install llvm-tools, by running ```rustup component add llvm-tools```
- Open Visual Studio code and install all suggested extensions

# Launch your bit
In VS Code terminal, enter 

```batch
cargo run --bin your_bit
```

### Tips and tricks
While you're iterating on the code, prefer running ```cargo clippy``` instead of ```cargo build```. It's much faster.

Always run ```cargo clippy``` before submitting, fix all the warnings discovered there.

# Documentation
- [Interactive Rust book](https://rust-book.cs.brown.edu/)
- [Bevy book](https://bevy-cheatbook.github.io/tutorial.html)

# Nomenclature
- Bit : It's a small game that has a length from a couple of seconds to 30 seconds.
- Ribbit : The platform. It controls the flow between the bits.

# License
This repository is free, open source and permissively licensed. All code in this repository is dual-licensed under either:

 - MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
 - Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer!

# Your contributions
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
