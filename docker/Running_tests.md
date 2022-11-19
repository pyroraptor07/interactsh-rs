# Test environment setup
Follow these steps to setup the test environment needed to run the crate tests on your local machine.

Note: This assumes docker and docker-compose are already set up on your local machine.

- Make a copy of the `.env.example` file in the project root and replace the
FQDN and token values accordingly. Name this file `.env`.

- Make sure `./docker/scripts/start_servers.sh` and `./docker/scripts/stop_servers.sh` have executable permissions.

- From the project root directory, run `./docker/scripts/start_servers.sh`. Both test servers should come up.
Use `./docker/scripts/stop_servers.sh` to stop the test servers once testing is completed.