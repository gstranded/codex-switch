import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { CodexHistoryArchiveSection } from "@/components/settings/CodexHistoryArchiveSection";

const saveDialogMock = vi.fn();
const openDialogMock = vi.fn();
const exportHistoryMock = vi.fn();
const importHistoryMock = vi.fn();
const toastSuccessMock = vi.fn();
const toastErrorMock = vi.fn();
const toastWarningMock = vi.fn();

vi.mock("@/lib/api", () => ({
  settingsApi: {
    saveCodexHistoryFileDialog: (...args: unknown[]) => saveDialogMock(...args),
    openCodexHistoryFileDialog: (...args: unknown[]) => openDialogMock(...args),
    exportCodexHistoryToFile: (...args: unknown[]) =>
      exportHistoryMock(...args),
    importCodexHistoryFromFile: (...args: unknown[]) =>
      importHistoryMock(...args),
  },
}));

vi.mock("sonner", () => ({
  toast: {
    success: (...args: unknown[]) => toastSuccessMock(...args),
    error: (...args: unknown[]) => toastErrorMock(...args),
    warning: (...args: unknown[]) => toastWarningMock(...args),
  },
}));

function renderSection() {
  const client = new QueryClient();
  return render(
    <QueryClientProvider client={client}>
      <CodexHistoryArchiveSection />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  saveDialogMock.mockReset();
  openDialogMock.mockReset();
  exportHistoryMock.mockReset();
  importHistoryMock.mockReset();
  toastSuccessMock.mockReset();
  toastErrorMock.mockReset();
  toastWarningMock.mockReset();
});

describe("CodexHistoryArchiveSection", () => {
  it("exports a portable chat archive through the native save dialog", async () => {
    const user = userEvent.setup();
    saveDialogMock.mockResolvedValueOnce("C:/exports/chat-history.zip");
    exportHistoryMock.mockResolvedValueOnce({
      filePath: "C:/exports/chat-history.zip",
      sessionFiles: 3,
      stateDatabases: 1,
    });
    renderSection();

    await user.click(
      screen.getByRole("button", { name: "settings.chatHistory.export" }),
    );

    expect(saveDialogMock).toHaveBeenCalledTimes(1);
    expect(exportHistoryMock).toHaveBeenCalledWith(
      "C:/exports/chat-history.zip",
    );
    expect(toastSuccessMock).toHaveBeenCalledTimes(1);
  });

  it("imports a selected archive and refreshes the visible session list", async () => {
    const user = userEvent.setup();
    openDialogMock.mockResolvedValueOnce("C:/imports/portable-history.zip");
    importHistoryMock.mockResolvedValueOnce({
      importedSessionFiles: 2,
      skippedSessionFiles: 1,
      importedSessionIndexEntries: 2,
      importedStateThreads: 2,
      warnings: [],
    });
    renderSection();

    await user.click(
      screen.getByRole("button", {
        name: "settings.chatHistory.selectImport",
      }),
    );
    expect(await screen.findByText("portable-history.zip")).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: "settings.chatHistory.import" }),
    );

    expect(importHistoryMock).toHaveBeenCalledWith(
      "C:/imports/portable-history.zip",
    );
    expect(toastSuccessMock).toHaveBeenCalledTimes(1);
  });
});
