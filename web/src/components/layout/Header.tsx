"use client";
import { Link, useLocation } from "react-router-dom";
import { useTheme } from "@/store/useTheme";

export function Header() {
  const theme = useTheme((s) => s.theme);
  const resolved = useTheme((s) => s.resolved);
  const setTheme = useTheme((s) => s.setTheme);
  // 固定 Twitter/X 关注账号
  const TWITTER_HANDLE = "wquguru";
  const barCls = `sticky top-0 z-50 w-full border-b-2 backdrop-blur`; /* 改为2px粗边框 */
  const textCls = "";
  const hoverLink = "";
  const brandCls = "";

  const location = useLocation();

  return (
    <header
      className={barCls}
      style={{
        background: "var(--header-bg)",
        borderColor: "var(--header-border)",
      }}
    >
      <div
        className={`ui-sans relative flex h-[var(--header-h)] w-full items-center px-3 text-xs`}
        style={{ color: "var(--foreground)" }}
      >
        {/* 左：品牌 */}
        <div className="flex min-w-0 flex-1">
          <Link
            to="/"
            className="font-semibold tracking-wide ui-sans text-base" /* 增大字体 */
            style={{ color: "var(--brand-accent)", textDecoration: "none" }}
          >
            nof0
          </Link>
        </div>

        {/* 中:主导航(绝对居中) - 移除边框 */}
        <nav
          className="ui-sans absolute left-1/2 -translate-x-1/2 flex items-center"
          aria-label="Primary"
          style={{ fontSize: "16px" }}
        >
          <NavItem to="/" label="交易所" currentPath={location.pathname} />
          <span style={{ color: "#d0d0d0", margin: "0 8px" }}>|</span>
          <NavItem
            to="/leaderboard"
            label="排行榜"
            currentPath={location.pathname}
          />
          <span style={{ color: "#d0d0d0", margin: "0 8px" }}>|</span>
          <NavItem
            to="/models"
            label="模型"
            currentPath={location.pathname}
          />
        </nav>

        {/* 右：主题切换占位，保证中间绝对定位不受挤压 */}
        <div className="flex min-w-0 flex-1 justify-end">
          {/* 右侧：外链 */}
          <div className="flex items-center gap-2">
            {/* GitHub */}
            <a
              href="https://github.com/wquguru/nof0"
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Open GitHub repository"
              className="inline-flex items-center justify-center w-7 h-7"
              style={{
                color: "var(--btn-inactive-fg)",
              }}
              title="GitHub"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                width="16"
                height="16"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M12 .5C5.73.5.97 5.26.97 11.54c0 4.86 3.15 8.98 7.52 10.43.55.1.75-.24.75-.53 0-.26-.01-1.13-.02-2.05-3.06.67-3.71-1.3-3.71-1.3-.5-1.28-1.22-1.63-1.22-1.63-.99-.68.08-.67.08-.67 1.09.08 1.66 1.12 1.66 1.12.98 1.67 2.56 1.19 3.19.91.1-.71.38-1.19.69-1.46-2.44-.28-5.01-1.22-5.01-5.42 0-1.2.43-2.18 1.12-2.95-.11-.28-.49-1.42.11-2.96 0 0 .93-.3 3.05 1.13.89-.25 1.84-.38 2.79-.38.95 0 1.9.13 2.79.38 2.12-1.43 3.05-1.13 3.05-1.13.6 1.54.22 2.68.11 2.96.69.77 1.12 1.75 1.12 2.95 0 4.21-2.57 5.14-5.02 5.41.39.34.73 1.01.73 2.03 0 1.46-.01 2.63-.01 2.98 0 .29.19.64.75.53 4.37-1.45 7.52-5.57 7.52-10.43C23.03 5.26 18.27.5 12 .5z" />
              </svg>
            </a>
            {/* Twitter/X 关注意图 */}
            <a
              href={`https://twitter.com/intent/follow?screen_name=${TWITTER_HANDLE}`}
              target="_blank"
              rel="noopener noreferrer"
              aria-label="Follow on X (Twitter)"
              className="inline-flex items-center justify-center w-7 h-7"
              style={{
                color: "var(--btn-inactive-fg)",
              }}
              title="Follow on X"
            >
              <svg
                xmlns="http://www.w3.org/2000/svg"
                viewBox="0 0 24 24"
                width="16"
                height="16"
                fill="currentColor"
                aria-hidden="true"
              >
                <path d="M18.244 2H21.5l-7.5 8.57L23 22h-6.555l-5.12-6.622L5.38 22H2.12l8.08-9.236L2 2h6.69l4.64 6.02L18.244 2zm-2.296 18h1.82L8.16 4H6.25l9.698 16z" />
              </svg>
            </a>
          </div>
        </div>
      </div>
    </header>
  );
}

export default Header;

function NavItem({
  to,
  label,
  currentPath,
}: {
  to: string;
  label: string;
  currentPath: string;
}) {
  const isActive =
    currentPath === to || (to !== "/" && currentPath.startsWith(to));
  return (
    <Link
      to={to}
      className={`ui-sans px-4 py-2 uppercase tracking-wider`}
      style={{
        color: "inherit",
        textDecoration: "none",
        fontWeight: isActive ? 600 : 400,
        fontSize: "inherit",
      }}
    >
      {label}
    </Link>
  );
}
