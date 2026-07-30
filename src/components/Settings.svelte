<script lang="ts">
  import { invoke } from "../utils/tauri-adapter";

  export let initialPath: string;
  export let onSave: (path: string) => void;
  export let isDark: boolean;
  export let onToggleTheme: () => void;
  export let colorScheme: string;
  export let onChangeColorScheme: (scheme: string) => void;
  export let bgStyle: string;
  export let onChangeBgStyle: (v: string, url?: string) => void;
  export let glassMode: string;
  export let onChangeGlassMode: (v: string) => void;
  export let shadowStyle: string;
  export let onChangeShadowStyle: (v: string) => void;
  export let motionLevel: string;
  export let onChangeMotionLevel: (v: string) => void;
  export let customBgUrl: string = "";
  export let bgOpacity: number = 0.5;
  export let onChangeBgOpacity: (v: number) => void;
  export let radiusScale: number = 1;
  export let onChangeRadiusScale: (v: number) => void;
  export let cardOpacity: number = 1;
  export let onChangeCardOpacity: (v: number) => void;
  export let titlebarOpacity: number = 0.8;
  export let onChangeTitlebarOpacity: (v: number) => void;

  let libraryPath = initialPath;
  let isSaving = false;

  $: bgUrlValue = customBgUrl;

  let showModal = false;
  let modalTitle = "";
  let modalMessage = "";

  const schemes = [
    { id: "sakura", name: "樱粉", light: "#ff5595", dark: "#ff6ba0" },
    { id: "indigo", name: "靛蓝", light: "#6366f1", dark: "#818cf8" },
    { id: "forest", name: "森绿", light: "#10b981", dark: "#34d399" },
    { id: "amber", name: "琥珀", light: "#f59e0b", dark: "#fbbf24" },
    { id: "violet", name: "紫罗兰", light: "#8b5cf6", dark: "#a78bfa" },
    { id: "teal", name: "青碧", light: "#14b8a6", dark: "#2dd4bf" },
    { id: "rose", name: "玫瑰", light: "#f43f5e", dark: "#fda4af" },
    { id: "ocean", name: "深海", light: "#0c87e8", dark: "#36a8f2" },
    { id: "sunset", name: "落日", light: "#f97316", dark: "#fdba74" },
    { id: "ink", name: "墨韵", light: "#64748b", dark: "#94a3b8" },
    { id: "primrose", name: "樱草", light: "#eab308", dark: "#fde047" },
    { id: "coral", name: "珊瑚", light: "#ff5a3c", dark: "#ffa88c" },
  ];

  const bgOptions = [
    { id: "pure", name: "纯净", desc: "纯色背景" },
    { id: "gradient", name: "渐变", desc: "光晕渐变" },
    { id: "custom", name: "自定义", desc: "图片背景" },
  ];

  const glassOptions = [
    { id: "solid", name: "实色", desc: "无模糊" },
    { id: "translucent", name: "半透", desc: "轻微毛玻璃" },
    { id: "glass", name: "玻璃态", desc: "强毛玻璃" },
  ];

  const shadowOptions = [
    { id: "flat", name: "平面", desc: "无阴影" },
    { id: "standard", name: "标准", desc: "柔和阴影" },
    { id: "float", name: "浮浮", desc: "强阴影悬浮" },
  ];

  const motionOptions = [
    { id: "minimal", name: "极简", desc: "0.1s 无位移" },
    { id: "standard", name: "标准", desc: "0.3s 轻位移" },
    { id: "rich", name: "丰富", desc: "0.5s 弹性动画" },
  ];

  function showMessage(title: string, message: string) {
    modalTitle = title;
    modalMessage = message;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
  }

  async function browseDirectory() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, multiple: false, defaultPath: libraryPath });
      if (selected && typeof selected === "string") libraryPath = selected;
    } catch (_) {}
  }

  async function pickBgImage() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const { readFile } = await import("@tauri-apps/plugin-fs");
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp"] }],
      });
      if (selected && typeof selected === "string") {
        const uint8 = await readFile(selected);
        const dataUrl = bytesToDataUrl(uint8);
        const compressed = await compressBackgroundImage(dataUrl);
        onChangeBgStyle("custom", compressed);
      }
    } catch (_) {
      const input = document.createElement("input");
      input.type = "file";
      input.accept = "image/*";
      input.onchange = () => {
        const file = input.files?.[0];
        if (file) {
          const reader = new FileReader();
          reader.onload = async () => {
            const compressed = await compressBackgroundImage(reader.result as string);
            onChangeBgStyle("custom", compressed);
          };
          reader.readAsDataURL(file);
        }
      };
      input.click();
    }
  }

  async function compressBackgroundImage(dataUrl: string): Promise<string> {
    return new Promise((resolve) => {
      const img = new Image();
      img.onload = () => {
        const maxW = 1920;
        const scale = Math.min(1, maxW / img.width);
        const w = Math.round(img.width * scale);
        const h = Math.round(img.height * scale);
        const canvas = document.createElement("canvas");
        canvas.width = w;
        canvas.height = h;
        const ctx = canvas.getContext("2d");
        if (!ctx) { resolve(dataUrl); return; }
        ctx.drawImage(img, 0, 0, w, h);
        resolve(canvas.toDataURL("image/jpeg", 0.85));
      };
      img.onerror = () => resolve(dataUrl);
      img.src = dataUrl;
    });
  }

  function bytesToDataUrl(uint8: Uint8Array): string {
    let mimeType = "image/jpeg";
    if (uint8.length >= 4) {
      const header = uint8.subarray(0, 4);
      if (header[0] === 0x89 && header[1] === 0x50 && header[2] === 0x4E && header[3] === 0x47) {
        mimeType = "image/png";
      } else if (header[0] === 0x47 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x38) {
        mimeType = "image/gif";
      } else if (header[0] === 0xFF && header[1] === 0xD8) {
        mimeType = "image/jpeg";
      } else if (uint8.length >= 12 && header[0] === 0x52 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x46) {
        const webpHeader = uint8.subarray(8, 12);
        if (webpHeader[0] === 0x57 && webpHeader[1] === 0x45 && webpHeader[2] === 0x42 && webpHeader[3] === 0x57) {
          mimeType = "image/webp";
        }
      }
    }
    let binary = "";
    const chunkSize = 8192;
    for (let i = 0; i < uint8.length; i += chunkSize) {
      const chunk = uint8.subarray(i, i + chunkSize);
      binary += String.fromCharCode.apply(null, chunk as unknown as number[]);
    }
    const base64 = btoa(binary);
    return `data:${mimeType};base64,${base64}`;
  }

  async function saveSettings() {
    isSaving = true;
    try { await invoke("set_library_path", { path: libraryPath }); onSave(libraryPath); }
    catch (e) { showMessage("保存失败", `${e}`); }
    finally { isSaving = false; }
  }
