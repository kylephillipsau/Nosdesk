# Plugin System Architecture for Nosdesk

## Objective
Design an extensible plugin system that allows developers to build custom UI components and integrations with external services, while preserving core functionality stability and security.

---

## Design Philosophy: Extend, Don't Replace

Plugins **add capabilities alongside** core features - they cannot remove or replace them. This ensures:
- Consistent, predictable user experience
- Core workflows remain stable
- Large features stay in core (plugins are for integrations/extensions)

```
┌─────────────────────────────────────────────────────────────┐
│                     CORE (Protected)                        │
│  Tickets, Documents, Devices, Users, Collaboration          │
│  - Cannot be removed or replaced by plugins                 │
│  - Behavior is consistent and predictable                   │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   Plugin API      │
                    │   (Controlled)    │
                    └─────────┬─────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                   EXTENSIONS (Plugins)                       │
│  - Add new panels, tabs, actions via custom Vue components  │
│  - Integrate with external services (Dell, Jira, Slack)     │
│  - Display/sync external data                               │
│  - React to events (observe, not block)                     │
└─────────────────────────────────────────────────────────────┘
```

---

## Plugin Capabilities

### What Plugins CAN Do

#### 1. Build Custom Vue Components
Plugins can create full Vue components with access to Nosdesk's design system:

```vue
<!-- Example: DellSupportPanel.vue -->
<template>
  <PluginPanel title="Dell Support">
    <div v-if="dellCase">
      <StatusBadge :status="dellCase.status" />
      <p>Case: {{ dellCase.caseNumber }}</p>
      <p>ETA: {{ dellCase.estimatedResolution }}</p>
      <Button @click="syncStatus">Refresh Status</Button>
      <a :href="dellCase.url" target="_blank">View in Dell Portal</a>
    </div>
    <Button v-else @click="createCase">Create Dell Case</Button>
  </PluginPanel>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { PluginPanel, StatusBadge, Button } from '@nosdesk/plugin-components';

const props = defineProps(['ticket', 'device']);
const dellCase = ref(null);

onMounted(async () => {
  // Fetch existing case from plugin storage
  const mapping = await pluginAPI.storage.get('case-mappings') || {};
  if (mapping[props.ticket.id]) {
    dellCase.value = await fetchDellCase(mapping[props.ticket.id]);
  }
});

async function createCase() {
  const response = await pluginAPI.fetch('https://api.dell.com/cases', {
    method: 'POST',
    body: JSON.stringify({
      serviceTag: props.device.serialNumber,
      description: props.ticket.title
    })
  });
  dellCase.value = response;

  // Add comment to ticket
  await pluginAPI.tickets.addComment(props.ticket.id, {
    content: `Dell Case #${response.caseNumber} created`,
    metadata: { pluginId: 'dell-support', caseNumber: response.caseNumber }
  });
}
</script>
```

#### 2. Register Components in UI Slots
```typescript
// Plugin registers where components appear
pluginAPI.registerPanel('ticket-sidebar', {
  component: DellSupportPanel,
  context: ['ticket', 'device'],  // Data passed to component
  title: 'Dell Support'
});
```

#### 3. Read Core Data (with permissions)
```typescript
const ticket = await pluginAPI.tickets.get(ticketId);
const device = await pluginAPI.devices.get(ticket.deviceId);
const documents = await pluginAPI.documents.list({ linkedTo: ticketId });
```

#### 4. Extend Core Features (add, not modify)
```typescript
// Add comments
await pluginAPI.tickets.addComment(ticketId, { content: '...' });

// Add external links
await pluginAPI.tickets.addExternalLink(ticketId, {
  provider: 'dell-support',
  externalId: caseNumber,
  url: `https://dell.com/support/case/${caseNumber}`
});

