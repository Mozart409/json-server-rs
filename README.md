# JSON Server RS

## Table of Contents

- [About](#about)
- [Installing](#installing)
- [Usage](#usage)
- [Contributing](../CONTRIBUTING.md)

## About <a name = "about"></a>

JSON Server RS is a simple REST API for testing and prototyping or CI/CD pipelines. Shove your JSON data into a file and move it to the /data folder. The name of the file, e.g. "articles.json", will be the name of the endpoint, e.g. "http://localhost:3000/api/articles". The data will be served as a JSON array. To view all "endpoints" visit "http://localhost:3000/api" or "http://localhost:3000/api/".

## Installing <a name = "installing"></a>

Download the executable from cargo and place it in your path.

**THIS IS NOT IMPLEMENTED YET**

```sh
cargo install json-server-rs
```

```sh
brew install json-server-rs
```

```sh
sudo dnf install json-server-rs
```

```sh
sudo apt-get install json-server-rs
```

## Usage <a name = "usage"></a>

Add notes about how to use the system.

```sh
json-server-rs -p 8888 -d ./api
```
