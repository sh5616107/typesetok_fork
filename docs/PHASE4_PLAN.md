# Phase 4: Continuous Flow (Magazine Layout)

## Prerequisites
- Phase 1: Infrastructure
- Phase 2: Measurement  
- Phase 3: Balance Solver

## Objective

Implement **single continuous flow** distributed across multiple columns (magazine/newspaper style), as opposed to multiple independent parallel flows.

## Current vs Target

### Current (Parallel Flows)
```
┌─────────────┬─────────────┐
│  Flow A     │  Flow B     │
│  Para 1     │  Para 1     │
│  Para 2     │  Para 2     │
│  Para 3     │  Para 3     │
│             │             │
└─────────────┴─────────────┘
```
Two separate `Flow` objects with their own paragraphs.

### Target (Continuous Flow)
```
┌─────────────┬─────────────┐
│  Para 1     │  Para 3     │
│  Para 2     │  Para 4     │
│             │  Para 5     │
│             │             │
└─────────────┴─────────────┘
```
One `Flow`, content "snakes" left-to-right (or right-to-left in Hebrew).

## Architecture

```
ParagraphStyle
  + column_count: usize           // 1, 2, 3, ...
  + column_gutter_pt: f32         // gap between columns
  + column_balance: BalanceMode   // Sequential | Balanced

         ↓
    
SnakeColumnBreaker
  distribute_lines(
    lines: Vec<LineBox>,
    num_columns: usize,
    column_height: f32,
  ) -> Vec<ColumnAllocation>

         ↓
         
BalanceSolver (Phase 3)
  solve() now receives:
    - snake_column_config
    - applies balance across snake columns
```

## Core Components

### 1. Style Extension
**File:** `crates/tok-core/src/styles.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnBalanceMode {
    /// Fill columns sequentially left-to-right (or right-to-left)
    Sequential,
    /// Balance column heights equally (like magazine layout)
    Balanced,
}

impl Default for ColumnBalanceMode {
    fn default() -> Self {
        Self::Sequential
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphStyle {
    // ... existing fields ...
    
    /// Number of columns for continuous flow (1 = single column, default)
    #[serde(default)]
    pub column_count: usize,
    
    /// Gap between continuous flow columns (pt)
    #[serde(default = "default_column_gutter")]
    pub column_gutter_pt: f32,
    
    /// How to distribute content across columns
    #[serde(default)]
    pub column_balance_mode: ColumnBalanceMode,
}

fn default_column_gutter() -> f32 {
    12.0
}
```

### 2. Snake Column Breaker
**File:** `crates/tok-typeset/src/snake_column_breaker.rs`

