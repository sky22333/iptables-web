<script setup lang="ts">
import { type HTMLAttributes, useAttrs } from 'vue'
import { textareaClass } from '@/lib/form-control'
import { cn } from '@/lib/utils'

defineOptions({ inheritAttrs: false })

defineProps<{ class?: HTMLAttributes['class'] }>()
const model = defineModel<string>()
const attrs = useAttrs()
</script>

<template>
  <textarea v-bind="attrs" v-model="model" :class="cn(textareaClass, $props.class)" />
</template>
