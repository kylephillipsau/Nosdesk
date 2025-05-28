<!-- # src/components/UserAvatar.vue -->
<script setup lang="ts">
import { computed, ref, onMounted, watch, defineExpose } from 'vue'
import { useRouter } from 'vue-router'
import userService from '@/services/userService'
import LazyImage from '@/components/LazyImage.vue'
import { useUserLookup } from '@/services/userLookupService'

// Method to refresh a specific user's data
const refreshUser = async (userUuid: string) => {
  try {
    const userData = await userService.getUserByUuid(userUuid);
    if (userData) {
      // Add to lookup cache
      addUsersToCache([userData]);
    }
  } catch (err) {
    console.error('Error refreshing user data:', err);
  }
}

interface Props {
  name: string;
  userName?: string; // Optional pre-loaded user name
  showName?: boolean;
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl' | 'full';
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

// User lookup service for efficient lookups
const { getUserName, getUserAvatar, lookupUser, addUsersToCache } = useUserLookup()

// Simple ref for the element
const elementRef = ref<HTMLElement | null>(null)

// Check if a string is a UUID
const isUuid = (str: string) => {
  const uuidPattern = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i
  return uuidPattern.test(str)
}

// Fetch user data only if not available in cache and no userName prop provided
onMounted(async () => {
  if (isUuid(props.name) && !props.userName) {
    // Check if user is in cache first
    const cachedName = getUserName(props.name)
    if (!cachedName) {
      // Try to fetch individual user instead of all users
      await lookupUser(props.name)
    }
  }
})

// Watch for changes to props.name
watch(() => props.name, async (newName) => {
  if (isUuid(newName) && !props.userName) {
    const cachedName = getUserName(newName)
    if (!cachedName) {
      await lookupUser(newName)
    }
  }
}, { immediate: true })

// Check if the name is a UUID and get the actual name if it is
const displayName = computed(() => {
  // If we have a pre-loaded user name, use it immediately
  if (props.userName) {
    return props.userName;
  }
  
  // Check if the name looks like a UUID
  if (isUuid(props.name)) {
    // Try to get name from lookup service cache
    const cachedName = getUserName(props.name);
    if (cachedName) {
      return cachedName;
    }
    
    // Fallback to UUID if no cached name available
    return props.name;
  }
  
  return props.name;
})

// Get the avatar URL either from props or from the user data if loaded by UUID
const effectiveAvatarUrl = computed(() => {
  // If an avatar was directly provided as a prop, use it
  if (props.avatar) {
    return props.avatar;
  }
  
  // If name is a UUID, try to get avatar from lookup service
  if (isUuid(props.name)) {
    const preferThumb = props.size !== 'full';
    return getUserAvatar(props.name, preferThumb);
  }

  return null;
})

const getInitials = (name: string) => {
  if (!name) {
    return '?';
  }
  
  // Split by space or hyphen to better handle formats like "User-123"
  const parts = name.split(/[\s-]+/);
  
  const initials = parts
    .filter(part => part.length > 0)
    .map(word => word.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2);
    
  return initials;
}

const getBackgroundColor = (name: string) => {
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
    xl: {
      base: 'h-[3.75rem] w-[3.75rem]', // 60px at default font size
      text: 'text-sm',
      responsive: 'sm:h-[3.5rem] sm:w-[3.5rem]' // 56px on small screens
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

// Track image loading state
const imageLoaded = ref(false)

// Reset image loaded state when avatar URL changes
watch(effectiveAvatarUrl, () => {
  imageLoaded.value = false
})

// Export the refreshUser method for external components to use
defineExpose({
  refreshUser
})
</script>

<template>
  <div 
    ref="elementRef"
    class="flex items-center"
    :class="[
      { 'cursor-pointer hover:opacity-80': clickable },
      size === 'full' ? 'h-full aspect-square' : '',
      showName ? 'gap-2' : ''
    ]"
    @click="navigateToProfile"
  >
    <!-- Avatar container -->
    <div 
      :class="[
        sizeClasses.base, 
        sizeClasses.responsive
      ]"
      class="rounded-full flex items-center justify-center flex-shrink-0 font-medium text-white relative overflow-hidden"
      :style="{ backgroundColor: getBackgroundColor(displayName) }"
    >
      <!-- Avatar image with lazy loading -->
      <LazyImage
        v-if="effectiveAvatarUrl"
        :src="effectiveAvatarUrl"
        :alt="displayName"
        class="w-full h-full rounded-full"
        loading="lazy"
        @load="imageLoaded = true"
        @error="imageLoaded = false"
      />
      
      <!-- Fallback initials (hidden when image loads) -->
      <span 
        v-show="!imageLoaded"
        :class="sizeClasses.text"
        class="absolute inset-0 flex items-center justify-center"
      >
        {{ getInitials(displayName) }}
      </span>
    </div>
    
    <!-- Text container with consistent spacing to prevent layout shift -->
    <span 
      v-if="showName" 
      :class="nameTextClasses"
      class="transition-all duration-500 ease-out truncate"
    >
      {{ displayName }}
    </span>
  </div>
</template>

<style scoped>
/* Smooth transitions */
img {
  transition: opacity 0.3s ease-out;
}

/* Smooth background color transitions */
div {
  transition: background-color 0.3s ease-out, opacity 0.3s ease-out;
}

/* Ensure text doesn't cause layout shifts */
.truncate {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>