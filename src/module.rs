use std::hash::Hash;

use abi_stable::{
    declare_root_module_statics,
    external_types::crossbeam_channel::RSender,
    library::RootModule,
    package_version_strings, sabi_trait,
    sabi_types::VersionStrings,
    std_types::{RBox, RBoxError, RHashMap, ROption, RResult, RStr, RString},
    StableAbi,
};

use crate::{NotImplementedError, SabiWidget};

pub type ModuleType = SabiModule_TO<'static, RBox<()>>;

#[sabi_trait]
pub trait SabiModule {
    /// Register activities and producers that should appear when dynisland starts
    /// When this function is called the config was already loaded from the config file
    /// Functions using the gtk api should be run inside `glib::MainContext::default().spawn_local()` because gtk has yet to be initialized
    ///
    /// # Examples
    /// ```
    /// fn init(&self) {
    ///     let base_module = self.base_module.clone();
    ///     let config = self.config.clone();
    ///     glib::MainContext::default().spawn_local(async move {
    ///         if config.example_value==42{
    ///             //create activity
    ///             let act: DynamicActivity = /* ... */;
    ///             //register activity and data producer
    ///             base_module.register_activity(act).unwrap();
    ///         }
    ///         base_module.register_producer(self::producer);
    ///     });
    /// }
    /// ```
    fn init(&self);

    /// Update the config struct from the section of the config file for this module
    ///
    /// # Examples
    /// ```
    /// #[derive(Serialize, Deserialize, Clone)]
    /// #[serde(default)]
    /// pub struct ModuleConfig{
    ///     example_value: i32,
    /// }
    ///
    /// impl Default for ModuleConfig{
    ///     fn default() -> Self {
    ///         Self { example_value: 42 }
    ///     }
    /// }
    ///
    /// fn update_config(&mut self, config: RString) -> RResult<(), RBoxError> {
    ///     let conf = ron::from_str::<ron::Value>(&config)
    ///         .with_context(|| "failed to parse config to value")
    ///         .unwrap();
    ///     let old_config = self.config.clone();
    ///     self.config = conf
    ///         .into_rust()
    ///         .unwrap_or_else(|err| {
    ///             log::error!("parsing error, using old config: {:#?}", err);
    ///             old_config
    ///         }
    ///     );
    ///     ROk(())
    /// }
    /// ```
    fn update_config(&mut self, config: RString) -> RResult<(), RBoxError>;

    /// Restart the producers registered on the BaseModule
    ///
    /// # Examples
    /// ```
    /// fn restart_producers(&self) {
    ///     self.producers_rt.shutdown_blocking();
    ///     self.producers_rt.reset_blocking();
    ///     for producer in self
    ///         .base_module
    ///         .registered_producers()
    ///         .blocking_lock()
    ///         .iter()
    ///     {
    ///         producer(self);
    ///     }
    /// }
    /// ```
    fn restart_producers(&self);

    /// Get the default config for the module in ron format
    ///
    /// # Examples
    /// ```
    /// fn default_config(&self) -> RResult<RString, RBoxError> {
    ///     match ron::ser::to_string_pretty(&ModuleConfig::default(), PrettyConfig::default()) {
    ///         Ok(conf) => ROk(RString::from(conf)),
    ///         Err(err) => RErr(RBoxError::new(err)),
    ///     }
    /// }
    /// ```
    fn default_config(&self) -> RResult<RString, RBoxError> {
        RResult::RErr(RBoxError::new(NotImplementedError::default()))
    }

    #[sabi(last_prefix_field)]
    fn cli_command(&self, _command: RString) -> RResult<RString, RBoxError> {
        RResult::RErr(RBoxError::new(NotImplementedError::default()))
    }
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = ModuleBuilderRef)))]
#[sabi(missing_field(panic))]
pub struct ModuleBuilder {
    /// Create a new instance of a module
    ///
    /// # Examples
    /// ```
    /// pub struct Module{
    ///     base_module: BaseModule<MusicModule>,
    ///     producers_rt: ProducerRuntime,
    ///     config: ModuleConfig,
    /// }
    /// impl SabiModule for Module{/* ... */}
    ///
    /// #[sabi_extern_fn]
    /// pub fn new(app_send: RSender<UIServerCommand>) -> RResult<ModuleType, RBoxError> {
    ///     let base_module = BaseModule::new(NAME, app_send.clone());
    ///     let producers_rt = ProducerRuntime::new();
    ///     let module = Module{
    ///         base_module,
    ///         producers_rt,
    ///         config: ModuleConfig::default(),
    ///     };
    ///     ROk(SabiModule_TO::from_value(module, TD_CanDowncast))
    /// }
    /// ```
    pub new: extern "C" fn(app_send: RSender<UIServerCommand>) -> RResult<ModuleType, RBoxError>,

