# Platform-specific Setup

Before we get started, let's make sure we have all the necessary ingredients for
installing Rust and building nannou projects.

Depending on what OS you are running, you might require an extra step or two.

- **All Platforms**

  For now, nannou requires that you have both `python` and `cmake` installed.
  These are required by a tool called `shaderc`, a part of nannou's graphics
  stack. The role of this tool is to compile GLSL shaders to SPIR-V so that we
  may run them using the system's Vulkan implementation. There are a few
  attempts at pure-rust alternatives to this tool in the works and we hope to
  switch to one of these in the near future to avoid the need for these extra
  dependencies.

- **macOS**: Ensure that you have xcode-tools installed:

  ```bash
  xcode-select --install
  ```

  In order to add support for [Vulkan](https://www.khronos.org/vulkan/) (the
  graphics backend used by nannou) to macOS, nannou will prompt you and attempt
  to automatically download and install the [MoltenVK
  SDK](https://github.com/KhronosGroup/MoltenVK). MoltenVK is a driver-level
  implementation of the Vulkan graphics and compute API, that runs on Apple's
  Metal graphics and compute framework on both iOS and macOS. If you wish to
  update your MoltenVK SDK version, simply remove the currently installed SDK
  (this should be at `~/.vulkan_sdk`) and nannou will prompt you about
  downloading and installing the next version the next time you attempt to build
  a nannou project.

- **Windows**: Install the `ninja` tool.

  This tool is another required by the `shaderc` tool as mentioned under the
  "All Platforms" section above.

  1. Download the latest release of ninja from the [ninja releases
     page](https://github.com/ninja-build/ninja/releases).
  2. Place the `ninja.exe` file somewhere you are happy for it to stay.
  3. Add the directory containing `ninja.exe` to your `Path` environment
     variable if it is not already included.

- **Linux**: ensure you have the following system packages installed:

  - **Basic dev packages**

    First make sure our basic dev packages are installed. `curl` will be
    required by `rustup` the rust toolchain manager. `build-essential` will be
    required by `rustc` the rust compiler for linking. `cmake` and `python` are
    necessary for nannou's `shaderc` dependency to build, as mentioned in the
    "All Platforms" section above. `pkg-config`will be used to retrieve 
    information about `alsa` during the building process.

    For Debian/Ubuntu users:
    ```bash
    sudo apt-get install curl build-essential python cmake pkg-config
    ```

  - **alsa dev package**

    For Fedora users:
    ```bash
    sudo dnf install alsa-lib-devel
    ```

    For Debian/Ubuntu users:
    ```bash
    sudo apt-get install libasound2-dev
    ```

    For Arch users:
    ```bash
    sudo pacman -S alsa-lib
    ```

  - **curl lib dev package**

    Nannou depends on the `curl-sys` crate. Some Linux distributions use
    LibreSSL instead of OpenSSL (such as AlpineLinux, Voidlinux, possibly
    [others](https://en.wikipedia.org/wiki/LibreSSL#Adoption) if manually
    installed).

  - **vulkan**

    Installing Vulkan support on Linux is generally quite easy using your
    distro's package manager. That said, there may be different driver
    options to consider depending on your graphics card and tolerance for
    proprietary software. The following are rough guidelines on how to get
    going quickly, however if you are at all concerned with finding the
    approach that suits you best we recommend searching for vulkan driver
    installation for your graphics card on your distro.

    For Fedora with AMD graphic cards:
    ```bash
    sudo dnf install vulkan vulkan-info
    ```

    For Fedora with NVIDIA graphic cards:
    Add the proprietary drivers
    ```bash
    sudo dnf install https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm https://download1.rpmfusion.org/nonfree/fedora/rpmfusion-nonfree-release-$(rpm -E %fedora).noarch.rpm
    ```
    and run
    ```bash
    sudo dnf install xorg-x11-drv-nvidia akmod-nvidia vulkan-tools
    ```

    For Debian with AMD or Intel graphic cards:
    ```bash
    sudo apt-get install libvulkan1 mesa-vulkan-drivers vulkan-utils
    ```

    For Debian with NVIDIA graphic cards:
    ```bash
    sudo apt-get install vulkan-utils
    ```

    For Ubuntu users with AMD or Intel graphic cards:
    Add a PPA for the latest drivers
    ```bash
    sudo add-apt-repository ppa:oibaf/graphics-drivers
    sudo apt-get update
    sudo apt-get upgrade
    ```
    and run
    ```bash
    sudo apt-get install libvulkan1 mesa-vulkan-drivers vulkan-utils
    ```

    For Ubuntu users with NVIDIA graphic cards:
    Add a PPA for the latest drivers
    ```bash
    sudo add-apt-repository ppa:graphics-drivers/ppa
    sudo apt-get update
    sudo apt-get upgrade
    ```
    and run
    ```bash
    sudo apt-get install nvidia-graphics-drivers-396 nvidia-settings vulkan vulkan-utils
    ```

    For Arch with AMD or Intel graphic cards:
    ```bash
    sudo pacman -S vulkan-radeon lib32-vulkan-radeon
    ```

    For Arch with NVIDIA graphic cards:
    ```bash
    sudo pacman -S nvidia lib32-nvidia-utils
    ```

OK, we should now be ready to install Rust!
