# pythonbox

Simple service to run Python 3 sandboxed in a Docker container and return the output back.

## Motivation
I originally tried to use Judge0 to do this, but it turns out that Judge0 only runs on systems with cgroups v1 enabled, and my current setup only supports cgroups v2.
Running code in a Docker container is pretty safe (as long as you don't privilege the container), so this is what this we do to contain the code.
Right now it only supports python, but it is very simple to extend this in the future to other scripting languages as long as they can run in a docker container.

For compilation, we might need to introduce a multiple entry script system, but that is a TODO at the moment.

### Installation
1. Ensure you have installed docker and the rust toolchain.
2. Ensure that the current user has permissions to use docker.
3. `git clone` the repository, and run `cargo build` inside

### Running the server
1. Run `./run.sh`

### Requests
Post requests in JSON to the endpoint `/run_code`.

Each request must have the following fields:
* `max_time_s`: number, in secconds that the code can run for
* `base_64_tar_gz`: string containing a base 64 representation of a tar file containing the code.
    The tar file must contain an executable file called `run` at it's base.
    `run` should run the main Python script.
    The entire tar will be unpacked in the `/opt` directory.


If the HTTP response status code is 200 the response will be a JSON object with the following fields:
* `stdout`: base 64 encoded string containing the stdout of the program
* `stderr`: base 64 encoded string containing the stdout of the program
* `exit_code`: the exit code of the program (may be null if we couldn't get the code)
