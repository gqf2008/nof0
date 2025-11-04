# Terminal Style Quick Reference

## CSS Classes

### Layout
```tsx
.terminal-border        // 2px black border, square corners
```

### Typography
```tsx
.terminal-header       // Bold uppercase headers (letter-spacing: 0.1em)
.terminal-text         // Monospace text with tabular numbers
```

### Buttons
```tsx
.terminal-button       // Standard terminal button (0.75rem)
.terminal-button-small // Small terminal button (responsive)
```

## Common Patterns

### Page Title
```tsx
<h1 className="terminal-header text-4xl" style={{ color: "var(--foreground)" }}>
  PAGE TITLE
</h1>
```

### Card Component
```tsx
<div 
  className="terminal-border p-8 transition-none"
  style={{
    borderColor: "var(--border)",
    background: "var(--surface)"
  }}
>
  {/* content */}
</div>
```

### Button
```tsx
<button 
  className="terminal-button"
  style={{
    borderColor: "var(--border)",
    background: "var(--surface-elevated)"
  }}
>
  BUTTON TEXT
</button>
```

### Status Badge
```tsx
<div
  className="terminal-button-small"
  style={{
    background: "rgba(0, 170, 0, 0.1)",
    color: "var(--terminal-green)",
    borderColor: "var(--terminal-green)"
  }}
>
  ACTIVE
</div>
```

## Color Variables

### Foreground Colors
```css
var(--foreground)        // Primary text (#000 light, #0f0 dark)
var(--foreground-muted)  // Secondary text (#333)
var(--foreground-subtle) // Tertiary text (#666)
```

### Surface Colors
```css
var(--surface)           // Default surface (#fff)
var(--surface-elevated)  // Elevated surface (#f8f9fa)
var(--surface-hover)     // Hover surface (#f1f3f4)
```

### Border Colors
```css
var(--border)            // Primary border (#000)
var(--border-subtle)     // Secondary border (#ccc)
```

### Terminal Colors
```css
var(--terminal-green)    // #00aa00
var(--terminal-red)      // #cc0000
var(--terminal-yellow)   // #b8860b
var(--terminal-blue)     // #0000aa
```

### Accent Colors
```css
var(--accent-primary)    // #0000ff (nof1.ai blue)
var(--accent-success)    // #00aa00
var(--accent-warning)    // #ffaa00
var(--accent-error)      // #ff0000
```

## Design Rules

✅ **DO**:
- Use UPPERCASE for all text
- Use `transition: none`
- Use IBM Plex Mono font
- Use 2px borders
- Use square corners (border-radius: 0)
- Use CSS variables for colors

❌ **DON'T**:
- Use rounded corners
- Use smooth transitions
- Use lowercase for UI text
- Use thin borders (1px)
- Use hardcoded colors
- Use mixed fonts

## Example Component

```tsx
import React from 'react';

export function TerminalCard({ title, children, status }: Props) {
  return (
    <div
      className="terminal-border p-8 transition-none"
      style={{
        borderColor: "var(--border)",
        background: "var(--surface)"
      }}
    >
      <div className="flex items-center justify-between mb-6">
        <h2 className="terminal-header text-2xl" style={{ color: "var(--foreground)" }}>
          {title.toUpperCase()}
        </h2>
        
        {status && (
          <div
            className="terminal-button-small"
            style={{
              background: status === "active" 
                ? "rgba(0, 170, 0, 0.1)" 
                : "rgba(170, 170, 170, 0.1)",
              color: status === "active"
                ? "var(--terminal-green)"
                : "var(--foreground-muted)",
              borderColor: status === "active"
                ? "var(--terminal-green)"
                : "var(--border-subtle)"
            }}
          >
            {status.toUpperCase()}
          </div>
        )}
      </div>
      
      <div className="terminal-text" style={{ color: "var(--foreground-muted)" }}>
        {children}
      </div>
    </div>
  );
}
```

## Migration Checklist

For each component:
- [ ] Replace all `rounded-*` with `border-radius: 0`
- [ ] Replace `transition-*` with `transition: none`
- [ ] Convert text to UPPERCASE
- [ ] Use `terminal-*` CSS classes
- [ ] Use CSS variables for colors
- [ ] Replace border with `terminal-border`
- [ ] Use IBM Plex Mono font

---

**See Also**: `TERMINAL_STYLE_MIGRATION.md` for full documentation
