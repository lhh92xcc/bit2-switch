import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { Bit2LoginDialog } from "./Bit2LoginDialog";

const { openLoginWindow, cancelLogin } = vi.hoisted(() => ({
  openLoginWindow: vi.fn(),
  cancelLogin: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@/lib/api", () => ({
  bit2Api: { openLoginWindow, cancelLogin },
}));

describe("Bit2LoginDialog", () => {
  it("opens the verified login window without a manual success button", async () => {
    openLoginWindow.mockResolvedValue(undefined);
    render(<Bit2LoginDialog open onOpenChange={() => undefined} />);

    expect(
      screen.queryByRole("button", { name: "我已完成登录" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "继续登录" }));
    await waitFor(() => expect(openLoginWindow).toHaveBeenCalledOnce());
  });

  it("shows a launch failure without reporting authentication success", async () => {
    openLoginWindow.mockRejectedValue(new Error("window unavailable"));
    render(<Bit2LoginDialog open onOpenChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: "继续登录" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "无法打开登录窗口，请重试",
    );
    expect(screen.queryByText(/登录成功/)).not.toBeInTheDocument();
  });
});

it("invalidates a pending login when the dialog is dismissed", async () => {
  openLoginWindow.mockResolvedValue(undefined);
  render(<Bit2LoginDialog open onOpenChange={() => undefined} />);
  fireEvent.click(screen.getByRole("button", { name: "继续登录" }));
  await screen.findByRole("status");
  fireEvent.keyDown(screen.getByRole("dialog"), {
    key: "Escape",
    code: "Escape",
  });
  await waitFor(() => expect(cancelLogin).toHaveBeenCalled());
});
