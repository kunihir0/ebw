# Exliar VFIO Automation Framework

## Project Vision

A modular, extensible system for automating VFIO GPU passthrough and VM management with Discord integration.

## Documentation Structure

This documentation directory contains the design and planning documents for the Exliar VFIO Automation Framework. Our Stage 1 focus is on building the core foundation with special emphasis on GPU detection and compatibility across different vendors.

### Key Documents

1. [Architecture Overview](./architecture.md) - Overall system architecture and component design
2. [GPU Compatibility](./gpu-compatibility.md) - Detailed information about GPU passthrough compatibility
3. [Plugin System](./plugin-system.md) - Plugin architecture for extending framework functionality

## Core Design Principles

1. **Distribution Agnostic**: Adapt to different Linux distributions and configurations
2. **Vendor Neutral**: Support AMD, NVIDIA, and Intel GPUs with vendor-specific optimizations
3. **Modular Architecture**: Core components are separate with clear interfaces
4. **Extensible**: Plugin system for extending functionality
5. **User-Friendly**: Terminal UI for easy management
6. **Safe Operations**: Backup creation and rollback capabilities

## Relationship to vfio-auto

**Important Note**: The Exliar VFIO Automation Framework is being built completely from scratch. The existing `vfio-auto` project is used ONLY as a reference to understand VFIO passthrough patterns and common approaches. We are:

1. Building a completely new codebase in Rust (versus Python in vfio-auto)
2. Designing a fundamentally different architecture with a true plugin system
3. Implementing a modern Terminal UI vs CLI approach
4. Focusing on different priorities (GPU compatibility, state tracking)
5. Using completely different internal APIs and module structures

The vfio-auto project serves as a learning resource to understand the domain, but our implementation is entirely original and independent.

## Project Stages

### Stage 1: Core Foundation (Current Focus)

- System detection module
- PCI management module with enhanced GPU detection
- Boot configuration module
- VFIO module manager
- State tracking system
- Basic plugin system
- Terminal UI foundation

### Stage 2: VM Management (Future)

- QEMU/KVM wrapper
- VM template system
- Storage management
- VM lifecycle hooks

### Stage 3: Advanced Features (Future)

- Performance monitoring
- USB passthrough manager
- Looking Glass integration
- Discord Bot integration

## Development Plan

Our development plan centers around building the core foundation first with robust GPU compatibility handling:

1. Implement system detection with bootloader and init system identification
2. Develop comprehensive GPU detection with vendor-specific handling
3. Build PCI device management with IOMMU group validation
4. Create boot configuration manager for different bootloaders
5. Implement VFIO configuration and binding
6. Develop state tracking for changes and rollbacks
7. Create plugin system foundation
8. Build initial Terminal UI

## Directory Structure

```
src/
  ├── main.rs
  ├── cli.rs
  ├── core/
  │   ├── mod.rs
  │   ├── system.rs       (System detection)
  │   ├── pci.rs          (PCI management)
  │   ├── bootloader.rs   (Boot configuration)
  │   ├── vfio.rs         (VFIO module management)
  │   └── state.rs        (State tracking)
  ├── gpu/
  │   ├── mod.rs
  │   ├── detection.rs    (GPU detection)
  │   ├── vendor/         (Vendor-specific modules)
  │   │   ├── amd.rs
  │   │   ├── nvidia.rs
  │   │   └── intel.rs
  │   ├── quirks.rs      (GPU quirks database)
  │   ├── vbios.rs       (VBIOS management)
  │   └── reset.rs       (GPU reset handling)
  ├── plugin/
  │   ├── mod.rs
  │   ├── api.rs          (Plugin API)
  │   └── manager.rs      (Plugin manager)
  ├── ui/
  │   ├── mod.rs
  │   └── tui.rs          (Terminal UI)
  └── utils/
      ├── mod.rs
      └── logging.rs      (Logging utilities)
```

## Next Steps

After finalizing this design, we'll proceed with implementation in the following order:

1. Create Rust project structure from scratch
2. Core system detection module
3. GPU detection with vendor-specific handling
4. PCI and IOMMU management
5. VFIO configuration and binding
6. State tracking and rollback
7. Basic TUI