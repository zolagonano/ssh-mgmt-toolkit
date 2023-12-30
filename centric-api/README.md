# Centric-API

## Overview

`centric-api` is a vital component of the project, built with the `rocket` framework (v0.5). It focuses on managing sales, payments, users, and serves as the interface to communicate with nodes through the `node-api`.

## Features

- **Ping Route**: Basic route to check API availability.
- **Node Management Routes**: Operations to manage nodes, including creation, updating, and deletion.
- **Service Management Routes**: CRUD operations for services.
- **User Management Routes**: Handle user creation and retrieve user references.
- **Sell Management Routes**: Operations related to selling, verification, and listing.
- **Authentication Routes**: Register, login, and token verification for secure operations.

## Configuration

As of now configurations are done in `src/lib.rs` and inside `consts` module but they will be moved to a JSON config file.

## Running the API

To launch the `centric-api`, use the following commands:

```bash
cd centric-api
cargo run
```

## License

This project is licensed under the terms of the **BSD 3-Clause License**. See the [LICENSE](LICENSE) file for details.

## Testing

Explore the `test.ipynb` notebook provided in `/centric-api` for tests and examples.
