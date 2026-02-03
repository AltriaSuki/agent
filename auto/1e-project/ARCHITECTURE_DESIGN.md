# 日省录 · 技术架构设计文档

> 基于现有系统的深度分析与重构方案

## 📋 文档概览

**文档版本**: v1.0
**创建日期**: 2026-02-03
**适用范围**: Daily Reckoning / 日省录 v2.0+
**设计目标**: 构建可扩展、高性能、易维护的技术架构

---

## 🎯 Executive Summary (执行摘要)

本文档基于对现有系统的全面评估，提出了一套系统性的架构改进方案。核心改进包括：

1. **数据层重构**: 从 JSON 文件迁移到嵌入式数据库（SQLite）
2. **代码架构优化**: 完成模块化重构，建立清晰的分层架构
3. **前端现代化**: 引入模块化、类型安全和状态管理
4. **性能优化**: 实现缓存机制、增量计算、异步处理
5. **安全增强**: 本地加密、安全存储、隐私保护
6. **可观测性**: 日志系统、性能监控、错误追踪

**预期成果**:
- 性能提升 3-5 倍（数据库索引 + 缓存）
- 代码可维护性提升 60%（模块化 + 类型安全）
- 数据安全性提升（端到端加密）
- 支持 10 万+ 条记录无性能衰减

---

## 🔍 现状分析

### 1.1 当前架构概览

```
┌─────────────────────────────────────────────────────┐
│                    Web / Android UI                 │
│            (Vanilla JS + Kotlin WebView)            │
└──────────────────┬──────────────────────────────────┘
                   │ HTTP API / JNI
┌──────────────────▼──────────────────────────────────┐
│              Rust Core (单体架构)                     │
│  ┌─────────────────────────────────────────────┐    │
│  │  core.rs (正在拆分中)                        │    │
│  │  - 业务逻辑 + 数据处理 + 存储耦合            │    │
│  └─────────────────────────────────────────────┘    │
└──────────────────┬──────────────────────────────────┘
                   │
┌──────────────────▼──────────────────────────────────┐
│            JSON 文件存储 (state.json)                │
│         - 全量读写 - 无索引 - 无事务                 │
└─────────────────────────────────────────────────────┘
```

### 1.2 核心技术栈

| 层级 | 技术 | 问题 |
|------|------|------|
| **前端 (Web)** | Vanilla JS (1625行) | 无模块化、无类型检查、难维护 |
| **前端 (Android)** | Kotlin + WebView + JNI | JNI 开销大、调试困难 |
| **后端核心** | Rust + Axum | 正在重构中，技术债务 |
| **存储** | JSON 文件 | 性能瓶颈、无事务保证 |
| **部署** | 本地 (无 CI/CD) | 手动构建、无自动化测试 |

### 1.3 关键痛点 (按优先级排序)

#### 🔴 P0 - 阻塞性问题

1. **数据存储瓶颈**
   - 每次读写都要加载/序列化整个 `state.json`
   - 数据量达到 1000+ 天后性能明显下降
   - 无事务保证，写入失败可能导致数据损坏
   - 无法进行复杂查询（如全文搜索、日期范围过滤）

2. **前端可维护性差**
   - 1625 行单文件 JS，无模块划分
   - 无类型系统，运行时错误难以预防
   - 状态管理混乱，`window.state` 全局变量

#### 🟡 P1 - 严重影响

3. **代码组织混乱**
   - 正在进行的重构尚未完成
   - 业务逻辑、数据访问、API 接口耦合
   - 测试覆盖率不足（无前端测试、缺少单元测试）

4. **安全性不足**
   - 数据明文存储，无加密保护
   - SMTP 密码以环境变量明文配置
   - 无身份验证机制

#### 🟢 P2 - 长期优化

5. **缺乏可观测性**
   - 无结构化日志系统
   - 无性能监控和指标采集
   - 线上问题难以定位

6. **开发流程不规范**
   - 无 CI/CD 流程
   - 手动测试为主
   - 无代码质量检查

---

## 🏗️ 架构重构设计

### 2.1 目标架构

```
┌──────────────────────────────────────────────────────────┐
│                    Presentation Layer                     │
│  ┌──────────────┐              ┌──────────────────────┐  │
│  │  Web UI      │              │   Android Native     │  │
│  │  (TypeScript)│              │   (Kotlin + Jetpack) │  │
│  └──────┬───────┘              └──────────┬───────────┘  │
└─────────┼──────────────────────────────────┼──────────────┘
          │ HTTP/WS                          │ FFI (uniffi)
┌─────────▼──────────────────────────────────▼──────────────┐
│                      API Gateway Layer                     │
│  ┌────────────────────────────────────────────────────┐   │
│  │  Axum Router + Middleware (认证/日志/限流)         │   │
│  └────────────────────┬───────────────────────────────┘   │
└───────────────────────┼───────────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────────┐
│                    Domain Layer (核心业务)                 │
│  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │ Entities    │  │  Use Cases   │  │  Domain Events  │  │
│  │ (领域模型) │  │  (业务逻辑)  │  │  (事件系统)     │  │
│  └─────────────┘  └──────────────┘  └─────────────────┘  │
└───────────────────────┬───────────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────────┐
│                  Infrastructure Layer                      │
│  ┌────────────┐  ┌──────────┐  ┌─────────┐  ┌─────────┐  │
│  │ Repository │  │  Cache   │  │ Crypto  │  │ Logger  │  │
│  │  (SQLite)  │  │ (Memory) │  │(AES-GCM)│  │(Tracing)│  │
│  └────────────┘  └──────────┘  └─────────┘  └─────────┘  │
└───────────────────────────────────────────────────────────┘
```

