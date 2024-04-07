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

# 删除被封锁的IP
curl -X DELETE http://127.0.0.1:12345/blocked_ip/delete -H "Content-Type: application/js
on" -d '{"ipv4": "172.0.0.1"}'

# 添加被封锁的IP
curl -X POST http://127.0.0.1:12345/blocked_ip/write -H "Content-Type: application/js
on" -d '{"ipv4": "172.0.0.1"}'

# 访问被封锁的IP列表
curl http://127.0.0.1:12345/blocked_ip/read

#
```
