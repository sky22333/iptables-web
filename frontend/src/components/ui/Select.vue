<script setup lang="ts">
import { type HTMLAttributes, useAttrs } from 'vue'
import { selectClass } from '@/lib/form-control'
import { cn } from '@/lib/utils'

defineOptions({ inheritAttrs: false })

defineProps<{ class?: HTMLAttributes['class'] }>()
const model = defineModel<string>()
const attrs = useAttrs()
</script>

<template>
  <select v-bind="attrs" v-model="model" :class="cn(selectClass, $props.class)">
    <slot />
  </select>
</template>