### 2.2 分层职责

#### **Presentation Layer (表现层)**
- **职责**: UI 渲染、用户交互、状态展示
- **技术**:
  - Web: TypeScript + 轻量级框架 (Svelte/Alpine.js)
  - Android: Kotlin + Jetpack Compose (逐步替代 WebView)
- **原则**:
  - 不包含业务逻辑
  - 只调用 API Gateway 提供的接口
  - 状态管理本地化

#### **API Gateway Layer (网关层)**
- **职责**: 路由、认证、限流、日志、错误处理
- **技术**: Axum + Tower Middleware
- **功能**:
  - RESTful API 路由
  - WebSocket 支持（实时通知）
  - 请求验证和日志记录
  - 统一错误响应格式

#### **Domain Layer (领域层)**
- **职责**: 核心业务逻辑、领域规则、数据验证
- **技术**: Pure Rust (无 I/O 依赖)
- **组件**:
  - **Entities**: `Reflection`, `Achievement`, `Statistic` 等领域实体
  - **Use Cases**: `SubmitReflection`, `CalculateStats`, `UnlockAchievement` 等用例
  - **Domain Events**: `ReflectionSubmitted`, `StreakBroken` 等事件
- **原则**:
  - 依赖倒置（不依赖基础设施层）
  - 单元测试友好
  - 业务规则集中管理

#### **Infrastructure Layer (基础设施层)**
- **职责**: 数据持久化、缓存、加密、日志等技术细节
- **技术**: SQLite, AES-GCM, Tracing
- **组件**:
  - **Repository**: 数据访问抽象（trait 定义 + SQLite 实现）
  - **Cache**: 内存缓存（统计数据、成就状态）
  - **Crypto**: 加密解密服务
  - **Logger**: 结构化日志

---

## 🗄️ 数据层重构设计

### 3.1 从 JSON 到 SQLite

#### 当前 JSON 方案问题
```rust
// 每次操作都要：
1. 读取整个 state.json 文件 (I/O 密集)
2. 反序列化为内存对象 (CPU 密集)
3. 修改数据
4. 序列化回 JSON (CPU 密集)
5. 写回文件 (I/O 密集)

// 随着数据增长：
- 100 天记录 ≈ 50KB  → 50ms  延迟
- 365 天记录 ≈ 180KB → 200ms 延迟
- 1000天记录 ≈ 500KB → 800ms 延迟
```

#### SQLite 方案优势
- ✅ **增量读写**: 只操作需要的数据
- ✅ **事务支持**: ACID 保证数据一致性
- ✅ **索引加速**: 日期、时间段查询毫秒级
- ✅ **全文搜索**: FTS5 扩展支持中文分词
- ✅ **并发安全**: WAL 模式支持读写并发
- ✅ **数据迁移**: 易于版本升级和 schema 演进

### 3.2 数据库 Schema 设计

```sql
-- 核心反思记录表
CREATE TABLE reflections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,                  -- YYYY-MM-DD
    slot TEXT NOT NULL,                  -- morning/noon/night
    content TEXT NOT NULL,               -- 反思内容
    word_count INTEGER NOT NULL,         -- 字数
    created_at INTEGER NOT NULL,         -- Unix timestamp
    sentiment_score REAL,                -- 情感得分 (-1.0 ~ 1.0)
    UNIQUE(date, slot)                   -- 每天每个时段只能有一条
);

-- 索引优化查询
CREATE INDEX idx_reflections_date ON reflections(date DESC);
CREATE INDEX idx_reflections_created_at ON reflections(created_at DESC);

-- 全文搜索 (支持中文)
CREATE VIRTUAL TABLE reflections_fts USING fts5(
    content,
    tokenize='unicode61 remove_diacritics 2'
);

-- 统计数据表 (缓存聚合结果)
CREATE TABLE statistics (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- 单例模式
    total_days INTEGER NOT NULL,
    completed_days INTEGER NOT NULL,
    total_words INTEGER NOT NULL,
    current_streak INTEGER NOT NULL,
    best_streak INTEGER NOT NULL,
    last_updated INTEGER NOT NULL
);

-- 成就记录表
CREATE TABLE achievements (
    id TEXT PRIMARY KEY,                 -- 成就 ID
    unlocked_at INTEGER NOT NULL,        -- 解锁时间
    seen BOOLEAN DEFAULT 0               -- 是否已查看
);

-- 时光胶囊表
CREATE TABLE time_capsules (
    id TEXT PRIMARY KEY,
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    unlock_date TEXT NOT NULL,           -- YYYY-MM-DD
    opened_at INTEGER,                   -- 开启时间 (NULL = 未开启)
    CHECK (opened_at IS NULL OR opened_at >= created_at)
);

-- 应用配置表
CREATE TABLE app_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 初始化配置
INSERT INTO app_config (key, value) VALUES
    ('first_used', date('now')),
    ('schema_version', '1');
```

### 3.3 数据访问层 (Repository Pattern)

