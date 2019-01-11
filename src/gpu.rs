//! Items related to interaction with GPUs via Vulkan.

use std::borrow::Cow;
use std::panic::RefUnwindSafe;
use std::sync::Arc;
use vulkano::format::Format;
use vulkano::instance::{ApplicationInfo, Instance, InstanceCreationError, InstanceExtensions,
                        PhysicalDevice};
use vulkano::instance::debug::{DebugCallback, DebugCallbackCreationError, Message, MessageTypes};
use vulkano::instance::loader::{FunctionPointers, Loader};
use vulkano_win;

/// The default application name used with the default `ApplicationInfo`.
pub const DEFAULT_APPLICATION_NAME: &'static str = "nannou-app";

/// The default application info
pub const DEFAULT_APPLICATION_INFO: ApplicationInfo<'static> = ApplicationInfo {
    application_name: Some(Cow::Borrowed(DEFAULT_APPLICATION_NAME)),
    application_version: None,
    engine_name: None,
    engine_version: None,
};

/// A builder struct that makes the process of building an instance more modular.
#[derive(Default)]
pub struct VulkanInstanceBuilder {
    pub app_info: Option<ApplicationInfo<'static>>,
    pub extensions: Option<InstanceExtensions>,
    pub layers: Vec<String>,
    pub loader: Option<FunctionPointers<Box<dyn Loader + Send + Sync>>>,
}

/// A builder struct that makes the process of building a debug callback more modular.
#[derive(Default)]
pub struct VulkanDebugCallbackBuilder {
    pub message_types: Option<MessageTypes>,
    pub user_callback: Option<BoxedUserCallback>,
}

// The user vulkan debug callback allocated on the heap to avoid complicated type params.
type BoxedUserCallback = Box<Fn(&Message) + 'static + Send + RefUnwindSafe>;

impl VulkanInstanceBuilder {
    /// Begin building a vulkano instance.
    pub fn new() -> Self {
        Default::default()
    }
}

impl VulkanInstanceBuilder {
    /// Specify the application info with which the instance should be created.
    pub fn app_info(mut self, app_info: ApplicationInfo<'static>) -> Self {
        self.app_info = Some(app_info);
        self
    }

    /// Specify the exact extensions to enable for the instance.
    pub fn extensions(mut self, extensions: InstanceExtensions) -> Self {
        self.extensions = Some(extensions);
        self
    }

    /// Add the given extensions to the set of existing extensions within the builder.
    ///
    /// Unlike the `extensions` method, this does not disable pre-existing extensions.
    pub fn add_extensions(mut self, ext: InstanceExtensions) -> Self {
        self.extensions = self.extensions.take()
            .map(|mut e| {
                // TODO: Remove this when `InstanceExtensions::union` gets merged.
                e.khr_surface |= ext.khr_surface;
                e.khr_display |= ext.khr_display;
                e.khr_xlib_surface |= ext.khr_xlib_surface;
                e.khr_xcb_surface |= ext.khr_xcb_surface;
                e.khr_wayland_surface |= ext.khr_wayland_surface;
                e.khr_android_surface |= ext.khr_android_surface;
                e.khr_win32_surface |= ext.khr_win32_surface;
                e.ext_debug_report |= ext.ext_debug_report;
                e.mvk_ios_surface |= ext.mvk_ios_surface;
                e.mvk_macos_surface |= ext.mvk_macos_surface;
                e.mvk_moltenvk |= ext.mvk_moltenvk;
                e.nn_vi_surface |= ext.nn_vi_surface;
                e.ext_swapchain_colorspace |= ext.ext_swapchain_colorspace;
                e.khr_get_physical_device_properties2 |= ext.khr_get_physical_device_properties2;
                e
            })
            .or(Some(ext));
        self
    }

    /// Specify the exact layers to enable for the instance.
    pub fn layers<L>(mut self, layers: L) -> Self
    where
        L: IntoIterator,
        L::Item: Into<String>,
    {
        self.layers = layers.into_iter().map(Into::into).collect();
        self
    }

    /// Extend the existing list of layers with the given layers.
    pub fn add_layers<L>(mut self, layers: L) -> Self
    where
        L: IntoIterator,
        L::Item: Into<String>,
    {
        self.layers.extend(layers.into_iter().map(Into::into));
        self
    }

    /// Build the vulkan instance with the existing parameters.
    pub fn build(self) -> Result<Arc<Instance>, InstanceCreationError> {
        let VulkanInstanceBuilder {
            app_info,
            extensions,
            layers,
            loader,
        } = self;

        let app_info = app_info.unwrap_or(DEFAULT_APPLICATION_INFO);
        let extensions = extensions.unwrap_or_else(required_windowing_extensions);
        let layers = layers.iter().map(|s| &s[..]);
        match loader {
            None => Instance::new(Some(&app_info), &extensions, layers),
            Some(loader) => Instance::with_loader(loader, Some(&app_info), &extensions, layers),
        }
    }
}

