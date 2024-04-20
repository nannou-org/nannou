use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};

use crate as wgpu;

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
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AdapterMapKey {
    power_preference: wgpu::PowerPreference,
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
    descriptor: wgpu::DeviceDescriptor<'static>,
}

/// A handle to a connected logical device and its associated queue.
#[derive(Debug)]
pub struct DeviceQueuePair {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl AdapterMap {
    #[cfg(not(target_os = "unknown"))]
    /// Check for an adaptor with the given options or request one.
    ///
    /// First checks to see if an adapter for the given set of options is active. If so, returns a
    /// handle to this adapter. Otherwise, requests a new adapter via `Adapter::request`.
    ///
    /// Returns `None` if there are no available adapters that meet the specified options.
    pub fn get_or_request<'a, 'b>(
        &'a self,
        options: wgpu::RequestAdapterOptions<'b>,
        instance: &'a wgpu::Instance,
    ) -> Option<Arc<ActiveAdapter>> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.get_or_request_async(options, instance))
    }

    #[cfg(not(target_os = "unknown"))]
    /// Request an adaptor with the given options.
    ///
    /// This will always request a new adapter and will never attempt to share an existing one. The
    /// new adapter will take the place of the old within the map in the case that an existing
    /// active adapter exists.
    ///
    /// Returns `None` if there are no available adapters that meet the specified options.
    pub fn request<'a, 'b>(
        &'a self,
        options: wgpu::RequestAdapterOptions<'b>,
        instance: &'a wgpu::Instance,
    ) -> Option<Arc<ActiveAdapter>> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.request_async(options, instance))
    }

    /// The async implementation of `get_or_request`.
    pub async fn get_or_request_async<'a, 'b>(
        &'a self,
        options: wgpu::RequestAdapterOptions<'b>,
        instance: &'a wgpu::Instance,
    ) -> Option<Arc<ActiveAdapter>> {
        let power_preference = options.power_preference;
        let key = AdapterMapKey { power_preference };
        let mut map = self
            .map
            .lock()
            .expect("failed to acquire `AdapterMap` lock");
        if let Some(adapter) = map.get(&key) {
            return Some(adapter.clone());
        }
        if let Some(adapter) = instance.request_adapter(&options).await {
            let device_map = Default::default();
            let adapter = Arc::new(ActiveAdapter {
                adapter,
                device_map,
            });
            return Some(map.entry(key).or_insert(adapter).clone());
        }
        None
    }

    /// The async implementation of `request`.
    pub async fn request_async<'a, 'b>(
        &'a self,
        options: wgpu::RequestAdapterOptions<'b>,
        instance: &'b wgpu::Instance,
    ) -> Option<Arc<ActiveAdapter>> {
        let adapter = instance.request_adapter(&options).await?;
        let device_map = Default::default();
        let adapter = Arc::new(ActiveAdapter {
            adapter,
            device_map,
        });
        let power_preference = options.power_preference;
        let key = AdapterMapKey { power_preference };
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
    pub fn clear_inactive_adapters_and_devices(&self) {
        let mut map = self
            .map
            .lock()
            .expect("failed to acquire `AdapterMap` lock");
        map.retain(|_, adapter| {
            adapter.clear_inactive_devices();
            adapter.device_count() > 0
        });
    }

    /// Poll all devices within all active adapters.
    pub(crate) fn _poll_all_devices(&self, maintain: wgpu::Maintain) {
        let map = self
            .map
            .lock()
            .expect("failed to acquire `AdapterMap` lock");
        for adapter in map.values() {
            adapter._poll_all_devices(maintain.clone()); // TODO: clone?
        }
    }
}

