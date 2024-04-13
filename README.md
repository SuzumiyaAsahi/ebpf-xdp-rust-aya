# 使用指南

## 前情提要

### Rust环境配置

其实Rust的环境配置超级简单，简单到让你认为一个编程语言的环境配置就应该这样简单。

```shell
# 由于ebpf程序需要在root权限下安装和关闭，所以rust需要在root下安装。

# 这条指令执行后会进入安装，整个过程中只需要按一次回车确认默认配置就可以
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 将Rust更新至前沿版本，如果这条指令执行不了，就重新开一个终端，可能是你的环境配置没有更新导致的
rustup update nightly

# 将前沿版本设置为默认版本
rustup default nightly
```

### sqlite安装

```shell
apt-get update
apt-get upgrade
apt install sqlite3 libsqlite3-dev
```

### openssl安装

```shell
apt install openssl
apt install pkg-config
apt install libssl-dev
```

### ebpf运行环境配置

主要是来自于这里，不过对这个项目有用的是在下面被挑了出来
https://aya-rs.dev/book/start/development/

```shell
# 也要在root下进行
cargo install bpf-linker

# 不过可能说没有cc之类的
# 那就安装一下
apt update
apt install build-essential
```

### 根据自己的网卡修改

```shell
# 查看网卡名称
ip a
```

然后修改 test-app/src/main.rs 代码

应该是在第十三行左右。

```rust

// 我们这个XDP程序绑定的是eth0网卡
#[derive(Debug, Parser)]
struct Opt {
    #[clap(short, long, default_value = "eth0")]
    iface: String,
}

```

将这里的eth0修改为自己的ebpf需要绑定的网卡名称，注意不要写错，然后保存退出即可。

### WSL迁移

```bash
wsl --export Ubuntu-22.04 E://wslubuntu//Ubuntu-22.04.tar
wsl --unregister Ubuntu-22.04
wsl --import Ubuntu-22.04 E://wslubuntu E://wslubuntu//ubuntu-22.04.tar
```

## 后端项目启动

```shell
# 由于ebpf程序需要在root权限下安装和关闭，所以下面的指令需要在root权限下运行。

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
