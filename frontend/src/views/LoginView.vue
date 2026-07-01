<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Loader2 } from 'lucide-vue-next'
import Button from '@/components/ui/Button.vue'
import Input from '@/components/ui/Input.vue'
import Label from '@/components/ui/Label.vue'
import { api } from '@/lib/api'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function onSubmit() {
  error.value = ''
  loading.value = true
  try {
    const res = await api.login(username.value.trim(), password.value.trim())
    if (!res.success || !res.token) {
      error.value = res.message ?? '登录失败'
      return
    }
    auth.saveToken(res.token)
    await router.push({ name: 'dashboard' })
  } catch (e) {
    error.value = e instanceof Error ? e.message : '网络错误'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-dvh flex items-center justify-center bg-background px-4">
    <div class="w-full max-w-sm rounded-lg border border-border bg-card p-6 shadow-sm">
      <h1 class="text-lg font-medium text-center mb-1">Realm 端口转发</h1>
      <p class="text-sm text-muted-foreground text-center mb-6">端口转发管理</p>
      <form class="space-y-4" @submit.prevent="onSubmit">
        <div class="space-y-2">
          <Label for="username">用户名</Label>
          <Input id="username" v-model="username" autocomplete="username" placeholder="用户名" required />
        </div>
        <div class="space-y-2">
          <Label for="password">密码</Label>
          <Input id="password" v-model="password" type="password" autocomplete="current-password" placeholder="密码" required />
        </div>
        <p v-if="error" class="text-sm text-destructive">{{ error }}</p>
        <Button type="submit" class="w-full" :disabled="loading">
          <Loader2 v-if="loading" class="h-4 w-4 animate-spin" />
          {{ loading ? '登录中…' : '登录' }}
        </Button>
      </form>
    </div>
  </div>
</template>
