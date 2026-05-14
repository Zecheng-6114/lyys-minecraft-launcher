# Minecraft 启动器 (Rust)

一个简化版的命令行 Minecraft 启动器，支持离线启动。

## 功能

- **list-versions**: 列出最近 20 个 Minecraft 版本
- **download**: 下载指定版本的客户端 JAR 文件
- **launch**: 完整启动流程（下载库、natives、资源）并启动游戏

## 安装

### 前提条件

- Rust 1.70+
- Java 8+（或在环境变量中配置 JAVA_PATH）

### 构建

```bash
cargo build --release
```

输出二进制文件位于 `target/release/lml.exe`。

## 使用

### 列出版本

```bash
lml list-versions
```

输出最近 20 个版本的 ID。

### 下载客户端

```bash
lml download <version>
```

例如：

```bash
lml download 1.20.1
```

客户端 JAR 将保存到 `%APPDATA%\.minecraft\versions\<version>\<version>.jar`。

### 启动游戏

```bash
lml launch <version> <username>
```

例如：

```bash
lml launch 1.20.1 MyPlayer
```

启动器将自动：
1. 下载所需的库文件（存储在 `%APPDATA%\.minecraft\libraries\`）
2. 提取 Windows natives（存储在 `%APPDATA%\.minecraft\natives\<version>\`）
3. 下载游戏资源（存储在 `%APPDATA%\.minecraft\assets\`）
4. 构建 classpath 并启动 Java 客户端

## Java 配置

启动器按以下顺序查找 Java：

1. `JAVA_HOME` 环境变量（查找 `JAVA_HOME\bin\java.exe`）
2. `JAVA_PATH` 环境变量（直接指向 java.exe）
3. 硬编码的常见路径（如 `D:\Game\Minecraft\Java\zulu*\bin\java.exe`）
4. 系统 PATH 中的 `java` 命令

### 手动配置 JAVA_PATH

在 PowerShell 中：

```powershell
$env:JAVA_PATH = "D:\Game\Minecraft\Java\zulu21.50.19-ca-fx-jdk21.0.11-win_x64\bin\java.exe"
lml launch 1.20.1 MyPlayer
```

或设置永久环境变量。

## 项目结构

```
src/
  main.rs       - CLI 入口，子命令处理
  manifest.rs   - 版本清单与元数据解析
  downloader.rs - HTTP 下载逻辑
  launcher.rs   - 库下载、natives 提取、启动逻辑
```

## 技术栈

- **clap**: 命令行参数解析
- **reqwest**: HTTP 客户端
- **serde/serde_json**: JSON 序列化
- **zip**: Native JAR 解压
- **dirs**: 跨平台目录处理

## 已知限制

- 仅支持离线启动（无微软账户集成）
- 仅在 Windows 上测试
- 不支持 Mod 管理
- 不支持多个配置文件

## 测试

在环境中设置 JAVA_PATH 后：

```powershell
$env:JAVA_PATH = "path\to\java.exe"
cargo run -- list-versions
cargo run -- download 1.12.2
cargo run -- launch 1.12.2 TestPlayer
```

## License

MIT

## 相关项目

- [console-minecraft-launcher](https://github.com/MrShieh-X/console-minecraft-launcher)
