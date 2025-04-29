# Extract,deNormalize & upload (Jira-tickets)
<img src="logo.svg" alt="Alt Text" width="300">

   Note: Work In Progress, will support more sources/transformers other than JIRA

## Usecase 

Making plaintext versions for ML inference jobs inside AWS Lambda.

Outside of scope; generation of embeddings and storage in VectorDB

## Dependecies (other than Cargo.toml)

[pandoc](https://pandoc.org/installing.html) see layer_pulish.sh 

TODO: For JIRA a custom rust-lib (memchr) would be better to avoid cpu/mem overkill in regex/pandoc here.


# Setup envs

Make sure to setup an app token in ~/.jira_token then call

    export JIRA_TOKEN=$(cat ~/.jira_token);
    export JIRA_ENDPOINT="https://jira.mydomain.no";
    export JIRA_JQL="project=some_code AND resolution=Done"
    # more to come
    cargo run
