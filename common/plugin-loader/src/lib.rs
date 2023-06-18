use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use libloading::Library;

use strategy_api::Strategy;

type StrategyLoader = extern fn() -> Box<dyn Strategy + Send>;

pub fn load(binary: &[u8]) -> Result<Plugin> {
    let temp_dir = tempfile::tempdir()?;
    let file_path = temp_dir.path().join("plugin.dll");
    {
        let mut file = File::create(&file_path)?;
        file.write_all(binary)?;
    }
    let plugin = unsafe {
        let library = Library::new(file_path)?;
        let load: libloading::Symbol<StrategyLoader> = library.get(b"load")?;
        let strategy = load();
        Plugin {
            library,
            strategy,
        }
    };
    Ok(plugin)
}
pub struct Plugin {
    // don't change order, strategy should drop before library
    pub strategy: Box<dyn Strategy + Send>,
    #[allow(unused)]
    library: Library,
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Strategy plugin, name: '{}', version: '{}'", self.strategy.name(), self.strategy.version())
    }
}
