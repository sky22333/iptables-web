<script setup lang="ts">
import { cn } from '@/lib/utils'

defineProps<{
  modelValue: string
  options: { value: string; label: string }[]
}>()

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
</script>

<template>
  <div class="inline-flex rounded-md border border-border bg-muted p-0.5">
    <button
      v-for="opt in options"
      :key="opt.value"
      type="button"
      :class="
        cn(
          'px-3 py-1.5 text-xs font-medium rounded-[4px] transition-colors',
          modelValue === opt.value
            ? 'bg-card text-foreground border border-border'
            : 'text-muted-foreground hover:text-foreground',
        )
      "
      @click="emit('update:modelValue', opt.value)"
    >
      {{ opt.label }}
    </button>
  </div>
</template>
