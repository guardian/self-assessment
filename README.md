# Guardian Self Assessment CLI tool

## What?
`self-assessment` is a tool that generates a list of PRs authored and reviewed by you, as well as an optional report of Trello boards and cards you are assigned to.

## Why? 
Assessing oneself is hard - this tool aims to make the process a little bit easier.\
It is not meant to be the be-all and end-all of the self-assessment journey. Use it as a starting point to remember your contributions to the Guardian. 

## How?

1. You need Rust to install `self-assessment`. Running the following command will install the Rust toolchain on your machine. If Rust is already installed, skip this step.
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Run `cargo install self-assessment` to install or update the CLI tool to the latest version.

3. You can now start using the CLI tool! But first, you need to generate a GitHub personal access token 👉 [here](https://github.com/settings/tokens/new) 👈\
This is required for the tool to access your pull requests in private repositories within the Guardian organisation.\
Set your preferred expiration date and make sure you grant the **repo** scopes (avoid "No expiration" for security reasons). Finally, click "Generate token".\
**NB**: You will need to re-authenticate once the token expires.
<img width="744" alt="image" src="https://user-images.githubusercontent.com/57295823/153786937-19a8bda1-2d2c-4df2-9fd0-682b6a15228f.png">


4. Set the access token by running the following command (replace `<AUTH_TOKEN>` with the generated token):
```shell
self-assessment --auth-token <AUTH_TOKEN>
```
5. You can now run the CLI tool using the following syntax:

```shell
self-assessment --from <YYYY-MM-DD> --to <YYYY-MM-DD>
```
Both the `--from` and `--to` flags are optional. **If you want to include Trello boards and cards in your report, read the [Trello report section](#trello-report).** \
If no flags are specified (i.e. if you just execute `self-assessment`), you will get a list of all PRs with no time constraints. This is not recommended, as it is likely to incur GitHub's secondary rate limit (particularly if you've been at the Guardian a long time and are a prolific contributor). 
Omitting one of the two flags also works (e.g `self-assessment --from 2021-10-01`).

If all goes well, you should see an automatically generated HTML page containing useful information about PRs authored and reviewed by you.

<img width="1766" alt="image" src="https://user-images.githubusercontent.com/57295823/154172206-6e7212c6-9d82-45d4-9937-c13c19177f5e.png">
<img width="1765" alt="image" src="https://user-images.githubusercontent.com/57295823/153787265-5afab18f-d26b-4357-acd9-2f999206b440.png">

## Trello report
In order to display the Trello cards your name is assigned to, you need to configure the CLI with a Trello API key and a token.
#### Step 1
You can obtain a key by [logging into Trello](https://trello.com/login) and then visiting https://trello.com/app-key (**please note that this page will throw an error if you are not logged in**).

Once you have obtained a key, run the following command:

```shell
self-assessment --trello-key <TRELLO_KEY>
```

#### Step 2
From the [same page](https://trello.com/app-key), click the link to generate a server token. 

<img width="613" alt="image" src="https://user-images.githubusercontent.com/57295823/155423334-0483e732-4bf5-4317-bc07-2a550473303f.png">

You will be taken to a page to generate a sever token, which is set to never expire by default (`authorize?expiration=never`). It is strongly recommended that you change the query parameter in the URL to `authorize?expiration=30days` for security reasons.

Once you have obtained your token, run the following command from the terminal:

```shell
self-assessment --trello-token <TRELLO_TOKEN>
```

Running `self-assessment` from the terminal will now generate a report including Trello cards assigned to you, as well as your authored and reviewed GitHub pull requests. The `--from <YYYY-MM-DD>` and `--to <YYYY--MM-DD>` flags are fully supported.
## CLI information
```
self-assessment 1.1.2
A CLI tool that generates a list of pull requests raised and reviewed in the Guardian's GitHub
organisation, as well as an optional summary of the user's Trello boards and cards.

USAGE:
    self-assessment [OPTIONS]

OPTIONS:
    -a, --auth-token <AUTH_TOKEN>
            Github authentication token. This is needed for the CLI tool to access the Guardian's
            private repositories to which the user has access. You can get a personal access token
            at <https://github.com/settings/tokens/new>

    -f, --from <FROM>
            Match PRs and Trello cards that were created after this date [default: *]

    -h, --help
            Print help information

    -s, --skip-trello
            Skip Trello report. Passing this flag generates a report that does not include Trello
            cards

    -t, --to <TO>
            Match PRs and Trello cards that were created up until this date [default: *]

        --trello-key <TRELLO_KEY>
            Trello API key. You can get an API key at <https://trello.com/app-key> Note: you need to
            be logged into Trello to be able to see the page. Both the API key and the server token
            need to be set for the generated report to include Trello cards

        --trello-token <TRELLO_TOKEN>
            Trello server token. You can get a server token by following the link at
            <https://trello.com/app-key> Both the API key and the server token need to be set for
            the generated report to include Trello cards

    -V, --version
            Print version information
```
