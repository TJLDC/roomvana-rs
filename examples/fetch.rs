//! Fetch the live Roomvana catalog and print what it contains.
//!
//! Run with: `cargo run --example fetch`

use roomvana::Catalog;

fn main() -> Result<(), roomvana::Error> {
    let catalog = Catalog::fetch()?;

    println!(
        "interior rooms: {}  exteriors: {}  gardens: {}  styles: {}",
        catalog.room_slugs().len(),
        catalog.exterior_slugs().len(),
        catalog.garden_slugs().len(),
        catalog.style_slugs().len(),
    );

    println!("\nrooms:");
    for room in catalog.rooms() {
        println!("  {:<16} {}", room.slug, room.name);
    }

    println!("\nstyles:");
    for style in catalog.styles() {
        println!("  {:<24} {}", style.slug, style.name);
    }

    println!("\nexample link:");
    println!("  {}", catalog.design_link("kitchen", "japandi")?);

    Ok(())
}
