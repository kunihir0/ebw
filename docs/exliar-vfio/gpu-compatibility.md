# GPU Passthrough Compatibility Guide

This document provides detailed information about GPU passthrough compatibility across different vendors and strategies for handling vendor-specific issues in the Exliar VFIO Automation Framework.

## Overview

GPU passthrough compatibility depends on several factors:
- GPU vendor and architecture
- Driver versions and compatibility
- IOMMU grouping
- Reset capabilities
- Host system configuration
- Guest OS requirements

## Vendor-Specific Considerations

### NVIDIA GPUs

NVIDIA GPUs are popular for passthrough but have some specific challenges:

#### Code 43 Error

NVIDIA drivers in Windows guests can detect virtualization and refuse to work with error code 43.

**Detection:**
- Windows guest shows "Error 43" in Device Manager
- Guest cannot use NVIDIA drivers

**Common Solutions:**
```xml
<!-- Hide hypervisor from NVIDIA drivers -->
<features>
  <hyperv>
    <vendor_id state='on' value='randomid'/>
  </hyperv>
  <kvm>
    <hidden state='on'/>
  </kvm>
</features>

<!-- Disable NVIDIA's ability to detect virtualization -->
<qemu:commandline>
  <qemu:arg value='-cpu'/>
  <qemu:arg value='host,kvm=off,hv_vendor_id=null'/>
</qemu:commandline>
```

#### Driver Version Compatibility

- Newer NVIDIA drivers (especially GeForce) are more likely to detect virtualization
- Quadro/Tesla drivers are often more compatible
- Driver version 465.xx and newer may require additional workarounds

#### Optimus/Prime Configurations

Laptops with hybrid Intel-NVIDIA configurations require special handling:
- ACPI tables may need modification
- Advanced ACS override patches may be required 
- Special treatment of power management

#### VBIOS Considerations

- May need to extract and pass custom VBIOS to VM
- ROMs larger than 1MB may need special handling

### AMD GPUs

AMD GPUs generally have good passthrough compatibility with some notable exceptions:

#### Reset Bug

Some AMD GPUs (particularly older generations) suffer from a reset bug that prevents the GPU from reinitializing after VM shutdown.

**Affected Architectures:**
- Most GCN 1.0-4.0 (Southern Islands, Sea Islands, Volcanic Islands, Polaris)
- Some Vega cards
- Generally not an issue with RDNA/RDNA2 (Navi) cards

**Workarounds:**
- Vendor Reset kernel module
- Reset workaround scripts
- Special shutdown sequence
- Hot reboot scripts

#### Driver Compatibility

- AMDGPU vs Radeon driver differences
- Open source vs proprietary driver considerations
- Special kernel parameters:
  - `amdgpu.ppfeaturemask=0xffffffff` (for overclocking support)
  - `amdgpu.dc=0` (for some display controller issues)

#### APU Considerations

- Shared memory and resources with CPU
- Framebuffer handling
- Special IOMMU considerations

#### Audio Function Handling

Many AMD GPUs include audio functions that need special handling:
- Passing through both PCI functions
- Ensuring audio device is properly configured in VM

### Intel GPUs

Intel GPUs have different considerations depending on whether they're integrated or discrete:

#### Integrated Graphics (iGPU)

Most Intel integrated GPUs don't support full GPU passthrough but do support:

**GVT-g Virtualization:**
- SR-IOV-like functionality
- Mediated device framework
- Allows sharing with multiple VMs
- Configuration via `/sys/devices/pci*/*/mdev_supported_types/`

**Setup Requirements:**
- Kernel with GVT-g support
- Kernel parameters: `i915.enable_gvt=1 intel_iommu=on`
- Compatible generations: Broadwell and newer

#### Intel Arc Discrete GPUs

The new Arc discrete GPUs have different considerations:
- Still limited community experience
- Similar to AMD discrete cards in handling
- May require specific driver versions

## IOMMU Group Management

### ACS Override

Many systems have poor IOMMU grouping, requiring ACS override patches:

```bash
# Check current IOMMU groups
for d in /sys/kernel/iommu_groups/*/devices/*; do 
    echo "$(basename $(dirname $(dirname $d))): $(lspci -nns $(basename $d))"
done | sort -V
```

**Implementing ACS Override:**
- Kernel patch or patched kernel required
- kernel parameter: `pcie_acs_override=downstream,multifunction`
- Security considerations: breaks IOMMU isolation guarantees

### Device Binding Strategies

Different approaches for binding devices to VFIO-PCI:

1. **Early Binding** (via kernel parameters)
   ```
   vfio-pci.ids=10de:1234,10de:4321
   ```

2. **Boot-time Scripts** (via initramfs)
   ```bash
   for dev in "$DEVICES"; do
     echo "vfio-pci" > /sys/bus/pci/devices/$dev/driver_override
     echo "$dev" > /sys/bus/pci/drivers/vfio-pci/bind
   done
   ```

