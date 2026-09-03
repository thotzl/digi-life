let styleSheet: HTMLStyleElement | null = null;
let classCounter = 0;

/**
 * Highly performant, typified, Zero-Dependency CSS-in-JS Engine for pure TypeScript.
 * Compiles styled-component template literals, injects styles dynamically into a 
 * single head stylesheet, and returns fully typified DOM nodes.
 */
export function styled<T extends keyof HTMLElementTagNameMap>(tag: T) {
  return (strings: TemplateStringsArray, ...values: any[]) => {
    return (props: Record<string, any> = {}): HTMLElementTagNameMap[T] => {
      // 1. Compile raw template CSS string
      const css = strings.reduce((acc, str, i) => acc + str + (values[i] !== undefined ? values[i] : ""), "");

      // 2. Generate unique class identifier
      const className = `styled-${tag}-${classCounter++}`;

      // 3. Ensure document-wide Stylesheet element exists in the head
      if (typeof document !== "undefined" && !styleSheet) {
        styleSheet = document.createElement("style");
        styleSheet.id = "styled-components-stylesheet";
        document.head.appendChild(styleSheet);
      }

      // 4. Parse flat & nested CSS blocks (resolving '&' with our class)
      if (styleSheet) {
        const rules = parseCSSRules(className, css);
        rules.forEach(rule => {
          try {
            styleSheet!.sheet?.insertRule(rule, styleSheet!.sheet.cssRules.length);
          } catch (e) {
            console.warn("[Styled TS] Failed to insert rule:", rule, e);
          }
        });
      }

      // 5. Create DOM Element with correct namespace
      const isSvgTag = ["svg", "path", "g", "circle", "rect", "line", "text"].includes(tag);
      const el = (isSvgTag 
        ? document.createElementNS("http://www.w3.org/2000/svg", tag)
        : document.createElement(tag)) as any;

      el.classList.add(className);

      // 6. Bind props, style objects, event listeners, and children
      for (const [key, val] of Object.entries(props)) {
        if (key === "children") {
          applyChildren(el, val);
        } else if (key === "style" && typeof val === "object") {
          Object.assign(el.style, val);
        } else if (key.startsWith("on") && typeof val === "function") {
          const eventName = key.slice(2).toLowerCase();
          el.addEventListener(eventName, val as any);
        } else {
          if (isSvgTag) {
            el.setAttribute(key, String(val));
          } else {
            el[key] = val;
          }
        }
      }

      return el;
    };
  };
}

/**
 * Extracts nested CSS blocks (e.g. '&:hover', '& .child') and converts them 
 * into valid stylesheet rules, keeping flat declarations on the base class.
 */
function parseCSSRules(className: string, cssText: string): string[] {
  const rules: string[] = [];
  
  // 1. Strip CSS comments to prevent syntax parsing bugs
  const cleanText = cssText.replace(/\/\*[\s\S]*?\*\//g, "").trim();
  if (!cleanText) return [];

  let mainStyles = "";
  let remaining = cleanText;

  while (true) {
    const startIdx = remaining.indexOf("{");
    if (startIdx === -1) {
      mainStyles += " " + remaining;
      break;
    }

    const precedingText = remaining.substring(0, startIdx).trim();
    
    // Track matching brace depth to support media queries or nested blocks
    let depth = 1;
    let endIdx = -1;
    for (let i = startIdx + 1; i < remaining.length; i++) {
      if (remaining[i] === "{") depth++;
      else if (remaining[i] === "}") {
        depth--;
        if (depth === 0) {
          endIdx = i;
          break;
        }
      }
    }

    if (endIdx === -1) {
      mainStyles += " " + remaining;
      break;
    }

    const blockBody = remaining.substring(startIdx + 1, endIdx).trim();

    // Split precedingText into main styles (base properties) and selector part
    let baseStylesPart = "";
    let selectorPart = precedingText;

    const lastSemicolonIdx = precedingText.lastIndexOf(";");
    if (lastSemicolonIdx !== -1) {
      baseStylesPart = precedingText.substring(0, lastSemicolonIdx + 1).trim();
      selectorPart = precedingText.substring(lastSemicolonIdx + 1).trim();
    }

    // Append base styles to main styles rule
    if (baseStylesPart) {
      mainStyles += " " + baseStylesPart;
    }

    // Process nested selector by replacing '&' with our class
    const resolvedSelector = selectorPart.replace(/&/g, `.${className}`);
    rules.push(`${resolvedSelector} { ${blockBody} }`);

    // Remove parsed block from remaining text
    remaining = remaining.substring(endIdx + 1).trim();
  }

  // Prepend main styles block so base class rules are registered first
  if (mainStyles.trim()) {
    rules.unshift(`.${className} { ${mainStyles.trim()} }`);
  }

  return rules;
}

/**
 * Helper to append diverse children nodes (strings, numbers, elements, or arrays thereof).
 * Auto-detects and compiles raw HTML/SVG markup safely into DOM Nodes.
 */
function applyChildren(parent: Element, children: any) {
  if (children === null || children === undefined) return;

  const appendSingle = (child: any) => {
    if (child === null || child === undefined) return;
    if (typeof child === "string" || typeof child === "number") {
      const str = String(child);
      if (str.trim().startsWith("<")) {
        // Safe live compilation of raw HTML/SVG vector tags
        const temp = document.createElement("div");
        temp.innerHTML = str;
        while (temp.firstChild) {
          parent.appendChild(temp.firstChild);
        }
      } else {
        parent.appendChild(document.createTextNode(str));
      }
    } else if (child instanceof HTMLElement || child instanceof SVGElement || child instanceof DocumentFragment || child instanceof Text) {
      parent.appendChild(child);
    }
  };

  if (Array.isArray(children)) {
    children.forEach(appendSingle);
  } else {
    appendSingle(children);
  }
}
