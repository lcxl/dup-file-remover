# dup-file-remover
[中文说明](./README_CN.md)

Dup File Remover is a software for detecting and removing duplicate files, supporting deployment in Docker mode. The backend is written in Rust, while the frontend uses the Ant Design Pro solution.
## Installation
### Docker Deployment
Docker deployment is very simple; you just need to pull the image and run the container. The command to run is as follows:
```bash
docker run -d --name dup-file-remover \
    -p 8081:8081 \
    -v /path/to/conf:/app/conf \
    -v /path/to/data:/app/data \
    lcxl/dup-file-remover:latest
```
Docker Compose deployment method:
```yaml
version: '3'
services:
  dfr:
    container_name: dfr
    image: lcxl/dup-file-remover:latest
    ports:
      - "8081:8081"
    restart: unless-stopped
    volumes:
      - /mnt/lcxlstorage/nasdata:/app/data
      - /mnt/lcxlstorage/appconfig/dfr:/app/conf
```
Here, `/path/to/conf` and `/path/to/data` are the directories for storing configuration files and data, respectively. The `/path/to/data` directory should point to where duplicate files will be removed and must have read/write permissions; otherwise, there may be issues with program execution.
## Building from Source
If you want to build from source code, you can follow these steps:
* Clone this code repository locally;
* Install Docker for image building. For Docker installation, refer to the [Docker official documentation](https://docs.docker.com/engine/install/)
* Execute the `build_docker.sh` command to build the image; the generated image name will be `dup-file-remover`
* Run the image using the `docker run` command, referencing the above configuration file.
## Application Development
If you want to develop, you can follow these steps:
* Clone this code repository locally;
* Install the necessary software:
    * Rust: for compiling the backend application; refer to [Rust official documentation](https://www.rust-lang.org/learn/get-started)
    * NPM: for compiling the Web frontend application; refer to [NPM official documentation](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
* Execute `cargo run` in the code repository root directory to run the backend application;
* Switch to the `web` directory, execute `npm run start` to run the frontend application;
* Open a browser and access `http://localhost:8000` to view the application.
The front-end and back-end of the application communicate using the OpenAPI protocol. When there is a change in the backend HTTP interface, run the following command during runtime in the backend: `./update_web_openapi.sh` to update the OpenAPI interface of the frontend application.
## Frontend Components
Antd usage: https://ant-design.antgroup.com/docs/react/introduce-cn  
Umi usage: https://ant-design.antgroup.com/docs/react/use-with-umi-cn