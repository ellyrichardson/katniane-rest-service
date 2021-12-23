# Katniane REST Service
Rust-based REST service for the Katniane blockchain

# Scope
The scope of this project is to handle the composing of extrinsics, which are logs, that needs to be 
saved in the Katniane blockchain. This rest service ensures that the logs saved in Katniane
comply to the Katniane log semantics. This rest service also handles the logic needed for the extrinsics
saved in Katniane, such as the signing of extrincs, conforming to Scale CODEC, communicating to Katniane Node, etc.

# Katniane Log Semantic
```rs
pub struct AuditLog {
    pub title: String,
    pub content: String,
    pub timestamp: String,
    pub reporter: Public,
}
```
The `title` field is the title of the log, in which the value is provided by the user/entity.
The `content` field is the content of the log, which the value is provided by the user/entity.
The `timestamp` field is the timestamp of the log, in which the REST service generates at the time it
received the log it needs to submit to Katniane. The timezone for the timestamp generated is the local
timezone in which the rest service runs on.
The `reporter` field is the public key of the user/entity who sent the logs. The REST service will
extract this data so theres no need for the user/entity to provide it in the request. This is to identify
entities that has added entries to the log file.

# Usage
In order for users/entities to be able to save logs to Katniane using this REST service, HTTP methods
are available for usage.

### Saving Logs
To save a log via the Katniane REST service, the HTTP `POST` method 
can be utilized for the `/v1/logs` endpoint.

`curl` example:
```sh
curl -X POST 127.0.0.1:3030/v1/logs \
-H 'Content-Type: application/json' \
-d '{
"filename":"test_log_file1",
"title":"test_four_title",
"content":"content4"
}'
```
The `filename` field of the JSON payload is the filename where the logs are supposed to 
be mapped and associated with. 

### Retrieving Logs
To retrieve logs via the Katniane REST service, the HTTP `GET` method can be utilized 
for the `/v1/logs/<filename>/<timestamp>` endpoint. 

URL example:
```
127.0.0.1:3030/v1/logs/test_log_file1/2021-12-14
```

This endpoint will return all the entries saved in a log file that falls under a date. The entries returned
will be in JSON format. The entries will be unordered, but can be sorted with once parsed as each entries
contain a timestamp.

# Project Status
This project is currently in `pre-alpha` stage and is in active development. This project is also being 
worked on concurrently with the Katniane chain.

## Getting Started

Follow the steps below to get started with the Katniane REST Service.

### Rust Setup

First, complete the [basic Rust setup instructions](./doc/rust-setup.md).

### Katniane Node

Setup and run a Katniane Node by following the guide in the [Katniane Node](https://github.com/ellyrichardson/katniane-node) repo

### Run

Use Rust's native `cargo` command to launch the project

```sh
cargo run
```

### Build

The `cargo run` command will perform an initial build. Use the following command to build the project
without launching it:

```sh
cargo build --release
```

