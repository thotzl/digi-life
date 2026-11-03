import { describe, expect, it, vi } from "vitest";

const callMock = vi.fn();
vi.mock("./ipc", () => ({ call: (...args: unknown[]) => callMock(...args) }));

import { getPhenotype, PHENOTYPE_RETRY_MS } from "./phenotypes";

describe("phenotype cache (TCK-160)", () => {
  it("a failed request is not repeated every frame, only after the retry delay", async () => {
    callMock.mockRejectedValue({ kind: "database", message: "bad genome" });
    expect(await getPhenotype("BROKEN", 1000)).toBeNull();
    expect(await getPhenotype("BROKEN", 1016)).toBeNull();
    expect(callMock).toHaveBeenCalledTimes(1);
    callMock.mockResolvedValue({ latinName: "X" });
    expect(await getPhenotype("BROKEN", 1000 + PHENOTYPE_RETRY_MS)).toEqual({ latinName: "X" });
    expect(callMock).toHaveBeenCalledTimes(2);
  });
});
