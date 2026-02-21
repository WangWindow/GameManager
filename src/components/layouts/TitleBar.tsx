import { Icon } from "@iconify/react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from "@/components/ui/dropdown-menu";
import { useWindowControls } from "@/hooks/useWindowControls";

interface TitleBarProps {
  onManage?: () => void;
  onImport?: () => void;
  onScan?: () => void;
  onSettings?: () => void;
}

export default function TitleBar({ onManage, onImport, onScan, onSettings }: TitleBarProps) {
  const { isTauri, isMaximized, minimize, toggleMaximize, close } = useWindowControls();

  return (
    <header
      data-tauri-drag-region
      className="fixed left-0 right-0 top-0 z-50 flex h-10 select-none items-center justify-between border-b bg-background/80 px-3 backdrop-blur supports-backdrop-filter:bg-background/60"
    >
      <div data-tauri-drag-region className="flex min-w-0 items-center gap-2">
        <div className="flex h-7 w-7 items-center justify-center rounded-md bg-primary text-primary-foreground">
          <Icon icon="ri:gamepad-fill" className="h-5 w-5" />
        </div>
        <span data-tauri-drag-region className="text-sm font-semibold tracking-tight">
          GameManager
        </span>
      </div>

      <div data-tauri-drag-region="false" className="flex items-center gap-2">
        <Button variant="ghost" size="sm" className="h-8 gap-2 px-2" title="导入" onClick={onImport}>
          <Icon icon="ri:add-line" className="h-4 w-4" />
          <span className="text-xs">导入</span>
        </Button>
        <Button variant="ghost" size="sm" className="h-8 gap-2 px-2" title="扫描" onClick={onScan}>
          <Icon icon="ri:scan-line" className="h-4 w-4" />
          <span className="text-xs">扫描</span>
        </Button>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="icon" className="h-8 w-8" title="更多">
              <Icon icon="ri:more-2-line" className="h-4 w-4" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onSelect={onManage}>管理中心</DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={onSettings}>设置</DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        {isTauri && (
          <>
            <Button variant="ghost" size="icon" className="h-8 w-9" title="最小化" onClick={minimize}>
              <Icon icon="ri:subtract-line" className="h-4 w-4" />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-9" title="最大化/还原" onClick={toggleMaximize}>
              <Icon
                icon={
                  isMaximized ? "ri:checkbox-multiple-blank-line" : "ri:checkbox-blank-line"
                }
                className="h-4 w-4"
              />
            </Button>
            <Button variant="ghost" size="icon" className="h-8 w-9" title="关闭" onClick={close}>
              <Icon icon="ri:close-line" className="h-4 w-4" />
            </Button>
          </>
        )}
      </div>
    </header>
  );
}
