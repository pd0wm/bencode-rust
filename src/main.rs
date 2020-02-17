extern crate clap;

use std::fs;

use clap::{Arg, App};

use bittorrent::bdecode::parse;

fn main() {
    let matches = App::new("Simple torrent client")
        .arg(Arg::with_name("input").required(true))
        .get_matches();

    let filename = matches.value_of("input").unwrap();
    let contents = fs::read(filename).expect("Error reading torrent file");
    let torrent = parse(&contents).expect("Error parsing torrent").1;

    // Main dict
    let torrent = torrent.get_dict();
    let comment = torrent.get("comment").unwrap().get_string();
    println!("torrent.comment: {}", comment);

    let announce = torrent.get("announce").unwrap().get_string();
    println!("torrent.announce: {}", announce);


    // Info dict
    let info = torrent.get("info").unwrap().get_dict();
    let name = info.get("name").unwrap().get_string().to_string();
    println!("info.name: {}", name);

    let piece_length = info.get("piece length").unwrap().get_number();
    println!("info.piece length: {}", piece_length);

    let length = info.get("length").unwrap().get_number();
    println!("info.length: {}", length);

    let pieces = info.get("pieces").unwrap().get_bytes();
    println!("len(info.pieces): {}", pieces.len());
}
