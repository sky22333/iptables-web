<script setup lang="ts">
import { type HTMLAttributes, computed, useAttrs } from 'vue'
import { inputClass } from '@/lib/form-control'
import { cn } from '@/lib/utils'

defineOptions({ inheritAttrs: false })

defineProps<{ class?: HTMLAttributes['class'] }>()
const model = defineModel<string | number | null>()
const attrs = useAttrs()

const displayValue = computed(() => {
  if (model.value == null) return ''
  return String(model.value)
})

function onInput(e: Event) {
  model.value = (e.target as HTMLInputElement).value
}
</script>

<template>
  <input
    v-bind="attrs"
    :value="displayValue"
    :class="cn(inputClass, $props.class)"
    @input="onInput"
  />
</template>