</script>

<div class="settings-wrap">
  <div class="settings-card">
    <div class="setting-row">
      <label>动漫库目录</label>
      <div class="path-row">
        <input type="text" bind:value={libraryPath} placeholder="选择目录..." />
        <button class="ghost" on:click={browseDirectory}>浏览</button>
      </div>
      <p class="hint">程序将扫描此目录下的所有动漫文件夹</p>
    </div>

    <div class="setting-row">
      <label>外观主题</label>
      <div class="theme-toggle-row">
        <button
          class="theme-option"
          class:active={!isDark}
          on:click={() => isDark && onToggleTheme()}
        >
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/></svg>
          <span>亮色</span>
        </button>
        <button
          class="theme-option"
          class:active={isDark}
          on:click={() => !isDark && onToggleTheme()}
        >
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
          <span>暗色</span>
        </button>
      </div>
      <p class="hint">切换亮色 / 暗色模式，设置自动保存</p>
    </div>

    <div class="setting-row">
      <label>配色方案</label>
      <div class="scheme-grid">
        {#each schemes as s}
          <button
            class="scheme-card"
            class:active={colorScheme === s.id}
            on:click={() => onChangeColorScheme(s.id)}
          >
            <div class="scheme-preview" style="background: linear-gradient(135deg, {s.light}, {s.dark})">
              <div class="scheme-dot" style="background: {s.dark}"></div>
            </div>
            <span class="scheme-name">{s.name}</span>
          </button>
        {/each}
      </div>
      <p class="hint">选择主题配色，设置自动保存</p>
    </div>

    <div class="setting-row">
      <label>圆角风格</label>
      <div class="slider-row">
        <span class="slider-label">圆角大小</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={radiusScale}
          on:input={(e) => onChangeRadiusScale(parseFloat(e.currentTarget.value))}
        />
        <span class="slider-value">{Math.round(radiusScale * 100)}%</span>
      </div>
      <p class="hint">拖动滑块调整圆角大小，0% 为直角，100% 为大圆角</p>
    </div>

    <div class="setting-row">
      <label>背景图片</label>
      <div class="option-row">
        {#each bgOptions as o}
          <button
            class="option-card"
            class:active={bgStyle === o.id}
            on:click={() => onChangeBgStyle(o.id, bgUrlValue)}
          >
            <div class="option-icon bg-{o.id}"></div>
            <span class="option-name">{o.name}</span>
            <span class="option-desc">{o.desc}</span>
          </button>
        {/each}
      </div>
      {#if bgStyle === "custom"}
        <div class="bg-picker-row">
          <button class="bg-picker-btn" on:click={pickBgImage}>
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>
            选择图片
          </button>
          {#if bgUrlValue}
            <div class="bg-preview" style="background-image: url('{bgUrlValue}')"></div>
            <button class="ghost bg-clear-btn" on:click={() => onChangeBgStyle("pure")}>清除</button>
          {/if}
        </div>
        <p class="hint">点击按钮选择本地图片作为背景</p>
        {#if bgUrlValue}
          <div class="slider-row">
            <span class="slider-label">背景透明度</span>
            <input
              type="range"
              min="0.1"
              max="1"
              step="0.05"
              value={bgOpacity}
              on:input={(e) => onChangeBgOpacity(parseFloat(e.currentTarget.value))}
            />
            <span class="slider-value">{Math.round(bgOpacity * 100)}%</span>
          </div>
        {/if}
      {/if}
    </div>

    <div class="setting-row">
      <label>透明度 / 毛玻璃</label>
      <div class="option-row">
        {#each glassOptions as o}
          <button
            class="option-card"
            class:active={glassMode === o.id}
            on:click={() => onChangeGlassMode(o.id)}
          >
            <div class="option-icon glass-{o.id}"></div>
            <span class="option-name">{o.name}</span>
            <span class="option-desc">{o.desc}</span>
          </button>
        {/each}
      </div>
      <div class="slider-row">
        <span class="slider-label">卡片透明度</span>
        <input
          type="range"
          min="0.2"
          max="1"
          step="0.05"
          value={cardOpacity}
          on:input={(e) => onChangeCardOpacity(parseFloat(e.currentTarget.value))}
        />
        <span class="slider-value">{Math.round(cardOpacity * 100)}%</span>
      </div>
      <p class="hint">拖动滑块调整卡片不透明度，范围 20% ~ 100%</p>
      <div class="slider-row">
        <span class="slider-label">标题栏透明度</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.05"
          value={titlebarOpacity}
          on:input={(e) => onChangeTitlebarOpacity(parseFloat(e.currentTarget.value))}
        />
        <span class="slider-value">{Math.round(titlebarOpacity * 100)}%</span>
      </div>
      <p class="hint">拖动滑块调整标题栏透明度，范围 0% ~ 100%</p>
    </div>

    <div class="setting-row">
      <label>卡片阴影</label>
      <div class="option-row">
        {#each shadowOptions as o}
          <button
            class="option-card"
            class:active={shadowStyle === o.id}
            on:click={() => onChangeShadowStyle(o.id)}
          >
            <div class="option-icon shadow-{o.id}"></div>
            <span class="option-name">{o.name}</span>
            <span class="option-desc">{o.desc}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="setting-row">
      <label>动画强度</label>
      <div class="option-row">
        {#each motionOptions as o}
          <button
            class="option-card"
            class:active={motionLevel === o.id}
            on:click={() => onChangeMotionLevel(o.id)}
          >
            <div class="option-icon motion-{o.id}"></div>
            <span class="option-name">{o.name}</span>
            <span class="option-desc">{o.desc}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="info-section">
      <h3>支持格式</h3>
      <div class="fmt-list">
        {#each ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "rmvb", "m4v"] as f}
          <span class="fmt">{f}</span>
        {/each}
      </div>
    </div>

    <div class="info-section">
      <h3>命名规则</h3>
      <div class="rule-list">
        <span class="rule">[中文名] 第X季 [字幕组]</span>
        <span class="rule">[中文名] [剧场版/OVA] [字幕组]</span>
      </div>
    </div>

    <div class="save-row">
      <button on:click={saveSettings} disabled={isSaving}>
        {isSaving ? "保存中..." : "保存设置"}
      </button>
    </div>
  </div>
</div>

{#if showModal}
  <div class="modal-overlay" on:click={closeModal}>
    <div class="modal-content" on:click|stopPropagation>
      <div class="modal-header">
        <h3 class="modal-title">{modalTitle}</h3>
        <button class="modal-close" on:click={closeModal}>
          <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
      <div class="modal-body">
        <p>{modalMessage}</p>
      </div>
      <div class="modal-footer">
        <button class="modal-confirm" on:click={closeModal}>确定</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-wrap { max-width: 600px; margin: 0 auto; }

  .settings-card {
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-8);
    box-shadow: var(--card-shadow);
    display: flex;
    flex-direction: column;
    gap: var(--space-8);
  }

  .setting-row label {
    display: block;
    font-weight: 600;
    font-size: 0.92rem;
    color: var(--text-primary);
    margin-bottom: var(--space-3);
  }

  .path-row { display: flex; gap: var(--space-3); }
  .path-row input { flex: 1; }

  .hint { color: var(--text-tertiary); font-size: 0.8rem; margin-top: var(--space-2); }

  .info-section h3 {
    font-size: 0.92rem;
    font-weight: 600;
    color: var(--text-primary);
    margin-bottom: var(--space-3);
  }

  .fmt-list { display: flex; flex-wrap: wrap; gap: var(--space-2); }
  .fmt {
    background: linear-gradient(135deg, var(--accent-100), var(--accent-50));
    color: var(--accent-600);
    padding: 4px 11px;
    border-radius: var(--radius-pill);
    font-size: 0.75rem;
    font-weight: 500;
    font-family: 'SF Mono', Menlo, Consolas, monospace;
  }

  .rule-list { display: flex; flex-direction: column; gap: var(--space-2); }
  .rule {
    background: var(--surface-dim);
    color: var(--text-secondary);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    font-size: 0.82rem;
    border: 1px solid var(--border);
  }

  .save-row { display: flex; justify-content: flex-end; }

  /* 主题切换 */
  .theme-toggle-row {
    display: flex;
    gap: var(--space-3);
  }

  .theme-option {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    font-size: 0.88rem;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-dim);
    border: 1px solid var(--border);
    cursor: pointer;
    transition: all 0.25s ease;
  }

  .theme-option:hover {
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  .theme-option.active {
    background: linear-gradient(135deg, var(--accent-500), var(--accent-400));
    color: white;
    border-color: transparent;
    box-shadow: 0 2px 10px rgba(var(--accent-rgb), 0.3);
  }

  /* 配色方案网格 */
  .scheme-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(88px, 1fr));
    gap: var(--space-3);
  }

  .scheme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-2);
    border-radius: var(--radius-md);
    background: var(--surface-dim);
    border: 2px solid transparent;
    cursor: pointer;
    transition: all 0.25s ease;
  }

  .scheme-card:hover {
    border-color: var(--border-strong);
    transform: translateY(-2px);
  }

  .scheme-card.active {
    border-color: var(--accent-500);
    box-shadow: 0 2px 12px rgba(var(--accent-rgb), 0.2);
  }

  .scheme-preview {
    width: 100%;
    height: 36px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    position: relative;
    overflow: hidden;
  }

  .scheme-dot {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    border: 2px solid white;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  }

  .scheme-name {
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .scheme-card.active .scheme-name {
    color: var(--accent-600);
    font-weight: 600;
  }

  /* ========== 暗色模式适配 ========== */
  :global([data-theme="dark"]) .settings-card {
    background: var(--card-bg);
    border-color: var(--border);
    box-shadow: var(--card-shadow);
  }

  :global([data-theme="dark"]) .setting-row label {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .path-row input {
    background: var(--surface);
    border-color: var(--border);
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .path-row input:focus {
    border-color: var(--accent-400);
  }

  :global([data-theme="dark"]) .path-row .ghost {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(var(--accent-rgb), 0.15);
    color: var(--text-secondary);
    box-shadow: none;
  }

  :global([data-theme="dark"]) .path-row .ghost:hover {
    background: rgba(var(--accent-rgb), 0.15);
    color: var(--accent-300);
    border-color: rgba(var(--accent-rgb), 0.35);
    box-shadow: 0 2px 8px rgba(var(--accent-rgb), 0.15);
  }

  :global([data-theme="dark"]) .hint {
    color: var(--text-tertiary);
  }

  :global([data-theme="dark"]) .info-section h3 {
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .fmt {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.2), rgba(var(--accent-rgb), 0.1));
    color: var(--accent-300);
  }

  :global([data-theme="dark"]) .rule {
    background: var(--surface-dim);
    color: var(--text-secondary);
    border-color: var(--border);
  }

  :global([data-theme="dark"]) .theme-option {
    background: var(--surface-dim);
    border-color: var(--border);
    color: var(--text-secondary);
  }

  :global([data-theme="dark"]) .theme-option:hover {
    border-color: var(--border-strong);
    color: var(--text-primary);
  }

  :global([data-theme="dark"]) .theme-option.active {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.95), rgba(var(--accent-rgb), 0.85));
    color: white;
    border-color: transparent;
    box-shadow: 0 2px 14px rgba(var(--accent-rgb), 0.4);
  }

  :global([data-theme="dark"]) .scheme-card {
    background: var(--surface-dim);
  }

  :global([data-theme="dark"]) .scheme-card.active {
    border-color: var(--accent-500);
  }

  :global([data-theme="dark"]) .scheme-card.active .scheme-name {
    color: var(--accent-300);
  }

  :global([data-theme="dark"]) .save-row button {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.95), rgba(var(--accent-rgb), 0.85));
    color: white;
    box-shadow: 0 2px 12px rgba(var(--accent-rgb), 0.35);
  }

  :global([data-theme="dark"]) .save-row button:hover:not(:disabled) {
    background: linear-gradient(135deg, rgba(var(--accent-rgb), 0.98), rgba(var(--accent-rgb), 0.92));
    box-shadow: 0 4px 20px rgba(var(--accent-rgb), 0.5);
  }

  /* ========== 主题定制选项卡片 ========== */
  .option-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
  }

  .option-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--surface-dim);
    border: 2px solid transparent;
    cursor: pointer;
    transition: all 0.25s ease;
    min-width: 90px;
  }

  .option-card:hover {
    border-color: var(--border-strong);
    transform: translateY(-2px);
  }

  .option-card.active {
    border-color: var(--accent-500);
    box-shadow: 0 2px 12px rgba(var(--accent-rgb), 0.2);
  }

  .option-icon {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    margin-bottom: 2px;
  }

  .option-name {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .option-card.active .option-name {
    color: var(--accent-600);
  }

  .option-desc {
    font-size: 0.7rem;
    color: var(--text-tertiary);
  }

  /* 背景图片预览图标 */
  .bg-pure { background: var(--surface); border: 1px solid var(--border); }
  .bg-gradient {
    background: radial-gradient(ellipse at 30% 30%, var(--accent-300), transparent 70%),
                radial-gradient(ellipse at 70% 70%, var(--accent-200), transparent 70%),
                var(--surface);
  }
  .bg-custom {
    background: var(--surface);
    border: 1px dashed var(--border);
    position: relative;
    overflow: hidden;
  }
  .bg-custom::after {
    content: "🖼";
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    opacity: 0.5;
  }

  /* 背景图片选择行 */
  .bg-picker-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-3);
    flex-wrap: wrap;
  }
  .bg-picker-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .bg-preview {
    width: 48px;
    height: 48px;
    border-radius: var(--radius-sm);
    background-size: cover;
    background-position: center;
    border: 1px solid var(--border);
    flex-shrink: 0;
  }
  .bg-clear-btn {
    font-size: 0.8rem;
    padding: 6px 12px;
  }

  /* 滑块 */
  .slider-row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-3);
  }
  .slider-label {
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-secondary);
    white-space: nowrap;
    min-width: 64px;
  }
  .slider-row input[type="range"] {
    flex: 1;
    height: 6px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--surface-dim);
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    outline: none;
    cursor: pointer;
  }
  .slider-row input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--accent-500), var(--accent-400));
    border: 2px solid white;
    box-shadow: 0 2px 6px rgba(var(--accent-rgb), 0.35);
    cursor: pointer;
  }
  .slider-row input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--accent-500);
    border: 2px solid white;
    box-shadow: 0 2px 6px rgba(var(--accent-rgb), 0.35);
    cursor: pointer;
  }
  .slider-value {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--accent-600);
    min-width: 40px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  /* 毛玻璃预览图标 */
  .glass-solid { background: var(--accent-300); }
  .glass-translucent {
    background: rgba(var(--accent-rgb), 0.5);
    backdrop-filter: blur(4px);
  }
  .glass-glass {
    background: rgba(var(--accent-rgb), 0.3);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(var(--accent-rgb), 0.3);
  }

  /* 阴影预览图标 */
  .shadow-flat { background: var(--accent-300); box-shadow: none; border: 1px solid var(--border); }
  .shadow-standard { background: var(--accent-300); box-shadow: 0 2px 8px rgba(0,0,0,0.15); }
  .shadow-float { background: var(--accent-300); box-shadow: 0 8px 20px rgba(var(--accent-rgb), 0.3); }

  /* 动画预览图标 */
  .motion-minimal { background: var(--accent-200); }
  .motion-standard { background: var(--accent-300); }
  .motion-rich {
    background: linear-gradient(135deg, var(--accent-400), var(--accent-300));
    animation: motionPreview 2s ease-in-out infinite;
  }
  @keyframes motionPreview {
    0%, 100% { transform: scale(1); }
    50% { transform: scale(1.08); }
  }

  /* 暗色模式适配 */
  :global([data-theme="dark"]) .option-card {
    background: var(--surface-dim);
  }
  :global([data-theme="dark"]) .option-card.active {
    border-color: var(--accent-500);
  }
  :global([data-theme="dark"]) .option-card.active .option-name {
    color: var(--accent-300);
  }
  :global([data-theme="dark"]) .bg-pure {
    background: var(--surface);
    border-color: var(--border);
  }
  :global([data-theme="dark"]) .bg-custom {
    background: var(--surface);
    border-color: var(--border);
  }
  :global([data-theme="dark"]) .btn-outline {
    border-color: var(--accent-400);
  }
</style>
