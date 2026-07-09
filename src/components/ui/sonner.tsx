import { Icon } from "@iconify/react"
import { Toaster as Sonner, type ToasterProps } from "sonner"

const Toaster = ({ theme = "system", ...props }: ToasterProps) => {
  return (
    <Sonner
      theme={theme}
      className="toaster group"
      icons={{
        success: <Icon icon="ri:checkbox-circle-line" className="size-4" />,
        info: <Icon icon="ri:information-line" className="size-4" />,
        warning: <Icon icon="ri:alert-line" className="size-4" />,
        error: <Icon icon="ri:close-circle-line" className="size-4" />,
        loading: <Icon icon="ri:loader-line" className="size-4 animate-spin" />,
      }}
      style={
        {
          "--normal-bg": "var(--popover)",
          "--normal-text": "var(--popover-foreground)",
          "--normal-border": "var(--border)",
          "--border-radius": "var(--radius)",
        } as React.CSSProperties
      }
      {...props}
    />
  )
}

export { Toaster }
