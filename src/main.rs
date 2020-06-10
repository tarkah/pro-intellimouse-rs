use anyhow::Error;
use env_logger::Env;

use std::result;

pub type Result<T> = result::Result<T, Error>;

fn main() -> Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("mouse=INFO"));

    let mouse = mouse::IntelliMouse::connect()?;
    let dpi = mouse.read_property(mouse::Property::Dpi)?;

    println!("{:?}", dpi);

    Ok(())
}
