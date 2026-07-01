<script setup lang="ts">
import { useToast } from '@/composables/useToast'

const { toasts } = useToast()
</script>

<template>
  <div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2 w-[min(100vw-2rem,320px)]">
    <div
      v-for="t in toasts"
      :key="t.id"
      class="rounded-md border px-3 py-2 text-sm bg-card"
      :class="{
        'border-emerald-200 text-emerald-800': t.type === 'success',
        'border-red-200 text-red-700': t.type === 'error',
        'border-border text-foreground': t.type === 'info',
      }"
    >
      {{ t.message }}
    </div>
  </div>
</template>
