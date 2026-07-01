<script setup lang="ts">
import { type HTMLAttributes, useAttrs } from 'vue'
import { cn } from '@/lib/utils'

defineProps<{ class?: HTMLAttributes['class'] }>()
const attrs = useAttrs()
</script>

<template>
  <input
    v-bind="attrs"
    :class="
      cn(
        'flex h-8 w-full rounded-md border border-input bg-card px-3 text-sm placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/30 disabled:opacity-50',
        $props.class,
      )
    "
  />
</template>