```rust
// src/infrastructure/repository/mod.rs

/// 数据访问抽象（方便未来切换数据库）
#[async_trait]
pub trait ReflectionRepository: Send + Sync {
    async fn save(&self, reflection: Reflection) -> Result<()>;
    async fn find_by_date_and_slot(&self, date: &str, slot: SlotKind) -> Result<Option<Reflection>>;
    async fn find_by_date_range(&self, start: &str, end: &str) -> Result<Vec<Reflection>>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Reflection>>;
    async fn delete(&self, id: i64) -> Result<()>;
}

/// SQLite 实现
pub struct SqliteReflectionRepository {
    pool: SqlitePool,
    cache: Arc<RwLock<LruCache<String, Reflection>>>,
}

impl SqliteReflectionRepository {
    pub async fn new(db_path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(db_path)
            .await?;

        // 运行迁移
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self {
            pool,
            cache: Arc::new(RwLock::new(LruCache::new(100))),
        })
    }
}

#[async_trait]
impl ReflectionRepository for SqliteReflectionRepository {
    async fn save(&self, reflection: Reflection) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query!(
            r#"
            INSERT INTO reflections (date, slot, content, word_count, created_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(date, slot) DO NOTHING
            "#,
            reflection.date,
            reflection.slot.as_str(),
            reflection.content,
            reflection.word_count,
            reflection.created_at
        )
        .execute(&mut *tx)
        .await?;

        // 更新全文搜索索引
        sqlx::query!(
            "INSERT INTO reflections_fts (rowid, content) VALUES (last_insert_rowid(), ?)",
            reflection.content
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        // 更新缓存
        let cache_key = format!("{}:{}", reflection.date, reflection.slot.as_str());
        self.cache.write().await.put(cache_key, reflection);

        Ok(())
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<Reflection>> {
        let results = sqlx::query_as!(
            Reflection,
            r#"
            SELECT r.* FROM reflections r
            JOIN reflections_fts fts ON r.rowid = fts.rowid
            WHERE reflections_fts MATCH ?
            ORDER BY rank
            LIMIT ?
            "#,
            query,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}
```

### 3.4 数据迁移方案

```rust
// src/infrastructure/migration.rs

/// 从 JSON 迁移到 SQLite
pub async fn migrate_from_json(
    json_path: &str,
    db_repo: &impl ReflectionRepository
) -> Result<MigrationReport> {
    let json_data = fs::read_to_string(json_path)?;
    let persisted: Persisted = serde_json::from_str(&json_data)?;

    let mut report = MigrationReport::default();

    // 迁移当天的反思
    for (slot, maybe_entry) in [
        (SlotKind::Morning, &persisted.morning),
        (SlotKind::Noon, &persisted.noon),
        (SlotKind::Night, &persisted.night),
    ] {
        if let Some(entry) = maybe_entry {
            let reflection = Reflection {
                date: persisted.journal_date.clone(),
                slot,
                content: entry.text.clone(),
                word_count: entry.text.chars().count(),
                created_at: Utc::now().timestamp(),
                sentiment_score: None,
            };
            db_repo.save(reflection).await?;
            report.migrated_count += 1;
        }
    }

    // 迁移历史记录（从 ledger）
    for day_status in &persisted.stats.ledger {
        for (slot, content) in [
            (SlotKind::Morning, &day_status.morning_text),
            (SlotKind::Noon, &day_status.noon_text),
            (SlotKind::Night, &day_status.night_text),
        ] {
            if let Some(text) = content {
                let reflection = Reflection {
                    date: day_status.date.clone(),
                    slot,
                    content: text.clone(),
                    word_count: text.chars().count(),
                    created_at: parse_date(&day_status.date)?.timestamp(),
                    sentiment_score: None,
                };
                db_repo.save(reflection).await?;
                report.migrated_count += 1;
            }
        }
    }

    // 备份原 JSON 文件
    let backup_path = format!("{}.backup.{}", json_path, Utc::now().timestamp());
    fs::copy(json_path, &backup_path)?;
    report.backup_path = backup_path;

    Ok(report)
}
```

---

## 💻 前端架构现代化

### 4.1 问题诊断

**当前前端代码结构** (static/app.js):
```javascript
// 1625 行单文件，包含：
- 全局变量 window.state
- 30+ 个函数混杂在一起
- 无模块划分
- 无类型检查
- 手动 DOM 操作
- 状态管理混乱
```

### 4.2 TypeScript + 模块化方案

#### 目录结构
```
static/
├── src/
│   ├── main.ts                 # 入口文件
│   ├── api/
│   │   ├── client.ts           # API 客户端
│   │   └── types.ts            # API 类型定义
│   ├── state/
│   │   ├── store.ts            # 状态管理
│   │   └── types.ts            # 状态类型
│   ├── components/
│   │   ├── ReflectionForm.ts   # 反思提交表单
│   │   ├── StatsPanel.ts       # 统计面板
│   │   ├── AchievementList.ts  # 成就列表
│   │   └── HistoryView.ts      # 历史浏览
│   ├── utils/
│   │   ├── date.ts             # 日期工具
│   │   ├── vibration.ts        # 振动反馈
│   │   └── toast.ts            # Toast 提示
│   └── styles/
│       ├── main.css            # 全局样式
│       └── themes.css          # 主题系统
├── index.html                  # HTML 模板
├── package.json
├── tsconfig.json
└── vite.config.ts              # 构建配置
```

