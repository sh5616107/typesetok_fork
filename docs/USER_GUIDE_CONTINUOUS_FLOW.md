# Using Continuous Flow Columns in TypesetOK

## Overview

Phase 6 adds UI controls for continuous flow column layouts, making it easy to create magazine-style documents with 2 or 3 columns without editing JSON files.

**What is Continuous Flow?**
- Text flows naturally from one column to the next, like in a magazine or newspaper
- Different from **Parallel Flow** (traditional Talmud layout with Gemara, Rashi, Tosafot)
- Columns can be automatically balanced for equal heights

---

## Quick Start

### Option 1: Use a Template

1. **File → New from Template**
2. Choose:
   - **Magazine (2 Columns)** - Two-column continuous flow
   - **Magazine (3 Columns)** - Three-column continuous flow  
   - **Mixed Article** - Single column intro + two-column body

Templates are pre-configured and ready to use!

### Option 2: Configure Manually

#### Via Section Settings Dialog

1. Right-click on a section → **Edit Section Settings**
2. In the **Column Layout** section:
   - Set **Column Count** to 1, 2, or 3
   - Choose **Distribution Mode**:
     - **Balanced** - All columns same height (recommended)
     - **Sequential** - Fill column by column (natural flow)
3. Click **Apply**

#### Via Column Toolbar (Quick Access)

1. Select a section
2. Use the toolbar buttons:
   - **[1]** - Single column
   - **[2]** - Two columns
   - **[3]** - Three columns
3. Toggle **Balance** for automatic height equalization

---

## Column Distribution Modes

### Balanced (מאוזן)
- All columns end at approximately the same height
- Best for clean, professional appearance
- Recommended for most use cases
- Uses Phase 3 balance solver with 7-stage adjustment

**Example:**
```
┌──────────┬──────────┐
│ Content  │ Content  │
│ flows    │ flows    │
│ here and │ continues│
│ fills    │ evenly   │
│ both     │ to end   │
└──────────┴──────────┘
```

### Sequential (רציף)
- Fills first column completely before moving to next
- Natural reading flow
- May result in unequal column heights
- Good for chronological content

**Example:**
```
┌──────────┬──────────┐
│ First    │ Then     │
│ column   │ second   │
│ fills    │ column   │
│ entirely │ starts   │
│ first    │          │
└──────────┴──────────┘
```

---

## Multi-Section Documents

You can mix different layouts in the same document!

### Example: Article with Intro + Body

**Section 1: Introduction**
- Column Count: 1 (full width)
- Use for: Title, author, abstract

**Section 2: Main Article**
- Column Count: 2
- Distribution: Balanced
- Use for: Body text

**Section 3: Conclusion**
- Column Count: 1 (full width)
- Use for: Summary, references

---

## Technical Details

### JSON Structure

When you set columns via UI, TypesetOK updates the section JSON:

```json
{
  "id": "sec-main",
  "name": "Main Article",
  "page_style": "Standard",
  "column_count": 2,
  "distribution_mode": "Balanced",
  "flows": [...]
}
```

**Fields:**
- `column_count` (number): 1, 2, or 3
- `distribution_mode` (string): "Sequential" or "Balanced"

**Note:** If `column_count` is absent or 1, the section uses parallel flows (traditional layout).

### Backend Processing

1. **Layout Classifier** (Phase 1) - Detects continuous flow from `column_count`
2. **Engine Router** (Phase 5) - Routes to `SnakeColumnBreaker` for continuous flow
3. **Snake Column Breaker** (Phase 4) - Creates columns with natural flow
4. **Balance Solver** (Phase 3) - Equalizes heights if `distribution_mode` is "Balanced"
5. **Balance Measurement** (Phase 2) - Validates final column heights

---

## Limitations

### Current Constraints

1. **Maximum 3 columns** via UI (backend supports more via JSON)
2. **Section-level only** - Cannot change columns mid-section
3. **No mixed parallel + continuous** - A document cannot have both multi-flow anchoring and continuous flow sections (Phase 5 diagnostic)

### Not Yet Supported (Future Phases)

- Custom column widths (currently equal widths)
- Column rules (vertical lines between columns)
- Spanning elements (headings across all columns)
- Mid-section column changes

---

## Troubleshooting

### "Columns look uneven"
- **Solution:** Enable **Balanced** distribution mode
- If still uneven, check balance measurement diagnostics

### "Text disappeared"
- **Solution:** Increase page height or reduce font size
- Check that column width is sufficient for text

### "Template doesn't load"
- **Solution:** Ensure `.json` files are in `packages/tok-ui/src/templates/`
- Validate JSON syntax

### "IPC handler not responding"
- **Solution:** Check Electron main process logs
- Verify `tok-cli` binary is accessible

---

## Examples

### Example 1: Two-Column Newsletter

```
Section: "Weekly Newsletter"
├─ Column Count: 2
├─ Distribution: Balanced
└─ Page Style: Letter
```

**Result:** Professional two-column layout with equal heights.

### Example 2: Three-Column Magazine

```
Section: "Magazine Cover Story"
├─ Column Count: 3
├─ Distribution: Balanced
└─ Page Style: A4
```

**Result:** Dense three-column layout like Time Magazine.

### Example 3: Academic Article

```
Section 1: "Abstract"
├─ Column Count: 1
└─ Distribution: N/A

Section 2: "Body"
├─ Column Count: 2
└─ Distribution: Sequential

Section 3: "References"
├─ Column Count: 1
└─ Distribution: N/A
```

**Result:** Full-width abstract, two-column body, full-width references.

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Ctrl+1** | Set 1 column (future) |
| **Ctrl+2** | Set 2 columns (future) |
| **Ctrl+3** | Set 3 columns (future) |
| **Ctrl+B** | Toggle balance (future) |

*Note: Shortcuts planned for Phase 7.*

---

## API Reference (For Developers)

### IPC Handlers

#### `tok:set-section-column-layout`
```typescript
ipcRenderer.invoke('tok:set-section-column-layout', {
  sectionId: 'sec-123',
  columnCount: 2,
  distributionMode: 'Balanced'
});
```

#### `tok:get-section-column-info`
```typescript
const info = await ipcRenderer.invoke('tok:get-section-column-info', {
  sectionId: 'sec-123'
});
// Returns: { sectionId, columnCount, distributionMode, flowMode }
```

### TypeScript Interfaces

```typescript
interface SectionSettings {
  id: string;
  name: string;
  pageStyle: string;
  columnCount?: number;
  distributionMode?: 'Sequential' | 'Balanced';
}
```

### Components

- `SectionSettingsDialog` - Full section editor
- `ColumnToolbar` - Quick column controls

---

## Further Reading

- [Phase 1-5 Implementation Report](./PHASE5_COMPLETION_REPORT.md)
- [Phase 6 Plan](./PHASE6_PLAN.md)
- [SPEC_ALIGNMENT.md](../SPEC_ALIGNMENT.md) - Technical architecture

---

## Feedback & Support

- **GitHub Issues:** [TypesetOK/typesetok/issues](https://github.com/TypesetOK/typesetok/issues)
- **Discussions:** Phase 6 feedback thread
- **Email:** support@typesetok.com

---

**Version:** Phase 6 (2025-01-15)  
**Author:** TypesetOK Team  
**Last Updated:** 2025-01-15
