# Microsoft Graph Integration Roadmap

## Current Status

The Microsoft Graph integration provides synchronization of organizational data from Microsoft Entra ID (Azure AD) and Microsoft Intune into Nosdesk.

### Implemented Features

- **User Sync**: Import user accounts and profiles from Microsoft Entra ID
  - User display name, email, job title, department
  - Profile photos (synced to local storage)
  - Manager relationships
  - Account status (enabled/disabled)

- **Device Sync**: Import managed devices from Microsoft Intune
  - Device name, serial number, model
  - OS type and version
  - Compliance status
  - User assignment (primary user)
  - Last sync time and enrollment date

- **Connection Management**
  - OAuth2 client credentials flow authentication
  - Configuration via environment variables
  - Connection status monitoring
  - Sync progress tracking with real-time updates

---

## Planned Features

### Phase 1: Group Management

**Status:** Not implemented (placeholder in UI)

**Scope:**
- Sync security groups and distribution groups from Microsoft Entra ID
- Store group membership relationships
- Display group memberships on user profiles
- Filter users by group membership

**Database Changes Required:**
```sql
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_id VARCHAR(255) NOT NULL UNIQUE,  -- Microsoft Graph group ID
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    group_type VARCHAR(50),  -- 'security', 'distribution', 'microsoft365'
    mail VARCHAR(255),
    mail_enabled BOOLEAN DEFAULT false,
    security_enabled BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    synced_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID REFERENCES groups(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(group_id, user_id)
);
```

**API Endpoints:**
- `GET /api/groups` - List all groups
- `GET /api/groups/:id` - Get group details with members
- `GET /api/users/:id/groups` - Get groups for a user

**Implementation Notes:**
- Use Microsoft Graph `/groups` endpoint with `$expand=members`
- Handle nested groups (groups containing other groups)
- Consider pagination for large organizations

---

### Phase 2: Organizational Hierarchy

**Status:** Planned

**Scope:**
- Sync organizational units/departments
- Manager-report relationships (already partially implemented)
- Org chart visualization

---

### Phase 3: License & Subscription Sync

**Status:** Planned

**Scope:**
- Sync Microsoft 365 license assignments
- Track license usage per user
- Alert on license availability

---

### Phase 4: Conditional Access & Security

**Status:** Planned

**Scope:**
- Sync conditional access policies
- Sign-in risk information
- MFA status per user

---

## Technical Debt

1. **Sync Conflict Resolution**: Need strategy for handling conflicts between local edits and Microsoft Graph data
2. **Delta Sync**: Implement delta queries for incremental sync instead of full sync
3. **Rate Limiting**: Add proper handling for Microsoft Graph API rate limits
4. **Retry Logic**: Implement exponential backoff for transient failures

---

## Environment Variables

Current configuration:
```env
MICROSOFT_CLIENT_ID=your-app-client-id
MICROSOFT_CLIENT_SECRET=your-app-client-secret
MICROSOFT_TENANT_ID=your-tenant-id
MICROSOFT_REDIRECT_URI=http://localhost:8080/api/auth/microsoft/callback
```

---

## Related Documentation

- [Microsoft Graph API Reference](https://learn.microsoft.com/en-us/graph/api/overview)
- [Microsoft Entra ID Groups](https://learn.microsoft.com/en-us/graph/api/resources/group)
- [Microsoft Intune Devices](https://learn.microsoft.com/en-us/graph/api/resources/intune-devices-manageddevice)
