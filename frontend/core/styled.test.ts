// Regression test for the styled() child handling (TCK-197): string children are always text, never parsed as HTML.
// The vitest environment is `node` and no DOM library is installed, so a minimal fake DOM records every HTML parse.
import { beforeAll, describe, expect, it } from "vitest";

const parsedMarkup: string[] = [];

class FakeNode {
  parentNode: FakeNode | null = null;
  childNodes: FakeNode[] = [];
  get firstChild(): FakeNode | null {
    return this.childNodes[0] ?? null;
  }
  appendChild(node: FakeNode): FakeNode {
    if (node.parentNode) node.parentNode.childNodes = node.parentNode.childNodes.filter((n) => n !== node);
    node.parentNode = this;
    this.childNodes.push(node);
    return node;
  }
}

class FakeText extends FakeNode {
  constructor(readonly data: string) {
    super();
  }
}

class FakeElement extends FakeNode {
  readonly classList = { add: (..._names: string[]) => {} };
  readonly style: Record<string, string> = {};
  constructor(readonly tagName: string) {
    super();
  }
  addEventListener(): void {}
  setAttribute(): void {}
  set innerHTML(markup: string) {
    parsedMarkup.push(markup);
    this.childNodes = [];
    this.appendChild(new FakeElement("parsed"));
  }
}

beforeAll(() => {
  const g = globalThis as any;
  g.HTMLElement = FakeElement;
  g.SVGElement = class {};
  g.DocumentFragment = class {};
  g.Text = FakeText;
  g.document = {
    head: new FakeElement("head"),
    createElement: (tag: string) => new FakeElement(tag),
    createElementNS: (_ns: string, tag: string) => new FakeElement(tag),
    createTextNode: (data: string) => new FakeText(data),
  };
});

describe("styled children", () => {
  it("renders a markup-like string child (user-entered creature name) as text", async () => {
    const { styled } = await import("./styled");
    parsedMarkup.length = 0;
    const name = "<img src=x onerror=alert(1)>";
    const el = styled("h3")``({ children: name }) as unknown as FakeElement;
    expect(parsedMarkup).toEqual([]);
    expect(el.childNodes).toHaveLength(1);
    expect(el.childNodes[0]).toBeInstanceOf(FakeText);
    expect((el.childNodes[0] as FakeText).data).toBe(name);
  });

  it("renders markup strings inside a children array as text", async () => {
    const { styled } = await import("./styled");
    parsedMarkup.length = 0;
    const el = styled("span")``({ children: ["  <svg onload=alert(1)>", 42] }) as unknown as FakeElement;
    expect(parsedMarkup).toEqual([]);
    expect(el.childNodes.map((n) => (n as FakeText).data)).toEqual(["  <svg onload=alert(1)>", "42"]);
  });

  it("parses markup only when it is marked with html()", async () => {
    const mod: any = await import("./styled");
    expect(typeof mod.html).toBe("function");
    parsedMarkup.length = 0;
    const icon = "<svg viewBox='0 0 24 24'><path d='M0 0'/></svg>";
    const el = mod.styled("h3")``({ children: [mod.html(icon), " Title"] }) as FakeElement;
    expect(parsedMarkup).toEqual([icon]);
    expect(el.childNodes).toHaveLength(2);
    expect((el.childNodes[1] as FakeText).data).toBe(" Title");
    expect(`${mod.html(icon)}`).toBe(icon); // icons still interpolate into innerHTML templates
  });
});
