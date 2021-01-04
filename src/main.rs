use rustnes::{NES, ROM};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom = ROM::load("nestest.nes")?;

    let mut nes = NES::default();
    nes.load(rom);

    nes.power_on();

    nes.nestest();

    Ok(())
}
