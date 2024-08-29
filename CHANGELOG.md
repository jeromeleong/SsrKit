# CHANGELOG

## 2024-08-15 至 2024-08-29

### 變更摘要
這個時期主要focus在增強島嶼渲染功能，優化配置管理，以及改進整體代碼結構。

### 詳細變更

#### 新增
- 在island模塊中使用Arc來包裝島嶼渲染器，提高性能和資源管理
- 引入SsrInitializer和SsrInitializerChanger結構，簡化初始化過程
- 新增CombinedIslandProcessor以組合多個島嶼處理器

#### 變更
- 將SsrkitConfig版本從0.1.1更新到0.1.2
- 重構config模塊，將SsrkitConfigBuilder重命名為SsrkitConfigChanger
- 在island模塊中增強島嶼渲染器的類型定義
- 重構render.rs以整合新的島嶼處理邏輯
- 改進template.rs中的渲染邏輯，支持島嶼腳本的生成和替換

#### 修復
- 優化了一些代碼以提高性能和可讀性

### 貢獻者
- Jerome Leong

---

## 2024-08-01 至 2024-08-14

### 變更摘要
這個時期主要focus在項目結構的優化、文檔更新和CI/CD流程的改進。

### 詳細變更

#### 新增
- 添加GitHub Actions工作流程進行rust-clippy分析，提高代碼質量
- 新增CHANGELOG.md文件記錄重要變更
- 在Cargo.toml中添加元數據字段，如作者、描述、倉庫等

#### 變更
- 更新lru依賴版本從0.12.3到0.12.4
- 重構島嶼管理器和配置構建器，提高可用性
- 將SsrkitConfigBuilder::build重命名為SsrkitConfigChanger::finish

#### 移除
- 移除.github/workflows/rust-clippy.yml工作流程

#### 修復
- 修正了一些文檔和代碼註釋中的錯誤

### 貢獻者
- Jerome Leong

---

## 2024-07-15 至 2024-07-30

### 變更摘要
這個期間主要聚焦於改進島嶼管理、模板渲染和緩存系統的實現，同時優化了整體性能和代碼結構。

### 詳細變更

#### 新增
- 引入 `SsrkitConfig` 結構體，用於配置緩存大小
- 實現通用的 LRU 緩存，用於島嶼和模板
- 新增 `init_cache` 函數初始化全局配置
- 為島嶼和模板添加緩存機制，提升性能

#### 變更
- 重構 `IslandManager` 以整合新的緩存系統
- 更新 `Template` 結構體，使用新的緩存機制
- 調整渲染邏輯以利用新的緩存系統
- 優化模塊導出，提高代碼組織性

#### 修復
- 修正島嶼佔位符替換邏輯，提高準確性
- 改進錯誤處理和日誌記錄

### 貢獻者
- Jerome

---

## 2024-07-01 至 2024-07-14

### 變更摘要
這個期間主要專注於改進島嶼管理系統、引入模板渲染功能，以及優化整體代碼結構。

### 詳細變更

#### 新增
- 實現 `IslandProcessor` 特性和 `CombinedIslandProcessor` 結構體
- 新增 `Template` 結構體用於 HTML 渲染
- 引入 `nanoid` 生成唯一實例 ID

#### 變更
- 重構 `IslandManager` 以支持多個渲染器
- 更新 `SsrRenderer` 以接受 `Arc<Template>` 進行渲染
- 優化 `lib.rs` 中的模塊導出和類型重導出

#### 修復
- 改進島嶼佔位符替換邏輯
- 優化錯誤處理和日誌記錄

### 貢獻者
- Jerome

---

## 2024-06-15 至 2024-06-30

### 變更摘要
這個期間主要集中在初始項目結構的建立和基本功能的實現。

### 詳細變更

#### 新增
- 創建初始項目結構，包括 `Cargo.toml` 和 `.gitignore`
- 實現 `ParamsProcessor` 特性和 `SsrRenderer` 結構體
- 添加 `CombinedParamsProcessor` 用於組合多個處理器
- 實現 `get_or_init_renderer` 函數用於初始化渲染器

#### 變更
- 重組代碼結構，將主要功能分離到獨立模塊中

#### 修復
- 改進錯誤處理和類型安全

### 貢獻者
- Jerome