# Platform-specific Setup

Before we get started, let's make sure we have all the necessary ingredients for
installing Rust and building nannou projects.

Depending on what OS you are running, you might require an extra step or two.

By the way, if you notice some steps are missing from this section of the guide,
feel free to open an issue or PR at [the nannou guide
repo](https://github.com/nannou-org/guide)!

## macOS

Ensure that you have xcode-tools installed:

```bash
xcode-select --install
```

Some examples require that you have `cmake` installed as well. The easiest way to achieve this is to use [Homebrew](https://brew.sh).

```bash
brew install cmake
```

This should provide all the developer tools needed for building nannou.

## Windows

Rust requires the C++ build tools for Visual Studio. The Rust book has this to
say:

> On Windows, go to
> [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
> and follow the instructions for installing Rust. At some point in the
> installation, you’ll receive a message explaining that you’ll also need the
> C++ build tools for Visual Studio 2013 or later. The easiest way to acquire
> the build tools is to install [Build Tools for Visual Studio
> 2019](https://www.visualstudio.com/downloads/#build-tools-for-visual-studio-2019).
> The tools are in the Other Tools and Frameworks section.

## Linux

Ensure you have the following system packages installed:

- **Basic dev packages**

  First make sure the basic dev packages are installed.
  - `curl` will be required by `rustup` the rust toolchain manager.
  - `build-essential` will be required by `rustc` the rust compiler for linking.
  - `pkg-config` is used by some build scripts to source information about
    certain libraries.
  - `alsa` dev packages are required for `nannou_audio`.

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

- **xcb**

  The XCB library provides inter-operability with Xlib.

  For Debian/Ubuntu users:
  ```bash
  sudo apt install libxcb-shape0-dev libxcb-xfixes0-dev
  ```

  You might also need `python3` for the `xcb` crate's build script.

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
  sudo apt-get install vulkan-tools
  ```

  Or, on older versions (pre-Buster i.e., < 10):
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

  For Arch with AMD graphic cards:
  ```bash
  sudo pacman -S vulkan-radeon lib32-vulkan-radeon
  ```

  For Arch with Intel graphics card:
  ```bash
  sudo pacman -S vulkan-intel
  ```

  For Arch with NVIDIA graphic cards:
  ```bash
  sudo pacman -S nvidia lib32-nvidia-utils
  ```

  For Gentoo run:
  ```bash
  sudo emerge --ask --verbose dev-util/vulkan-tools dev-util/vulkan-headers
  ```

OK, we should now be ready to install Rust!
