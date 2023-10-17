use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::io::Write;

use anyhow::Result;
use libloading::Library;

use plugin_api::PluginApi;

#[allow(improper_ctypes_definitions)]
type PluginLoader = extern "C" fn() -> Box<dyn PluginApi + Send + Sync>;

pub fn load(binary: &[u8]) -> Result<Plugin> {
    let file_name = if cfg!(target_os = "windows") {
        "plugin.dll"
    } else {
        "plugin.so"
    };
    let temp_dir = tempfile::tempdir()?;
    let file_path = temp_dir.path().join(file_name);
    {
        let mut file = File::create(&file_path)?;
        file.write_all(binary)?;
    }
    let plugin = unsafe {
        let library = Library::new(file_path)?;
        let plugin_api = library.get::<PluginLoader>(b"load")?();
        Plugin {
            library,
            api: plugin_api,
        }
    };
    Ok(plugin)
}

pub struct Plugin {
    // don't change order, api should drop before library
    pub api: Box<dyn PluginApi + Send + Sync>,
    #[allow(unused)]
    library: Library,
}

impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Plugin name: '{}', version: '{}'",
            self.api.id().name,
            self.api.id().version
        )
    }
}
