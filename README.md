## Guardian Self Assessment CLI tool

A tool that generates a list of PRs authored and reviewed by you.

### Usage

1. Generate a GitHub personal access token from [here](https://github.com/settings/tokens/new). \
Set your preferred expiration date and make sure you grant the **repo** scopes. Click "Generate token".

2. Set the access token by running the following (replace `<AUTH_TOKEN>` with the generated token).
```shell
cargo run -- --auth-token <AUTH_TOKEN>
```
3. You can now run the CLI tool using the following syntax:

```shell
cargo run -- --from YYYY-MM-DD --to YYYY-MM-DD 
```

## CLI usage
```shell
self-assessment 0.1.0

USAGE:
    self-assessment [OPTIONS]

OPTIONS:
    -a, --auth-token <AUTH_TOKEN>    Github authentication token. This is needed to run the CLI
                                     tool. You can get a personal access token at
                                     https://github.com/settings/tokens/new [default: ]
    -f, --from <FROM>                Match PRs that were created after this date [default: *]
    -h, --help                       Print help information
    -t, --to <TO>                    Match PRs that were created up until this date [default: *]
    -V, --version                    Print version information
```