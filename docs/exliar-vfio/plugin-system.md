# Plugin System Architecture

This document outlines the plugin architecture for the Exliar VFIO Automation Framework, which enables flexible extension of the core functionality.

## Plugin System Goals

1. **Extensibility**: Allow users to extend the framework without modifying core code
2. **Modularity**: Keep core functionality clean and separate from extensions
3. **Isolation**: Ensure plugins can't destabilize the core system
4. **Discoverability**: Make available hooks and extension points clear
5. **Versioning**: Support clear plugin API versioning and compatibility

## Plugin Interface

The core of our plugin system is the `VfioPlugin` trait:

```rust
pub trait VfioPlugin: Send + Sync {
    // Plugin identity
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    fn author(&self) -> &str;
    
    // API compatibility 
    fn api_version(&self) -> (u32, u32);  // (major, minor)
    
    // Lifecycle hooks
    fn on_load(&self, context: &mut PluginContext) -> Result<(), PluginError>;
    fn on_unload(&self, context: &mut PluginContext) -> Result<(), PluginError>;
    
    // Main extension points (implement only what's needed)
    fn on_system_detected(&self, context: &mut PluginContext, system: &SystemInfo) -> Result<(), PluginError> {
        // Default empty implementation
        Ok(())
    }
    
    fn on_device_enumeration(&self, context: &mut PluginContext, devices: &[PciDevice]) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_gpu_detected(&self, context: &mut PluginContext, gpu: &GpuDevice) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_pre_vfio_bind(&self, context: &mut PluginContext, device: &PciDevice) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_post_vfio_bind(&self, context: &mut PluginContext, device: &PciDevice) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_pre_initramfs_update(&self, context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_post_initramfs_update(&self, context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_pre_bootloader_update(&self, context: &mut PluginContext, bootloader: &BootloaderConfig) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_post_bootloader_update(&self, context: &mut PluginContext, bootloader: &BootloaderConfig) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_pre_vm_start(&self, context: &mut PluginContext, vm_config: &mut VmConfig) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_post_vm_shutdown(&self, context: &mut PluginContext, vm_stats: &VmStats) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_cleanup(&self, context: &mut PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    
    // UI integration points
    fn register_ui_elements(&self, context: &mut PluginContext, ui_registry: &mut UiRegistry) -> Result<(), PluginError> {
        Ok(())
    }
    
    fn on_ui_event(&self, context: &mut PluginContext, event: &UiEvent) -> Result<(), PluginError> {
        Ok(())
    }
    
    // Command registration
    fn register_commands(&self, context: &mut PluginContext, registry: &mut CommandRegistry) -> Result<(), PluginError> {
        Ok(())
    }
}
```

## Plugin Context

Each plugin receives a `PluginContext` that provides:

1. Access to core services
2. Storage for plugin-specific state
3. Communication mechanisms with other plugins
4. Configuration access

```rust
pub struct PluginContext {
    // Plugin-specific storage
    storage: HashMap<String, Box<dyn Any + Send + Sync>>,
    
    // Service providers
    logger: Arc<dyn Logger>,
    config_provider: Arc<dyn ConfigProvider>,
    event_bus: Arc<EventBus>,
    
    // Core service accessors
    system_info: Option<Arc<SystemInfo>>,
    device_manager: Option<Arc<DeviceManager>>,
    vfio_manager: Option<Arc<VfioManager>>,
    state_tracker: Option<Arc<StateTracker>>,
}

impl PluginContext {
    // Storage API for plugin state
    pub fn set<T: 'static + Send + Sync>(&mut self, key: &str, value: T);
    pub fn get<T: 'static + Send + Sync>(&self, key: &str) -> Option<&T>;
    pub fn get_mut<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<&mut T>;
    pub fn remove<T: 'static + Send + Sync>(&mut self, key: &str) -> Option<T>;
    
    // Event subscription
    pub fn subscribe<E: Event>(&mut self, handler: Box<dyn Fn(&E) + Send + Sync>);
    pub fn emit<E: Event>(&self, event: E);
    
    // Service access
    pub fn logger(&self) -> &dyn Logger;
    pub fn config(&self) -> &dyn ConfigProvider;
    
    // Core service access (if available at hook time)
    pub fn system_info(&self) -> Option<&SystemInfo>;
    pub fn device_manager(&self) -> Option<&DeviceManager>;
    pub fn vfio_manager(&self) -> Option<&VfioManager>;
    pub fn state_tracker(&self) -> Option<&StateTracker>;
}
```