3. **Runtime Binding** (via sysfs)
   ```bash
   echo "$dev" > /sys/bus/pci/drivers/current_driver/unbind
   echo "vfio-pci" > /sys/bus/pci/devices/$dev/driver_override
   echo "$dev" > /sys/bus/pci/drivers/vfio-pci/bind
   ```

## GPU Database Structure

Our GPU compatibility database will store structured information about known GPUs:

```json
{
  "vendors": {
    "AMD": {
      "families": {
        "GCN": {
          "reset_bug": true,
          "recommended_driver": "amdgpu",
          "quirks": ["vendor_reset", "fbc_disabled"]
        },
        "RDNA": {
          "reset_bug": false,
          "recommended_driver": "amdgpu",
          "quirks": ["rom_loading_required"]
        }
      },
      "models": {
        "1002:67df": {
          "name": "Radeon RX 580",
          "family": "GCN",
          "architecture": "Polaris",
          "reset_bug": true,
          "specific_quirks": [
            "May require vendor_reset module",
            "ROM must be loaded with vendor ROM"
          ],
          "recommended_vm_config": {
            "xml_snippets": {}
          }
        },
        "1002:73bf": {
          "name": "Radeon RX 6800 XT",
          "family": "RDNA",
          "architecture": "RDNA2",
          "reset_bug": false,
          "specific_quirks": []
        }
      }
    },
    "NVIDIA": {
      "families": {
        "Turing": {
          "code_43_vulnerable": true,
          "recommended_quirks": ["hyperv_vendor", "hidden_kvm"]
        },
        "Ampere": {
          "code_43_vulnerable": true,
          "recommended_quirks": ["hyperv_vendor", "hidden_kvm", "host_passthrough"]
        }
      },
      "models": {
        "10de:2206": {
          "name": "NVIDIA RTX 3080",
          "family": "Ampere",
          "code_43_vulnerable": true,
          "specific_quirks": [
            "May require specific vBIOS loading",
            "ROM bar size adjustment recommended"
          ],
          "recommended_vm_config": {
            "xml_snippets": {}
          }
        }
      }
    },
    "Intel": {
      "families": {
        "Gen12": {
          "supports_gvt_g": false,
          "supports_sriov": false
        },
        "Arc": {
          "supports_gvt_g": false,
          "supports_sriov": false,
          "recommended_quirks": ["rom_loading_required"]
        }
      }
    }
  }
}
```

## Practical Detection and Configuration Workflow

1. **Detect system GPUs**
   - List all GPUs in system using PCI vendor/device IDs
   - Correlate with known database entries

2. **Analyze IOMMU groups**
   - Determine if ACS override is needed
   - Check for proper separation of GPU functions
   - Identify related devices that must be passed through together

3. **Assess vendor-specific requirements**
   - Apply vendor detection logic
   - Check for known issues based on GPU model and architecture
   - Determine necessary quirks and workarounds

4. **Configure system for selected GPU**
   - Generate appropriate kernel parameters
   - Create necessary modprobe configuration
   - Set up appropriate driver binding

5. **Generate VM configuration**
   - Create required XML snippets for libvirt
   - Include appropriate vendor hiding techniques
   - Configure ROM loading if required
   - Set up appropriate bus/device/function assignments

6. **Test and validate**
   - Verify GPU appears correctly in VM
   - Confirm driver loads properly
   - Test reset functionality
   - Check performance characteristics
   - Validate multiple VM start/stop cycles

## Troubleshooting Common Issues

### Unable to bind GPU to VFIO

**Common causes:**
- Driver already in use and cannot be unbound
- Secure Boot preventing unsigned VFIO modules from loading
- Missing kernel modules or incorrect configuration

**Solutions:**
- Check current driver: `lspci -k -s <BDF>`  
- Force unbind current driver before binding to VFIO
- Disable Secure Boot or sign the VFIO modules
- Ensure VFIO modules are properly loaded: `lsmod | grep vfio`

### VM crashes when starting with GPU passthrough

**Common causes:**
- Improper IOMMU grouping
- Missing related devices (audio function, etc.)
- Reset bug affecting GPU
- Incorrect VM configuration

**Solutions:**
- Pass through all devices in the same IOMMU group
- Verify XML configuration for common mistakes
- Add appropriate ROM file if needed
- Try alternative PCIe port configurations
- Apply vendor-specific quirks

### GPU works but resets between VM reboots

**Common causes:**
- AMD reset bug
- Incomplete unbinding from host
- Power management issues

**Solutions:**
- Apply vendor_reset module for AMD GPUs
- Create proper shutdown scripts
- Configure appropriate ROM loading
- Use PCIe power management settings correctly

## Future Compatibility Expansions

Our framework will continue to evolve GPU compatibility handling:

1. **Community-sourced compatibility database**
   - User reports of working configurations
   - Centralized repository of known issues and solutions

2. **Automated quirk detection**
   - Runtime testing of reset capabilities
   - Automatic determination of needed workarounds

3. **Configuration generators**
   - Templates for common VM configurations
   - Vendor-specific best practices

4. **Telemetry and analysis** (opt-in)
   - Anonymous collection of compatibility data
   - Machine learning to predict compatibility issues