import { fireEvent, render, screen } from "@testing-library/react";
import { CodexConsole } from "./CodexConsole";

const provider = {
  id: "bit2-codex",
  name: "bit2.ai Codex",
  settingsConfig: {
    config: `model_provider = "custom"\nmodel = "gpt-5.6-sol"\n[model_providers.custom]\nbase_url = "https://bit2.ai/v1"`,
  },
};

describe("CodexConsole", () => {
  it("shows verified account, endpoint, model and CLI readiness", () => {
    render(
      <CodexConsole
        authStatus={{ connected: true, baseUrl: "https://bit2.ai" }}
        provider={provider}
        cliStatus={{ state: "ready", version: "1.4.2" }}
        onConnect={() => undefined}
        onLogout={() => undefined}
        onLaunch={() => undefined}
        onAddProvider={() => undefined}
        onOpenProviders={() => undefined}
        onOpenSettings={() => undefined}
        onOpenAbout={() => undefined}
      />,
    );

    expect(screen.getByText("已连接 bit2.ai")).toBeInTheDocument();
    expect(screen.getByText("https://bit2.ai/v1")).toBeInTheDocument();
    expect(screen.getByText("gpt-5.6-sol")).toBeInTheDocument();
    expect(screen.getByText("Codex CLI 1.4.2")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "启动 Codex" })).toBeEnabled();
  });

  it("does not claim readiness and offers both onboarding paths", () => {
    const onConnect = vi.fn();
    const onAddProvider = vi.fn();
    render(
      <CodexConsole
        authStatus={{ connected: false, baseUrl: "https://bit2.ai" }}
        provider={undefined}
        cliStatus={{ state: "missing" }}
        onConnect={onConnect}
        onLogout={() => undefined}
        onLaunch={() => undefined}
        onAddProvider={onAddProvider}
        onOpenProviders={() => undefined}
        onOpenSettings={() => undefined}
        onOpenAbout={() => undefined}
      />,
    );

    expect(screen.getByText("尚未连接")).toBeInTheDocument();
    expect(screen.getByText("未检测到 Codex CLI")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "安装并启动 Codex" }),
    ).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "连接 bit2.ai" }));
    fireEvent.click(screen.getByRole("button", { name: "手动配置 API Key" }));
    expect(onConnect).toHaveBeenCalledOnce();
    expect(onAddProvider).toHaveBeenCalledOnce();
  });
});
