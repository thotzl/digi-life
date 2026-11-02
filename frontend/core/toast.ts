// Minimal toast notifications (TCK-157): IPC errors reach the user instead of only the console.
// Text goes through `textContent` only (backend error text is untrusted).

export type ToastKind = "error" | "info";

const TOAST_MS = 6000;
let stack: HTMLDivElement | null = null;

function toastStack(): HTMLDivElement {
  if (!stack || !stack.isConnected) {
    stack = document.createElement("div");
    stack.className = "toast-stack";
    document.body.appendChild(stack);
  }
  return stack;
}

/** Shows `message` for a few seconds; an identical message that is still visible is not repeated. */
export function showToast(message: string, kind: ToastKind = "info") {
  if (typeof document === "undefined") return;
  const container = toastStack();
  for (const existing of Array.from(container.children)) {
    if (existing.textContent === message) return;
  }
  const toast = document.createElement("div");
  toast.className = `toast toast-${kind}`;
  toast.textContent = message;
  toast.addEventListener("click", () => toast.remove());
  container.appendChild(toast);
  setTimeout(() => toast.remove(), TOAST_MS);
}
