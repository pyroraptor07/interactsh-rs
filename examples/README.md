# Examples
This directory contains 3 variations of a CLI client application using the Interactsh-rs crate:
- `cli_client_tokio` - Built with the [tokio](https://github.com/tokio-rs/tokio) async runtime
- `cli_client_asyncstd` - Built with the [async-std](https://github.com/async-rs/async-std) async runtime
- `cli_client_smol` - Built with the [smol](https://github.com/smol-rs/smol) async runtime

The `cli_client_shared` folder is a shared library crate used by all three examples that contains code not directly related to the Interactsh-rs crate (though it does contain example log formatting functions).

Run the examples with the `--help` option to see how to use them.