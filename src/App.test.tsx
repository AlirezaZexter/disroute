import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import App from "./App";

afterEach(cleanup);

describe("DisRoute dashboard", () => {
  it("explains that only Discord is proxied", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "فقط Discord" })).toBeInTheDocument();
    expect(screen.getByText("بازی‌ها")).toBeInTheDocument();
    expect(screen.getAllByText("Direct")).toHaveLength(2);
  });

  it("announces preview connection failures with text instead of color alone", async () => {
    render(<App />);
    fireEvent.change(screen.getByPlaceholderText("vless://uuid@server:443?..."), {
      target: { value: "vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls" },
    });
    fireEvent.click(screen.getByRole("button", { name: "اتصال Discord" }));
    await waitFor(() => expect(screen.getByRole("heading", { name: "خطای اتصال" })).toBeInTheDocument());
  });
});
