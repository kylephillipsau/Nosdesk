# Voice-to-Text Transcription

## Summary
Add automatic transcription to voice notes in comments/attachments using Whisper on the backend. Transcriptions are stored in the database and can be edited by users.

## Requirements
- Transcribe after recording completes
- Backend-first using Whisper (no browser Web Speech API)
- Store transcriptions in database
- Allow users to edit transcriptions

---

## Phase 1: Database Schema

**New migration**: `migrations/YYYY-MM-DD_add_transcription_to_attachments/up.sql`

```sql
ALTER TABLE attachments ADD COLUMN transcription TEXT;
ALTER TABLE attachments ADD COLUMN transcription_status VARCHAR(20) DEFAULT 'none';
ALTER TABLE attachments ADD COLUMN transcription_method VARCHAR(20);
ALTER TABLE attachments ADD COLUMN transcription_error TEXT;

-- Valid statuses: none, pending, processing, completed, failed
```

---

## Phase 2: Backend Changes

### Files to modify:
- `/backend/src/models.rs` - Add transcription fields to Attachment struct
- `/backend/src/schema.rs` - Auto-updated by Diesel after migration

### New files:
- `/backend/src/handlers/transcription.rs` - API endpoints
- `/backend/src/services/transcription.rs` - Whisper integration

### API Endpoints:
| Method | Endpoint | Purpose |
|--------|----------|---------|
| GET | `/attachments/{id}/transcription` | Get transcription |
| PUT | `/attachments/{id}/transcription` | Update/edit transcription |
| POST | `/attachments/{id}/transcription/start` | Trigger backend transcription |

### Whisper Integration:
- Use `whisper-rs` crate for local Whisper model
- Or call OpenAI Whisper API (configurable via env var)
- Run transcription async, update status when complete

---

## Phase 3: Frontend Changes

### New composable: `/frontend/src/composables/useSpeechTranscription.ts`

```typescript
export function useSpeechTranscription() {
  // State: text, status, error, method
  // Methods:
  //   - transcribe(attachmentId) - triggers backend transcription
  //   - loadTranscription(attachmentId) - fetches existing
  //   - saveTranscription(attachmentId, text) - saves edits
  //   - pollStatus(attachmentId) - polls until complete
}
```

### Modify: `/frontend/src/components/ticketComponents/AudioPreview.vue`
- Add transcription section below audio player
- Show status: processing spinner, completed text, or error
- Auto-trigger transcription on mount (after recording)
- Allow editing before submission

### Modify: `/frontend/src/components/ticketComponents/AudioPlayer.vue`
- Add optional `attachmentId` prop
- Load and display transcription if available
- Add edit button to modify transcription
- Show transcription as italic text below waveform

### Modify: `/frontend/src/components/ticketComponents/CommentsAndAttachments.vue`
- Pass `attachment.id` to AudioPlayer components

### Update types: `/frontend/src/types/comment.ts`
- Add transcription fields to Attachment interface

---

## Phase 4: UI Flow

### Recording Flow:
1. User records voice note via VoiceRecorder
2. AudioPreview shows with "Transcribing..." spinner
3. Backend processes with Whisper
4. Transcription appears, user can edit
5. User clicks "Add" to submit with transcription

### Playback Flow:
1. AudioPlayer loads with attachmentId
2. Fetches transcription from backend
3. Displays transcription below waveform
4. User can click "Edit" to modify

---

## Implementation Order

1. **Database migration** - Add transcription columns
2. **Backend models** - Update Attachment struct
3. **Backend handlers** - Create transcription endpoints
4. **Backend service** - Integrate Whisper
5. **Frontend composable** - useSpeechTranscription
6. **AudioPreview** - Add transcription during preview
7. **AudioPlayer** - Add transcription display/edit
8. **CommentsAndAttachments** - Wire up attachment IDs

---

## Critical Files

| File | Changes |
|------|---------|
| `backend/src/models.rs` | Add transcription fields to Attachment |
| `backend/src/handlers/transcription.rs` | New file: API endpoints |
| `backend/src/services/transcription.rs` | New file: Whisper integration |
| `frontend/src/composables/useSpeechTranscription.ts` | New file: transcription logic |
| `frontend/src/components/ticketComponents/AudioPreview.vue` | Add transcription UI |
| `frontend/src/components/ticketComponents/AudioPlayer.vue` | Add transcription display |
| `frontend/src/types/comment.ts` | Update Attachment interface |

---

## Configuration

Add to `docker.env`:
```
# Optional: OpenAI API key for Whisper API (fallback if local unavailable)
OPENAI_API_KEY=sk-...

# Or use local Whisper model
WHISPER_MODEL_PATH=/app/models/whisper-base.bin
```
