# Node-API

## Overview

`node-api` is an API component designed for managing individual servers in the context of SSH VPN user operations. The API primarily focuses on server statistics retrieval and user management.

## Features

- **Ping Route**: Basic route to check API availability.
- **Node Information**: Retrieve detailed information about a specific node.
- **Statistics Routes**: Access network and hardware statistics.
- **User Management Routes**: Manage users, including addition, deletion, and modification.

## Configuration

For proper functionality, create a configuration file named `config_sample.json` (located in `/node-api`) and place it in `/etc/sshmgmt_config.json`.

## Running the API

To launch the `node-api`, use the following commands:

```bash
cd node-api
cargo run
```

## License

This project is licensed under the terms of the **BSD 3-Clause License**. See the [LICENSE](LICENSE) file for details.

## Testing

Explore the `test.ipynb` notebook provided in `/node-api` for tests and examples.
