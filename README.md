# dup-file-remover

dup file remover是一款查找和删除重复文件的软件，支持docker模式部署。后端使用rust编写，前端使用antd和react。


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
