# Guardian Self Assessment CLI tool

## What?
`self-assessment` is a tool that generates a list of PRs authored and reviewed by you.

## Why? 
Assessing oneself is hard - this tool aims to make the process a little bit easier.\
It is not meant to be the be-all and end-all of the self-assessment journey. Use it as a starting point to remember your contributions to the Guardian. 

## Usage

1. You need Rust to install `self-assessment`. Running the following command will install the Rust toolchain on your machine. If Rust is already installed, skip this step.
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Run `cargo install self-assessment` to install the CLI tool.

3. You can now use the CLI tool! But first, you need to generate a GitHub personal access token 👉 [here](https://github.com/settings/tokens/new) 👈\
Set your preferred expiration date and make sure you grant the **repo** scopes (avoid "No expiration" for security reasons). Finally, click "Generate token".\
**NB**: You will need to re-authenticate once the token expires.
<img width="744" alt="image" src="https://user-images.githubusercontent.com/57295823/153786937-19a8bda1-2d2c-4df2-9fd0-682b6a15228f.png">


6. Set the access token by running the following command (replace `<AUTH_TOKEN>` with the generated token):
```shell
self-assessment --auth-token <AUTH_TOKEN>
```
7. You can now run the CLI tool using the following syntax:

```shell
self-assessment --from <YYYY-MM-DD> --to <YYYY-MM-DD>
```
Both the `--from` and `--to` flags are optional.\
If no flags are specified (i.e. if you just execute `self-assessment`), you will get a list of all PRs with no time constraints. This is not recommended, as it is likely to incur GitHub's secondary rate limit (particularly if you've been at the Guardian a long time and are a prolific contributor). 
Omitting one of the two flags also works (e.g `self-assessment --from 2021-10-01`).

If all goes well, you should see an automatically generated HTML page containing useful information about PRs authored and reviewed by you.

<img width="1769" alt="image" src="https://user-images.githubusercontent.com/57295823/153787048-c8b8174b-ce81-43c2-9620-52ddc7077848.png">
<img width="1765" alt="image" src="https://user-images.githubusercontent.com/57295823/153787265-5afab18f-d26b-4357-acd9-2f999206b440.png">

## CLI information
```
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
