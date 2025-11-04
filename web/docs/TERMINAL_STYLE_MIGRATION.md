# Terminal Style Migration - nof1.ai Design System

## 概述
本文档记录了 nof0 web 项目从原有设计向 nof1.ai terminal 风格的迁移过程。

## 设计参考
- **参考网站**: https://nof1.ai (Alpha Arena)
- **参考文件**: `/AI trading in real markets.html` 及其资源文件夹

## 核心设计原则

### 1. Typography (字体系统)
- **主字体**: IBM Plex Mono (全站等宽字体)
- **字体特性**:
  ```css
  font-family: "IBM Plex Mono", monospace;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.01em;
  ```

### 2. Terminal 组件样式

#### Terminal Header (终端标题)
```css
.terminal-header {
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  font-size: 0.875rem;
}
```

#### Terminal Text (终端文本)
```css
.terminal-text {
  font-family: "IBM Plex Mono", monospace;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.01em;
}
```

#### Terminal Button (终端按钮)
```css
.terminal-button {
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--foreground);
  font-family: "IBM Plex Mono", monospace;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 0.25rem 0.75rem;
  cursor: pointer;
  transition: none; /* 无过渡效果 */
  font-size: 0.75rem;
  border-radius: 0 !important; /* 完全方形 */
}
```

#### Terminal Border (终端边框)
```css
.terminal-border {
  border: 2px solid var(--border);
  border-collapse: collapse;
  border-radius: 0 !important; /* 无圆角 */
}
```

### 3. 色彩系统

#### Light Theme (亮色主题)
```css
--background: #ffffff;
--foreground: #000000;          /* 纯黑文字 */
--foreground-muted: #333333;    /* 次级文字 */
--foreground-subtle: #666666;   /* 三级文字 */

--surface: #ffffff;
--surface-elevated: #f8f9fa;
--surface-hover: #f1f3f4;

--border: #000000;              /* 纯黑边框 */
--border-subtle: #cccccc;       /* 次级边框 */

--terminal-green: #00aa00;
--terminal-red: #cc0000;
--terminal-yellow: #b8860b;
--terminal-blue: #0000aa;

--accent-primary: #0000ff;      /* 纯蓝色 */
--accent-success: #00aa00;
--accent-warning: #ffaa00;
--accent-error: #ff0000;
```

#### Dark Theme (暗色主题)
```css
--background: #000000;
--foreground: #00ff00;          /* Terminal Green */
--terminal-green: #00ff00;
--terminal-red: #ff0000;
/* ... 其他颜色 */
```

### 4. 布局特点
- **无圆角**: 所有组件使用 `border-radius: 0 !important`
- **粗线条**: 边框使用 `border: 2px solid`
- **无动画**: 按钮和交互元素使用 `transition: none`
- **方块设计**: 所有卡片、按钮、输入框都是完全方形

### 5. 纹理效果
```css
/* SVG Noise Filter 背景纹理 */
background-image: url("data:image/svg+xml,...");
background-size: 180px 180px;
pointer-events: none;
z-index: 1;
```

## 已实施的改造

### NavigationPage.tsx
**改造前**:
- 使用 rounded-xl 圆角
- zinc-400 等柔和灰色
- transition-all duration-300 平滑过渡
- hover:scale-105 缩放效果

**改造后**:
- 完全方形设计 (terminal-border)
- 纯黑边框和文字
- 无过渡效果 (transition-none)
- Terminal 风格按钮
- 大写字母标题 (UPPERCASE)
- IBM Plex Mono 等宽字体

### globals.css
**新增样式类**:
- `.terminal-border` - 2px 纯黑边框，无圆角
- `.terminal-button` - 标准尺寸终端按钮
- `.terminal-button-small` - 小尺寸终端按钮
- `.terminal-header` - 终端标题样式
- `.terminal-text` - 终端文本样式

