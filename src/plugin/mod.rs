// Plugin system for Exliar VFIO Automation Framework
//
// This module defines the plugin interface and plugin management system

use std::collections::HashMap;
use std::path::Path;
use std::any::Any;

/// Represents an error that occurred in a plugin
#[derive(Debug)]
pub struct PluginError {
    pub message: String,
}

impl std::fmt::Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Plugin error: {}", self.message)
    }
}

impl std::error::Error for PluginError {}

/// Context provided to plugins, giving them access to core services and storage
pub struct PluginContext {
    // Plugin-specific storage
    storage: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl PluginContext {
    /// Creates a new plugin context
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }
    
    /// Sets a value in the plugin context
    pub fn set<T: 'static + Send + Sync>(&mut self, key: &str, value: T) {
        self.storage.insert(key.to_string(), Box::new(value));
    }
    
    /// Gets a value from the plugin context
    pub fn get<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T> {
        self.storage.get(key)
            .and_then(|boxed| boxed.downcast_ref::<T>())
    }
    
    /// Gets a mutable value from the plugin context
    pub fn get_mut<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<&mut T> {
        self.storage.get_mut(key)
            .and_then(|boxed| boxed.downcast_mut::<T>())
    }
    
    /// Removes a value from the plugin context
    pub fn remove<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<T> {
        self.storage.remove(key)
            .and_then(|boxed| boxed.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }
}

/// Core interface for plugins
pub trait VfioPlugin: Send + Sync {
    /// Returns the plugin name
    fn name(&self) -> &str;
    
    /// Returns the plugin version
    fn version(&self) -> &str;
    
    /// Returns a description of the plugin
    fn description(&self) -> &str;
    
    /// Returns the plugin author
    fn author(&self) -> &str;
    
    /// Called when the plugin is loaded
    fn on_load(&self, context: &mut PluginContext) -> Result<(), PluginError>;
    
    /// Called when the plugin is unloaded
    fn on_unload(&self, context: &mut PluginContext) -> Result<(), PluginError>;
}

/// Manager for loading and interacting with plugins
pub struct PluginManager {
    plugins: Vec<Box<dyn VfioPlugin>>,
    contexts: HashMap<String, PluginContext>,
}

impl PluginManager {
    /// Creates a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            contexts: HashMap::new(),
        }
    }
    
    /// Registers a plugin with the manager
    pub fn register_plugin(&mut self, plugin: Box<dyn VfioPlugin>) -> Result<(), PluginError> {
        let plugin_name = plugin.name().to_string();
        
        // Create context for the plugin
        let mut context = PluginContext::new();
        
        // Call on_load to initialize the plugin
        plugin.on_load(&mut context)?;
        
        // Store the plugin and its context
        self.plugins.push(plugin);
        self.contexts.insert(plugin_name, context);
        
        Ok(())
    }
    
    /// Returns a list of loaded plugins
    pub fn list_plugins(&self) -> Vec<(&str, &str)> {
        self.plugins.iter()
            .map(|p| (p.name(), p.version()))
            .collect()
    }
}

// Placeholder for plugin loading from dynamic libraries (to be implemented)
pub mod dynamic {
    use super::*;
    
    /// Loads a plugin from a dynamic library
    pub fn load_plugin(_path: &Path) -> Result<Box<dyn VfioPlugin>, PluginError> {
        // This is a placeholder - actual implementation would use libloading crate
        Err(PluginError { message: "Dynamic plugin loading not implemented yet".to_string() })
    }
}