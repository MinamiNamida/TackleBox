完全同意。把中心化架构平滑升级到「可运行的 P2P 版本」确实要一份**详细的设计说明**。下面给你一份“能落地”的高保真设计文档（**不写具体代码**），覆盖：目标、角色与组件、连接建链与通道管理、消息协议、房间/游戏生命周期、路由与转发、NAT/Relay、安全、错误恢复、资源与限流、监控度量，以及灰度上线策略。你可以直接拿它当执行蓝本。

# 0. 设计目标（非功能/功能）

* **稳定性**：节点上下线、房间解散、游戏结束要自然收敛（无悬挂任务/通道）。
* **低延迟**：同房间内尽量直连；必要时可通过近端 relay。
* **可演进**：保留中心化 Platform 能力（Bootstrap/Tracker/Relay/协调），允许关闭/收缩以趋近纯 P2P。
* **最小改动**：沿用你现有的 Client / Platform / Room / Game 概念与数据流。
* **安全留接口**：初期可不启用鉴权，但协议天然预留签名/可鉴别字段。

# 1. 角色与组件

* **Client（对等节点）**

  * 持有：WS（或QUIC/TCP）到本地服务的通道（对外与UI）、到其他节点的多条 P2P 链路（直连或经 relay）。
  * 维护：`room_id → PeerSet`、`peer_id → Link`、`room_id → game_state proxy`。
* **Platform（Bootstrap/Tracker/可选 Relay）**

  * 提供**发现服务**（room 成员候选 peer 列表、候选 relay 列表）。
  * 负责**初始协调**（谁是临时房主/协调者），**不接管数据通道**。
  * 可选充当 **TURN/中继**（在打洞失败时转发）。
* **Room（逻辑 overlay）**

  * 不是中心化进程，而是**各 Client 的一个本地逻辑体**：保存本房间成员、路由表、转发策略、广播策略。
  * 每个 Client 的 Room 逻辑只管理**自己这份视图**（最终一致）。
* **Game（状态机/业务逻辑）**

  * 从 Room 接收玩家动作事件，产出事件再由 Room 广播。
  * 不直接管理链路或注册 client 通道（Room 做网关）。

# 2. 链路/通道模型（强约束）

* **控制通道（Control Plane）**：用于建链、成员变更、拓扑维护。

  * 载体：与 Platform 的 WS/HTTP(S)，与 Peers 的轻量控制流（可共用数据连接的一个逻辑子流）。
* **数据通道（Data Plane）**：玩家事件、房间广播、游戏结果。

  * 载体：优先 Peer↔Peer 直连；失败时 Peer↔Relay↔Peer。
* **通道单位**：以“**双向通道对**”计（语义同你之前的计数口径）。
* **Room 内 fan-out**：

  * 默认用**每 Peer 一条上行发送通道**（对端各自接收），或
  * 用**broadcast 语义**（单发多收，注意丢包与缓冲上限）。

# 3. 建链与加入流程（时序）

**场景：Client X 加入已存在 Room R（目标是尽量直连成员）**

1. `Client→Platform`: JoinRoom(room\_id)
2. `Platform`: 返回 `JoinGrant`（候选 peers 列表：`[(peer_id, addr_set, pubkey), …]`、候选 relay 列表、临时 token）
3. `Client`: 对每个候选 peer 发起 **NAT 穿透/直连尝试**（UDP hole punching / QUIC / TCP fallback）
4. 若直连失败，尝试通过 `relay` 建立 **中继数据通道**
5. 建链成功后，`Client` 向房内成员广播 `Hello(own_peer_meta)`（经任一已连通成员转发到全网）
6. 房内成员更新各自 `PeerSet` 与 `RoutingTable`，回发 `Welcome/PeersDelta`
7. `Client` 的 Room 层把本地 `client_tx` 注册进房间参与者集；Game 仍只面对 Room

> **退路**：若组网全部失败，Platform 可提供“代转发”模式（中心化兜底）。

# 4. 拓扑与路由（Room 内）