impl ActiveAdapter {
    #[cfg(not(target_os = "unknown"))]
    /// Check for a device with the given descriptor or request one.
    ///
    /// First checks for a connected device that matches the given descriptor. If one exists, it is
    /// returned. Otherwise, a new device connection is requested via `Adapter::request_device`.
    pub fn get_or_request_device(
        &self,
        descriptor: wgpu::DeviceDescriptor<'static>,
    ) -> Arc<DeviceQueuePair> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.get_or_request_device_async(descriptor))
    }

    #[cfg(not(target_os = "unknown"))]
    /// Request a device with the given descriptor.
    ///
    /// This will always request a new device connection and will never attempt to share an
    /// existing one. The new device will take the place of the old within the map in the case that
    /// an existing connected device exists.
    pub fn request_device(
        &self,
        descriptor: wgpu::DeviceDescriptor<'static>,
    ) -> Arc<DeviceQueuePair> {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(self.request_device_async(descriptor))
    }

    /// Check for a device with the given descriptor or request one.
    ///
    /// First checks for a connected device that matches the given descriptor. If one exists, it is
    /// returned. Otherwise, a new device connection is requested via `Adapter::request_device`.
    pub async fn get_or_request_device_async(
        &self,
        descriptor: wgpu::DeviceDescriptor<'static>,
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
        let (device, queue) = self
            .adapter
            .request_device(&key.descriptor, None)
            .await
            .expect("could not get or request device");
        let device = Arc::new(DeviceQueuePair { device, queue });
        map.insert(key, Arc::downgrade(&device));
        device
    }

    /// Request a device with the given descriptor.
    ///
    /// This will always request a new device connection and will never attempt to share an
    /// existing one. The new device will take the place of the old within the map in the case that
    /// an existing connected device exists.
    pub async fn request_device_async(
        &self,
        descriptor: wgpu::DeviceDescriptor<'static>,
    ) -> Arc<DeviceQueuePair> {
        let (device, queue) = self
            .adapter
            .request_device(&descriptor, None)
            .await
            .expect("could not request device async");
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
    pub fn device_count(&self) -> usize {
        let map = self
            .device_map
            .map
            .lock()
            .expect("failed to acquire `DeviceMap` lock");
        map.len()
    }

    /// Clear all device queue pairs that have been dropped.
    pub fn clear_inactive_devices(&self) {
        let mut map = self
            .device_map
            .map
            .lock()
            .expect("failed to acquire `DeviceMap` lock");
        map.retain(|_, pair| pair.upgrade().is_some());
    }

    /// Poll all of the active devices within the map.
    fn _poll_all_devices(&self, maintain: wgpu::Maintain) {
        let map = self
            .device_map
            .map
            .lock()
            .expect("failed to acquire `DeviceMap` lock");
        for weak in map.values() {
            if let Some(pair) = weak.upgrade() {
                pair.device().poll(maintain.clone()); // TODO: clone?
            }
        }
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
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }
}

impl Deref for ActiveAdapter {
    type Target = wgpu::Adapter;
    fn deref(&self) -> &Self::Target {
        &self.adapter
    }
}

impl Hash for DeviceMapKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_device_descriptor(&self.descriptor, state);
    }
}

impl PartialEq for DeviceMapKey {
    fn eq(&self, other: &Self) -> bool {
        eq_device_descriptor(&self.descriptor, &other.descriptor)
    }
}

impl Eq for DeviceMapKey {}

// NOTE: This should be updated as fields are added to the `wgpu::DeviceDescriptor` type.
fn eq_device_descriptor(
    a: &wgpu::DeviceDescriptor<'static>,
    b: &wgpu::DeviceDescriptor<'static>,
) -> bool {
    a.label == b.label && a.features == b.features && a.limits == b.limits
}

// NOTE: This should be updated as fields are added to the `wgpu::DeviceDescriptor` type.
fn hash_device_descriptor<H>(desc: &wgpu::DeviceDescriptor<'static>, state: &mut H)
where
    H: Hasher,
{
    desc.label.hash(state);
    desc.features.hash(state);
    desc.limits.hash(state);
}