## Plugin Manager

The plugin system is coordinated through the `PluginManager`:

```rust
pub struct PluginManager {
    plugins: Vec<Box<dyn VfioPlugin>>,
    contexts: HashMap<String, PluginContext>,
    hooks: HashMap<String, Vec<String>>, // Hook name -> List of plugin names
}

impl PluginManager {
    // Plugin discovery and loading
    pub fn discover_plugins(&mut self, plugin_dir: &Path) -> Result<Vec<PluginMetadata>>;
    pub fn load_plugin(&mut self, path: &Path) -> Result<()>;
    pub fn unload_plugin(&mut self, name: &str) -> Result<()>;
    
    // Plugin management
    pub fn enable_plugin(&mut self, name: &str) -> Result<()>;
    pub fn disable_plugin(&mut self, name: &str) -> Result<()>;
    
    // Plugin listing
    pub fn list_plugins(&self) -> Vec<&PluginMetadata>;
    pub fn get_plugin(&self, name: &str) -> Option<&dyn VfioPlugin>;
    
    // Hook invocation
    pub fn invoke_hook<T>(&mut self, hook_name: &str, arg: &T) -> Result<()>;
    pub fn invoke_hook_mut<T>(&mut self, hook_name: &str, arg: &mut T) -> Result<()>;
}
```

## Plugin Discovery and Loading

Plugins can be loaded in multiple ways:

1. **Dynamic Libraries (Preferred)**
   - `.so` files on Linux
   - Must export a `create_plugin()` function that returns a `Box<dyn VfioPlugin>`

2. **Rust Crate Plugins**
   - Compiled directly into the main application
   - Registered via a plugin registry

3. **Plugin Directories**
   - Standard location: `~/.config/exliar-vfio/plugins/`
   - System-wide location: `/usr/share/exliar-vfio/plugins/`
   - Custom locations via configuration

## Plugin Categories

Our system will support several categories of plugins:

### 1. Hardware Support Plugins

Extend support for specific hardware:
- GPU vendor-specific plugins
- Special device handling
- Laptop-specific optimizations

### 2. VM Integration Plugins

Enhance VM management:
- QEMU/Libvirt integration extensions  
- VM templating
- Storage management

### 3. UI Plugins

Extend the user interface:
- Additional TUI screens
- Improved visualizations
- Web interface components

### 4. System Integration Plugins

Integrate with other system components:
- Kernel module management
- Distribution-specific handling
- Other virtualization platforms

### 5. Automation Plugins

Add automation capabilities:
- Discord integration
- Remote management
- Scheduled operations

## Plugin Security Model

Plugins have significant system access, so security is important:

1. **Plugin Verification**
   - Optional signature verification
   - Hash checking for known plugins
   - Warning for unsigned plugins

2. **Permission System**
   - Plugins declare required permissions
   - User must approve permissions during installation
   - Fine-grained access control

3. **Execution Isolation**
   - Plugin operations recorded in change tracker
   - Critical operations require explicit approval
   - Potential for sandboxing in future versions

## Common Plugin Development Patterns

### State Management

