# Bencode rust

Parse .torrent file:

```rust
use std::fs;
use bittorrent::bdecode::parse;

fn main() {
    let contents = fs::read("assets/debian-10.3.0-amd64-netinst.iso.torrent").unwrap();
    let torrent = parse(&contents).unwrap().1.get_dict();
    let comment = torrent.get("comment").unwrap().get_string(); // "Debian CD from cdimage.debian.org"
}
```
See main.rs for more examples on how to use the parser.
