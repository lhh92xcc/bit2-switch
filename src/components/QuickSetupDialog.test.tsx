import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QuickSetupDialog } from "./QuickSetupDialog";

const api = vi.hoisted(() => ({
  getToolVersions: vi.fn(),
  runToolLifecycleAction: vi.fn(),
  storeBit2Secret: vi.fn(),
}));
vi.mock("@/lib/api", () => ({ settingsApi: api }));

it("does not save or launch before the installer produces a usable CLI", async () => {
  api.getToolVersions.mockResolvedValue([{ version: null }]);
  api.runToolLifecycleAction.mockResolvedValue(undefined);
  const complete = vi.fn();
  render(
    <QuickSetupDialog
      open
      onOpenChange={() => undefined}
      onComplete={complete}
    />,
  );
  expect(screen.queryByText("Claude Code")).not.toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "下一步" }));
  expect(screen.getByLabelText("API 请求地址")).toHaveValue(
    "https://bit2.ai/v1",
  );
  fireEvent.change(screen.getByLabelText("API Key"), {
    target: { value: "example-secret" },
  });
  fireEvent.click(screen.getByRole("button", { name: "下一步" }));
  fireEvent.click(screen.getByRole("button", { name: "保存并打开" }));
  await waitFor(() =>
    expect(api.runToolLifecycleAction).toHaveBeenCalledWith(
      ["codex"],
      "install",
    ),
  );
  expect(await screen.findByRole("status")).toHaveTextContent("等待检测结果");
  expect(api.storeBit2Secret).not.toHaveBeenCalled();
  expect(complete).not.toHaveBeenCalled();
  api.getToolVersions.mockResolvedValue([
    { version: "1.2.3", installed_but_broken: false },
  ]);
  await waitFor(() => expect(complete).toHaveBeenCalledOnce(), {
    timeout: 4500,
  });
  expect(api.storeBit2Secret).toHaveBeenCalledWith(
    "bit2-switch",
    expect.stringMatching(/^codex-/),
    "example-secret",
  );
});
