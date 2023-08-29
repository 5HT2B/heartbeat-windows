# heartbeat-windows

This is the Windows client for [Heartbeat](https://github.com/lmaotrigine/heartbeat). It
ships with three binaries:

- `heartbeat-task`: Pings the central server if an input device has been used in the
  last two minutes, and the workstation is unlocked. Requires the `task_runner` feature
  which installs some dependencies to make web requests, read configuration, and write
  logs.
- `heartbeat-config`: A helper binary to (over)write the configuration used by the task
  runner. You can use this script to set the `Authorization` token and base URL for your
  server. Requires the `config` feature which installs dependencies to figure out the
  location for the config file on your system.
- `heartbeat-register`: This script generates an XML file that can be imported into Task
  Scheduler. The `heartbeat-task` binary must be compiled/installed ***prior*** to
  running this script.

## Prerequisites

These tools are tested on the latest stable and Insiders builds of Windows 11, but
should work fine on older versions that are still supported by Microsoft.

## Installation

Pre-compiled, statically-linked binaries are available in the
[releases](https://github.com/lmaotrigine/heartbeat-windows/releases) page.

Alternatively, you can clone the repository and build from source if you have a Rust
toolchain installed.

## Troubleshooting

The panic hook in the `heartbeat-task` binary simply writes the panic info to the log file. The
location of this file is `%APPDATA%\heartbeat\heartbeat.log`. If you see something
untoward in the logs, please open an issue and include the relevant lines.
