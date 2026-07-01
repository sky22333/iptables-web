<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  variant?: 'success' | 'muted' | 'warning' | 'danger'
}>()

const classes = computed(() =>
  cn(
    'inline-flex items-center rounded px-1.5 py-0.5 text-xs border font-normal',
    props.variant === 'success' && 'border-emerald-200 bg-emerald-50 text-emerald-700',
    props.variant === 'warning' && 'border-amber-200 bg-amber-50 text-amber-700',
    props.variant === 'danger' && 'border-red-200 bg-red-50 text-red-700',
    (!props.variant || props.variant === 'muted') && 'border-border bg-muted text-muted-foreground',
  ),
)
</script>

<template>
  <span :class="classes"><slot /></span>
</template>
