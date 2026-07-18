import { useMemo, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import {
  ArchiveRestore,
  Download,
  FileArchive,
  Loader2,
  Upload,
  X,
} from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { settingsApi } from "@/lib/api";
import { extractErrorMessage } from "@/utils/errorUtils";

const formatArchiveStamp = (date: Date) =>
  `${date.getFullYear()}${String(date.getMonth() + 1).padStart(2, "0")}${String(
    date.getDate(),
  ).padStart(2, "0")}_${String(date.getHours()).padStart(2, "0")}${String(
    date.getMinutes(),
  ).padStart(2, "0")}${String(date.getSeconds()).padStart(2, "0")}`;

export function CodexHistoryArchiveSection() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [selectedFile, setSelectedFile] = useState("");
  const [isExporting, setIsExporting] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [isRestarting, setIsRestarting] = useState(false);
  const [restartPromptOpen, setRestartPromptOpen] = useState(false);

  const selectedFileName = useMemo(() => {
    if (!selectedFile) return "";
    const segments = selectedFile.split(/[\\/]/);
    return segments[segments.length - 1] || selectedFile;
  }, [selectedFile]);

  const selectImportFile = async () => {
    try {
      const path = await settingsApi.openCodexHistoryFileDialog();
      if (path) setSelectedFile(path);
    } catch (error) {
      toast.error(
        extractErrorMessage(error) ||
          t("settings.chatHistory.selectFailed", {
            defaultValue: "选择聊天记录归档失败",
          }),
      );
    }
  };

  const exportHistory = async () => {
    if (isExporting) return;
    setIsExporting(true);
    try {
      const defaultName = `codex-switch-workspace-${formatArchiveStamp(new Date())}.zip`;
      const destination =
        await settingsApi.saveCodexHistoryFileDialog(defaultName);
      if (!destination) return;
      const result = await settingsApi.exportCodexHistoryToFile(destination);
      toast.success(
        t("settings.chatHistory.exported", {
          defaultValue: "已导出 {{count}} 个 Codex 会话",
          count: result.sessionFiles,
        }),
        { description: result.filePath, closeButton: true },
      );
    } catch (error) {
      toast.error(
        extractErrorMessage(error) ||
          t("settings.chatHistory.exportFailed", {
            defaultValue: "导出聊天记录失败",
          }),
      );
    } finally {
      setIsExporting(false);
    }
  };

  const importHistory = async () => {
    if (!selectedFile) {
      await selectImportFile();
      return;
    }
    if (isImporting) return;
    setIsImporting(true);
    try {
      const result = await settingsApi.importCodexHistoryFromFile(selectedFile);
      await queryClient.invalidateQueries({ queryKey: ["sessions"] });
      await queryClient.invalidateQueries({ queryKey: ["providers", "codex"] });
      await queryClient.invalidateQueries({ queryKey: ["settings"] });
      toast.success(
        t("settings.chatHistory.imported", {
          defaultValue: "已导入 {{count}} 个会话，并同步到当前供应商",
          count: result.importedSessionFiles,
        }),
        { closeButton: true },
      );
      if (result.warnings.length > 0) {
        toast.warning(
          t("settings.chatHistory.importWarning", {
            defaultValue:
              "聊天记录已导入；部分本地索引未能合并，重新打开 Codex 后会自动重建。",
          }),
        );
      }
      setSelectedFile("");
      setRestartPromptOpen(true);
    } catch (error) {
      toast.error(
        extractErrorMessage(error) ||
          t("settings.chatHistory.importFailed", {
            defaultValue: "导入聊天记录失败",
          }),
      );
    } finally {
      setIsImporting(false);
    }
  };

  const restartCodex = async () => {
    if (isRestarting) return;
    setIsRestarting(true);
    try {
      await settingsApi.restartCodexClient();
      toast.success(
        t("settings.chatHistory.restartSuccess", {
          defaultValue: "Codex 已重新打开并加载导入的数据",
        }),
      );
      setRestartPromptOpen(false);
    } catch (error) {
      toast.error(
        extractErrorMessage(error) ||
          t("settings.chatHistory.restartFailed", {
            defaultValue: "无法自动重新打开 Codex，请手动退出后再打开",
          }),
      );
    } finally {
      setIsRestarting(false);
    }
  };

  return (
    <section className="space-y-4 border-t border-border/50 pt-6">
      <header className="space-y-2">
        <div className="flex items-center gap-2">
          <FileArchive className="h-4 w-4 text-emerald-500" />
          <h3 className="text-base font-semibold text-foreground">
            {t("settings.chatHistory.title")}
          </h3>
        </div>
        <p className="text-sm text-muted-foreground">
          {t("settings.chatHistory.description")}
        </p>
      </header>

      <div className="grid gap-3 sm:grid-cols-2">
        <Button
          type="button"
          variant="outline"
          className="min-h-11 justify-center"
          onClick={() => void exportHistory()}
          disabled={isExporting || isImporting}
        >
          {isExporting ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Download className="mr-2 h-4 w-4" />
          )}
          {t("settings.chatHistory.export")}
        </Button>
        <Button
          type="button"
          className="min-h-11 justify-center"
          onClick={() => void importHistory()}
          disabled={isExporting || isImporting}
        >
          {isImporting ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : selectedFile ? (
            <ArchiveRestore className="mr-2 h-4 w-4" />
          ) : (
            <Upload className="mr-2 h-4 w-4" />
          )}
          {selectedFile
            ? t("settings.chatHistory.import")
            : t("settings.chatHistory.selectImport")}
        </Button>
      </div>

      {selectedFile ? (
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span className="min-w-0 flex-1 truncate font-mono">
            {selectedFileName}
          </span>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="h-7 w-7 shrink-0"
            onClick={() => setSelectedFile("")}
            title={t("common.clear")}
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      ) : null}

      <p className="text-xs leading-relaxed text-muted-foreground">
        {t("settings.chatHistory.securityNote")}
      </p>

      <ConfirmDialog
        isOpen={restartPromptOpen}
        title={t("settings.chatHistory.restartTitle")}
        message={t("settings.chatHistory.restartMessage")}
        confirmText={
          isRestarting
            ? t("common.loading")
            : t("settings.chatHistory.restartConfirm")
        }
        cancelText={t("settings.chatHistory.restartLater")}
        variant="info"
        onConfirm={() => void restartCodex()}
        onCancel={() => {
          if (!isRestarting) setRestartPromptOpen(false);
        }}
      />
    </section>
  );
}