```rust
pub struct SnakeColumnBreaker {
    config: SnakeConfig,
}

pub struct SnakeConfig {
    pub num_columns: usize,
    pub column_width: f32,
    pub column_height: f32,
    pub gutter: f32,
    pub balance_mode: ColumnBalanceMode,
}

#[derive(Debug, Clone)]
pub struct ColumnAllocation {
    pub column_index: usize,
    pub lines: Vec<LineBox>,
    pub x_offset: f32,
    pub y_offset: f32,
    pub height: f32,
}

impl SnakeColumnBreaker {
    pub fn new(config: SnakeConfig) -> Self {
        Self { config }
    }
    
    /// Distribute lines across columns
    pub fn distribute(
        &self,
        lines: Vec<LineBox>,
    ) -> Result<Vec<ColumnAllocation>, SnakeError> {
        match self.config.balance_mode {
            ColumnBalanceMode::Sequential => {
                self.distribute_sequential(lines)
            }
            ColumnBalanceMode::Balanced => {
                self.distribute_balanced(lines)
            }
        }
    }
    
    /// Fill columns left-to-right sequentially
    fn distribute_sequential(
        &self,
        lines: Vec<LineBox>,
    ) -> Result<Vec<ColumnAllocation>, SnakeError> {
        let mut allocations = Vec::new();
        let mut current_column = 0;
        let mut current_lines = Vec::new();
        let mut current_height = 0.0;
        
        for line in lines {
            // Check if line fits in current column
            if current_height + line.height > self.config.column_height 
                && current_column < self.config.num_columns - 1 
            {
                // Finalize current column
                allocations.push(ColumnAllocation {
                    column_index: current_column,
                    lines: std::mem::take(&mut current_lines),
                    x_offset: self.column_x_offset(current_column),
                    y_offset: 0.0,
                    height: current_height,
                });
                
                // Move to next column
                current_column += 1;
                current_height = 0.0;
            }
            
            current_lines.push(line.clone());
            current_height += line.height;
        }
        
        // Finalize last column
        if !current_lines.is_empty() {
            allocations.push(ColumnAllocation {
                column_index: current_column,
                lines: current_lines,
                x_offset: self.column_x_offset(current_column),
                y_offset: 0.0,
                height: current_height,
            });
        }
        
        Ok(allocations)
    }
    
    /// Balance column heights as equally as possible
    fn distribute_balanced(
        &self,
        lines: Vec<LineBox>,
    ) -> Result<Vec<ColumnAllocation>, SnakeError> {
        // Calculate total height
        let total_height: f32 = lines.iter().map(|l| l.height).sum();
        let target_per_column = total_height / self.config.num_columns as f32;
        
        // Use first-fit decreasing or dynamic programming
        let allocations = self.balance_columns_dp(lines, target_per_column)?;
        
        Ok(allocations)
    }
    
    /// Dynamic programming for balanced column allocation
    fn balance_columns_dp(
        &self,
        lines: Vec<LineBox>,
        target_height: f32,
    ) -> Result<Vec<ColumnAllocation>, SnakeError> {
        // State: (line_index, column_index) -> (min_imbalance, split_points)
        // Goal: minimize max(column_height) - min(column_height)
        
        // Simplified: greedy with look-ahead
        let mut allocations = Vec::new();
        let mut remaining_lines = lines;
        
        for col_idx in 0..self.config.num_columns {
            let remaining_cols = self.config.num_columns - col_idx;
            let remaining_height: f32 = remaining_lines.iter().map(|l| l.height).sum();
            let target_this_col = remaining_height / remaining_cols as f32;
            
            let mut col_lines = Vec::new();
            let mut col_height = 0.0;
            
            while !remaining_lines.is_empty() {
                let next_line = &remaining_lines[0];
                
                // Would adding this line exceed target significantly?
                if col_height > 0.0 
                    && col_height + next_line.height > target_this_col * 1.2 
                {
                    break;
                }
                
                let line = remaining_lines.remove(0);
                col_height += line.height;
                col_lines.push(line);
            }
            
            allocations.push(ColumnAllocation {
                column_index: col_idx,
                lines: col_lines,
                x_offset: self.column_x_offset(col_idx),
                y_offset: 0.0,
                height: col_height,
            });
        }
        
        Ok(allocations)
    }
    
    fn column_x_offset(&self, col_idx: usize) -> f32 {
        col_idx as f32 * (self.config.column_width + self.config.gutter)
    }
}
```

### 3. Integration with Engine
**File:** Update `crates/tok-typeset/src/engine.rs`

```rust
impl TypesettingEngine {
    fn typeset_paragraph_with_columns(
        &self,
        paragraph: &ParagraphNode,
        style: &ParagraphStyle,
        available_width: f32,
        available_height: f32,
    ) -> Result<Vec<ColumnAllocation>, EngineError> {
        
        // Check if paragraph uses snake columns
        if style.column_count > 1 {
            let gutter = style.column_gutter_pt;
            let total_gutter = gutter * (style.column_count - 1) as f32;
            let column_width = (available_width - total_gutter) / style.column_count as f32;
            
            // 1. Break lines at column width (not full width)
            let lines = self.typeset_paragraph_lines(
                paragraph,
                column_width,
                style,
            )?;
            
            // 2. Distribute to columns
            let breaker = SnakeColumnBreaker::new(SnakeConfig {
                num_columns: style.column_count,
                column_width,
                column_height: available_height,
                gutter,
                balance_mode: style.column_balance_mode,
            });
            
            return breaker.distribute(lines);
        }
        
        // Regular single-column layout
        let lines = self.typeset_paragraph_lines(paragraph, available_width, style)?;
        Ok(vec![ColumnAllocation {
            column_index: 0,
            lines,
            x_offset: 0.0,
            y_offset: 0.0,
            height: lines.iter().map(|l| l.height).sum(),
        }])
    }
}
```

### 4. Balance Solver Integration

The Phase 3 `BalanceSolver` should work with snake columns:

```rust
impl BalanceSolver {
    pub fn solve_snake_columns(
        &self,
        allocations: Vec<ColumnAllocation>,
        policy: &PagePolicy,
    ) -> BalanceResult {
        // Measure bottom delta across snake columns
        let measurements: Vec<_> = allocations
            .iter()
            .map(|a| ColumnMeasurement {
                column_index: a.column_index,
                last_baseline_y: a.lines.last().map(|l| Mpt::from_pt(l.y + l.height)),
                line_count: a.lines.len(),
                is_empty: a.lines.is_empty(),
            })
            .collect();
        
        let delta = BottomDelta::from_columns(&measurements, baseline_step)?;
        
        if delta.delta == Mpt(0) {
            return BalanceResult::Exact(/* ... */);
        }
        
        // Apply adjustment stages (Stage 1-5 from Phase 3)
        // ...
    }
}
```

## Section Model Extension