#### 类型安全的 API 客户端
```typescript
// src/api/types.ts
export interface StateView {
    journal_date: string;
    morning_status: SlotStatus;
    noon_status: SlotStatus;
    night_status: SlotStatus;
    stats: Statistics;
    achievements: Achievement[];
}

export enum SlotStatus {
    Locked = "Locked",
    Open = "Open",
    Submitted = "Submitted",
    Missed = "Missed"
}

export interface SlotStatusDetail {
    variant: SlotStatus;
    data?: {
        opens_at?: string;
        closes_at?: string;
        text?: string;
        submitted_at?: string;
    };
}

// src/api/client.ts
export class ApiClient {
    private baseUrl: string;

    constructor(baseUrl: string = '/api') {
        this.baseUrl = baseUrl;
    }

    async getState(): Promise<StateView> {
        const response = await fetch(`${this.baseUrl}/state`);
        if (!response.ok) {
            throw new ApiError(response.status, await response.text());
        }
        return response.json();
    }

    async submit(text: string): Promise<void> {
        const response = await fetch(`${this.baseUrl}/submit`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ text })
        });

        if (!response.ok) {
            const error = await response.json();
            throw new ApiError(response.status, error.error);
        }
    }

    async getHistory(options?: {
        limit?: number;
        offset?: number;
        includeEmpty?: boolean;
    }): Promise<DayStatus[]> {
        const params = new URLSearchParams();
        if (options?.limit) params.set('limit', String(options.limit));
        if (options?.offset) params.set('offset', String(options.offset));
        if (options?.includeEmpty) params.set('include_empty', 'true');

        const response = await fetch(`${this.baseUrl}/history?${params}`);
        return response.json();
    }

    async search(query: string, limit = 20): Promise<Reflection[]> {
        const response = await fetch(`${this.baseUrl}/search?q=${encodeURIComponent(query)}&limit=${limit}`);
        return response.json();
    }
}
```

#### 状态管理
```typescript
// src/state/store.ts
import { reactive, computed, watch } from 'vue'; // 或使用其他状态库

interface AppState {
    currentState: StateView | null;
    loading: boolean;
    error: string | null;
    draftText: string;
}

export const store = reactive<AppState>({
    currentState: null,
    loading: false,
    error: null,
    draftText: localStorage.getItem('draft') || ''
});

// 计算属性
export const currentSlot = computed(() => {
    if (!store.currentState) return null;

    const now = new Date();
    const hour = now.getHours();

    if (hour >= 4 && hour < 10) return 'morning';
    if (hour >= 10 && hour < 18) return 'noon';
    return 'night';
});

export const canSubmit = computed(() => {
    const slot = currentSlot.value;
    if (!slot || !store.currentState) return false;

    const status = store.currentState[`${slot}_status`];
    return status.variant === SlotStatus.Open && store.draftText.length >= 15;
});

// 自动保存草稿
watch(() => store.draftText, (newText) => {
    localStorage.setItem('draft', newText);
});

// Actions
export const actions = {
    async loadState() {
        store.loading = true;
        store.error = null;
        try {
            store.currentState = await api.getState();
        } catch (e) {
            store.error = e.message;
            throw e;
        } finally {
            store.loading = false;
        }
    },

    async submitReflection() {
        if (!canSubmit.value) return;

        try {
            await api.submit(store.draftText);
            store.draftText = '';
            await actions.loadState(); // 重新加载状态
            toast.success('提交成功！');
            vibrate([10, 20, 10, 20]);
        } catch (e) {
            toast.error(e.message);
            vibrate([10, 50, 10, 50, 10]);
            throw e;
        }
    }
};
```

#### 组件化（以 Alpine.js 为例）
```html
<!-- static/index.html -->
<div x-data="reflectionForm()">
    <!-- 当前时段 -->
    <div class="slot-indicator" :class="`slot-${currentSlot}`">
        <h2 x-text="slotName"></h2>
        <p x-text="slotQuote"></p>
    </div>

    <!-- 提交表单 -->
    <template x-if="isOpen">
        <form @submit.prevent="submit">
            <textarea
                x-model="text"
                :placeholder="placeholder"
                @input="updateWordCount"
                @focus="vibrate(5)"
            ></textarea>

            <div class="word-count" :class="wordCountClass">
                <span x-text="wordCountText"></span>
            </div>

            <button
                type="submit"
                :disabled="!canSubmit"
                @click="vibrate(15)"
            >
                提交反思
            </button>
        </form>
    </template>

    <!-- 已提交 -->
    <template x-if="isSubmitted">
        <div class="submitted-view">
            <div class="seal-stamp">✓</div>
            <p x-text="submittedText"></p>
            <small x-text="submittedTime"></small>
        </div>
    </template>
</div>

<script>
function reflectionForm() {
    return {
        text: Alpine.store('draft'),

        get currentSlot() {
            return Alpine.store('currentSlot');
        },

        get slotName() {
            return { morning: '晨省', noon: '午思', night: '夜省' }[this.currentSlot];
        },

        get isOpen() {
            return Alpine.store('state').morning_status.variant === 'Open';
        },

        get canSubmit() {
            return this.text.length >= 15;
        },

        updateWordCount() {
            Alpine.store('wordCount', this.text.length);
        },

        async submit() {
            await Alpine.store('actions').submitReflection();
        }
    }
}
</script>
```

### 4.3 构建配置 (Vite)

```typescript
// vite.config.ts
import { defineConfig } from 'vite';

export default defineConfig({
    root: 'static',
    build: {
        outDir: '../dist',
        emptyOutDir: true,
        rollupOptions: {
            output: {
                manualChunks: {
                    vendor: ['alpine.js'], // 或其他依赖
                }
            }
        }
    },
    server: {
        proxy: {
            '/api': 'http://localhost:8080'
        }
    }
});
```

---

## 🔒 安全性增强设计

### 5.1 数据加密存储

