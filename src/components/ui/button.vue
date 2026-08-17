<script setup lang="ts">
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/cn";

withDefaults(
  defineProps<{
    variant?: VariantProps<typeof btn>["variant"];
    size?: VariantProps<typeof btn>["size"];
    disabled?: boolean;
    type?: "button" | "submit";
  }>(),
  { variant: "primary", size: "md", type: "button" },
);

const btn = cva(
  "inline-flex items-center justify-center gap-2 rounded-xl font-medium transition-all focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-brand-500 disabled:pointer-events-none disabled:opacity-40 cursor-pointer select-none",
  {
    variants: {
      variant: {
        primary:
          "bg-brand-600 text-white shadow-sm shadow-brand-600/20 hover:bg-brand-700 active:scale-[0.98]",
        secondary:
          "bg-white text-slate-700 border border-slate-200 hover:bg-slate-50 active:scale-[0.98] dark:bg-slate-900 dark:text-slate-200 dark:border-slate-700 dark:hover:bg-slate-800",
        outline:
          "border border-brand-300 text-brand-700 hover:bg-brand-50 dark:border-brand-700 dark:text-brand-300 dark:hover:bg-brand-950",
        ghost:
          "text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800",
        danger: "bg-rose-600 text-white hover:bg-rose-700 active:scale-[0.98]",
      },
      size: {
        sm: "text-xs px-3 py-1.5",
        md: "text-sm px-4 py-2",
        lg: "text-base px-6 py-3",
      },
    },
    defaultVariants: { variant: "primary", size: "md" },
  },
);
</script>

<template>
  <button
    :type="type"
    :disabled="disabled"
    :class="cn(btn({ variant, size }), $attrs.class as string | undefined)"
  >
    <slot />
  </button>
</template>