**CSS变量更新**:
- 更新 `--foreground` 为纯黑 (#000000)
- 添加 `--foreground-muted` 和 `--foreground-subtle`
- 更新 `--border` 为纯黑 (#000000)
- 添加 terminal 色彩系列

## 组件迁移指南

### 按钮组件
```tsx
// 旧样式
<button className="rounded-lg px-4 py-2 bg-blue-500 hover:bg-blue-600 transition-all">
  Click Me
</button>

// 新样式 (Terminal)
<button className="terminal-button">
  CLICK ME
</button>
```

### 卡片组件
```tsx
// 旧样式
<div className="rounded-xl border shadow-lg hover:shadow-2xl transition-all">
  Content
</div>

// 新样式 (Terminal)
<div className="terminal-border p-8 transition-none" style={{
  borderColor: "var(--border)",
  background: "var(--surface)"
}}>
  Content
</div>
```

### 文本组件
```tsx
// 标题
<h1 className="terminal-header text-4xl">TITLE</h1>

// 正文
<p className="terminal-text" style={{ color: "var(--foreground-muted)" }}>
  Description text
</p>
```

## 待迁移组件

以下组件还需要应用 terminal 风格改造:

### 优先级 P0 (核心页面)
- [ ] Header/Navigation
- [ ] Dashboard
- [ ] Trading Interface

### 优先级 P1 (功能页面)
- [ ] Position Management
- [ ] Order Book
- [ ] Account Analytics
- [ ] Settings Page

### 优先级 P2 (辅助组件)
- [ ] Modal/Dialog
- [ ] Toast/Notification
- [ ] Form Components
- [ ] Table Components
- [ ] Chart Components

## 关键差异对比

| 方面 | 原设计 | Terminal 风格 |
|------|--------|--------------|
| 边框 | 1px, 柔和灰色 | 2px, 纯黑色 |
| 圆角 | rounded-xl (12px) | 0 (方形) |
| 字体 | 混合字体 | IBM Plex Mono |
| 文字大小写 | 正常 | 全大写 |
| 过渡效果 | 300ms 平滑 | 无 (none) |
| 颜色 | 柔和渐变 | 高对比度 |
| 悬停效果 | scale, shadow | 背景变化 |

## 性能优化建议

### 1. 移除不必要的过渡
Terminal 风格的核心是"无过渡"，这可以提升性能:
```css
transition: none !important;
```

### 2. 使用 CSS 变量
所有颜色通过 CSS 变量管理，便于主题切换:
```tsx
style={{ color: "var(--foreground)" }}
```

### 3. 简化 Tailwind 类
减少复杂的 Tailwind 组合，使用自定义类:
```tsx
// 避免
className="border-2 border-black rounded-none p-8 hover:bg-gray-100"

// 推荐
className="terminal-border p-8"
```

## 参考资源

### 外部链接
- [IBM Plex Mono Font](https://fonts.google.com/specimen/IBM+Plex+Mono)
- [nof1.ai](https://nof1.ai)

### 内部文档
- `/web/docs/theme.md` - 主题变量文档
- `/web/src/app/globals.css` - 全局样式定义
- `/AI trading in real markets.html` - 参考 HTML 文件

## 维护指南

### 添加新组件时
1. ✅ 使用 `terminal-*` 样式类
2. ✅ 文本内容使用大写
3. ✅ 颜色使用 CSS 变量
4. ✅ 移除所有圆角
5. ✅ 移除过渡效果
6. ✅ 使用 IBM Plex Mono 字体

### 主题切换支持
所有组件必须支持 light/dark 主题切换:
```css
:root[data-theme="light"] { /* light 样式 */ }
:root[data-theme="dark"] { /* dark 样式 */ }
```

## 版本历史

### v1.0.0 (2025-01-04)
- ✅ 创建 terminal 风格设计系统
- ✅ 迁移 NavigationPage 到 terminal 风格
- ✅ 添加核心 CSS 类和变量
- ✅ 更新颜色系统为高对比度

---

**最后更新**: 2025-01-04  
**负责人**: GitHub Copilot  
**状态**: 🚧 进行中
