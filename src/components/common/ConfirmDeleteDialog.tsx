import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { useI18n } from "@/i18n";

interface ConfirmDeleteDialogProps {
  open: boolean;
  title?: string;
  onOpenChange?: (open: boolean) => void;
  onConfirm?: () => void;
}

export default function ConfirmDeleteDialog({
  open,
  title,
  onOpenChange,
  onConfirm,
}: ConfirmDeleteDialogProps) {
  const { t } = useI18n();
  const titleSegment = title ? `「${title}」` : "";

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{t("confirmDelete.title")}</DialogTitle>
          <DialogDescription>
            {t("confirmDelete.description", { title: titleSegment })}
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button variant="ghost" onClick={() => onOpenChange?.(false)}>
            {t("common.cancel")}
          </Button>
          <Button variant="destructive" onClick={onConfirm}>
            {t("common.delete")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