#### 加密方案: AES-256-GCM
```rust
// src/infrastructure/crypto.rs

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use rand::{RngCore, thread_rng};

pub struct CryptoService {
    cipher: Aes256Gcm,
}

impl CryptoService {
    /// 从用户密码派生密钥
    pub fn from_password(password: &str, salt: &[u8]) -> Result<Self> {
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), salt)?;

        let key_bytes = &password_hash.hash.unwrap().as_bytes()[..32];
        let cipher = Aes256Gcm::new(key_bytes.into());

        Ok(Self { cipher })
    }

    /// 加密文本
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // 格式: [nonce(12) || ciphertext || tag(16)]
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// 解密文本
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String> {
        if encrypted.len() < 12 {
            bail!("Invalid encrypted data");
        }

        let nonce = Nonce::from_slice(&encrypted[..12]);
        let ciphertext = &encrypted[12..];

        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(Into::into)
    }
}

// 使用示例
pub struct EncryptedReflectionRepository {
    inner: SqliteReflectionRepository,
    crypto: CryptoService,
}

impl EncryptedReflectionRepository {
    pub async fn save(&self, reflection: Reflection) -> Result<()> {
        let encrypted_content = self.crypto.encrypt(&reflection.content)?;
        let encrypted_reflection = Reflection {
            content: base64::encode(encrypted_content),
            ..reflection
        };
        self.inner.save(encrypted_reflection).await
    }
}
```

#### 密钥管理
```rust
// 方案一: 用户密码保护（首次启动设置）
// 1. 用户设置密码
// 2. 生成随机 salt 存储在本地
// 3. 使用 Argon2 派生加密密钥
// 4. 每次启动时要求输入密码解锁

// 方案二: Android Keystore (移动端)
#[cfg(target_os = "android")]
pub fn get_encryption_key() -> Result<Vec<u8>> {
    // 使用 Android Keystore API
    // JNI 调用 Java 层的 KeyStore
    android_keystore::get_or_create_key("daily_reckoning_master_key")
}

// 方案三: 生物识别 + Keychain (iOS)
#[cfg(target_os = "ios")]
pub fn get_encryption_key() -> Result<Vec<u8>> {
    // 使用 iOS Keychain Services
    ios_keychain::get_or_create_key("daily_reckoning_master_key")
}
```

### 5.2 敏感配置保护

```rust
// src/config.rs

use secrecy::{Secret, ExposeSecret};

#[derive(Debug)]
pub struct AppConfig {
    pub database_path: String,
    pub smtp: Option<SmtpConfig>,
}

#[derive(Debug)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,  // 使用 secrecy crate 防止内存泄露
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        // 从环境变量或加密配置文件读取
        let smtp = if let Ok(host) = env::var("SMTP_HOST") {
            Some(SmtpConfig {
                host,
                port: env::var("SMTP_PORT")?.parse()?,
                username: env::var("SMTP_USERNAME")?,
                password: Secret::new(env::var("SMTP_PASSWORD")?),
            })
        } else {
            None
        };

        Ok(Self {
            database_path: env::var("DB_PATH").unwrap_or_else(|_| "data/app.db".to_string()),
            smtp,
        })
    }
}

// 使用时
async fn send_email(config: &SmtpConfig, content: &str) -> Result<()> {
    let credentials = Credentials::new(
        config.username.clone(),
        config.password.expose_secret().clone(),  // 只在必要时暴露
    );
    // ... 发送邮件
}
```

---

## ⚡ 性能优化策略

### 6.1 多级缓存架构

```rust
// src/infrastructure/cache.rs

use moka::future::Cache;
use std::time::Duration;

pub struct CacheLayer {
    // L1: 热数据缓存 (最近访问)
    hot_cache: Cache<String, Arc<StateView>>,

    // L2: 统计数据缓存 (较少变化)
    stats_cache: Cache<String, Arc<Statistics>>,

    // L3: 成就状态缓存
    achievement_cache: Cache<String, Vec<Achievement>>,
}

impl CacheLayer {
    pub fn new() -> Self {
        Self {
            hot_cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(60))
                .build(),

            stats_cache: Cache::builder()
                .max_capacity(10)
                .time_to_live(Duration::from_secs(300))  // 5分钟
                .build(),

            achievement_cache: Cache::builder()
                .max_capacity(50)
                .time_to_live(Duration::from_secs(600))  // 10分钟
                .build(),
        }
    }

    pub async fn get_or_compute<F, Fut>(
        &self,
        key: &str,
        compute: F,
    ) -> Result<Arc<StateView>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<StateView>>,
    {
        self.hot_cache
            .try_get_with(key.to_string(), async {
                let value = compute().await?;
                Ok(Arc::new(value))
            })
            .await
            .map_err(|e| anyhow!("Cache error: {}", e))
    }

    /// 提交反思后失效相关缓存
    pub fn invalidate_on_submission(&self, date: &str) {
        self.hot_cache.invalidate(date);
        self.stats_cache.invalidate_all();
        self.achievement_cache.invalidate_all();
    }
}
```

### 6.2 增量统计计算

```rust
// src/domain/services/statistics.rs

/// 增量更新统计数据（不需要每次全量计算）
pub struct IncrementalStatistics {
    repo: Arc<dyn StatisticsRepository>,
}

impl IncrementalStatistics {
    /// 提交反思时增量更新
    pub async fn on_reflection_submitted(
        &self,
        reflection: &Reflection,
    ) -> Result<()> {
        let mut stats = self.repo.get_current().await?;

        // 只更新受影响的字段
        stats.total_reflections += 1;
        stats.total_words += reflection.word_count;

        // 检查是否完成了新的一天
        let day_completed = self.repo
            .check_day_completed(&reflection.date)
            .await?;

        if day_completed {
            stats.completed_days += 1;
            stats.current_streak = self.calculate_new_streak(&reflection.date).await?;
        }

        self.repo.update(stats).await?;
        Ok(())
    }

    /// 只计算从上次更新以来的 streak（不是全量）
    async fn calculate_new_streak(&self, current_date: &str) -> Result<i64> {
        let yesterday = subtract_days(current_date, 1)?;
        let yesterday_completed = self.repo.is_day_completed(&yesterday).await?;

        if yesterday_completed {
            let prev_streak = self.repo.get_current().await?.current_streak;
            Ok(prev_streak + 1)
        } else {
            Ok(1) // 新的 streak 开始
        }
    }
}
```

