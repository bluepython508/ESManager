use std::fs;

use gui::App;

use anyhow::*;
use es_manager::{Instance, INSTANCE_DIR};

mod gui;

fn main() -> Result<()> {
    // let instances = fs::read_dir(&*INSTANCE_DIR)?
        // .filter_map(|x| x.ok())
        // .map(|x| Instance::open(x.path()))
        // .collect::<Result<Vec<_>>>()?;
    // let app = App::new(instances);
    // eframe::run_native(Box::new(app));
    for inst in std::env::args().skip(1) {
    	Instance::create(inst, Default::default())?;
    }
Ok(())
}