* **拓扑选择**（默认）

  * 小房间（d ≤ 6\~8）：**全连**（Mesh），最低延迟。
  * 中房间（d 10\~32）：**稀疏网 + 局部最短路**（每节点维持 k 个最近/稳定邻居，k≈√d 或固定上限）。
  * 大房间（d > 32）：**分片/簇**（cluster-head/relay-peers），簇内全连，簇间由簇头互连。
* **路由**

  * 房内数据采用**房间 ID + 发起 peer\_id + 消息序号**标识，路由策略：

    * 若直达：直发；
    * 否则：按邻居表/簇路由至下一跳；
  * **环路防护**：`seen(message_id)` 去重；TTL 限制。
* **广播**

  * 优先用**树形/簇形广播**（减少重复），Mesh 下可用 gossip 减冗余。

# 5. 消息协议（不含签名也可跑）

通用头（所有消息前）：

```
Header {
  proto_ver,
  room_id,
  from_peer,
  kind: Control | Data,
  seq,               // 局部序号
  ts,                // 发送时间戳
}
```

控制面（示例）：

* `Hello { peer_meta }`
* `Welcome { peers_delta }`
* `PeersDelta { joined:[], left:[] }`
* `LinkProbe { rtt_echo }` / `LinkPong { echo }`
* `Bye { reason }`
* `RelayOffer/RelayAck`（协商通过某 relay）

数据面（示例）：

* `RoomEvent { subtype: JoinAck/LeaveAck/Broadcast/... , payload }`
* `GameAction { actor_id, action_type, data }`
* `GameResult { to: One/All, payload }`

> 约束：**Game 只消费数据面**；**Room 负责把数据面进出映射到本地 `client_tx`**。

# 6. 生命周期与自动注销

* **Client 断开（本地 WS 关闭）**

  * 本地 Room 逻辑向房内 peers 发 `Bye`（或由对端超时检测）。
  * 停止所有对外链接（直连 & relay），清空映射。
* **Room 清理（本地视角）**

  * 本地 `PeerSet` 为空 → 本地 Room 任务退出。
  * 房间全局收敛：各节点都如此退出 → Overlay 自然消散。
* **Game 清理**

  * Game 完成/中止 → 发 `GameClosed` 事件到本地 Room → Room 广播。
  * 本地 Game 任务退出；如房内全部 Game 结束且无 peer，Room 退出。
* **Platform 清理**（仅保存弱态数据）

  * 仅维护“房间存在/成员索引”的弱态；长时间无心跳的房间从 Tracker 列表剔除。

# 7. NAT 穿透与中继（必需考虑）

* **优先**：QUIC/UDP 打洞（对称 NAT 失败概率高时减少重试）。
* **回退**：TCP 直连（通过端口映射/UPnP）。
* **兜底**：Platform 提供可伸缩 `Relay`（多实例、分区域）。
* **选择策略**：

  * 先近（地理/延迟）后远，先直连后 relay。
  * 定期 `LinkProbe` 评估质量，必要时**无缝切换路径**（Make-before-break）。

# 8. 可靠性与一致性

* **消息有序/去重**：`(from_peer, seq)` + `seen set`；关键路径可按房内会话分 `stream` 单独排队。
* **幂等**：设计 GameAction/RoomEvent 的语义幂等或可重放（带版本/帧号）。
* **流控**：发送端基于对端反馈/通道长度做背压；丢弃策略区分控制面/数据面（控制面优先）。
* **断线重连**：

  * 短暂断链保留本地 state，重连后通过 `PeersDelta`/`StateDigest` 做**快速增量同步**。
  * 超时未恢复 → 视为离开，Room 收敛。

# 9. 资源与限流（避免“通道爆炸”）

* **每节点连接上限**：

  * 小房：全连；
  * 中/大房：每节点邻居上限 `k`（默认 6\~10），超过通过簇/relay 承担。
* **每房间通道上限**：对 `room_id` 层面设置 `max_links_per_peer`。
* **按需建立/回收**：

  * 只有在「近 30s 内有真实互动」才维持直连；否则降级为经簇/relay。
  * 活跃度阈值与 d/c 预算挂钩（你之前的 F/S 比例公式可用来选阈值）。