For sections that should use continuous flow:

```rust
impl SectionNode {
    pub fn with_continuous_flow_columns(mut self, count: usize) -> Self {
        // Mark section for continuous flow layout
        self.layout_kind = Some("continuous_flow".to_string());
        self.column_proportions = None;  // Not applicable
        
        // Update all paragraph styles
        for flow in &mut self.flows {
            for para in &mut flow.paragraphs {
                // Apply column_count to style
            }
        }
        
        self
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_snake_sequential_fill() {
    let lines = create_test_lines(10, 10.0);  // 10 lines, 10pt each
    
    let breaker = SnakeColumnBreaker::new(SnakeConfig {
        num_columns: 2,
        column_width: 100.0,
        column_height: 50.0,  // Fits 5 lines
        gutter: 12.0,
        balance_mode: ColumnBalanceMode::Sequential,
    });
    
    let allocations = breaker.distribute(lines).unwrap();
    
    assert_eq!(allocations.len(), 2);
    assert_eq!(allocations[0].lines.len(), 5);
    assert_eq!(allocations[1].lines.len(), 5);
}

#[test]
fn test_snake_balanced_distribution() {
    let lines = vec![
        create_line(30.0),  // tall
        create_line(10.0),
        create_line(10.0),
        create_line(10.0),
        create_line(10.0),
    ];
    // Total: 70pt, target per column: 35pt
    
    let breaker = SnakeColumnBreaker::new(SnakeConfig {
        num_columns: 2,
        column_height: 100.0,
        balance_mode: ColumnBalanceMode::Balanced,
        // ...
    });
    
    let allocations = breaker.distribute(lines).unwrap();
    
    // Should put tall line in first column with one small
    // And three small lines in second column
    assert_eq!(allocations[0].height, 40.0);
    assert_eq!(allocations[1].height, 30.0);
    // Relatively balanced!
}
```

### Integration Test
```rust
#[test]
fn test_magazine_layout_two_columns() {
    let mut doc = DocumentRoot::new("Magazine Article");
    
    let mut style = ParagraphStyle::default();
    style.column_count = 2;
    style.column_gutter_pt = 12.0;
    style.column_balance_mode = ColumnBalanceMode::Balanced;
    
    doc.paragraph_styles = vec![style];
    
    let section = &mut doc.sections[0];
    let flow = section.main_flow_mut().unwrap();
    
    // Add long continuous text
    for i in 0..20 {
        flow.add_paragraph(ParagraphNode::new(
            FractionalIndex::new(format!("p{}", i)),
            "body",
            &format!("Paragraph {} with enough text to span lines.", i),
        ));
    }
    
    let engine = TypesettingEngine::new(config);
    let result = engine.typeset_document(&doc);
    
    assert_eq!(result.status, LayoutStatus::Valid);
    
    // Should have multiple pages with balanced columns
    for page in &result.pages {
        if page.columns.len() == 2 {
            let delta = (page.columns[0].occupied_height.0 
                - page.columns[1].occupied_height.0).abs();
            
            // Balanced within one line height
            assert!(delta < 15000);  // 15pt in mpt
        }
    }
}
```

## Edge Cases

### 1. Column Span Elements
Future extension: Some elements (headings, images) might span all columns.

```rust
pub enum ColumnSpan {
    Single,      // Fits in one column
    All,         // Spans all columns (heading, wide image)
}
```

### 2. Column Break Control
```rust
pub enum ColumnBreak {
    Auto,           // Let algorithm decide
    Before,         // Force break before this paragraph
    After,          // Force break after
    Avoid,          // Try to keep in same column
}
```

### 3. Widows/Orphans Across Columns
The existing widow/orphan logic should work, treating column breaks similarly to page breaks.

## Spec Alignment

The spec mentions (section 4):
> `Section { kind: front | body | back, wordSection: nextPage | continuous | oddPage | evenPage }`

The `continuous` mode means "start mid-page" not "continuous columns". However, continuous flow columns are a natural extension for the two-column requirement.

## Non-Goals for Phase 4

- ❌ Column-spanning elements (headings, images)
- ❌ Complex column shapes (L-shape already exists for Talmud)
- ❌ More than 4 columns (spec focuses on 2-3)

## Success Criteria

- [ ] `column_count` in ParagraphStyle works
- [ ] Sequential fill distributes correctly
- [ ] Balanced mode achieves reasonable balance
- [ ] Integration with Phase 3 balance solver
- [ ] Magazine layout test passes
- [ ] No regression in multi-flow (Talmud) layouts
- [ ] Performance acceptable (< 2x slowdown)

## Timeline

Estimated: 1-2 weeks (3-6 work sessions)

---

*This phase is relatively independent and can be done in parallel with Phase 3 refinements.*
