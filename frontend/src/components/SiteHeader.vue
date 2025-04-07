<script setup lang="ts">
import { useRouter, useRoute } from "vue-router";
import { computed, ref, onMounted, onUnmounted, nextTick } from "vue";
import UserAvatar from "./UserAvatar.vue";
import HeaderTitle from "./HeaderTitle.vue";
import DocumentIconSelector from "./DocumentIconSelector.vue";
import TicketIdentifier from "./TicketIdentifier.vue";
import PageUrlDisplay from "./PageUrlDisplay.vue";
import ticketService from '@/services/ticketService';
import { useAuthStore } from '@/stores/auth';
import { RouterLink } from "vue-router";

const router = useRouter();
const authStore = useAuthStore();

interface Props {
  title?: string;
  showCreateButton?: boolean;
  useRouteTitle?: boolean;
  ticket: { id: number; title: string } | null;
  document: { id: string; title: string; icon: string } | null;
  isTransitioning?: boolean;
  pageUrl?: string;
  navbarCollapsed?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  useRouteTitle: false,
  ticket: null,
  document: null,
  isTransitioning: false,
  pageUrl: undefined,
  navbarCollapsed: false,
});

const emit = defineEmits(["updateTicketTitle", "updateDocumentTitle", "updateDocumentIcon", "previewTicketTitle", "previewDocumentTitle"]);

const isTicketView = computed(() => {
  return props.ticket !== null;
});

const isDocumentView = computed(() => {
  return props.document !== null;
});

// Add console log to debug title
console.log("SiteHeader rendering with:", {
  isTicketView: isTicketView.value,
  isDocumentView: isDocumentView.value,
  ticket: props.ticket,
  document: props.document,
  title: props.title,
});

// Use the provided title if available
const displayTitle = computed(() => {
  if (props.title) {
    return props.title;
  }
  return '';
});

const handleUpdateTitle = (newTitle: string) => {
  if (props.ticket) {
    emit("updateTicketTitle", newTitle);
  }
};

const handlePreviewTitle = (newTitle: string) => {
  if (props.ticket) {
    emit("previewTicketTitle", newTitle);
  }
};

const handleUpdateDocumentTitle = (newTitle: string) => {
  if (props.document) {
    console.log(`SiteHeader: Updating document title to "${newTitle}" for document:`, props.document);
    emit("updateDocumentTitle", newTitle);
  }
};

const handlePreviewDocumentTitle = (newTitle: string) => {
  if (props.document) {
    console.log(`SiteHeader: Previewing document title as "${newTitle}" for document:`, props.document);
    emit("previewDocumentTitle", newTitle);
  }
};

const handleUpdateDocumentIcon = (newIcon: string) => {
  if (props.document) {
    emit("updateDocumentIcon", newIcon);
  }
};

const showUserMenu = ref(false);
const menuRef = ref<HTMLElement | null>(null);
const buttonRef = ref<HTMLElement | null>(null);

// Replace mock user data with actual user data from auth store
const user = computed(() => {
  if (authStore.user) {
    return {
      name: authStore.user.name,
      email: authStore.user.email,
      avatar: null // Auth user doesn't have avatar property, so we set it to null
    };
  }
  return {
    name: "Guest",
    email: "guest@example.com",
    avatar: null
  };
});

const handleClickOutside = (event: MouseEvent) => {
  if (!menuRef.value || !buttonRef.value) return;

  const target = event.target as Node;
  if (!menuRef.value.contains(target) && !buttonRef.value.contains(target)) {
    showUserMenu.value = false;
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === "Escape") {
    showUserMenu.value = false;
    buttonRef.value?.focus();
  }
};

const toggleUserMenu = () => {
  showUserMenu.value = !showUserMenu.value;
  if (showUserMenu.value) {
    // Focus first interactive element in menu when opened
    nextTick(() => {
      const firstFocusable = menuRef.value?.querySelector(
        "a, button"
      ) as HTMLElement;
      firstFocusable?.focus();
    });
  }
};

// Add isCreatingTicket ref
const isCreatingTicket = ref(false);

const navigateToCreateTicket = async () => {
  console.log('navigateToCreateTicket called');
  if (isCreatingTicket.value) {
    console.log('Already creating ticket, returning');
    return; // Prevent multiple clicks
  }
  
  try {
    console.log('Setting isCreatingTicket to true');
    isCreatingTicket.value = true;
    
    console.log('Creating empty ticket');
    // Create an empty ticket
    const newTicket = await ticketService.createEmptyTicket();
    
    console.log('Empty ticket created:', newTicket);
    // Navigate to the new ticket
    router.push(`/tickets/${newTicket.id}`);
  } catch (error) {
    console.error('Failed to create empty ticket:', error);
    // You could show an error notification here
    
    console.log('Falling back to create ticket page');
    // Fallback to the tickets page if creation fails
    router.push("/tickets");
  } finally {
    console.log('Setting isCreatingTicket to false');
    isCreatingTicket.value = false;
  }
};

