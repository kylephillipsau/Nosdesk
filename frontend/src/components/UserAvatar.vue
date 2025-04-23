<!-- # src/components/UserAvatar.vue -->
<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import userService from '@/services/userService'

// Create a shared store for users to prevent multiple fetches
// This will be shared across all instances of UserAvatar
const sharedUsers = {
  data: ref<any[]>([]),
  loading: ref(false),
  error: ref<string | null>(null),
  initialized: ref(false)
}

interface Props {
  name: string;
  showName?: boolean;
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'full';
  avatar?: string | null;
  clickable?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  showName: true,
  size: 'md',
  avatar: null,
  clickable: true
})

const router = useRouter()
// Use the shared store instead of local refs
const users = sharedUsers.data
const loading = sharedUsers.loading
const error = sharedUsers.error

// Check if a string is a UUID
const isUuid = (str: string) => {
  const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
  return uuidPattern.test(str)
}

// Fetch users from the API - Define this function before using it in watchers or lifecycle hooks
const fetchUsers = async () => {
  // If we already have users or are currently loading, don't fetch again
  if (sharedUsers.data.value.length > 0 || sharedUsers.loading.value) {
    sharedUsers.initialized.value = true
    return
  }
  
  sharedUsers.loading.value = true
  sharedUsers.error.value = null
  
  try {
    sharedUsers.data.value = await userService.getUsers()
    sharedUsers.initialized.value = true
  } catch (err) {
    console.error('Error fetching users in UserAvatar:', err)
    sharedUsers.error.value = 'Failed to load user data'
  } finally {
    sharedUsers.loading.value = false
  }
}

// Fetch users on component mount
onMounted(async () => {
  if (isUuid(props.name)) {
    await fetchUsers()
  }
})

// Watch for changes to props.name
watch(() => props.name, async (newName) => {
  if (isUuid(newName) && !sharedUsers.initialized.value) {
    await fetchUsers()
  }
}, { immediate: true })

// Check if the name is a UUID and get the actual name if it is
const displayName = computed(() => {
  // console.log(`UserAvatar: Processing name prop: "${props.name}" (type: ${typeof props.name})`);
  
  // Check if the name looks like a UUID
  if (isUuid(props.name)) {
    if (loading.value) return 'Loading...';
    const user = users.value.find(u => u.uuid === props.name);
    // console.log(`UserAvatar: UUID match found:`, user || 'No match');
    return user ? user.name : props.name;
  }
  return props.name;
})

const getInitials = (name: string) => {
  // console.log(`UserAvatar.getInitials: Input name: "${name}" (type: ${typeof name})`);
  
  if (!name) {
    // console.log('UserAvatar.getInitials: Empty name, returning "?"');
    return '?';
  }
  
  // If we're loading a UUID, just return a placeholder
  if (loading.value && isUuid(props.name)) {
    // console.log('UserAvatar.getInitials: Loading UUID, returning empty string');
    return '';
  }
  
  // Split by space or hyphen to better handle formats like "User-123"
  const parts = name.split(/[\s-]+/);
  // console.log(`UserAvatar.getInitials: Name parts:`, parts);
  
  const initials = parts
    .filter(part => part.length > 0)
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2);
    
  // console.log(`UserAvatar.getInitials: Calculated initials: "${initials}"`);
  return initials;
}

const getBackgroundColor = (name: string) => {
  // Return transparent background when loading a UUID
  if (loading.value && isUuid(props.name)) return 'transparent';
  
  if (!name) return 'hsl(0, 70%, 35%)';
  
  // Get the first letter and convert to uppercase
  const firstLetter = name.charAt(0).toUpperCase();
  
  // Get position in alphabet (A = 0, B = 1, etc.)
  const position = firstLetter.charCodeAt(0) - 65;
  
  // Convert position to a value between 0 and 360 (hue range)
  // We'll mod by 26 (alphabet length) first to handle non-letters
  const hue = (position % 26) * (360 / 26);
  
  // Return HSL color with fixed saturation and lightness
  return `hsl(${hue}, 70%, 35%)`;
}

