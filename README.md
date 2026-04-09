# VDA5050 Robot Simulator

This project is a VDA5050-compliant robot simulator written in Rust. It simulates the behavior of automated guided vehicles (AGVs) following the VDA5050 standard, using an MQTT broker to communicate. The simulator is configurable via a TOML file, and it supports basic customization like vehicle configuration, state update frequency, and more. Also simulator supoorts create multiple simulator at the same time.

## Features

- Simulates AGVs using the VDA5050 standard.
- Communicates with a broker via MQTT.
- Configurable vehicle, map, and simulator settings.
- Supports visualization updates and actions.
- Supports trajectory

## Configuration

You can configure the simulator using a `config.toml` file. Below is an example configuration:

```toml
[mqtt_broker]
host = "localhost"                  # MQTT broker address
port = "1883"                        # MQTT broker port
vda_interface = "uagv"               # VDA interface to use
username = ""                   # MQTT broker username (optional)
password = ""                   # MQTT broker password (optional)

[vehicle]
serial_number = "s1"                 # Serial number of the AGV
manufacturer = "rikeb"               # Manufacturer name
vda_version = "v2"                   # VDA standard version
vda_full_version = "2.0.0"           # Full VDA version

[settings]
map_id = "webots"                    # Map identifier
state_frequency = 1                  # Frequency for state updates (in Hz)
visualization_frequency = 1          # Frequency for visualization updates (in Hz)
action_time = 1.0                    # Action execution time (in seconds)
robot_count = 1                      # Number of robots to simulate
speed = 0.05                         # Robot speed in meters per second
# Log outgoing visualization payloads to per-vehicle files (see "Visualization log file" below)
log_visualization_messages = false
```

### Visualization log file

Outgoing **visualization** MQTT payloads can optionally be written to disk, separately from the main vehicle log (`logs/<serial>/vehicle.log*`).

| Setting | Values | Effect |
|--------|--------|--------|
| `log_visualization_messages` | `true` / `false` | **`true`**: append each published visualization JSON to `logs/<vehicle serial>/visualization.log` (with the same size-based rotation as other logs). **`false`**: do not write visualization bodies to a file. MQTT visualization messages are still published according to `visualization_frequency`; only file logging is toggled. |

Add or edit under **`[settings]`** in `config.toml`:

```toml
[settings]
# ... other settings ...
log_visualization_messages = true   # enable visualization log file
```

To **disable** visualization file logging, set it to `false` or omit it (the default is `false`).

Related options (also under `[settings]`): `log_max_file_bytes` and `log_max_files` apply to each log stream, including `visualization.log`.

### MQTT Broker Section
- **host**: The address of the MQTT broker (default: localhost).
- **port**: The port of the MQTT broker (default: 1883).
- **vda_interface**: The type of VDA interface used.
- **username**: The username for authenticating with the MQTT broker. Leave empty or omit if the broker requires no authentication.
- **password**: The password for authenticating with the MQTT broker. Leave empty or omit if the broker requires no authentication.

### Vehicle Section

- **serial_number**: The serial number of the simulated robot.
- **manufacturer**: The name of the robot manufacturer.
- **vda_version**: The version of the VDA standard being used.
- **vda_full_version**: The full version number of the VDA standard.

### Settings Section

- **map_id**: Identifier for the map used in the simulation (e.g., "webots").
- **state_frequency**: Frequency of state updates in Hertz (Hz). Determines how often the robot sends its current state to the broker.
- **visualization_frequency**: Frequency of visualization updates in Hertz (Hz). Controls how often the simulator will send data for visualization purposes.
- **action_time**: The time it takes to complete an action (in seconds). This controls how long each task or action will take for the robot to execute.
- **robot_count**: The number of robots being simulated. This allows you to simulate multiple robots within the same environment.
- **speed**: The speed of the robot in meters per second, which dictates how fast the robot will move in the simulation.
- **log_visualization_messages**: When `true`, records outgoing visualization JSON to `logs/<serial>/visualization.log`. When `false` (default), those payloads are not written to that file. Does not stop MQTT visualization traffic.
- **log_max_file_bytes**: Maximum size in bytes per log file before rotation (default 10 MiB).
- **log_max_files**: Maximum number of files per log stream, including the active file and numbered backups (default 10).

## Docker 

The repository ships a `Dockerfile` and two Compose files that let you build and run the simulator without installing Rust locally.

| File | Purpose |
|---|---|
| `docker-compose.yml` | Base file. Uses a pre-built image and expects an external MQTT broker. |
| `docker-compose.override.yml` | Development override. Adds a build context so the image is compiled from source. Applied automatically by Docker Compose when both files are present. |


## Requirements

- **Rust**: Ensure that Rust is installed. You can follow the installation instructions [here](https://www.rust-lang.org/tools/install) if you don’t have it installed.
  
- **MQTT Broker**: You'll need an MQTT broker such as [Mosquitto](https://mosquitto.org/). Install and run it on your machine to handle communication between the simulator and the system.

## Getting Started

To set up and run the VDA5050 robot simulator, follow these steps:

1. Build the project using Cargo, Rust’s package manager:

    ```bash
    cargo build --release
    ```

2. Configure the simulator by modifying the `config.toml` file. You can adjust parameters such as the MQTT broker address, vehicle details, and simulation settings.

3. Run the simulator:

    ```bash
    cargo run --release
    ```

## Usage

Once the simulator is running, it will start sending messages to the MQTT broker according to the configuration in the `config.toml` file. You can monitor the robot's state, actions, and other telemetry by subscribing to the relevant MQTT topics using a client or tool such as [MQTT Explorer](https://mqtt-explorer.com/).

To visualize the robot's status and actions, you can adjust the `visualization_frequency` setting in `config.toml`.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.


## Libraries Used

This simulator was built using the [`vda5050-types-rs`](https://github.com/kKdH/vda5050-types-rs) library for handling VDA5050 standard data types and message structures.

## Reference documentation and code (local paths)

The following paths point to checked-out specs and related backends on the developer machine. They are recorded here so tools and contributors can open the same material when aligning behavior (protocol text, JSON schemas, OpenTCS plant model, AOS integration).

| Resource | Local path |
|----------|------------|
| VDA5050 **2.1.0** release (markdown, schemas, assets) | `D:\source\yeefung\vda5050\VDA5050-release-2.1.0\` |
| VDA5050 **3.0.0** release | `D:\source\yeefung\vda5050\VDA5050-release-3.0.0\` |
| **OpenTCS** 7.2.1 | `D:\source\yeefung\openTCS\opentcs-7.2.1\` |
| **AOS** backend (Yeefung) | `D:\source\yeefung\YFAOS\aos-backend\` |

Adjust drive letters or parent folders if your checkout lives elsewhere; keep this table updated when switching machines or versions.
