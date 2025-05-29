<script setup lang="ts">
import UserAvatar from '@/components/UserAvatar.vue'

interface Props {
  userId: string
  userName: string
  email?: string
  avatarSize?: 'xs' | 'sm' | 'md' | 'lg'
  showAvatar?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  avatarSize: 'sm',
  showAvatar: true
})
</script>

<template>
  <div class="flex items-center gap-2 min-w-0">
    <UserAvatar
      v-if="showAvatar"
      :name="userId"
      :userName="userName"
      :size="avatarSize"
      :clickable="false"
      :show-name="false"
    />
    <div class="min-w-0">
      <div class="font-medium truncate text-slate-200">{{ userName }}</div>
      <div v-if="email" class="text-xs text-slate-400 truncate">{{ email }}</div>
    </div>
  </div>
</template> 