```rust
// Plugin initialization with state
fn on_load(&self, context: &mut PluginContext) -> Result<(), PluginError> {
    // Store plugin state in context
    context.set("my_plugin_state", MyState::new());
    Ok(())
}

// Access state in other hooks
fn on_gpu_detected(&self, context: &mut PluginContext, gpu: &GpuDevice) -> Result<(), PluginError> {
    if let Some(state) = context.get_mut::<MyState>("my_plugin_state") {
        state.record_gpu(gpu);
    }
    Ok(())
}
```

### Configuration Access

```rust
fn on_load(&self, context: &mut PluginContext) -> Result<(), PluginError> {
    // Read plugin configuration
    let config = context.config().get_section("my_plugin")?;
    let feature_enabled = config.get_bool("feature_enabled").unwrap_or(false);
    
    context.set("feature_enabled", feature_enabled);
    Ok(())
}
```

### Event Publishing

```rust
// Define custom event
#[derive(Debug, Clone)]
struct GpuOptimizationEvent {
    gpu_id: String,
    optimizations: Vec<String>,
}

impl Event for GpuOptimizationEvent {
    fn event_type(&self) -> &'static str {
        "gpu_optimization"
    }
}

// Emit event
fn on_post_vfio_bind(&self, context: &mut PluginContext, device: &PciDevice) -> Result<(), PluginError> {
    // Do some optimizations
    let optimizations = vec!["power_management".to_string(), "fan_control".to_string()];
    
    // Emit event for other plugins
    context.emit(GpuOptimizationEvent {
        gpu_id: device.bdf().to_string(),
        optimizations,
    });
    
    Ok(())
}
```

### Command Registration

```rust
fn register_commands(&self, context: &mut PluginContext, registry: &mut CommandRegistry) -> Result<(), PluginError> {
    registry.register_command(
        "optimize-gpu",
        "Apply GPU optimizations",
        Box::new(|args, ctx| {
            // Command implementation
            println!("Optimizing GPU with args: {:?}", args);
            Ok(())
        }),
    )?;
    
    Ok(())
}
```

## Plugin Dependencies

Plugins can declare dependencies on other plugins:

```rust
pub struct MyPlugin {
    dependencies: Vec<PluginDependency>,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            dependencies: vec![
                PluginDependency {
                    name: "nvidia-handler".to_string(),
                    version_req: ">=1.0.0".to_string(),
                    optional: false,
                },
                PluginDependency {
                    name: "performance-monitor".to_string(),
                    version_req: ">=0.5.0".to_string(), 
                    optional: true,
                },
            ],
        }
    }
}

impl VfioPlugin for MyPlugin {
    fn get_dependencies(&self) -> &[PluginDependency] {
        &self.dependencies
    }
    
    // Other trait implementations
}
```

## Plugin Versioning and Compatibility

Our plugin system uses semantic versioning:

1. **API Version**
   - Major version changes: Breaking API changes
   - Minor version changes: Non-breaking additions
   - Plugin declares compatible API version range

2. **Plugin Version**
   - Each plugin has its own semantic version
   - Used for dependency resolution

3. **Compatibility Checking**
   - Plugin API version checked at load time
   - Dependencies resolved before loading
   - Warnings for untested version combinations

## Plugin Lifecycle Example

Below is an example lifecycle of plugin interactions within the framework:

1. **Discovery and Loading**
   - Framework scans plugin directories
   - Plugin metadata is loaded
   - Plugin dependencies are resolved
   - Plugins are loaded in dependency order

2. **System Initialization**
   - Core systems initialized
   - `on_load` hooks called for each plugin
   - Plugins register hooks, commands, and UI elements

3. **Detection Phase**
   - System detection runs
   - `on_system_detected` hooks called
   - PCI device enumeration occurs
   - `on_device_enumeration` hooks called
   - GPU detection runs
   - `on_gpu_detected` hooks called for each GPU

