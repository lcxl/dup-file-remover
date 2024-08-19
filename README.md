# dup-file-remover

dup file remover是一款查找和删除重复文件的软件，支持docker模式部署。后端使用rust编写，前端使用antd和react。

## 安装

### Docker部署

docker部署非常简单，只需要拉取镜像并运行容器即可。运行命令如下：
```bash
docker run -d --name dup-file-remover \
    -p 80:80 \
    -v /path/to/config:/app/config \
    -v /path/to/data:/app/data \
    lcxl/dup-file-remover:latest
```

这里`/path/to/config`和`/path/to/data`是配置文件和数据的存储目录，可以根据实际情况进行修改。

## 数据库表设计

表名：`file_info`

| 字段 | 类型 | 说明 |
|----|----|---|
| id | int(10) unsigned | 主键ID |
| file_path | varchar(256) | 文件路径 |
| file_name | varchar(128) | 文件名 |
| md5 | char(32) | MD5值 |
| size | bigint(20) | 文件大小 |
| create_time | datetime | 创建时间 |
| update_time | timestamp | 更新时间 |

索引：
- 主键索引，自增 (id)
- 唯一索引 (md5, size)
- 唯一索引 (file_path)  
- 普通索引 (file_name)

## 前端组件

antd使用：https://ant-design.antgroup.com/docs/react/introduce-cn
umi使用 ：https://ant-design.antgroup.com/docs/react/use-with-umi-cn