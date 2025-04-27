# JIRA tickets extractor,deNormalizer and uploader (ETL)

![the logo](./logo.svg)

## Usecase 

Making plaintext versions for ML inference jobs inside AWS,
outside of scope is; generation of embeddings and store in VectorDB

# Dependecies (outside Cargo.toml)

[pandoc](https://pandoc.org/installing.html)


# Setup envs

Make sure to setup an app token in ~/.jira_token then call

    export JIRA_TOKEN=$(cat ~/.jira_token);
    export JIRA_ENDPOINT="https://jira.mydomain.no";
    export JIRA_JQL="project=CMPT AND resolution=Done"
    # more to come
    cargo run
