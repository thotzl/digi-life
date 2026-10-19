/**
 * Escapes HTML-significant characters so backend/DB/user data can be safely
 * interpolated into innerHTML template strings (text content and quoted attributes).
 */
export function escapeHtml(s: unknown): string {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
