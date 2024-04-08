# 使用指南

## 后端项目启动

```shell
# 启动ebpf程序
make ebpf

# 然后再开一个终端,启动http服务
make server

# 注意：一定要先启动ebpf程序，再启动http程序
```

## http访问构造

```shell
# 这里的"ipv4"这个JSON数据要根据实际情况来变化

# 不过实际上想添加或者删除封锁IP的数据都需要在ebpf程序运行之前才有效，因为ebpf程序是先在运行前读取数据库之后才运行。
# ebpf运行之后也能添加或者删除IP地址，但要再次运行ebpf程序才能生效。

# 删除被封锁的IP
curl -X DELETE http://127.0.0.1:12345/blocked_ip/delete -H "Content-Type: application/js
on" -d '{"ipv4": "172.0.0.1"}'

# 添加被封锁的IP
curl -X POST http://127.0.0.1:12345/blocked_ip/write -H "Content-Type: application/js
on" -d '{"ipv4": "172.0.0.1"}'

# 访问被封锁的IP列表
curl http://127.0.0.1:12345/blocked_ip/read

# 访问捕获的IP数据信息
# 暂时就返回这么几样
# pub struct PackageInfo {
#   pub source_ip: String,
#   pub source_port: i64,
#   pub destination_port: i64,
#   pub proto_type: String,
# }
curl http://127.0.0.1:12345/package_info/read
```

## 数据库

数据库使用的是sqlite3，因为它比较简单，一个文件就是一个数据库，就是 identifier.sqlite

具体使用方法我看的是这个

https://www.runoob.com/sqlite/sqlite-tutorial.html

## 开发环境

不过我已经安装好了，这里仅当备忘

https://aya-rs.dev/book/start/development/