4. **Configuration Phase**
   - VFIO configuration created
   - `on_pre_vfio_bind` hooks called
   - Devices bound to VFIO
   - `on_post_vfio_bind` hooks called
   - Initramfs update prepared
   - `on_pre_initramfs_update` hooks called
   - Initramfs updated
   - `on_post_initramfs_update` hooks called
   - Bootloader update prepared
   - `on_pre_bootloader_update` hooks called
   - Bootloader updated
   - `on_post_bootloader_update` hooks called

5. **VM Management (if applicable)**
   - VM configuration loaded
   - `on_pre_vm_start` hooks called
   - VM started
   - VM shuts down
   - `on_post_vm_shutdown` hooks called

6. **Cleanup**
   - `on_cleanup` hooks called
   - Plugins are unloaded
   - `on_unload` hooks called

## Plugin Development Guide

### Creating a Basic Plugin

Here's a minimal plugin implementation:

```rust
use exliar_vfio::plugin::{VfioPlugin, PluginContext, PluginError};
use exliar_vfio::system::SystemInfo;
use exliar_vfio::gpu::GpuDevice;

#[derive(Default)]
pub struct ExamplePlugin {
    name: String,
    version: String,
    description: String,
}

impl ExamplePlugin {
    pub fn new() -> Self {
        Self {
            name: "example-plugin".to_string(),
            version: "0.1.0".to_string(),
            description: "An example plugin".to_string(),
        }
    }
}

impl VfioPlugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn version(&self) -> &str {
        &self.version
    }
    
    fn description(&self) -> &str {
        &self.description
    }
    
    fn author(&self) -> &str {
        "Example Author"
    }
    
    fn api_version(&self) -> (u32, u32) {
        (1, 0)  // Compatible with API version 1.0
    }
    
    fn on_load(&self, context: &mut PluginContext) -> Result<(), PluginError> {
        context.logger().info("Example plugin loaded!");
        Ok(())
    }
    
    fn on_system_detected(&self, context: &mut PluginContext, system: &SystemInfo) -> Result<(), PluginError> {
        context.logger().info(&format!("System detected: {:?}", system.distribution));
        Ok(())
    }
    
    fn on_gpu_detected(&self, context: &mut PluginContext, gpu: &GpuDevice) -> Result<(), PluginError> {
        context.logger().info(&format!("GPU detected: {} ({})", 
            gpu.model_name(), gpu.vendor()));
        Ok(())
    }
}

// Export plugin creation function (for dynamic loading)
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn VfioPlugin> {
    Box::new(ExamplePlugin::new())
}
```

### Building and Packaging Plugins

1. **Library Creation**
   ```toml
   # Cargo.toml
   [lib]
   name = "my_plugin"
   crate-type = ["cdylib"]  # Dynamic library
   ```

2. **Plugin Manifest**
   ```toml
   # plugin.toml
   name = "my-plugin"
   version = "0.1.0"
   description = "My awesome plugin"
   author = "Plugin Author"
   api_version = "1.0"
   
   [dependencies]
   nvidia-handler = ">=1.0.0"
   ```

3. **Packaging**
   ```
   my-plugin/
   ├── lib.so             # Compiled plugin binary
   ├── plugin.toml        # Plugin manifest
   ├── LICENSE            # License file
   └── README.md          # Documentation
   ```

## Example Plugin Scenarios

### 1. NVIDIA-specific handler plugin

