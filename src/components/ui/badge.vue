<script setup lang="ts">
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "../../lib/cn";

withDefaults(
  defineProps<{ variant?: VariantProps<typeof badge>["variant"]; class?: string }>(),
  { variant: "default" },
);

const badge = cva(
  "inline-flex items-center gap-1 rounded-full px-2.5 py-0.5 text-xs font-medium",
  {
    variants: {
      variant: {
        default: "bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300",
        brand: "bg-brand-50 text-brand-700 dark:bg-brand-950 dark:text-brand-300",
        success: "bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300",
        warning: "bg-amber-50 text-amber-700 dark:bg-amber-950 dark:text-amber-300",
        danger: "bg-rose-50 text-rose-700 dark:bg-rose-950 dark:text-rose-300",
      },
    },
    defaultVariants: { variant: "default" },
  },
);
</script>

<template>
  <span :class="cn(badge({ variant }), $props.class)">
    <slot />
  </span>
</template>
