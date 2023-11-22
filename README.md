# heartbeat-windows

This is the Windows client for [Heartbeat](https://github.com/lmaotrigine/heartbeat). It
ships with two binaries.

- `heartbeat-task`: Pings the central server if an input device has been used in the
  last two minutes, and the workstation is unlocked. Requires the `task_runner` feature
  which installs some dependencies to make web requests, read configuration, and write
  logs. This binary is linked with `/SUBSYSTEM:WINDOWS`, so that a console doesn't pop
  up each time it runs.
- `heartbeat-client`: This is the CLI app you will need to use to configure the client and
  register the task using Task Scheduler. The `heartbeat-task` binary must be
  compiled/installed ***prior*** to running this script, and both the executables
  must be placed in the same directory to work properly. I place these constraints
  on you because Windows places other constraints on me, which I've tried really
  hard to work around.

## Prerequisites

These tools are tested on the latest stable and Insiders builds of Windows 11, but
should work fine on older versions that are still supported by Microsoft.

## Installation

Pre-compiled, statically-linked binaries are available in the
[releases](https://github.com/5HT2B/heartbeat-windows/releases) page.

Alternatively, you can clone the repository and build from source if you have a Rust
toolchain installed.

## Troubleshooting

The panic hook in the `heartbeat-task` binary simply writes the panic info to the log file. The
location of this file is `%HEARTBEAT_HOME%\logs\heartbeat.log`. `%HEARTBEAT_HOME%` by default
is `%USERPROFILE%\.heartbeat`. If you see something untoward in the logs, please open an
issue and include the relevant lines.