impl VulkanDebugCallbackBuilder {
    /// Begin building a vulkan debug callback.
    pub fn new() -> Self {
        Default::default()
    }

    /// The message types to be emitted to the debug callback.
    ///
    /// If unspecified, nannou will use `MessageTypes::errors_and_warnings`.
    pub fn message_types(mut self, msg_tys: MessageTypes) -> Self {
        self.message_types = Some(msg_tys);
        self
    }

    /// The function that will be called for handling messages.
    ///
    /// If unspecified, nannou will use a function that prints to `stdout` and `stderr`.
    pub fn user_callback<F>(mut self, cb: F) -> Self
    where
        F: Fn(&Message) + 'static + Send + RefUnwindSafe,
    {
        self.user_callback = Some(Box::new(cb) as Box<_>);
        self
    }

    /// Build the debug callback builder for the given vulkan instance.
    pub fn build(
        self,
        instance: &Arc<Instance>,
    ) -> Result<DebugCallback, DebugCallbackCreationError> {
        let VulkanDebugCallbackBuilder {
            message_types,
            user_callback,
        } = self;
        let message_types = message_types.unwrap_or_else(|| MessageTypes {
            error: true,
            warning: true,
            performance_warning: true,
            information: true,
            debug: true,
        });
        let user_callback = move |msg: &Message| {
            match user_callback {
                Some(ref cb) => (**cb)(msg),
                None => {
                    let ty = if msg.ty.error {
                        "error"
                    } else if msg.ty.warning {
                        "warning"
                    } else if msg.ty.performance_warning {
                        "performance_warning"
                    } else if msg.ty.information {
                        "information"
                    } else if msg.ty.debug {
                        "debug"
                    } else {
                        println!("[vulkan] <unknown message type>");
                        return;
                    };
                    println!("[vulkan] {} {}: {}", msg.layer_prefix, ty, msg.description);
                }
            };
        };
        DebugCallback::new(instance, message_types, user_callback)
    }
}

/// The default set of required extensions used by Nannou.
///
/// This is the same as calling `vulkano_win::required_extensions()`.
pub fn required_windowing_extensions() -> InstanceExtensions {
    vulkano_win::required_extensions()
}

/// Whether or not the format is sRGB.
pub fn format_is_srgb(format: Format) -> bool {
    use vulkano::format::Format::*;
    match format {
        R8Srgb |
        R8G8Srgb |
        R8G8B8Srgb |
        B8G8R8Srgb |
        R8G8B8A8Srgb |
        B8G8R8A8Srgb |
        A8B8G8R8SrgbPack32 |
        BC1_RGBSrgbBlock |
        BC1_RGBASrgbBlock |
        BC2SrgbBlock |
        BC3SrgbBlock |
        BC7SrgbBlock |
        ETC2_R8G8B8SrgbBlock |
        ETC2_R8G8B8A1SrgbBlock |
        ETC2_R8G8B8A8SrgbBlock |
        ASTC_4x4SrgbBlock |
        ASTC_5x4SrgbBlock |
        ASTC_5x5SrgbBlock |
        ASTC_6x5SrgbBlock |
        ASTC_6x6SrgbBlock |
        ASTC_8x5SrgbBlock |
        ASTC_8x6SrgbBlock |
        ASTC_8x8SrgbBlock |
        ASTC_10x5SrgbBlock |
        ASTC_10x6SrgbBlock |
        ASTC_10x8SrgbBlock |
        ASTC_10x10SrgbBlock |
        ASTC_12x10SrgbBlock |
        ASTC_12x12SrgbBlock => true,
        _ => false,
    }
}

/// Given some target MSAA samples, limit it by the capabilities of the given `physical_device`.
///
/// This is useful for attempting a specific multisampling sample count but falling back to a
/// supported count in the case that the desired count is unsupported.
///
/// Specifically, this function limits the given `target_msaa_samples` to the minimum of the color
/// and depth sample count limits.
pub fn msaa_samples_limited(physical_device: &PhysicalDevice, target_msaa_samples: u32) -> u32 {
    let color_limit = physical_device.limits().framebuffer_color_sample_counts();
    let depth_limit = physical_device.limits().framebuffer_depth_sample_counts();
    let msaa_limit = std::cmp::min(color_limit, depth_limit);
    std::cmp::min(msaa_limit, target_msaa_samples)
}
