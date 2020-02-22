use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Weak};

/// A map from `RequestAdapterOptions` to active adapters.
///
/// Each time an adapter is requested via the `App`, it keeps track of which adapters are active.
/// This is done in order to allow re-use of adapters and in turn re-use of logical devices and the
/// sharing of resources between windows.
///
/// At the end of the application loop (after `update` and `view` have been called), adapters
/// containing no active device connections are removed from the map.
#[derive(Default)]
pub struct AdapterMap {
    map: Mutex<HashMap<AdapterMapKey, Arc<ActiveAdapter>>>,
}

/// The key into the adapter map.
///
/// This type is a thin wrapper around `wgpu::RequestAdapterOptions` that provides implementations
/// of `Eq` and `Hash`.
#[derive(Clone, Debug)]
pub struct AdapterMapKey {
    options: wgpu::RequestAdapterOptions,
}

/// A single active adapter and its map of connected devices.
pub struct ActiveAdapter {
    adapter: wgpu::Adapter,
    device_map: DeviceMap,
}

/// A map of actively connected devices for an adapter.
///
/// This is used so that windows built to target the same physical device may also target the same
/// logical device and share the same queue. This allows for resources like textures and buffers to
/// be shared between windows.
///
/// The map contains only weak handles to active adapters and cleans up unused entries at the end
/// of each application loop.
#[derive(Default)]
pub struct DeviceMap {
    map: Mutex<HashMap<DeviceMapKey, Weak<DeviceQueuePair>>>,
}

/// The key into the device map.
///
/// This type is a thin wrapper around `wgpu::DeviceDesriptor` that provides implementations of
/// `Eq` and `Hash`.
#[derive(Clone, Debug)]
pub struct DeviceMapKey {
    descriptor: wgpu::DeviceDescriptor,
}

/// A handle to a connected logical device and its associated queue.
#[derive(Debug)]
pub struct DeviceQueuePair {
    device: wgpu::Device,
    queue: Mutex<wgpu::Queue>,
}

impl AdapterMap {
    /// Check for an adaptor with the given options or request one.
    ///
    /// First checks to see if an adapter for the given set of options is active. If so, returns a
    /// handle to this adapter. Otherwise, requests a new adapter via `Adapter::request`.
    ///
    /// Returns `None` if there are no available adapters that meet the specified options.
    pub fn get_or_request(
        &self,
        options: wgpu::RequestAdapterOptions,
    ) -> Option<Arc<ActiveAdapter>> {
        let key = AdapterMapKey { options };
        let mut map = self
            .map
            .lock()
            .expect("failed to acquire `AdapterMap` lock");
        if let Some(adapter) = map.get(&key) {
            return Some(adapter.clone());
        }
        if let Some(adapter) = wgpu::Adapter::request(&key.options) {
            let device_map = Default::default();
            let adapter = Arc::new(ActiveAdapter {
                adapter,
                device_map,
            });
            return Some(map.entry(key).or_insert(adapter).clone());
        }
        None
    }

    /// Request an adaptor with the given options.
    ///
    /// This will always request a new adapter and will never attempt to share an existing one. The
    /// new adapter will take the place of the old within the map in the case that an existing
    /// active adapter exists.
    ///
    /// Returns `None` if there are no available adapters that meet the specified options.
    pub fn request(&self, options: wgpu::RequestAdapterOptions) -> Option<Arc<ActiveAdapter>> {
        let adapter = wgpu::Adapter::request(&options)?;
        let device_map = Default::default();
        let adapter = Arc::new(ActiveAdapter {
            adapter,
            device_map,
        });
        let key = AdapterMapKey { options };
        let mut map = self
            .map
            .lock()
            .expect("failed to acquire `AdapterMap` lock");
        map.insert(key, adapter.clone());
        Some(adapter)
    }

    /// Clear all adapters that currently have no connected devices.
    ///
    /// First clears all devices that no longer have any external references.
    pub(crate) fn clear_inactive_adapters_and_devices(&self) {
        let mut map = self
            .map
            .lock()
            .expect("failed to acquire `AdapterMap` lock");
        map.retain(|_, adapter| {
            adapter.clear_inactive_devices();
            adapter.device_count() > 0
        });
    }
}