// Add attachments
await pluginAPI.tickets.addAttachment(ticketId, file);
```

#### 5. React to Events (observe only)
```typescript
// Subscribe to events - cannot block or prevent them
pluginAPI.on('ticket:created', async (ticket) => {
  if (ticket.category === 'hardware') {
    await createExternalCase(ticket);
  }
});
```

#### 6. Make External API Calls (proxied through backend)
```typescript
// All external requests go through Nosdesk backend for security
const warranty = await pluginAPI.fetch('https://api.dell.com/warranty', {
  method: 'POST',
  body: JSON.stringify({ serviceTag: device.serialNumber })
});
```

#### 7. Store Plugin-Specific Data
```typescript
// Namespaced storage - plugins can't access each other's data
await pluginAPI.storage.set('dell-case-mapping', { [ticketId]: caseNumber });
const mapping = await pluginAPI.storage.get('dell-case-mapping');
```

### What Plugins CANNOT Do

| Blocked Action | Reason |
|----------------|--------|
| Remove/hide core UI elements | Ensures consistent UX |
| Intercept/block core operations | Prevents breaking workflows |
| Access raw database | Data integrity protection |
| Modify other plugins' data | Plugin isolation |
| Override authentication | Security |
| Access data beyond permissions | Privacy |
| Replace core components | Core stability |

---

## UI Slot System

Plugins register components into predefined slots:

```typescript
const PLUGIN_SLOTS = {
  // Global slots
  'navbar-items': { multiple: true },           // Add navigation items
  'settings-integrations': { multiple: true },  // Integration settings pages

  // Ticket context
  'ticket-header-actions': { multiple: true },  // Buttons in ticket header
  'ticket-sidebar': { multiple: true },         // Sidebar panels
  'ticket-tabs': { multiple: true },            // Additional tabs
  'ticket-footer-actions': { multiple: true },  // Footer action buttons

  // Document context
  'document-toolbar': { multiple: true },       // Toolbar actions
  'document-sidebar': { multiple: true },       // Sidebar panels

  // Device context
  'device-header-actions': { multiple: true },  // Device page actions
  'device-info-panels': { multiple: true },     // Info panels
};
```

---

## Plugin Manifest

```json
{
  "name": "dell-support",
  "displayName": "Dell Hardware Support",
  "version": "1.0.0",
  "description": "Integrate with Dell support for hardware tickets",
  "permissions": [
    "tickets:read",
    "tickets:comment",
    "tickets:link",
    "devices:read",
    "storage",
    "external:api.dell.com"
  ],
  "components": {
    "DellSupportPanel": {
      "slot": "ticket-sidebar",
      "entry": "./components/DellSupportPanel.vue",
      "context": ["ticket", "device"]
    },
    "DellSettings": {
      "slot": "settings-integrations",
      "entry": "./components/DellSettings.vue",
      "label": "Dell Integration",
      "icon": "server"
    }
  },
  "events": ["ticket:created", "ticket:updated"],
  "settings": [
    { "key": "api_key", "type": "secret", "label": "Dell API Key" },
    { "key": "auto_create_cases", "type": "boolean", "label": "Auto-create cases for hardware tickets" }
  ]
}
```

---

## Plugin API Reference

```typescript
interface PluginAPI {
  // === READ: Access core data ===
  tickets: {
    get(id: number): Promise<Ticket>;
    list(filters?: TicketFilters): Promise<Ticket[]>;
  };
  devices: {
    get(id: number): Promise<Device>;
    list(filters?: DeviceFilters): Promise<Device[]>;
  };
  documents: {
    get(id: number): Promise<Document>;
    list(filters?: DocumentFilters): Promise<Document[]>;
  };
  users: {
    getCurrent(): User;
    get(id: number): Promise<User>;
  };

  // === EXTEND: Add to core features ===
  tickets: {
    addComment(id: number, comment: PluginComment): Promise<void>;
    addExternalLink(id: number, link: ExternalLink): Promise<void>;
    addAttachment(id: number, file: File): Promise<void>;
  };

  // === INTEGRATE: External services ===
  fetch(url: string, options?: RequestInit): Promise<Response>;

  // === STORE: Plugin data ===
  storage: {
    get<T>(key: string): Promise<T | null>;
    set<T>(key: string, value: T): Promise<void>;
    delete(key: string): Promise<void>;
  };

  // === OBSERVE: React to events ===
  on(event: EventType, handler: EventHandler): () => void;

  // === UI: Register components ===
  registerPanel(slot: SlotName, config: PanelConfig): void;
  registerAction(slot: SlotName, config: ActionConfig): void;

  // === NOTIFY: User feedback ===
  notify(message: string, type: 'info' | 'success' | 'warning' | 'error'): void;

  // === CONTEXT: Current state ===
  context: {
    ticket: Ticket | null;    // Current ticket (if on ticket page)
    device: Device | null;    // Current device (if on device page)
    document: Document | null; // Current document (if on doc page)
  };

  // === SETTINGS: Plugin configuration ===
  settings: {
    get<T>(key: string): Promise<T>;
    getAll(): Promise<Record<string, any>>;
  };
}

type EventType =
  | 'ticket:created' | 'ticket:updated' | 'ticket:status_changed'
  | 'ticket:assigned' | 'ticket:comment_added'
  | 'document:created' | 'document:updated'
  | 'device:created' | 'device:updated';
