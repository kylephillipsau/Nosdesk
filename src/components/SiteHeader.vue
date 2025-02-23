<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import { computed, ref, onMounted, onUnmounted, nextTick } from 'vue';
import UserAvatar from './UserAvatar.vue';
import { usePageTitle } from '@/composables/usePageTitle';
import AnimatedTitle from './AnimatedTitle.vue';
import TicketTitle from './ticketComponents/TicketTitle.vue';

const router = useRouter();
const route = useRoute();

interface Props {
  title?: string;
  showCreateButton?: boolean;
  useRouteTitle?: boolean;
  ticket: { id: number; title: string } | null;
  isTransitioning?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  useRouteTitle: false,
  ticket: null,
  isTransitioning: false
});

const emit = defineEmits(['updateTicketTitle']);

const { pageTitle, setCustomTitle } = usePageTitle();

const isTicketView = computed(() => {
  return props.ticket !== null;
});

const handleUpdateTitle = (newTitle: string) => {
  if (props.ticket) {
    setCustomTitle(`#${props.ticket.id} ${newTitle}`);
    emit('updateTicketTitle', newTitle);
  }
};

// Initialize title from route meta if needed
onMounted(() => {
  if (props.useRouteTitle && !props.title) {
    // Title will be handled by router navigation guard
    return;
  }
  if (props.title) {
    setCustomTitle(props.title);
  }
});

const showUserMenu = ref(false);
const menuRef = ref<HTMLElement | null>(null);
const buttonRef = ref<HTMLElement | null>(null);

// Mock user data - replace with actual user data later
const user = {
  name: 'Kyle Phillips',
  email: 'kyle@example.com',
  avatar: null
};

const handleClickOutside = (event: MouseEvent) => {
  if (!menuRef.value || !buttonRef.value) return;
  
  const target = event.target as Node;
  if (!menuRef.value.contains(target) && !buttonRef.value.contains(target)) {
    showUserMenu.value = false;
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    showUserMenu.value = false;
    buttonRef.value?.focus();
  }
};

const toggleUserMenu = () => {
  showUserMenu.value = !showUserMenu.value;
  if (showUserMenu.value) {
    // Focus first interactive element in menu when opened
    nextTick(() => {
      const firstFocusable = menuRef.value?.querySelector('a, button') as HTMLElement;
      firstFocusable?.focus();
    });
  }
};

const navigateToCreateTicket = () => {
  router.push('/tickets/create');
};

const navigateToSettings = () => {
  showUserMenu.value = false;
  router.push('/settings');
};

const handleLogout = () => {
  // API endpoint needed: POST /api/auth/logout
};

// Event listeners for click outside
onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside);
  document.addEventListener('keydown', handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside);
  document.removeEventListener('keydown', handleKeydown);
});
</script>

<template>
  <header class="bg-slate-800 border-b border-slate-700">
    <div class="flex items-center justify-between h-16 px-6">
      <!-- Left side - Title area -->
      <div class="flex items-center flex-1 relative overflow-hidden">
        <div 
          class="w-full transition-all duration-300 ease-in-out"
          :class="{ 'opacity-0 -translate-y-4': isTransitioning }"
        >
          <template v-if="isTicketView && props.ticket">
            <TicketTitle 
              :ticket-id="props.ticket.id"
              :initial-title="props.ticket.title"
              @update-title="handleUpdateTitle"
            />
          </template>
          <template v-else>
            <AnimatedTitle :title="pageTitle" />
          </template>
        </div>
      </div>

      <!-- Right side -->
      <div class="flex items-center gap-4">
        <!-- Create Ticket Button -->
        <button
          v-if="props.showCreateButton"
          @click="navigateToCreateTicket"
          class="px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          Create Ticket
        </button>

        <!-- User Profile Menu -->
        <div class="relative">
          <button
            ref="buttonRef"
            @click="toggleUserMenu"
            class="flex items-center justify-center hover:ring-2 hover:ring-blue-500 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-500"
            aria-haspopup="true"
            :aria-expanded="showUserMenu"
          >
            <UserAvatar
              :showName="false"
              :name="user.name"
              :avatar="user.avatar"
              size="lg"
            />
          </button>

          <!-- Dropdown Menu -->
          <div
            v-if="showUserMenu"
            ref="menuRef"
            class="absolute right-0 mt-2 w-48 bg-slate-800 border border-slate-700 rounded-lg shadow-lg py-1 z-50"
            role="menu"
            tabindex="-1"
          >
            <!-- User Info -->
            <div class="px-4 py-2 border-b border-slate-700">
              <div class="text-sm font-medium text-white">{{ user.name }}</div>
              <div class="text-xs text-slate-400">{{ user.email }}</div>
            </div>

            <!-- Menu Items -->
            <div class="py-1">
              <a
                href="#"
                class="block px-4 py-2 text-sm text-slate-300 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
                role="menuitem"
              >
                Profile Settings
              </a>
              <button
                @click="navigateToSettings"
                class="w-full text-left px-4 py-2 text-sm text-slate-300 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
                role="menuitem"
              >
                Preferences
              </button>
              <button
                @click="handleLogout"
                class="w-full text-left px-4 py-2 text-sm text-red-400 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
                role="menuitem"
              >
                Sign Out
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped>
.transition-all {
  transition-property: all;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
}

@media (prefers-reduced-motion: reduce) {
  .transition-all {
    transition-duration: 0.1s;
    transform: none !important;
  }
}
</style>