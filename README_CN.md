# dup-file-remover

dup file remover是一款查找和删除重复文件的软件，支持docker模式部署。后端使用 rust 编写，前端使用 ant design pro 解决方案。

## 安装

### Docker部署

docker部署非常简单，只需要拉取镜像并运行容器即可。运行命令如下：
```bash
docker run -d --name dup-file-remover \
    -p 8081:8081 \
    -v /path/to/conf:/app/conf \
    -v /path/to/data:/app/data \
    lcxl/dup-file-remover:latest
```

docker compose 部署方式：
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

这里 `/path/to/conf` 和 `/path/to/data` 是配置文件和数据的存储目录，`/path/to/data` 要指向到删除重复文件的目录，并且需要有读写权限，否则程序运行可能会有问题。

## 从源码构建

如果你想从源代码开始进行构建，可以按照以下方式：

* 克隆此代码仓库到本地；
* 安装 docker，用于构建镜像，docker 安装参考 [docker 官方文档](https://docs.docker.com/engine/install/)
* 执行 `build_docker.sh` 命令构建镜像，生成的镜像名称为 `dup-file-remover`
* 使用 `docker run` 命令运行镜像，参考上面的配置文件。

## 应用开发

如果你想进行开发，可以按照以下方式：
* 克隆此代码仓库到本地；
* 安装必须的软件：
    * Rust：用于编译后端应用，参考 [Rust 官方文档](https://www.rust-lang.org/learn/get-started)
    * NPM: 用于编译Web前端应用，参考 [NPM 官方文档](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm)
* 在代码仓库根目录中执行 `cargo run` 运行后端应用；
* 切到 `web` 目录，执行 `npm run start` 运行前端应用；
* 打开游览器，访问 `http://localhost:8000` 查看应用。

应用前端和后端使用openapi协议进行通信。当后端 HTTP 接口有变更时，在后端运行期间执行以下命令 `./update_web_openapi.sh` 对前端应用的 openapi 接口进行更新操作。

## 前端组件

antd使用：https://ant-design.antgroup.com/docs/react/introduce-cn

umi使用 ：https://ant-design.antgroup.com/docs/react/use-with-umi-cn