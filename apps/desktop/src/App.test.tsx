import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, test } from "vitest";
import { App } from "./App";

afterEach(cleanup);

test("renders the product name as the main heading", () => {
  render(<App />);
  const heading = screen.getByRole("heading", { level: 1 });
  expect(heading.textContent).toBe("Atabey");
});