impl ActiveAdapter {
    /// Check for a device with the given descriptor or request one.
    ///
    /// First checks for a connected device that matches the given descriptor. If one exists, it is
    /// returned. Otherwise, a new device connection is requested via `Adapter::request_device`.
    pub fn get_or_request_device(
        &self,
        descriptor: wgpu::DeviceDescriptor,
    ) -> Arc<DeviceQueuePair> {
        let key = DeviceMapKey { descriptor };
        let mut map = self
            .device_map
            .map
            .lock()
            .expect("failed to acquire `AdapterMap` lock");
        if let Some(device_ref) = map.get(&key) {
            if let Some(device) = device_ref.upgrade() {
                return device;
            }
        }
        let (device, queue) = self.adapter.request_device(&key.descriptor);
        let queue = Mutex::new(queue);
        let device = Arc::new(DeviceQueuePair { device, queue });
        map.insert(key, Arc::downgrade(&device));
        device
    }

    /// Request a device with the given descriptor.
    ///
    /// This will always request a new device connection and will never attempt to share an
    /// existing one. The new device will take the place of the old within the map in the case that
    /// an existing connected device exists.
    pub fn request_device(&self, descriptor: wgpu::DeviceDescriptor) -> Arc<DeviceQueuePair> {
        let (device, queue) = self.adapter.request_device(&descriptor);
        let queue = Mutex::new(queue);
        let device = Arc::new(DeviceQueuePair { device, queue });
        let key = DeviceMapKey { descriptor };
        let mut map = self
            .device_map
            .map
            .lock()
            .expect("failed to acquire `DeviceMap` lock");
        map.insert(key, Arc::downgrade(&device));
        device
    }

    /// A count of devices that are currently active.
    fn device_count(&self) -> usize {
        let map = self
            .device_map
            .map
            .lock()
            .expect("failed to acquire `DeviceMap` lock");
        map.len()
    }

    /// Clear all device queue pairs that have been dropped.
    fn clear_inactive_devices(&self) {
        let mut map = self
            .device_map
            .map
            .lock()
            .expect("failed to acquire `DeviceMap` lock");
        map.retain(|_, pair| pair.strong_count() > 0);
    }
}

impl DeviceQueuePair {
    /// A reference to the inner `wgpu::Device`.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// A reference to the inner `wgpu::Queue`.
    ///
    /// The queue is guarded by a `Mutex` in order to synchronise submissions of command buffers in
    /// cases that the queue is shared between more than one window.
    pub fn queue(&self) -> &Mutex<wgpu::Queue> {
        &self.queue
    }
}

impl Hash for AdapterMapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_request_adapter_options(&self.options, state);
    }
}

impl Hash for DeviceMapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_device_descriptor(&self.descriptor, state);
    }
}

impl PartialEq for AdapterMapKey {
    fn eq(&self, other: &Self) -> bool {
        eq_request_adapter_options(&self.options, &other.options)
    }
}

impl PartialEq for DeviceMapKey {
    fn eq(&self, other: &Self) -> bool {
        eq_device_descriptor(&self.descriptor, &other.descriptor)
    }
}

impl Eq for AdapterMapKey {}

impl Eq for DeviceMapKey {}

// NOTE: This should be updated as fields are added to the `wgpu::RequestAdapterOptions` type.
fn eq_request_adapter_options(
    a: &wgpu::RequestAdapterOptions,
    b: &wgpu::RequestAdapterOptions,
) -> bool {
    a.power_preference == b.power_preference && a.backends == b.backends
}

// NOTE: This should be updated as fields are added to the `wgpu::DeviceDescriptor` type.
fn eq_device_descriptor(a: &wgpu::DeviceDescriptor, b: &wgpu::DeviceDescriptor) -> bool {
    a.extensions.anisotropic_filtering == b.extensions.anisotropic_filtering
        && a.limits.max_bind_groups == b.limits.max_bind_groups
}

// NOTE: This should be updated as fields are added to the `wgpu::RequestAdapterOptions` type.
fn hash_request_adapter_options<H>(opts: &wgpu::RequestAdapterOptions, state: &mut H)
where
    H: Hasher,
{
    opts.power_preference.hash(state);
    opts.backends.hash(state);
}

// NOTE: This should be updated as fields are added to the `wgpu::DeviceDescriptor` type.
fn hash_device_descriptor<H>(desc: &wgpu::DeviceDescriptor, state: &mut H)
where
    H: Hasher,
{
    desc.extensions.anisotropic_filtering.hash(state);
    desc.limits.max_bind_groups.hash(state);
}
