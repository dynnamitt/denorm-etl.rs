# Extract,deNormalize & upload (Jira-tickets)
<img src="logo.svg" alt="Alt Text" width="300">

     Note: Work In Progress, will support more sources/deNormalizers other than JIRA

## Usecase 

Making plaintext versions before ML inference jobs inside AWS Lambda.
Can be a [During ingestion Lambda](https://docs.aws.amazon.com/bedrock/latest/userguide/kb-data-source-customize-ingestion.html#kb-data-source-customize-lambda) once the handler-code is in place.

Outside of scope; generation of embeddings and put/query in VectorDB 

## Dependecies (other than Cargo.toml)

[pandoc](https://pandoc.org/installing.html) see Makefile + layer_publish.sh 

TODO: For JIRA a custom rust-lib (using memchr) would be better to avoid cpu/mem overkill in regex/pandoc here.


# Setup envs

Make sure to setup an app token in ~/.jira_token then call

    export JIRA_TOKEN=$(cat ~/.jira_token);
    export JIRA_ENDPOINT="https://jira.mydomain.no";
    export JIRA_JQL="project=some_code AND resolution=Done"
    # more to come
    cargo run


