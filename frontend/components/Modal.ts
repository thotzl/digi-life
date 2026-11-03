// Modal dialog (TCK-160): one component for every dialog (formerly three hand-built copies, each appending its own
// `<style>` tag with conflicting `.modal-card` rules on every open). A modal lives in a child scope of the view
// that opened it: leaving the view closes it, closing it stops its preview loops.
import type { Scope } from "../core/scope";
import { styled, type TrustedHtml } from "../core/styled";

const Backdrop = styled("div")`
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(2, 6, 23, 0.85);
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(8px);
`;

const Box = styled("div")`
  background: rgba(15, 23, 42, 0.9);
  border: 1px solid var(--modal-accent-border);
  border-radius: 8px;
  padding: 24px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  gap: 14px;
  box-shadow: 0 0 35px var(--modal-accent-glow);
  animation: slide-fade-in 0.2s ease-out;
  font-family: monospace;
  color: #f1f5f9;
`;

const Title = styled("h3")`
  margin: 0;
  font-size: 1.05rem;
  font-weight: bold;
  color: #fff;
  text-transform: uppercase;
  letter-spacing: 1px;
  border-bottom: 1px solid rgba(148, 163, 184, 0.1);
  padding-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 6px;

  svg { color: var(--modal-accent); }
`;

const Text = styled("p")`
  margin: 0;
  font-size: 0.68rem;
  color: #94a3b8;
  line-height: 1.45;
`;

const ButtonRow = styled("div")`
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  border-top: 1px solid rgba(148, 163, 184, 0.1);
  padding-top: 12px;
  margin-top: 4px;
`;

const Button = styled("button")`
  border-radius: 4px;
  font-family: monospace;
  font-size: 0.7rem;
  font-weight: bold;
  cursor: pointer;
  padding: 8px 16px;
  background: rgba(148, 163, 184, 0.06);
  border: 1px solid rgba(148, 163, 184, 0.15);
  color: #94a3b8;

  &.modal-primary {
    background: var(--modal-accent);
    border: none;
    color: #020617;
  }
`;

const CheckLabel = styled("label")`
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.65rem;
  cursor: pointer;
  user-select: none;
  color: #94a3b8;

  input { cursor: pointer; width: 15px; height: 15px; margin: 0; }
`;

export interface ModalAction {
  label: string;
  primary?: boolean;
  /** Runs on click; the modal closes afterwards unless it returns `false` (e.g. a rejected request). */
  onClick?: () => void | boolean | Promise<void | boolean>;
}

export interface ModalOptions {
  title: string;
  icon?: TrustedHtml;
  /** Accent colour as "r, g, b" (border, glow, primary button) */
  accent: string;
  width?: number;
  text?: string;
  body?: (Node | null)[];
  /** Defaults to a single "Cancel" */
  actions?: ModalAction[];
}

export interface ModalHandle {
  readonly element: HTMLElement;
  /** Child scope of the opener; disposed when the modal closes */
  readonly scope: Scope;
  close(): void;
}

/** Opens a modal over `#app` in a child scope of `scope`; disposing `scope` closes it. */
export function openModal(scope: Scope, options: ModalOptions): ModalHandle {
  const modalScope = scope.child();
  const box = Box({ style: { width: `${options.width ?? 410}px` } });
  box.style.setProperty("--modal-accent", `rgb(${options.accent})`);
  box.style.setProperty("--modal-accent-border", `rgba(${options.accent}, 0.35)`);
  box.style.setProperty("--modal-accent-glow", `rgba(${options.accent}, 0.14)`);
  const backdrop = Backdrop({ children: box });

  const handle: ModalHandle = {
    element: box,
    scope: modalScope,
    close: () => modalScope.dispose(),
  };
  modalScope.add(() => backdrop.remove());

  box.appendChild(Title({ children: options.icon ? [options.icon, options.title] : options.title }));
  if (options.text) box.appendChild(Text({ children: options.text }));
  for (const node of options.body ?? []) if (node) box.appendChild(node);

  const actions = options.actions ?? [{ label: "Cancel" }];
  const row = ButtonRow();
  for (const action of actions) {
    const button = Button({ class: action.primary ? "modal-primary" : "", children: action.label });
    button.addEventListener("click", async () => {
      const keepOpen = (await action.onClick?.()) === false;
      if (!keepOpen) handle.close();
    });
    row.appendChild(button);
  }
  box.appendChild(row);

  (document.getElementById("app") ?? document.body).appendChild(backdrop);
  return handle;
}

/** Labelled checkbox for modal bodies. */
export function checkboxRow(label: string, checked = false): { element: HTMLElement; input: HTMLInputElement } {
  const input = document.createElement("input");
  input.type = "checkbox";
  input.checked = checked;
  const span = document.createElement("span");
  span.textContent = label;
  return { element: CheckLabel({ children: [input, span] }), input };
}

/** Paragraph for modal bodies (text only). */
export function modalText(text: string): HTMLElement {
  return Text({ children: text });
}
