# 使用指南

## 后端项目启动

```shell
# 启动ebpf程序和http_server
make run

# 关闭ebpf程序和http_server
make kill
```

## http访问构造

```shell
# 这里的"ipv4"这个JSON数据要根据实际情况来变化

# 添加被封锁的IP
curl -X POST http://127.0.0.1:12345/blocked_ip/write -H "Content-Type: application/json" -d '{"ipv4": "172.0.0.1"}'

# 删除被封锁的IP
curl -X DELETE http://127.0.0.1:12345/blocked_ip/delete -H "Content-Type: application/json" -d '{"ipv4": "172.0.0.1"}'

# 删除所有被封锁的IP
curl -X DELETE http://127.0.0.1:12345/blocked_ip/flush

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


# 下面这些前端不用看

# 关闭ebpf程序
curl http://127.0.0.1:12345/kill_restart/kill

# 启动ebpf程序
curl http://127.0.0.1:12345/kill_restart/restart

# 关闭ebpf程序并重新启动
curl http://127.0.0.1:12345/kill_restart/kill_and_restart
```

## 数据库

数据库使用的是sqlite3，因为它比较简单，一个文件就是一个数据库，就是 identifier.sqlite

具体使用方法我看的是这个

https://www.runoob.com/sqlite/sqlite-tutorial.html

## ebpf运行环境配置

不过我已经安装好了，这里仅当备忘

https://aya-rs.dev/book/start/development/