const sizeClasses = computed(() => {
  // Base sizes in rem units
  const sizes = {
    xs: {
      base: 'h-[1.25rem] w-[1.25rem]', // 20px at default font size
      text: 'text-[10px]',
      responsive: 'sm:h-[1.375rem] sm:w-[1.375rem]' // 22px on small screens
    },
    sm: {
      base: 'h-[1.5rem] w-[1.5rem]', // 24px at default font size
      text: 'text-xs',
      responsive: 'sm:h-[1.75rem] sm:w-[1.75rem]' // Slightly larger on small screens
    },
    md: {
      base: 'h-[2rem] w-[2rem]', // 32px at default font size
      text: 'text-sm',
      responsive: 'sm:h-[2.25rem] sm:w-[2.25rem]' // Slightly larger on small screens
    },
    lg: {
      base: 'h-[2.25rem] w-[2.25rem]', // 36px at default font size
      text: 'text-sm',
      responsive: 'sm:h-[2.5rem] sm:w-[2.5rem]' // 40px on small screens
    },
    full: {
      base: 'w-full h-full min-w-full min-h-full', // Full size with minimum dimensions
      text: 'text-4xl sm:text-5xl',
      responsive: '' // No responsive changes needed
    }
  }

  return sizes[props.size] || sizes.md
})

// Computed class for the name text that scales with viewport
const nameTextClasses = computed(() => {
  const baseClasses = 'text-white transition-all duration-200'
  
  switch (props.size) {
    case 'xs':
      return `${baseClasses} text-[10px] sm:text-xs`
    case 'sm':
      return `${baseClasses} text-xs sm:text-sm`
    case 'lg':
      return `${baseClasses} text-sm` // Keeping text size consistent for header
    case 'full':
      return `${baseClasses} text-xl` // Larger text for full size
    default: // md
      return `${baseClasses} text-xs sm:text-sm`
  }
})

const navigateToProfile = () => {
  if (props.clickable) {
    // If the name is a UUID, use it directly for navigation
    // Otherwise, we can't navigate to a profile (profiles are accessed by UUID)
    if (isUuid(props.name)) {
      router.push(`/users/${props.name}`)
    } else {
      console.warn('Cannot navigate to profile: name is not a UUID', props.name)
    }
  }
}
</script>

<template>
  <div 
    class="flex items-center gap-2"
    :class="[
      { 'cursor-pointer hover:opacity-80': clickable },
      size === 'full' ? 'h-full aspect-square' : ''
    ]"
    @click="navigateToProfile"
  >
    <div 
      :class="[
        sizeClasses.base, 
        sizeClasses.responsive,
        { 'animate-pulse bg-transparent': loading && isUuid(props.name) }
      ]"
      class="rounded-full flex items-center justify-center flex-shrink-0 font-medium text-white transition-all duration-300"
      :style="{ backgroundColor: getBackgroundColor(displayName) }"
    >
      <img 
        v-if="avatar" 
        :src="avatar" 
        :alt="displayName"
        class="w-full h-full rounded-full object-cover transition-opacity duration-300"
        :class="{ 'opacity-0': loading && isUuid(props.name) }"
      />
      <span v-else-if="!loading || !isUuid(props.name)" :class="sizeClasses.text">{{ getInitials(displayName) }}</span>
      <span v-else class="opacity-0">...</span>
    </div>
    <span 
      v-if="showName" 
      :class="[
        nameTextClasses,
        { 'animate-pulse text-slate-400': loading && isUuid(props.name) }
      ]"
    >
      {{ displayName }}
    </span>
  </div>
</template>

<style scoped>
@keyframes pulse {
  0%, 100% {
    opacity: 0.4;
  }
  50% {
    opacity: 0.1;
  }
}

.animate-pulse {
  animation: pulse 1.5s ease-in-out infinite;
}
</style>