import "@testing-library/jest-dom/vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import App from "./App";

afterEach(cleanup);

describe("DisRoute dashboard", () => {
  it("explains that only Discord is proxied", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "مسیر ترافیک" })).toBeInTheDocument();
    expect(screen.getByText("فقط Discord از پروکسی عبور می‌کند.")).toBeInTheDocument();
    expect(screen.getByText("بازی‌ها")).toBeInTheDocument();
    expect(screen.getAllByText("Direct")).toHaveLength(2);
    expect(screen.getByRole("heading", { name: "کانفیگ شخصی" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "بررسی آپدیت" })).toBeInTheDocument();
    expect(screen.getByText(/مجوز Firewall موردنیاز/)).toBeInTheDocument();
    expect(screen.getByLabelText(/مسیر دوطرفهٔ شبکه/).querySelectorAll("svg")).toHaveLength(2);
  });

  it("announces preview connection failures with text instead of color alone", async () => {
    render(<App />);
    await waitFor(() => expect(screen.getByPlaceholderText("vless:// · vmess:// · trojan:// · ss://")).toBeEnabled());
    fireEvent.change(screen.getByPlaceholderText("vless:// · vmess:// · trojan:// · ss://"), {
      target: { value: "vless://00000000-0000-4000-8000-000000000000@example.com:443?security=tls" },
    });
    fireEvent.click(screen.getByRole("button", { name: "اتصال Discord" }));
    await waitFor(() => expect(screen.getByRole("heading", { name: "خطای اتصال" })).toBeInTheDocument());
    expect(screen.getByRole("alert")).toBeInTheDocument();
  });

  it("reports the current version when the updater runs outside Tauri", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "بررسی آپدیت" }));
    await waitFor(() => expect(screen.getByText("نسخه جدیدی منتشر نشده")).toBeInTheDocument());
  });

  it("requires an explicit community warning acknowledgement", async () => {
    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "اتصال سریع رایگان" }));
    expect(screen.getByRole("alertdialog", { name: "پیش از استفاده بخوانید" })).toBeInTheDocument();
    const continueButton = screen.getByRole("button", { name: "ادامه" });
    expect(continueButton).toBeDisabled();
    fireEvent.click(screen.getByRole("checkbox", { name: /این هشدار را خواندم/ }));
    fireEvent.click(continueButton);
    await waitFor(() => expect(screen.getByText("مدیریت منابع اتصال سریع")).toBeInTheDocument());
  });
});
