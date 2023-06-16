pub mod endpoints {
    pub const GET_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION: &str = "/api/v1/registry/plugins/info";
    pub const GET_BINARY_BY_ID_OR_NAME_OR_VERSION: &str = "/api/v1/registry/plugins/binary";
    pub const POST_PLUGINS: &str = "/api/v1/registry/plugins";
    pub const DELETE_PLUGINS_INFO_BY_ID_OR_NAME_OR_VERSION: &str = "/api/v1/registry/plugins";
}

pub mod dto {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use registry_core::model::Plugin;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PluginInfo {
        pub id: Uuid,
        pub name: String,
        pub version: String,
    }

    impl From<Plugin> for PluginInfo {
        fn from(value: Plugin) -> Self {
            Self {
                id: value.id,
                name: value.name,
                version: value.version,
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PluginBinary {
        pub id: Uuid,
        pub name: String,
        pub version: String,
        pub binary: Vec<u8>,
    }

    impl From<Plugin> for PluginBinary {
        fn from(value: Plugin) -> Self {
            Self {
                id: value.id,
                name: value.name,
                version: value.version,
                binary: value.binary,
            }
        }
    }
}

pub mod path_query {
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PluginQuery {
        pub id: Option<Uuid>,
        pub name: Option<String>,
        pub version: Option<String>,
    }
}
