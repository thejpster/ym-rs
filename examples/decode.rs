extern crate ym;

use std::io::Read;

fn main() -> std::io::Result<()> {
    let filename = std::env::args().nth(1).unwrap();
    let mut f = std::fs::File::open(&filename)?;
    let mut data = Vec::new();
    f.read_to_end(&mut data)?;
    println!("Read {} bytes from {}.", data.len(), filename);

    let player = ym::YmFile::new(&data).unwrap();

    println!("Num VBLs: {}", player.num_vbl());
    println!("Song Attributes: {}", player.song_attributes());
    println!("Num Digi Drums: {}", player.num_digi_drums());
    println!("External Frequency: {} Hz", player.external_frequency());
    println!("Player Frequency: {} Hz", player.player_frequency());
    println!("VBL Loop Number: {}", player.vbl_loop_number());
    println!("Num Registers: {}", player.num_registers());

    for idx in 0..player.num_registers() {
        println!("Register {}: {:?}", idx, player.register(idx));
    }

    Ok(())
}