```rust
pub struct NvidiaHandlerPlugin {
    // Plugin metadata
    // ...
}

impl VfioPlugin for NvidiaHandlerPlugin {
    // Basic plugin methods
    // ...
    
    fn on_gpu_detected(&self, context: &mut PluginContext, gpu: &GpuDevice) -> Result<(), PluginError> {
        // Only handle NVIDIA GPUs
        if gpu.vendor() != GpuVendor::NVIDIA {
            return Ok(());
        }
        
        context.logger().info(&format!("NVIDIA GPU detected: {}", gpu.model_name()));
        
        // Check driver version and store compatibility info
        let driver_version = detect_nvidia_driver_version()?;
        let is_code_43_vulnerable = is_vulnerable_to_code_43(&gpu, &driver_version);
        
        // Store information for later hooks
        context.set(&format!("gpu_{}_code_43_vulnerable", gpu.bdf()), is_code_43_vulnerable);
        
        Ok(())
    }
    
    fn on_pre_vm_start(&self, context: &mut PluginContext, vm_config: &mut VmConfig) -> Result<(), PluginError> {
        // Find NVIDIA GPUs in VM config
        let nvidia_gpus: Vec<_> = vm_config.devices.iter()
            .filter(|dev| dev.is_gpu() && dev.vendor_id() == "10de")
            .collect();
            
        for gpu in nvidia_gpus {
            let bdf = gpu.bdf();
            // Check if this GPU needs Code 43 workaround
            if context.get::<bool>(&format!("gpu_{}_code_43_vulnerable", bdf)).unwrap_or(&true) {
                // Apply Code 43 workarounds to VM config
                vm_config.add_hypervisor_hidden(true)?;
                vm_config.add_vendor_id("123456789ab")?;
                vm_config.add_kvm_hidden(true)?;
                
                context.logger().info(&format!(
                    "Applied NVIDIA Code 43 workarounds for GPU {}", bdf));
            }
        }
        
        Ok(())
    }
}
```

### 2. Performance monitoring plugin

```rust
pub struct PerfMonitorPlugin {
    // Plugin metadata
    // ...
}

impl VfioPlugin for PerfMonitorPlugin {
    // Basic plugin methods
    // ...
    
    fn on_load(&self, context: &mut PluginContext) -> Result<(), PluginError> {
        // Initialize performance monitor state
        context.set("perf_stats", PerfStats::new());
        Ok(())
    }
    
    fn on_pre_vm_start(&self, context: &mut PluginContext, vm_config: &mut VmConfig) -> Result<(), PluginError> {
        // Reset stats before VM start
        if let Some(stats) = context.get_mut::<PerfStats>("perf_stats") {
            stats.reset();
            stats.vm_start_time = SystemTime::now();
        }
        
        // Start monitoring thread
        let (tx, rx) = mpsc::channel();
        context.set("perf_channel", tx);
        
        std::thread::spawn(move || {
            monitor_performance(rx);
        });
        
        Ok(())
    }
    
    fn on_post_vm_shutdown(&self, context: &mut PluginContext, vm_stats: &VmStats) -> Result<(), PluginError> {
        // Store performance stats
        if let Some(stats) = context.get_mut::<PerfStats>("perf_stats") {
            stats.vm_end_time = SystemTime::now();
            stats.runtime_seconds = stats.vm_end_time
                .duration_since(stats.vm_start_time)
                .ok()
                .map(|d| d.as_secs())
                .unwrap_or(0);
                
            // Log performance summary
            context.logger().info(&format!(
                "VM Performance: Runtime: {}s, Avg GPU Usage: {}%, Avg CPU: {}%, Max Memory: {}MB",
                stats.runtime_seconds,
                stats.avg_gpu_usage,
                stats.avg_cpu_usage,
                stats.max_memory_mb
            ));
        }
        
        // Stop monitoring thread
        if let Some(tx) = context.remove::<mpsc::Sender<()>>("perf_channel") {
            let _ = tx.send(());  // Signal thread to stop
        }
        
        Ok(())
    }
    
    // Register UI elements to show performance stats
    fn register_ui_elements(&self, context: &mut PluginContext, ui_registry: &mut UiRegistry) -> Result<(), PluginError> {
        ui_registry.register_panel(
            "performance",
            "Performance",
            Box::new(|ui, ctx| {
                // Draw performance UI panel
                if let Some(stats) = ctx.get::<PerfStats>("perf_stats") {
                    ui.draw_stats(stats);
                }
                Ok(())
            }),
        )?;
        
        Ok(())
    }
}