    /// The name of the module
    #[sabi(last_prefix_field)]
    pub name: RStr<'static>,
}

impl RootModule for ModuleBuilderRef {
    declare_root_module_statics! {ModuleBuilderRef}
    const BASE_NAME: &'static str = "module";
    const NAME: &'static str = "module";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

/// A command from a module to the app thread
#[repr(C)]
#[derive(StableAbi)]
pub enum UIServerCommand {
    /// Add an ActivityWidget to the LayoutManager
    AddActivity {
        activity_id: ActivityIdentifier,
        widget: SabiWidget,
    },
    // AddProducer(RString, Producer),
    /// Remove an ActivityWidget from the LayoutManager.
    ///
    /// The module should drop all the other references to the widget before sending this command
    RemoveActivity { activity_id: ActivityIdentifier },
    /// Send a request for the app to call `SabiModule::restart_producers()`.
    ///
    /// This is useful when you don't have a reference to the module
    RestartProducers { module_name: RString },

    RequestNotification {
        activity_id: ActivityIdentifier,
        mode: u8,
        duration: ROption<u64>,
    },
}

/// Module and activity name, used to uniquely identify a dynamic activity
///
/// Also includes metadata, this is not used for identification but for additional information
/// storage and comunication from the module to the layout manager
///
/// This struct must not change once the activity is registered
#[repr(C)]
#[derive(StableAbi, Clone, Debug, PartialOrd, Ord)]
pub struct ActivityIdentifier {
    /// Module name, must be the same as the on provided in `ModuleBuilder`
    pub(crate) module: RString,
    /// Activity name, must be the same as `activityWidget.name()`
    pub(crate) activity: RString,

    #[sabi(last_prefix_field)]
    pub(crate) metadata: ActivityMetadata,
}

impl Hash for ActivityIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.module.hash(state);
        self.activity.hash(state);
    }
}
impl Eq for ActivityIdentifier {}
impl PartialEq for ActivityIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.module == other.module && self.activity == other.activity
    }
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug, Default, PartialEq, Eq)]
pub struct ActivityMetadata {
    pub(crate) window_name: ROption<RString>,

    #[sabi(last_prefix_field)]
    pub(crate) additional_metadata: RHashMap<RString, RString>,
}

impl PartialOrd for ActivityMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.window_name.cmp(&other.window_name))
    }
}
impl Ord for ActivityMetadata {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.window_name.cmp(&other.window_name)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_activity_identifier_hash() {
        let act = ActivityIdentifier {
            module: RString::from("module"),
            activity: RString::from("activity"),
            metadata: ActivityMetadata::default(),
        };
        let mut act2 = ActivityIdentifier {
            module: RString::from("module"),
            activity: RString::from("activity"),
            metadata: ActivityMetadata::default(),
        };
        act2.metadata
            .additional_metadata
            .insert(RString::from("test"), RString::from("test"));
        act2.metadata.window_name = ROption::RSome(RString::from("window"));
        assert_eq!(act, act2);
        let mut set = std::collections::HashSet::new();
        set.insert(act.clone());
        assert!(set.contains(&act));
    }

    #[test]
    fn test_activity_identifier_cmp() {
        let mut act = ActivityIdentifier {
            module: RString::from("module"),
            activity: RString::from("activity"),
            metadata: ActivityMetadata::default(),
        };
        let mut act2 = ActivityIdentifier {
            module: RString::from("module"),
            activity: RString::from("activity"),
            metadata: ActivityMetadata::default(),
        };

        act2.metadata
            .additional_metadata
            .insert(RString::from("test"), RString::from("test"));
        act2.metadata.window_name = ROption::RSome(RString::from("window"));
        let cmp = act.cmp(&act2);
        assert_eq!(cmp, std::cmp::Ordering::Greater);

        act.metadata.window_name = ROption::RSome(RString::from("window"));
        let cmp = act.cmp(&act2);
        assert_eq!(cmp, std::cmp::Ordering::Equal);
    }
}