* **消息速率限制**：对单 peer、单房、全局分别做 rate limit（令牌桶/滑动窗口）。

# 10. 监控与度量（最小集）

* **链路健康**：RTT、丢包率、重试次数、relay 使用率。
* **拓扑形态**：房间 d、每节点度数、簇规模、路径长度分布。
* **负载**：消息吞吐（Control/Data）、背压触发次数、缓冲占用。
* **回收效果**：房间空闲时长、自动退出次数、孤儿任务检测为 0。

# 11. 安全与演进（先留口，后启用）

* **身份**：每节点生成长期公私钥；`peer_id = hash(pubkey)`。
* **消息签名**：对控制面消息签名；数据面可选签名或会话 MAC。
* **重放保护**：`ts` + 窗口校验。
* **权限**：Room 级白名单/黑名单；房主/协调者角色可迁移（view-change）。
* **隐私**：地址只经 Platform 发放给同房候选；跨房间不互泄。

# 12. 与中心化模式的兼容/灰度

* **模式开关**：按房间开启 P2P；未开启的房间沿用中心化（Platform 转发）。
* **阶段 1**：P2P 仅在小房间（d≤6）启用，全连 Mesh，验证消息顺序/丢包处理。
* **阶段 2**：启用 NAT 穿透 + Relay 兜底，收集链路质量分布。
* **阶段 3**：引入簇/稀疏拓扑与自动选路，放开到中房间。
* **回退策略**：任意时刻可切回中心化（Platform 强制接管数据面）。

# 13. 关键数据结构（抽象，无具体代码）

* `PeerMeta { peer_id, addrs[], caps, pubkey?, rtt_est, last_active }`
* `Link { peer_id, kind: Direct|Relay(peer_id/relay_id), streams{control,data...}, health }`
* `RoomLocalState { room_id, peers: HashMap<peer_id, PeerMeta>, links: HashMap<peer_id, Link>, topology: ClusterInfo?, routing: RoutingTable }`
* `RoutingTable { next_hop: HashMap<peer_id, peer_id>, version }`
* `SeenSet { (from, seq) -> ts }`（LRU）
* `RateLimiter { scope_key -> tokens }`

# 14. 典型时序小抄（无代码）

**房内广播（小房全连）：**
`Game -> Room(local) -> for each peer link: send(Data.Broadcast)`

**单播到指定玩家（可能跨簇）：**
`Game -> Room(local) -> route(to_peer_id) -> ...hops... -> target Room(remote) -> client_tx`

**成员退出：**
`Client(local) closes -> Room(local) sends Bye -> peers remove -> if peers empty -> Room(local) exit`

**链路切换（降级到 relay）：**
`LinkProbe spikes -> Room picks relay -> establish relay link -> drain in-flight -> cut old link`

# 15. 你需要落地时的默认参数（开箱即用）

* 小房阈值 `d_small = 6`：全连 Mesh
* 邻居上限 `k = 8`（中房）
* 直连尝试超时 `1.5s` / 重试 2 次；失败即尝试 relay
* 消息缓冲：控制面 128、数据面 1024（按需调）
* 活跃降级阈值：`30s` 无互动降级到经簇/relay
* SeenSet 容量：10k（LRU），TTL 5s（按包率调）
* 速率限制：单 peer 200 msg/s、单房 2k msg/s、全局 20k msg/s（示例）

---

## 最后总结（一句话）

把 Room 变成**各节点本地的“逻辑网关 + 路由器”**，Game 专注业务；Platform 收缩成 Tracker/Relay。通过**直连优先、relay 兜底、簇化路由、按需建链与自动回收**，你能在不牺牲你当前分层模型的前提下，稳稳把系统演进到 P2P，同时保住可控的通道数与延迟。

如果你想，我可以把这份设计转换成一张“单页蓝图”（ASCII 架构图 + 时序图），方便你团队评审和任务拆解。
