import "@testing-library/jest-dom/vitest";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "./App";

describe("DisRoute dashboard", () => {
  it("explains that only Discord is proxied", () => {
    render(<App />);
    expect(screen.getByRole("heading", { name: "فقط Discord" })).toBeInTheDocument();
    expect(screen.getByText("بازی‌ها")).toBeInTheDocument();
    expect(screen.getAllByText("Direct")).toHaveLength(2);
  });
});

