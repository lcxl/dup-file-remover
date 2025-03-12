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

## 数据库表设计

表名：`file_info`

| 字段 | 类型 | 说明 |
|----|----|---|
| id | int(10) unsigned | 主键ID |
| file_name | varchar(128) | 文件名 |
| file_path | varchar(256) | 文件路径 |
| md5 | char(32) | MD5值 |
| size | bigint(20) | 文件大小 |
| create_time | datetime | 创建时间 |
| update_time | timestamp | 更新时间 |

索引：
- 主键索引，自增 (id)
- 唯一索引 (md5, size)
- 唯一索引 (file_path)  
- 普通索引 (file_name)

## 扫描文件的技术实现

为了保证扫描任务的唯一性，我们采用了全局锁的方式。在扫描开始前，会尝试获取一个全局锁，如果获取成功，则进行扫描任务；如果获取失败，则表示有其他实例正在执行扫描任务，当前请求返回错误信息。

## 接口实现

### 开始扫描

curl 示例：
```bash
curl -X POST http://localhost:8081/api/dfr/scan/start \
-H "Content-Type: application/json" \
-d '{"scan_path": "/home/coder/dup-file-remover/target/release"}'
```

### 停止扫描

curl 示例：

```bash
curl -X POST http://localhost:8081/api/dfr/scan/stop \
-H "Content-Type: application/json"
```

## 前端组件

antd使用：https://ant-design.antgroup.com/docs/react/introduce-cn

umi使用 ：https://ant-design.antgroup.com/docs/react/use-with-umi-cn