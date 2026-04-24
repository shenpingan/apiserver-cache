# apiserver-cache

基于 Rust + kube-rs + actix-web 的 Kubernetes EndpointSlice 缓存服务。

## 功能

- 使用 Kubernetes Informer 机制实时监听 EndpointSlice 变化
- 内存缓存所有 EndpointSlice 数据
- 提供 HTTP API 查询 EndpointSlice 信息
- 支持 x-api-token 认证（排除 /health/check）
- 日志中间件
- 健康检查接口
- 命令行参数管理（clap）
- YAML 配置文件

## 项目结构

```
apiserver-cache/
├── Cargo.toml              # Rust 依赖
├── config.yaml             # 配置文件
├── README.md               # 说明文档
└── src/
    ├── main.rs             # 入口
    ├── config.rs           # 配置加载
    ├── k8s/
    │   ├── mod.rs          # 模块声明
    │   ├── cache.rs        # EndpointSlice 缓存
    │   └── informer.rs     # K8s Informer
    ├── api/
    │   ├── mod.rs          # 路由配置
    │   ├── endpoint.rs     # /api/endpoint/:namespace/:name
    │   └── health.rs       # /health/check
    └── middleware/
        ├── mod.rs          # 模块声明
        └── auth.rs         # x-api-token 认证中间件
```

## 构建

```bash
cargo build --release
```

## 运行

```bash
# 使用默认配置文件
./target/release/apiserver-cache

# 指定配置文件
./target/release/apiserver-cache --config config.yaml

# 指定日志级别
./target/release/apiserver-cache --log-level debug
```

## API 接口

### 健康检查（无需认证）

```bash
curl http://localhost:8080/health/check
```

响应：
```json
{
  "status": "ok",
  "timestamp": "2026-04-24T15:39:00+00:00"
}
```

### 查询 EndpointSlice（需要认证）

```bash
curl -H "x-api-token: your-secret-token-here" \
  http://localhost:8080/api/endpoint/default/my-service
```

响应：
```json
{
  "endpoint_slice_len": 3
}
```

## 配置

编辑 `config.yaml`：

```yaml
kubernetes:
  cluster_url: ""
  kubeconfig_path: ""

server:
  host: "0.0.0.0"
  port: 8080

auth:
  api_token: "your-secret-token-here"

logging:
  level: "info"
```

## 环境变量

支持通过环境变量覆盖配置（前缀 `APP_`）：

```bash
APP_SERVER_PORT=9090
APP_AUTH_API_TOKEN=my-token
```