### 6.3 异步处理

```rust
// src/api/handlers.rs

use tokio::task;

/// 提交反思接口（异步处理统计更新）
pub async fn submit_reflection(
    State(app): State<AppState>,
    Json(req): Json<SubmitRequest>,
) -> Result<StatusCode, ApiError> {
    // 1. 同步：保存反思（必须立即完成）
    let reflection = app.submit_use_case.execute(req.text).await?;

    // 2. 异步：更新统计（可以后台处理）
    let stats_service = app.stats_service.clone();
    task::spawn(async move {
        if let Err(e) = stats_service.on_reflection_submitted(&reflection).await {
            error!("Failed to update statistics: {}", e);
        }
    });

    // 3. 异步：检查成就解锁
    let achievement_service = app.achievement_service.clone();
    task::spawn(async move {
        if let Err(e) = achievement_service.check_unlocks().await {
            error!("Failed to check achievements: {}", e);
        }
    });

    // 4. 立即返回成功（不等待异步任务完成）
    Ok(StatusCode::CREATED)
}
```

### 6.4 数据库性能优化

```sql
-- 启用 WAL 模式（写不阻塞读）
PRAGMA journal_mode = WAL;

-- 增大缓存
PRAGMA cache_size = -64000;  -- 64MB

-- 优化查询计划
PRAGMA optimize;

-- 定期 VACUUM（清理碎片）
-- 在后台任务中每周执行一次
```

---

## 📊 可观测性设计

### 7.1 结构化日志系统

```rust
// src/infrastructure/logging.rs

use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()  // JSON 格式便于解析
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true)
        )
        .with(
            // 根据环境变量设置日志级别
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into())
        )
        .init();

    Ok(())
}

// 使用示例
#[instrument(skip(repo), fields(user_id = %user_id))]
pub async fn submit_reflection(
    repo: &impl ReflectionRepository,
    user_id: &str,
    text: String,
) -> Result<()> {
    info!(
        word_count = text.len(),
        "Starting reflection submission"
    );

    match repo.save(Reflection::new(text)).await {
        Ok(_) => {
            info!("Reflection saved successfully");
            Ok(())
        }
        Err(e) => {
            error!(error = %e, "Failed to save reflection");
            Err(e)
        }
    }
}
```

### 7.2 性能指标采集

```rust
// src/infrastructure/metrics.rs

use prometheus::{Registry, Counter, Histogram, HistogramOpts};

pub struct Metrics {
    // 计数器
    pub reflections_submitted: Counter,
    pub api_requests: Counter,
    pub errors: Counter,

    // 直方图（延迟分布）
    pub submission_duration: Histogram,
    pub query_duration: Histogram,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Result<Self> {
        let reflections_submitted = Counter::new(
            "reflections_submitted_total",
            "Total number of reflections submitted"
        )?;

        let submission_duration = Histogram::with_opts(
            HistogramOpts::new(
                "submission_duration_seconds",
                "Time to submit a reflection"
            ).buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        )?;

        registry.register(Box::new(reflections_submitted.clone()))?;
        registry.register(Box::new(submission_duration.clone()))?;

        Ok(Self {
            reflections_submitted,
            submission_duration,
            // ... 其他指标
        })
    }
}

// 使用示例
pub async fn submit_handler(/* ... */) -> Result<Response> {
    let timer = metrics.submission_duration.start_timer();

    let result = submit_reflection(/* ... */).await;

    timer.observe_duration();

    if result.is_ok() {
        metrics.reflections_submitted.inc();
    } else {
        metrics.errors.inc();
    }

    result
}
```

### 7.3 错误追踪

```rust
// src/domain/error.rs

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Slot {slot:?} is not open (status: {status})")]
    SlotNotOpen {
        slot: SlotKind,
        status: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Content validation failed: {reason}")]
    InvalidContent {
        reason: String,
        content_length: usize,
    },

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Encryption error")]
    Encryption(#[from] aes_gcm::Error),
}

// 统一错误处理中间件
pub async fn error_handler(
    err: BoxError,
) -> (StatusCode, Json<ErrorResponse>) {
    let (status, message, error_code) = match err.downcast::<DomainError>() {
        Ok(DomainError::SlotNotOpen { slot, status, .. }) => {
            (
                StatusCode::FORBIDDEN,
                format!("时段 {:?} 未开放（状态：{}）", slot, status),
                "SLOT_NOT_OPEN"
            )
        }
        Ok(DomainError::InvalidContent { reason, .. }) => {
            (
                StatusCode::BAD_REQUEST,
                format!("内容验证失败：{}", reason),
                "INVALID_CONTENT"
            )
        }
        _ => {
            error!("Unhandled error: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "服务器内部错误".to_string(),
                "INTERNAL_ERROR"
            )
        }
    };

    (status, Json(ErrorResponse {
        error: message,
        code: error_code.to_string(),
    }))
}
```

---

## 🚀 实施路线图

