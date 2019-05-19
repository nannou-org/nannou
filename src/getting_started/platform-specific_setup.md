# Platform-specific Setup

Depending on what OS you are running, you might require an extra step or two.

- **macOS**: Ensure that you have xcode-tools installed:

  ```bash
  xcode-select --install
  ```

- **Linux**: ensure you have the following system packages installed:

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