```

---

## Plugin Component Library

Plugins have access to Nosdesk's design system components:

```typescript
// @nosdesk/plugin-components
export {
  // Layout
  PluginPanel,      // Container with title and styling
  PluginTab,        // Tab content wrapper

  // Data display
  StatusBadge,      // Status indicators
  DataTable,        // Tables with sorting/filtering
  KeyValue,         // Key-value pairs display

  // Inputs
  Button,
  Input,
  Select,
  Checkbox,
  TextArea,

  // Feedback
  Spinner,
  Alert,
  EmptyState,

  // Icons
  Icon,             // Access to icon library
};
```

---

## Security Model

### Permission Levels

| Permission | Grants |
|------------|--------|
| `tickets:read` | Read ticket data |
| `tickets:comment` | Add comments to tickets |
| `tickets:link` | Add external links to tickets |
| `devices:read` | Read device data |
| `documents:read` | Read document data |
| `storage` | Use plugin storage |
| `external:<domain>` | Make requests to specific domain |

### Trust Levels

| Level | Description | DOM Access | External Requests |
|-------|-------------|------------|-------------------|
| **Official** | Nosdesk-built | Full Vue integration | Direct |
| **Verified** | Reviewed third-party | Full Vue integration | Backend-proxied |
| **Community** | Unreviewed | Shadow DOM isolation | Backend-proxied + whitelist |

### Security Measures

1. **All external requests proxied** through backend (logging, rate limiting)
2. **Permissions declared** in manifest, approved by admin on install
3. **Plugin storage namespaced** - can't access other plugins' data
4. **Events are observe-only** - plugins can't block core operations
5. **Audit logging** of all plugin actions

---

## Implementation Phases

### Phase 1: Event System & Webhooks
Build the foundation for plugin reactivity:
- Backend event emitter service
- Webhook delivery system for external integrations
- Event types for all core entity changes

### Phase 2: Plugin Runtime & API
Build the frontend plugin infrastructure:
- Plugin loader and manifest parser
- Plugin API implementation
- UI slot system
- Plugin component library
- Backend proxy for external requests

### Phase 3: Plugin Management UI
Admin interface for plugins:
- Install/uninstall plugins
- Configure plugin settings
- View plugin activity/logs
- Manage permissions

### Phase 4: Developer Experience
Tools for plugin developers:
- Plugin SDK/CLI for scaffolding
- Documentation site
- Example plugins (Slack, Jira, Dell)
- Local development server

---

## Database Schema

```sql
-- Installed plugins
CREATE TABLE plugins (
    id UUID PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    manifest JSONB NOT NULL,
    enabled BOOLEAN DEFAULT true,
    installed_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Plugin settings (admin-configured)
CREATE TABLE plugin_settings (
    id UUID PRIMARY KEY,
    plugin_id UUID REFERENCES plugins(id) ON DELETE CASCADE,
    key VARCHAR(100) NOT NULL,
    value JSONB,
    UNIQUE(plugin_id, key)
);

-- Plugin storage (plugin-managed data)
CREATE TABLE plugin_storage (
    id UUID PRIMARY KEY,
    plugin_id UUID REFERENCES plugins(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    UNIQUE(plugin_id, key)
);

-- Webhooks (Phase 1)
CREATE TABLE webhooks (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    secret VARCHAR(255) NOT NULL,
    events TEXT[] NOT NULL,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL
);

-- Webhook delivery log
CREATE TABLE webhook_deliveries (
    id UUID PRIMARY KEY,
    webhook_id UUID REFERENCES webhooks(id) ON DELETE CASCADE,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    response_status INTEGER,
    response_body TEXT,
    delivered_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL
);

-- Plugin activity log (audit)
CREATE TABLE plugin_activity (
    id UUID PRIMARY KEY,
    plugin_id UUID REFERENCES plugins(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    details JSONB,
    user_id UUID REFERENCES users(id),
    created_at TIMESTAMP NOT NULL
);
```

---

## Example Use Case: Dell Hardware Support Plugin

**Scenario:** IT team wants to integrate Dell support for hardware tickets.

**Plugin provides:**
1. **Sidebar panel** on tickets showing Dell case status
2. **Auto-create cases** when hardware tickets are created
3. **Sync warranty info** from Dell API to device records
4. **Settings page** for API credentials and preferences

**User flow:**
1. Admin installs "dell-support" plugin
2. Admin configures Dell API credentials in Settings > Integrations > Dell
3. Tech opens a hardware ticket with associated device
4. Dell Support panel appears in sidebar showing warranty status
5. Tech clicks "Create Dell Case" - case created, comment added to ticket
6. Plugin periodically syncs case status updates

---

## Summary

This plugin architecture provides:

- **Full custom Vue components** for rich UI extensions
- **Controlled API access** to core data and features
- **External service integration** via proxied requests
- **Event reactivity** for automation
- **Security by design** - extend, don't replace

Plugins enhance Nosdesk without compromising core stability or security.