### Phase 1: 基础设施重构 (2-3周)

**目标**: 建立新的技术基础

#### Week 1-2: 数据层迁移
- [ ] 设计 SQLite schema
- [ ] 实现 Repository trait 和 SQLite 实现
- [ ] 编写数据迁移工具（JSON → SQLite）
- [ ] 完成单元测试（100% 覆盖）
- [ ] 性能基准测试（对比 JSON 方案）

#### Week 2-3: 后端架构重构
- [ ] 完成模块化拆分（domain/services/infrastructure）
- [ ] 实现依赖注入容器
- [ ] 重构 API 层（新的错误处理、中间件）
- [ ] 添加结构化日志（tracing）
- [ ] 集成测试覆盖核心流程

**里程碑**:
- ✅ 新后端可以完全替代旧版本
- ✅ 所有测试通过
- ✅ 性能至少提升 2倍

---

### Phase 2: 前端现代化 (2-3周)

#### Week 4-5: TypeScript 迁移
- [ ] 配置构建工具（Vite）
- [ ] 定义类型系统（API types, State types）
- [ ] 实现 API 客户端
- [ ] 模块化拆分（10+ 个模块）
- [ ] 引入状态管理（Alpine.js stores）

#### Week 5-6: UI 组件重构
- [ ] 组件化改造（表单、统计、历史等）
- [ ] 响应式优化
- [ ] 动画效果增强（CSS transitions）
- [ ] 无障碍访问（ARIA 标签）
- [ ] 端到端测试（Playwright）

**里程碑**:
- ✅ 前端代码量减少 30%
- ✅ 类型覆盖率 > 90%
- ✅ Bundle 大小 < 100KB (gzipped)

---

### Phase 3: 安全性增强 (1-2周)

#### Week 7-8: 加密系统
- [ ] 实现 AES-256-GCM 加密服务
- [ ] 集成 Android Keystore（移动端）
- [ ] 用户密码设置流程
- [ ] 加密迁移工具（明文 → 密文）
- [ ] 安全审计

**里程碑**:
- ✅ 数据加密存储
- ✅ 通过安全扫描（无高危漏洞）

---

### Phase 4: 性能优化 (1-2周)

#### Week 9: 缓存系统
- [ ] 实现多级缓存
- [ ] 增量统计计算
- [ ] 数据库查询优化（索引、预编译）
- [ ] 异步任务处理

#### Week 10: 压力测试
- [ ] 编写压力测试脚本
- [ ] 性能基准测试
- [ ] 优化瓶颈
- [ ] 监控指标接入

**里程碑**:
- ✅ API 响应时间 < 100ms (p99)
- ✅ 支持 10 万+ 记录无性能衰减
- ✅ 并发 100 请求/秒无压力

---

### Phase 5: 可观测性 (1周)

#### Week 11: 监控系统
- [ ] Prometheus 指标暴露
- [ ] 日志聚合（结构化日志）
- [ ] 错误追踪（Sentry 集成，可选）
- [ ] 性能监控面板

**里程碑**:
- ✅ 完整的可观测性体系
- ✅ 问题定位时间 < 5 分钟

---

### Phase 6: CI/CD & DevOps (1周)

#### Week 12: 自动化流程
- [ ] GitHub Actions 配置
  - [ ] Rust 测试 + Clippy
  - [ ] TypeScript 类型检查 + ESLint
  - [ ] 自动化构建（Web + Android）
  - [ ] 代码覆盖率报告
- [ ] 发布流程自动化
- [ ] 文档自动生成

**里程碑**:
- ✅ 每次 PR 自动运行测试
- ✅ 一键发布新版本

---

## 📈 成功指标 (KPIs)

### 技术指标

| 指标 | 当前值 | 目标值 | 测量方式 |
|------|--------|--------|----------|
| **API 响应时间 (p99)** | ~500ms | < 100ms | 压力测试 |
| **前端 Bundle 大小** | ~200KB | < 100KB | webpack-bundle-analyzer |
| **测试覆盖率** | ~40% | > 80% | cargo tarpaulin |
| **代码重复率** | ~15% | < 5% | jscpd |
| **TypeScript 覆盖率** | 0% | > 90% | tsc --noEmit |
| **支持记录数** | 1000 | 100,000+ | 性能基准测试 |
| **内存占用** | ~50MB | < 30MB | 性能分析 |

### 质量指标

| 指标 | 目标 |
|------|------|
| **零高危安全漏洞** | ✅ 通过 cargo audit |
| **零编译警告** | ✅ Clippy 无警告 |
| **代码规范统一** | ✅ rustfmt + prettier |
| **文档完整性** | ✅ 所有公开 API 有文档 |

---

## ⚠️ 风险评估与缓解

### 高风险项

#### 1. 数据迁移失败导致数据丢失
**风险等级**: 🔴 高
**影响**: 用户数据永久丢失
**缓解措施**:
- 迁移前强制备份
- 迁移过程保留原 JSON 文件
- 提供回滚机制
- 灰度发布（先小范围测试）
- 迁移后数据一致性校验

#### 2. 性能优化导致功能回归
**风险等级**: 🟡 中
**影响**: 核心功能异常
**缓解措施**:
- 完善的自动化测试
- 性能基准测试对比
- 金丝雀部署
- 快速回滚机制

#### 3. 前端重构导致用户体验倒退
**风险等级**: 🟡 中
**影响**: 用户流失
**缓解措施**:
- 保持 UI 一致性
- 渐进式重构（不是一次性替换）
- A/B 测试
- 用户反馈收集

### 中风险项

