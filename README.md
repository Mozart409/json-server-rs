# JSON Server RS

## Table of Contents

- [About](#about)
- [Installing](#installing)
- [Usage](#usage)
- [Contributing](../CONTRIBUTING.md)

## About <a name = "about"></a>

JSON Server RS is a user-friendly REST API designed to facilitate testing, prototyping, and continuous integration/continuous deployment (CI/CD) pipelines. The server allows users to easily store JSON data in a file and transfer it to the designated "/data" folder. By naming the file, for example, "articles.json", the corresponding endpoint will also be named "http://localhost:3000/api/articles". The data is then served as a JSON array.

To view all available endpoints, users can navigate to "http://localhost:3000/api" or "http://localhost:3000/api/" in their web browser. This straightforward approach enables users to efficiently manage and manipulate their data, making it a valuable tool in various development and testing scenarios.

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
