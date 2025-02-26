<script setup lang="ts">
import { useRouter, useRoute } from "vue-router";
import { computed, ref, onMounted, onUnmounted, nextTick } from "vue";
import UserAvatar from "./UserAvatar.vue";
import { usePageTitle } from "@/composables/usePageTitle";
import AnimatedTitle from "./AnimatedTitle.vue";
import HeaderTitle from "./HeaderTitle.vue";
import DocumentIconSelector from "./DocumentIconSelector.vue";
import TicketIdentifier from "./TicketIdentifier.vue";
import PageUrlDisplay from "./PageUrlDisplay.vue";

const router = useRouter();
const route = useRoute();

interface Props {
  title?: string;
  showCreateButton?: boolean;
  useRouteTitle?: boolean;
  ticket: { id: number; title: string } | null;
  document: { id: string; title: string; icon: string } | null;
  isTransitioning?: boolean;
  pageUrl?: string;
}

const props = withDefaults(defineProps<Props>(), {
  useRouteTitle: false,
  ticket: null,
  document: null,
  isTransitioning: false,
  pageUrl: undefined,
});

const emit = defineEmits(["updateTicketTitle", "updateDocumentTitle", "updateDocumentIcon"]);

const { pageTitle, setCustomTitle } = usePageTitle();

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
  pageTitle: pageTitle.value,
  useRouteTitle: props.useRouteTitle,
  title: props.title,
});

// Use the provided title if available
const displayTitle = computed(() => {
  if (props.title) {
    console.log("Using provided title:", props.title);
    return props.title;
  }
  console.log("Using pageTitle:", pageTitle.value);
  return pageTitle.value;
});

const handleUpdateTitle = (newTitle: string) => {
  if (props.ticket) {
    setCustomTitle(`#${props.ticket.id} ${newTitle}`);
    emit("updateTicketTitle", newTitle);
  }
};

const handleUpdateDocumentTitle = (newTitle: string) => {
  if (props.document) {
    setCustomTitle(newTitle);
    emit("updateDocumentTitle", newTitle);
  }
};

const handleUpdateDocumentIcon = (newIcon: string) => {
  if (props.document) {
    emit("updateDocumentIcon", newIcon);
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
  name: "Kyle Phillips",
  email: "kyle@example.com",
  avatar: null,
};

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

const navigateToCreateTicket = () => {
  router.push("/tickets/create");
};

const navigateToSettings = () => {
  showUserMenu.value = false;
  router.push("/settings");
};

const handleLogout = () => {
  // API endpoint needed: POST /api/auth/logout
};

// Event listeners for click outside
onMounted(() => {
  document.addEventListener("mousedown", handleClickOutside);
  document.addEventListener("keydown", handleKeydown);
});

onUnmounted(() => {
  document.removeEventListener("mousedown", handleClickOutside);
  document.removeEventListener("keydown", handleKeydown);
});

// Add a computed property to determine if we should show the page URL
const showPageUrl = computed(() => {
  return !!props.pageUrl && !isTicketView.value && !isDocumentView.value;
});
</script>

<template>
  <header class="bg-slate-800 border-b border-slate-700">
    <div class="flex items-center justify-between h-16 px-6 gap-2">
      <!-- Left side - Title area -->
      <div class="flex items-center flex-1 relative overflow-hidden">
        <div
          class="w-full transition-all duration-300 ease-in-out"
          :class="{ 'opacity-0 -translate-y-4': isTransitioning }"
        >
          <template v-if="isTicketView && props.ticket">
            <div class="flex items-center gap-2">
              <TicketIdentifier :ticketId="props.ticket.id" size="lg" />
              <HeaderTitle
                :initial-title="props.ticket.title"
                :placeholder-text="'Enter ticket title...'"
                @update-title="handleUpdateTitle"
              />
            </div>
          </template>
          <template v-else-if="isDocumentView && props.document">
            <div class="flex items-center gap-2">
              <DocumentIconSelector
                :initial-icon="props.document.icon"
                @update:icon="handleUpdateDocumentIcon"
              />
              <HeaderTitle
                :initial-title="props.document.title"
                :placeholder-text="'Enter document title...'"
                @update-title="handleUpdateDocumentTitle"
              />
            </div>
          </template>
          <template v-else>
            <div class="flex items-center gap-2">
              <AnimatedTitle :title="displayTitle" />
              <PageUrlDisplay v-if="showPageUrl" :url="props.pageUrl" size="sm" class="ml-2" />
            </div>
          </template>
        </div>
      </div>

      <!-- Debug info (hidden in production) -->
      <div
        v-if="false"
        class="fixed top-0 right-0 bg-black text-white p-2 text-xs z-50"
      >
        isTicketView: {{ isTicketView }}<br />
        isDocumentView: {{ isDocumentView }}<br />
        pageTitle: {{ pageTitle }}<br />
        useRouteTitle: {{ props.useRouteTitle }}<br />
        title: {{ props.title }}<br />
        displayTitle: {{ displayTitle }}<br />
        ticket:
        {{
          props.ticket?.id
            ? `#${props.ticket?.id} ${props.ticket?.title}`
            : "null"
        }}<br />
        document:
        {{
          props.document?.id
            ? `${props.document?.id} ${props.document?.title}`
            : "null"
        }}
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
              @click="router.push(`/users/${encodeURIComponent(user.name)}`)"
            >
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
