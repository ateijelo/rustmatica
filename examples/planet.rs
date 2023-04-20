//! Example taken from [litemapy README](https://github.com/SmylerMC/litemapy#example)

use rustmatica::{util::Vec3, BlockState, Litematic, Region};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut region = Region::new("Planet".into(), Vec3::new(0, 0, 0), Vec3::new(21, 21, 21));

    for (pos, _) in region.clone().blocks() {
        if (((pos.x - 10).pow(2) + (pos.y - 10).pow(2) + (pos.z - 10).pow(2)) as f64)
            .sqrt()
            .round()
            <= 10.0
        {
            region.set_block(
                pos,
                BlockState {
                    name: std::borrow::Cow::Borrowed("minecraft:light_blue_concrete"),
                    properties: None,
                },
            )
        }
    }

    let planet = region.as_litematic("Made with rustmatica".into(), "RubixDev".into());
    planet.write_file("planet.litematic")?;

    let planet = Litematic::read_file("planet.litematic")?;
    let region = &planet.regions[0];

    let air = BlockState {
        name: std::borrow::Cow::Borrowed("minecraft:air"),
        properties: None,
    };

    for x in region.x_range() {
        for z in region.z_range() {
            if region.get_block(Vec3::new(x, 10, z)) == &air {
                print!(" ");
            } else {
                print!("#")
            }
        }
        println!();
    }

    Ok(())
}
