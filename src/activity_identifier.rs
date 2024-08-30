use std::fmt::Display;

use abi_stable::std_types::ROption;

use crate::module::{ActivityIdentifier, ActivityMetadata};

impl ActivityIdentifier {
    pub fn new(module_name: &str, activity_name: &str) -> Self {
        Self {
            module: module_name.to_string().into(),
            activity: activity_name.to_string().into(),
            metadata: ActivityMetadata::default(),
        }
    }
    pub fn module(&self) -> String {
        self.module.clone().into()
    }

    pub fn activity(&self) -> String {
        self.activity.clone().into()
    }

    pub fn metadata_mut(&mut self) -> &mut ActivityMetadata {
        &mut self.metadata
    }
    pub fn metadata(&self) -> ActivityMetadata {
        self.metadata.clone()
    }

    pub fn set_metadata(&mut self, metadata: ActivityMetadata) {
        self.metadata = metadata;
    }
}

impl ActivityMetadata{
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_window_name(&mut self, window_name: &str) {
        self.window_name = ROption::RSome(window_name.to_string().into());
    }
    pub fn window_name(&self) -> Option<String> {
        match &self.window_name {
            ROption::RSome(name) => Some(name.clone().into()),
            ROption::RNone => None,
        }
    }
    pub fn set_additional_metadata(&mut self, additional_metadata: &str) {
        self.additional_metadata = ROption::RSome(additional_metadata.to_string().into());
    }
    pub fn additional_metadata(&self) -> Option<String> {
        match &self.additional_metadata {
            ROption::RSome(metadata) => Some(metadata.clone().into()),
            ROption::RNone => None,
        }
    }
    
}

impl Display for ActivityIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.activity, self.module)
    }
}
