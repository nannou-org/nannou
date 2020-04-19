**Tutorial Info**

- Author: [madskjeldgaard](https://madskjeldgaard.dk)
- Required Knowledge: [Anatomy of a nannou app](/tutorials/basics/anatomy-of-a-nannou-app.md)
- Reading Time: 5 minutes
---

# OSC communication

Open Sound Control ([OSC](http://http://opensoundcontrol.org/)) is a protocol for communicating between different pieces of software and/or computers. It is based on network technology and offers a flexible way to share control data between processes with a high level of precision, either internally on your local machine or through a network connection.

In nannou it's possible to both send and receive OSC data, allowing you to control other software or let nannou be controlled by other software.

## Setting up OSC
To use OSC in nannou, it is necessary to add the `nannou_osc` crate as a dependency in your nannou project.

Open up your `Cargo.toml` file at the root of your nannou project and add the following line under the `[dependencies]` tag:

`nannou_osc = "0.1.0"`

The value in the quotes is the version of the OSC package. At the time of writing this, `"0.1.0"` is the latest version.

To get the latest version of the osc library, execute `cargo search nannou_osc` on the command line and read the resulting version from there.

To use the crate in your nannou-projects you can add a use-statement at the top of your `main.rs` file to import the OSC-functionality.
```use nannou_osc as osc;```