const navigateToSettings = () => {
  showUserMenu.value = false;
  router.push("/settings");
};

const handleLogout = () => {
  try {
    // Close the user menu
    showUserMenu.value = false;
    
    // Log the user out using the auth store
    // The auth store will handle the redirect to the login page
    authStore.logout();
  } catch (error) {
    console.error('Logout failed:', error);
    // You could show an error notification here
  }
};

// Check if we're on mobile
const isMobile = ref(window.innerWidth < 768); // md breakpoint

// Update mobile status on resize
const updateMobileStatus = () => {
  isMobile.value = window.innerWidth < 768;
};

// Event listeners for click outside
onMounted(() => {
  document.addEventListener("mousedown", handleClickOutside);
  document.addEventListener("keydown", handleKeydown);
  
  // Add resize listener
  window.addEventListener('resize', updateMobileStatus);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleClickOutside);
  document.removeEventListener("keydown", handleKeydown);
  
  // Remove resize listener
  window.removeEventListener('resize', updateMobileStatus);
});
</script>

<template>
  <header class="bg-slate-800 border-b border-slate-700 relative z-[999]">
    <div class="flex items-center justify-between h-16 px-4 md:px-6 gap-2">
      <!-- Left side - Title area -->
      <div class="flex items-center flex-1 relative overflow-hidden min-w-0">
        <div
          class="w-full transition-all duration-300 ease-in-out min-w-0"
          :class="{ 'opacity-0 -translate-y-4': isTransitioning }"
        >
          <template v-if="isTicketView && props.ticket">
            <div class="flex items-center gap-2 min-w-0">
              <TicketIdentifier :ticketId="props.ticket.id" size="lg" class="flex-shrink-0" />
              <HeaderTitle
                :initialTitle="props.ticket.title"
                :placeholder-text="'Enter ticket title...'"
                @update-title="handleUpdateTitle"
                @update-title-preview="handlePreviewTitle"
                class="min-w-0 truncate"
              />
            </div>
          </template>
          <template v-else-if="isDocumentView && props.document">
            <div class="flex items-center gap-2 min-w-0">
              <DocumentIconSelector
                :initial-icon="props.document.icon"
                @update:icon="handleUpdateDocumentIcon"
                class="flex-shrink-0"
              />
              <HeaderTitle
                :initialTitle="props.document.title"
                :placeholder-text="'Enter document title...'"
                @update-title="handleUpdateDocumentTitle"
                @update-title-preview="handlePreviewDocumentTitle"
                class="min-w-0 truncate"
              />
            </div>
          </template>
          <template v-else>
            <div class="flex items-center gap-2 min-w-0">
              <h1 class="text-xl font-semibold text-white truncate">{{ displayTitle }}</h1>
            </div>
          </template>
        </div>
      </div>

      <!-- Right side -->
      <div class="flex items-center gap-2 md:gap-4 flex-shrink-0">
        <!-- Create Ticket Button - Hide text on mobile -->
        <button
          v-if="props.showCreateButton"
          @click="navigateToCreateTicket"
          :disabled="isCreatingTicket"
          class="px-2 md:px-4 py-2 text-sm font-medium bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-1 md:gap-2"
          :aria-label="isCreatingTicket ? 'Creating new ticket...' : 'Create new ticket'"
        >
          <!-- Always show icon -->
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path v-if="isCreatingTicket" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" class="animate-spin" />
            <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
          
          <!-- Only show text on larger screens -->
          <span class="hidden md:inline">{{ isCreatingTicket ? 'Creating...' : 'Create Ticket' }}</span>
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
              size="md" 
              :clickable="false"
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
            <div
              class="px-4 py-2 border-b border-slate-700 hover:bg-slate-700 cursor-pointer"
              @click="authStore.user ? router.push(`/users/${authStore.user.uuid}`) : {}"
            >
              <div class="text-sm font-medium text-white">{{ user.name }}</div>
              <div class="text-xs text-slate-400">{{ user.email }}</div>
            </div>

            <!-- Menu Items -->
            <div class="py-1">
              <RouterLink
                to="/profile/settings"
                class="block px-4 py-2 text-sm text-slate-300 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
                role="menuitem"
                @click="showUserMenu = false"
              >
                Profile Settings
              </RouterLink>
              <button
                @click="navigateToSettings"
                class="w-full text-left px-4 py-2 text-sm text-slate-300 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
                role="menuitem"
              >
                Preferences
              </button>
              <router-link
                v-if="authStore.user?.role === 'admin'"
                to="/admin/settings"
                class="block w-full text-left px-4 py-2 text-sm text-slate-300 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
                role="menuitem"
              >
                <div class="flex items-center">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                  Administration
                </div>
              </router-link>
              
              <div v-if="authStore.user?.role === 'admin'" class="border-t border-slate-600 my-1"></div>
              
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
.dropdown-menu {
  position: fixed;
  /* Ensure it's positioned relative to the viewport */
  transform: translateZ(0);
  /* Force a new stacking context */
  will-change: transform;
  /* Hint to the browser to create a new layer */
}

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