#### 4. 加密实现不当导致安全漏洞
**风险等级**: 🟡 中
**影响**: 数据泄露
**缓解措施**:
- 使用成熟的加密库（aes-gcm）
- 安全审计
- 密钥管理最佳实践
- 渗透测试

#### 5. 第三方依赖漏洞
**风险等级**: 🟢 低
**影响**: 安全风险
**缓解措施**:
- 定期运行 `cargo audit`
- Dependabot 自动更新
- 依赖最小化原则

---

## 🔄 迁移策略

### 用户数据迁移流程

```
┌────────────────────────────────────────────────────────┐
│ Step 1: 版本检测                                        │
│  - 检测 data/state.json 是否存在                        │
│  - 检测 data/app.db 是否存在                            │
└────────────┬───────────────────────────────────────────┘
             │
             ├─ 新用户 ──────────────────> 直接使用 SQLite
             │
             └─ 老用户 ──> Step 2: 备份
                            │
┌───────────────────────────▼────────────────────────────┐
│ Step 2: 自动备份                                        │
│  - 复制 state.json -> state.json.backup.{timestamp}    │
│  - 显示备份路径给用户                                   │
└────────────┬───────────────────────────────────────────┘
             │
┌────────────▼───────────────────────────────────────────┐
│ Step 3: 数据迁移                                        │
│  - 创建 app.db                                          │
│  - 逐条插入反思记录                                     │
│  - 迁移统计数据                                         │
│  - 迁移成就记录                                         │
│  - 显示进度条                                           │
└────────────┬───────────────────────────────────────────┘
             │
┌────────────▼───────────────────────────────────────────┐
│ Step 4: 数据校验                                        │
│  - 对比记录数量                                         │
│  - 校验统计数据一致性                                   │
│  - 抽样检查内容完整性                                   │
└────────────┬───────────────────────────────────────────┘
             │
             ├─ 校验失败 ──> 回滚 + 报错
             │
             └─ 校验成功 ──> Step 5
                             │
┌────────────────────────────▼───────────────────────────┐
│ Step 5: 完成迁移                                        │
│  - 重命名 state.json -> state.json.old (保留)          │
│  - 写入迁移标记文件                                     │
│  - 显示迁移成功消息                                     │
└────────────────────────────────────────────────────────┘
```

### 版本兼容性策略

```rust
// src/infrastructure/migration/version.rs

#[derive(Debug)]
pub enum AppVersion {
    V1,  // JSON 存储
    V2,  // SQLite 存储
}

pub async fn detect_and_migrate() -> Result<()> {
    let current_version = detect_version().await?;

    match current_version {
        AppVersion::V1 => {
            info!("Detected v1 data, starting migration...");
            migrate_v1_to_v2().await?;
        }
        AppVersion::V2 => {
            info!("Already on v2, no migration needed");
        }
    }

    Ok(())
}

async fn migrate_v1_to_v2() -> Result<()> {
    // 1. 备份
    backup_v1_data().await?;

    // 2. 迁移
    let migrator = V1ToV2Migrator::new().await?;
    migrator.migrate().await?;

    // 3. 验证
    migrator.validate().await?;

    // 4. 清理
    finalize_migration().await?;

    Ok(())
}
```

---

## 📚 附录

### A. 技术选型对比

#### A.1 数据库选型

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| **JSON 文件** | 简单、无依赖 | 性能差、无查询能力 | ❌ 不适合长期使用 |
| **SQLite** | 高性能、SQL 支持、成熟 | 需要迁移 | ✅ **推荐** |
| **sled** | 纯 Rust、高性能 | 生态不成熟、无 SQL | ❌ 风险较高 |
| **PostgreSQL** | 功能最强大 | 部署复杂、过重 | ❌ 过度工程 |

#### A.2 前端框架选型

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| **Vanilla JS** | 无依赖、轻量 | 难维护、无类型 | ❌ 当前方案 |
| **Alpine.js** | 轻量 (15KB)、简单 | 功能有限 | ✅ **推荐** |
| **Svelte** | 性能好、编译时优化 | 生态较小 | ✅ 备选方案 |
| **React** | 生态最好 | 体积大 (40KB+) | ❌ 过重 |
| **Vue** | 平衡性好 | 体积中等 (30KB) | 🟡 备选方案 |

#### A.3 加密库选型

| 方案 | 优势 | 劣势 | 结论 |
|------|------|------|------|
| **aes-gcm** | 成熟、标准、安全 | - | ✅ **推荐** |
| **chacha20poly1305** | 性能略好 | 硬件加速少 | ✅ 备选方案 |
| **自实现** | 完全控制 | 风险极高 | ❌ 绝不推荐 |

### B. 参考资料

- [SQLite Performance Tuning](https://www.sqlite.org/optoverview.html)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [TypeScript Deep Dive](https://basarat.gitbook.io/typescript/)
- [Web Performance Best Practices](https://web.dev/performance/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)

---

## 🎯 下一步行动

### 立即行动项 (本周)

1. **确认架构方向**: 和团队/用户讨论本文档，收集反馈
2. **搭建开发环境**: 配置 SQLite、TypeScript、测试框架
3. **创建开发分支**: `feature/architecture-v2`
4. **编写 POC**: 实现一个最小可行的 SQLite + TypeScript 原型

### 下周计划

1. 开始 Phase 1: 数据层重构
2. 每日站会同步进度
3. 持续更新本文档

---

**文档维护者**: Architecture Team
**最后更新**: 2026-02-03
**下次审核**: 2026-02-17 (每两周审核一次)
