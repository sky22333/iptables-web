<script setup lang="ts">
import { type HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

defineProps<{ class?: HTMLAttributes['class'] }>()
</script>

<template>
  <label :class="cn('text-sm font-medium leading-none', $props.class)">
    <slot />
  </label>
</template>
