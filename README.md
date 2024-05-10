# Bankid mock 

This is a service that can be spun up to act as a mock of the Swedish identity provider Bankid.

## Features

- Easy access with preconfigured quick-users
- Muliple servers pointing to the same instance (can point multiple enviorments to the same instance)

## Running the mock for development 

```bash
cargo leptos watch
```

## Executing a Server on a Remote Machine Without the Toolchain
After running a `cargo leptos build --release` the minimum files needed are:

1. The server binary located in `target/server/release`
2. The `site` directory and all files within located in `target/site`

Copy these files to your remote server. The directory structure should be:
```text
bankid-mock
config.toml
site/
```
Set the following environment variables (updating for your project as needed):
```text
LEPTOS_OUTPUT_NAME="bankid-mock"
LEPTOS_SITE_ROOT="site"
LEPTOS_SITE_PKG_DIR="pkg"
LEPTOS_SITE_ADDR="127.0.0.1:3000"
LEPTOS_RELOAD_PORT="3001"
```
Finally, run the server binary.
