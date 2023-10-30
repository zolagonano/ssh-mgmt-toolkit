# SSH VPN Management Project

## Project Overview

This project provides a comprehensive API solution for managing SSH VPN users, supporting referrals, and managing sales and payments. It consists of two main components - `node-api` and `centric-api`. Both APIs are part of the same cargo workspace.

## Getting Started

1. Clone the repository:

```bash
git clone https://github.com/zolagonano/ssh-mgmt-toolkit.git
cd ssh-mgmt-toolkit
```

2. Run each API individually:

- For `node-api`:

```bash
cd node-api
cargo run
```

- For `centric-api`:

```bash
cd centric-api
cargo run
```

## Configuration

Ensure both APIs have their configuration files. Create a configuration file based on `config_sample.json` in each API and place them in requested path for example `/etc/sshmgmt_config.json` for `node-api`.

## Testing

Explore the `test.ipynb` notebooks provided in both `/node-api` and `/centric-api` for tests and examples.

## Communication Overview

### Centric-API to Multiple Node-APIs

```plaintext
+----------------------+          +------------------------+
|                      |          |                        |
|      Centric-API     |          |         Node-APIs       |
|                      |          |                        |
|                      |   HTTP   |      +--------+        |
|    +-------------+   +---------->      | Node-1 |        |
|    |             |   |           |      +--------+        |
|    |             |   |           |                        |
|    |             |   |           |      +--------+        |
|    |             |   |           |      | Node-2 |        |
|    |             |   |           |      +--------+        |
|    |             |   |           |                        |
|    +-------------+   |           |      +--------+        |
|                      |           |      | Node-3 |        |
+----------------------+           |      +--------+        |
                                   |                        |
                                   |      +--------+        |
                                   |      | Node-4 |        |
                                   |      +--------+        |
                                   |                        |
```

- **Centric-API to Multiple Node-APIs Communication**:

  - **Overview**: The `centric-api` communicates with multiple `node-api` instances to manage individual servers through one API.
  - **HTTP Requests**: The communication is established through HTTP requests.
  - **Communication Flow**: The `centric-api` sends requests to multiple `node-api` instances (Node-1, Node-2, Node-3, Node-4), which independently process the requests.

### Note

This project was initially developed for personal use. Feel free to explore, modify, and adapt the codebase according to your requirements. If you encounter issues or have suggestions, please open an issue on the GitHub repository.
